use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::manifest::{self, Manifest, PerTableStats};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpLogEntry {
    pub tx_id: String,
    pub status: String,
    pub table: String,
    pub key_hash: String,
    pub op: String,
    pub ts_start: u64,
    pub ts_end: u64,
    pub checksum: String,
}

pub fn write_oplog_entry(path: &Path, entry: &OpLogEntry) {
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
    let line = serde_json::to_string(entry).unwrap();
    writeln!(f, "{}", line).unwrap();
    f.sync_all().unwrap();
}

pub fn read_oplog(path: &Path) -> Vec<OpLogEntry> {
    if !path.exists() { return vec![]; }
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content.lines().filter(|l| !l.is_empty()).filter_map(|l| serde_json::from_str(l).ok()).collect()
}

pub fn update_oplog_status(path: &Path, tx_id: &str, new_status: &str) {
    let mut entries = read_oplog(path);
    for e in &mut entries {
        if e.tx_id == tx_id { e.status = new_status.to_string(); }
    }
    let mut f = std::fs::File::create(path).unwrap();
    for e in &entries { writeln!(f, "{}", serde_json::to_string(e).unwrap()).unwrap(); }
    f.sync_all().unwrap();
}