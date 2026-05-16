//! MVP-18 Phase A · AI-Aware Pane 联动关系数据类型 + 验证器 + DAO。
//!
//! spec §G（schema · migrate_v8 pane_links 表）+ §K（IPC contract source-of-truth ·
//! ts-rs derives）+ §E B.1-B.4（link 创建/持久化 acceptance）。
//!
//! 本切片范围（A1 关键路径 · 首切片）：
//! - §K 核心 types（`PaneLinkKind`/`PaneLinkStatus`/`PaneKind`/`PaneLinkRequest`/
//!   `PaneLink`/`PaneLinkError`）
//! - 纯验证器 `validate_link_request`（§E B.1 cross-workspace · B.2 parent type ·
//!   B.3 child type）—— pane kind/workspace 由 IPC 层从 pane registry 解析后传入，
//!   本层纯逻辑（spec §F core unit 层 = in-memory rusqlite + synthetic pane rows）
//! - `PaneLinkDao::create`（§E B.4 重复 (ws,parent,child,kind) 返回已有不插重复 ·
//!   over migrate_v8 表 · migrate_v8 UNIQUE 作 DB 层 backstop）
//!
//! 后续切片：unlink / list / set_enabled DAO · `pane:linked`/`pane:trigger` events ·
//! `pane:*` Tauri IPC 命令注册 · ts-rs build.rs 接线（A1 续 · 文件域归 A1）。

use crate::db::{DbError, DbPool};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// §G.1 `link_kind` · §K.3 binding 2。首版仅 `failureFeedback`（未来 kind 需 migration）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PaneLinkKind {
    FailureFeedback,
}

impl PaneLinkKind {
    /// DB / 事件文案用稳定串（与 §K.6 payload 例 `"failureFeedback"` 一致）。
    pub fn as_db_str(&self) -> &'static str {
        match self {
            PaneLinkKind::FailureFeedback => "failureFeedback",
        }
    }

    fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "failureFeedback" => Some(PaneLinkKind::FailureFeedback),
            _ => None,
        }
    }
}

/// §K.6 `status` · §K.3 binding 3。link 生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PaneLinkStatus {
    Enabled,
    Disabled,
    Stale,
    Removed,
}

/// Pane 种类（§E B.2/B.3 验证用）。由 IPC 层从 live pane registry 解析 pane_id →
/// kind 后传入验证器；本 enum 不改 MVP-14 LayoutNode schema（spec §C / §H.7）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PaneKind {
    Ai,
    Runner,
    Watch,
    Log,
    Build,
    Shell,
    Other,
}

impl PaneKind {
    /// §E B.2：只有 AI Pane 能作接收上下文的 parent。
    pub fn can_be_parent(self) -> bool {
        matches!(self, PaneKind::Ai)
    }

    /// §E B.3：child 必须是 Runner/Watch/Log/Build/Shell（执行型）；纯 UI Pane 不行。
    pub fn can_be_child(self) -> bool {
        matches!(
            self,
            PaneKind::Runner | PaneKind::Watch | PaneKind::Log | PaneKind::Build | PaneKind::Shell
        )
    }
}

/// §K.4 `PaneLinkRequest` · `pane:link` 命令请求。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneLinkRequest {
    pub workspace_id: String,
    pub parent_pane_id: String,
    pub child_pane_id: String,
    pub link_kind: PaneLinkKind,
}

/// §K.3 binding 1 · §G schema 行映射。仅 metadata（spec §G.3：不存 raw/prompt/token）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneLink {
    pub id: String,
    pub workspace_id: String,
    pub parent_pane_id: String,
    pub child_pane_id: String,
    pub link_kind: PaneLinkKind,
    pub enabled: bool,
    /// §G.1 `'structured' | 'rawText' | 'disabledByParser'`。
    pub fallback_mode: String,
    /// §G.1 `'user' | 'preset' | 'migration'`；MVP-18 用 `user`。
    pub created_by: String,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
    #[ts(type = "number")]
    pub last_triggered_at: Option<i64>,
}

/// §K.5 `PaneLinkError` · UI-facing binding 18。tagged enum：machine 可读 `kind` +
/// 人类可读 message（thiserror `Display`）。serde adjacently-tagged → JSON
/// `{ "kind": "...", "detail"?: "..." }`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, thiserror::Error)]
#[ts(export)]
#[serde(tag = "kind", content = "detail", rename_all = "camelCase")]
pub enum PaneLinkError {
    #[error("cross-workspace link denied")]
    CrossWorkspaceDenied,
    #[error("parent pane is not an AI pane")]
    InvalidParentPaneType,
    #[error("child pane cannot feed AI context")]
    InvalidChildPaneType,
    #[error("pane not found: {0}")]
    PaneNotFound(String),
    #[error("link not found: {0}")]
    LinkNotFound(String),
    #[error("unsupported cli kind: {0}")]
    UnsupportedCliKind(String),
    #[error("database error: {0}")]
    Db(String),
}

impl From<DbError> for PaneLinkError {
    fn from(e: DbError) -> Self {
        PaneLinkError::Db(e.to_string())
    }
}

