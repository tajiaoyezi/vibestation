use std::path::Path;

use rusqlite::Connection;

use crate::manifest::{self, Manifest, PerTableStats};

pub fn create_backup(src_db: &Path, backup_dir: &Path) {
    std::fs::create_dir_all(backup_dir).unwrap();
    let dst = backup_dir.join("data.sqlite");
    std::fs::copy(src_db, &dst).unwrap();
    let conn = Connection::open(&dst).unwrap();
    let count = count_rows(&conn);
    let version = get_user_version(&conn);
    let checksum = sha256_file(&dst);
    drop(conn);
    let mut per_table = std::collections::HashMap::new();
    per_table.insert("snapshots".to_string(), PerTableStats { row_count: count, sha256_checksum: checksum.clone() });
    let m = Manifest {
        user_version: version,
        per_table,
        last_committed_tx_id: None,
        export_timestamp: ts(),
    };
    manifest::write_manifest(backup_dir, &m);
}

/// Create a periodic backup with timestamp-based naming and retention policy.
/// Keeps the most recent `retention_count` backups plus 1 last-known-good.
/// Returns the backup directory path.
pub fn create_periodic_backup(db: &Path, backup_root: &Path, retention_count: usize) -> String {
    std::fs::create_dir_all(backup_root).unwrap();

    // 1. Create new backup with timestamp name
    let ts = ts();
    let backup_name = format!("auto-{}.backup", ts);
    let backup_dir = backup_root.join(&backup_name);
    create_backup(db, &backup_dir);

    // 2. Scan existing backups, sort by timestamp (newest first)
    let mut entries: Vec<u64> = std::fs::read_dir(backup_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("auto-") && name.ends_with(".backup") {
                // Parse "auto-<ts>.backup"
                let ts_str = name.trim_start_matches("auto-").trim_end_matches(".backup");
                ts_str.parse::<u64>().ok()
            } else {
                None
            }
        })
        .collect();
    entries.sort();
    entries.reverse(); // newest first

    // 3. Retention: keep retention_count + 1 last-known-good
    let max_keep = retention_count + 1; // regular backups + last-known-good
    for old_ts in entries.iter().skip(max_keep) {
        let old_name = format!("auto-{}.backup", old_ts);
        let old_path = backup_root.join(&old_name);
        if old_path.exists() {
            let _ = std::fs::remove_dir_all(&old_path);
        }
    }

    backup_name
}

/// Update last-known-good if self-check passes.
pub fn update_last_known_good(db: &Path, backup_root: &Path) {
    let lkg_dir = backup_root.join("last-known-good");
    if lkg_dir.exists() {
        let _ = std::fs::remove_dir_all(&lkg_dir);
    }
    create_backup(db, &lkg_dir);
}

fn count_rows(c: &Connection) -> u64 {
    c.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get::<_, u64>(0)).unwrap_or(0)
}

fn get_user_version(c: &Connection) -> u32 {
    c.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0)).unwrap_or(0)
}

fn sha256_file(p: &Path) -> String {
    let d = std::fs::read(p).unwrap_or_default();
    use sha2::Digest;
    let mut h = sha2::Sha256::new(); sha2::Digest::update(&mut h, &d); format!("{:x}", sha2::Sha256::finalize(h))
}

fn ts() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() }