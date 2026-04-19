use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerTableStats {
    pub row_count: u64,
    pub sha256_checksum: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub user_version: u32,
    pub per_table: HashMap<String, PerTableStats>,
    pub last_committed_tx_id: Option<String>,
    pub export_timestamp: u64,
}

pub fn write_manifest(dir: &std::path::Path, m: &Manifest) {
    std::fs::create_dir_all(dir).unwrap();
    let j = serde_json::to_string_pretty(m).unwrap();
    let tmp = dir.join("manifest.json.tmp");
    std::fs::write(&tmp, j).unwrap();
    std::fs::rename(&tmp, dir.join("manifest.json")).unwrap();
}

pub fn read_manifest(dir: &std::path::Path) -> Manifest {
    let j = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    serde_json::from_str(&j).unwrap()
}