/// 纯验证器（§E B.1/B.2/B.3）。`parent_*`/`child_*` 由调用方（IPC 层）从 live
/// pane registry 解析得到；本函数不触 DB / registry，便于 §F core unit 单测。
///
/// - §E B.1：req.workspace_id 必须同时等于 parent / child 所在 workspace，
///   否则 `CrossWorkspaceDenied`（跨 workspace context 泄漏硬边界 · §J R3）。
/// - §E B.2：parent 必须 AI Pane，否则 `InvalidParentPaneType`。
/// - §E B.3：child 必须执行型 Pane，否则 `InvalidChildPaneType`。
pub fn validate_link_request(
    req: &PaneLinkRequest,
    parent_workspace_id: &str,
    parent_kind: PaneKind,
    child_workspace_id: &str,
    child_kind: PaneKind,
) -> Result<(), PaneLinkError> {
    if parent_workspace_id != req.workspace_id || child_workspace_id != req.workspace_id {
        return Err(PaneLinkError::CrossWorkspaceDenied);
    }
    if !parent_kind.can_be_parent() {
        return Err(PaneLinkError::InvalidParentPaneType);
    }
    if !child_kind.can_be_child() {
        return Err(PaneLinkError::InvalidChildPaneType);
    }
    Ok(())
}

/// `pane_links` 表 DAO（over migrate_v8 · 镜像 `TabsDao` idiom）。
pub struct PaneLinkDao;

