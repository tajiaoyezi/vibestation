//! MVP-19 W1-A.0 · canonical session + commit-link contract types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// §G.2 `ai_sessions` row mapping.
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AiSession {
    pub id: String,
    pub workspace_id: String,
    pub cli_kind: String,
    /// §G.2 `auto | manual`.
    pub source: String,
    pub title: String,
    #[ts(type = "number")]
    pub started_at: i64,
    #[ts(type = "number | null")]
    pub ended_at: Option<i64>,
    pub end_reason: Option<String>,
    // i64 counts cross the IPC boundary as serde JSON numbers; map to TS `number`
    // (not ts-rs default `bigint`) to match the project binding convention used by
    // every existing `*Count` field (branchCount/commitCount/paneCount/...).
    #[ts(type = "number")]
    pub prompt_count: i64,
    #[ts(type = "number | null")]
    pub token_count: Option<i64>,
    #[ts(type = "number")]
    pub event_count: i64,
    pub status: SessionStatus,
    pub parser_version: Option<String>,
    pub strategy_version: Option<String>,
    /// §G.2 defaults to `{}`. Do not store plaintext secrets or sensitive user data.
    pub metadata_json: String,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

/// §G.2 `ai_sessions.status`.
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    Active,
    Ended,
    IdleCutoff,
    Archived,
}

/// §G.3 `session_commit_links` row mapping.
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionCommitLink {
    pub id: String,
    pub workspace_id: String,
    pub session_id: String,
    pub commit_sha: String,
    /// SQL INTEGER 0/1 maps at the DAO boundary; canonical Rust contract is bool.
    pub is_primary: bool,
    pub link_state: LinkState,
    pub auto_bound: bool,
    pub confidence: f32,
    pub confidence_reason: String,
    pub strategy_version: String,
    pub source_event_id: Option<String>,
    #[ts(type = "number")]
    pub linked_at: i64,
    #[ts(type = "number | null")]
    pub unlinked_at: Option<i64>,
    pub unlinked_reason: Option<String>,
    pub superseded_by_link_id: Option<String>,
    pub created_by: String,
    pub reviewed_by: Option<String>,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

/// §G.3 `session_commit_links.link_state`.
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum LinkState {
    Pending,
    ConfirmedAuto,
    ConfirmedManual,
    Unlinked,
    Superseded,
    Stale,
}

/// §G.6 tagged error contract.
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, thiserror::Error)]
#[ts(export)]
#[serde(tag = "kind", content = "detail", rename_all = "camelCase")]
pub enum SessionError {
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("link not found: {0}")]
    LinkNotFound(String),
    #[error("cross-workspace operation denied")]
    CrossWorkspaceDenied,
    #[error("invalid state transition: {0}")]
    InvalidStateTransition(String),
    #[error("database error: {0}")]
    DbError(String),
}
