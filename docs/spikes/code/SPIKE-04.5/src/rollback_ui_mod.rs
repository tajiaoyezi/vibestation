use std::path::Path;

/// CLI mock for auto-rollback UI.
/// In production, this would present a dialog to the user.
/// In this spike, it simulates the user choosing to roll back.
pub fn mock_rollback_ui(db_path: &Path, backup_dir: &Path) -> Result<u64, String> {
    let backup_db = backup_dir.join("data.sqlite");
    if !backup_db.exists() {
        return Err(format!("Backup not found at {}", backup_db.display()));
    }

    let backup_count = {
        let conn = rusqlite::Connection::open(&backup_db).map_err(|e| format!("open backup: {}", e))?;
        let count: u64 = conn.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get::<_, u64>(0)).unwrap_or(0);
        count
    };

    println!("    [MOCK UI] DB integrity check... FAILED");
    println!("    [MOCK UI] Prompt: 数据库损坏 · 从 {} 恢复？预计丢失 0 行 · [Y/n]", backup_dir.display());
    println!("    [MOCK UI] User chose: Y");

    // Rollback: copy backup -> db
    std::fs::copy(&backup_db, db_path).map_err(|e| format!("copy: {}", e))?;

    // Verify after rollback
    let restored_count = {
        let conn = rusqlite::Connection::open(db_path).map_err(|e| format!("open restored: {}", e))?;
        let count: u64 = conn.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get::<_, u64>(0)).unwrap_or(0);
        count
    };

    println!("    Rollback complete: {} rows restored", restored_count);
    Ok(restored_count)
}