impl PaneLinkDao {
    fn row_to_link(row: &rusqlite::Row<'_>) -> Result<PaneLink, rusqlite::Error> {
        let kind_str: String = row.get(4)?;
        let link_kind = PaneLinkKind::from_db_str(&kind_str).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                format!("unknown link_kind: {kind_str}").into(),
            )
        })?;
        let enabled_int: i64 = row.get(5)?;
        Ok(PaneLink {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            parent_pane_id: row.get(2)?,
            child_pane_id: row.get(3)?,
            link_kind,
            enabled: enabled_int != 0,
            fallback_mode: row.get(6)?,
            created_by: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            last_triggered_at: row.get(10)?,
        })
    }

    /// §E B.4：已存在同 `(workspace_id, parent_pane_id, child_pane_id, link_kind)`
    /// 且未软删的 link → 返回它（幂等 · 不插重复行）；否则插入新行。
    /// migrate_v8 的 `UNIQUE(...)` 作 DB 层 backstop（双保险）。
    pub fn create(pool: &DbPool, req: &PaneLinkRequest) -> Result<PaneLink, PaneLinkError> {
        let conn = pool.get().map_err(DbError::from)?;

        if let Some(existing) = Self::find_active(&conn, req)? {
            return Ok(existing);
        }

        let id = Uuid::new_v4().to_string();
        // §G.1：created_at / updated_at = unix millis（注意 tabs 用 secs · 本表用 millis）。
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO pane_links
                (id, workspace_id, parent_pane_id, child_pane_id, link_kind,
                 enabled, fallback_mode, created_by, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 'structured', 'user', ?6, ?6)",
            rusqlite::params![
                id,
                req.workspace_id,
                req.parent_pane_id,
                req.child_pane_id,
                req.link_kind.as_db_str(),
                now,
            ],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(ref err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                PaneLinkError::Db(format!("constraint violation inserting pane_link: {e}"))
            }
            other => PaneLinkError::Db(other.to_string()),
        })?;

        Ok(PaneLink {
            id,
            workspace_id: req.workspace_id.clone(),
            parent_pane_id: req.parent_pane_id.clone(),
            child_pane_id: req.child_pane_id.clone(),
            link_kind: req.link_kind,
            enabled: true,
            fallback_mode: "structured".to_string(),
            created_by: "user".to_string(),
            created_at: now,
            updated_at: now,
            last_triggered_at: None,
        })
    }

    /// 查同 unique tuple 的未软删 link（`deleted_at IS NULL`）。
    fn find_active(
        conn: &rusqlite::Connection,
        req: &PaneLinkRequest,
    ) -> Result<Option<PaneLink>, PaneLinkError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, workspace_id, parent_pane_id, child_pane_id, link_kind,
                        enabled, fallback_mode, created_by, created_at, updated_at,
                        last_triggered_at
                 FROM pane_links
                 WHERE workspace_id = ?1 AND parent_pane_id = ?2
                   AND child_pane_id = ?3 AND link_kind = ?4
                   AND deleted_at IS NULL
                 LIMIT 1",
            )
            .map_err(DbError::from)?;
        let mut rows = stmt
            .query_map(
                rusqlite::params![
                    req.workspace_id,
                    req.parent_pane_id,
                    req.child_pane_id,
                    req.link_kind.as_db_str(),
                ],
                Self::row_to_link,
            )
            .map_err(DbError::from)?;
        match rows.next() {
            Some(r) => Ok(Some(r.map_err(DbError::from)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::workspace::WorkspaceStore;
    use tempfile::TempDir;

    fn setup() -> (TempDir, DbPool, String) {
        let dir = TempDir::new().unwrap();
        let pool = db::open_pool(&dir.path().join("test_pane_links.db")).unwrap();
        let ws_dir = dir.path().join("ws");
        std::fs::create_dir_all(&ws_dir).unwrap();
        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
        (dir, pool, ws.workspace_id)
    }

    fn req(ws: &str) -> PaneLinkRequest {
        PaneLinkRequest {
            workspace_id: ws.to_string(),
            parent_pane_id: "ai-pane".to_string(),
            child_pane_id: "runner-pane".to_string(),
            link_kind: PaneLinkKind::FailureFeedback,
        }
    }

    // ── 验证器 §E B.1/B.2/B.3 ──────────────────────────────────────

    #[test]
    fn validate_accepts_same_ws_ai_parent_runner_child() {
        let r = req("w1");
        assert!(validate_link_request(&r, "w1", PaneKind::Ai, "w1", PaneKind::Runner).is_ok());
        // child 也接受 Watch/Log/Build/Shell
        for k in [
            PaneKind::Watch,
            PaneKind::Log,
            PaneKind::Build,
            PaneKind::Shell,
        ] {
            assert!(validate_link_request(&r, "w1", PaneKind::Ai, "w1", k).is_ok());
        }
    }

    #[test]
    fn validate_rejects_cross_workspace_b1() {
        let r = req("w1");
        // child 在别的 workspace
        assert_eq!(
            validate_link_request(&r, "w1", PaneKind::Ai, "w2", PaneKind::Runner),
            Err(PaneLinkError::CrossWorkspaceDenied)
        );
        // parent 在别的 workspace
        assert_eq!(
            validate_link_request(&r, "wX", PaneKind::Ai, "w1", PaneKind::Runner),
            Err(PaneLinkError::CrossWorkspaceDenied)
        );
    }

    #[test]
    fn validate_rejects_non_ai_parent_b2() {
        let r = req("w1");
        assert_eq!(
            validate_link_request(&r, "w1", PaneKind::Runner, "w1", PaneKind::Runner),
            Err(PaneLinkError::InvalidParentPaneType)
        );
    }

    #[test]
    fn validate_rejects_non_executor_child_b3() {
        let r = req("w1");
        for bad in [PaneKind::Ai, PaneKind::Other] {
            assert_eq!(
                validate_link_request(&r, "w1", PaneKind::Ai, "w1", bad),
                Err(PaneLinkError::InvalidChildPaneType)
            );
        }
    }

    // ── DAO create / 幂等 §E B.4 ───────────────────────────────────

    #[test]
    fn create_inserts_link_with_defaults() {
        let (_d, pool, ws) = setup();
        let link = PaneLinkDao::create(&pool, &req(&ws)).unwrap();
        assert!(!link.id.is_empty());
        assert_eq!(link.workspace_id, ws);
        assert_eq!(link.parent_pane_id, "ai-pane");
        assert_eq!(link.child_pane_id, "runner-pane");
        assert_eq!(link.link_kind, PaneLinkKind::FailureFeedback);
        assert!(link.enabled);
        assert_eq!(link.fallback_mode, "structured");
        assert_eq!(link.created_by, "user");
        assert_eq!(link.created_at, link.updated_at);
        assert!(link.last_triggered_at.is_none());
        // §G.1 unix millis：13 位数量级（> 1e12）
        assert!(link.created_at > 1_000_000_000_000);
    }

    #[test]
    fn create_duplicate_tuple_returns_existing_no_new_row_b4() {
        let (_d, pool, ws) = setup();
        let first = PaneLinkDao::create(&pool, &req(&ws)).unwrap();
        let again = PaneLinkDao::create(&pool, &req(&ws)).unwrap();
        assert_eq!(first.id, again.id, "duplicate must return existing link id");

        let count: i64 = pool
            .get()
            .unwrap()
            .query_row("SELECT count(*) FROM pane_links", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "duplicate must NOT create a second DB row (§B.4)");
    }

    #[test]
    fn create_distinct_child_makes_separate_link() {
        let (_d, pool, ws) = setup();
        let a = PaneLinkDao::create(&pool, &req(&ws)).unwrap();
        let mut r2 = req(&ws);
        r2.child_pane_id = "other-runner".to_string();
        let b = PaneLinkDao::create(&pool, &r2).unwrap();
        assert_ne!(a.id, b.id);
        let count: i64 = pool
            .get()
            .unwrap()
            .query_row("SELECT count(*) FROM pane_links", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn pane_link_error_serializes_with_machine_kind_k5() {
        // §K.5：UI-facing · machine 可读 kind + 人类可读 message。
        let json = serde_json::to_string(&PaneLinkError::CrossWorkspaceDenied).unwrap();
        assert!(
            json.contains("crossWorkspaceDenied"),
            "serialized error must carry machine kind, got {json}"
        );
        assert_eq!(
            PaneLinkError::InvalidParentPaneType.to_string(),
            "parent pane is not an AI pane"
        );
    }
}
