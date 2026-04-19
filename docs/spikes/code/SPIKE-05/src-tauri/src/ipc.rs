use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnSessionRequest {
    pub label: String,
    pub command: String,
    pub cols: u16,
    pub rows: u16,
    pub queue_capacity: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteSessionRequest {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeSessionRequest {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrainSessionRequest {
    pub session_id: String,
    pub max_chunks: Option<usize>,
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactWriteRequest {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactReadRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub label: String,
    pub command: String,
    pub queue_depth: usize,
    pub queued_bytes: usize,
    pub avg_queue_depth: f64,
    pub max_queue_depth: usize,
    pub total_read_bytes: u64,
    pub total_drained_bytes: u64,
    pub dropped_chunks: u64,
    pub dropped_bytes: u64,
    pub exit_status: Option<String>,
    pub created_at_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrainResponse {
    pub chunks: Vec<String>,
    pub queue_depth: usize,
    pub queued_bytes: usize,
    pub avg_queue_depth: f64,
    pub max_queue_depth: usize,
    pub total_read_bytes: u64,
    pub total_drained_bytes: u64,
    pub dropped_chunks: u64,
    pub dropped_bytes: u64,
    pub exit_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStats {
    pub pid: u32,
    pub rss_kb: u64,
    pub fd_count: usize,
    pub session_count: usize,
    pub reader_thread_alive: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    pub scenario: Option<String>,
    pub output_dir: Option<String>,
    pub report_dir: Option<String>,
    pub run_label: Option<String>,
    pub close_on_complete: bool,
}
