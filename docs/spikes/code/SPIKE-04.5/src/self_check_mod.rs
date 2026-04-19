use std::path::Path;

use rusqlite::Connection;

use crate::manifest;
use crate::op_log;

pub fn self_check(db_path: &Path, oplog_path: &Path) -> Result<String, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("open: {}", e))?;
    let actual_count = count_rows(&conn);
    let entries = op_log::read_oplog(oplog_path);

    for entry in &entries {
        match entry.status.as_str() {
            "pending" => {
                if actual_count > 0 {
                    op_log::update_oplog_status(oplog_path, &entry.tx_id, "committed");
                    return Ok(format!("ReconciledForward (pending→committed, DB has {} rows)", actual_count));
                } else {
                    op_log::update_oplog_status(oplog_path, &entry.tx_id, "aborted");
                    return Ok("ReconciledForward (pending→aborted, DB empty)".to_string());
                }
            }
            "committed" => {
                let expected_count = entry.key_hash.parse::<u64>().unwrap_or(0);
                if expected_count > 0 && actual_count < expected_count {
                    return Ok(format!("SilentLossDetected (oplog says committed {} rows, DB has {})", expected_count, actual_count));
                }
                return Ok("Consistent".to_string());
            }
            "aborted" => {
                return Ok("Consistent (aborted tx ignored)".to_string());
            }
            _ => {}
        }
    }

    Ok(format!("Consistent (no oplog entries, {} rows)", actual_count))
}

pub fn self_check_with_manifest(db_path: &Path, manifest_dir: &Path) -> Result<String, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("open: {}", e))?;
    let actual_count = count_rows(&conn);
    let current_checksum = sha256_file(db_path);
    let m = manifest::read_manifest(manifest_dir);

    // Compare DB file checksum against per_table "snapshots" entry
    if let Some(snap_stats) = m.per_table.get("snapshots") {
        if current_checksum != snap_stats.sha256_checksum {
            return Ok(format!("ChecksumMismatch (expected {}..., got {}...)",
                &snap_stats.sha256_checksum[..snap_stats.sha256_checksum.len().min(16)],
                &current_checksum[..current_checksum.len().min(16)]));
        }
    } else {
        // Fallback: if per_table empty, just check consistency
        return Ok(format!("Consistent ({} rows, no per_table checksum to verify)", actual_count));
    }
    Ok(format!("Consistent ({} rows)", actual_count))
}

fn count_rows(c: &Connection) -> u64 {
    c.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get::<_, u64>(0)).unwrap_or(0)
}

fn sha256_file(p: &Path) -> String {
    let d = std::fs::read(p).unwrap_or_default();
    use sha2::Digest;
    let mut h = sha2::Sha256::new(); sha2::Digest::update(&mut h, &d); format!("{:x}", sha2::Sha256::finalize(h))
}