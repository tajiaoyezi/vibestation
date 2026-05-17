//! MVP-19 W1-A.1 · `ai_sessions` + `session_commit_links` DAO（over migrate_v9）。
//!
//! 本切片范围（spec §I.2.1 · §G）：
//! - `AiSessionDao` CRUD（insert / get_by_id / list_by_workspace / update / archive）
//! - `SessionCommitLinkDao` CRUD（insert / get / list_by_session / list_by_commit /
//!   update_state / unlink〔软删 §H5〕/ supersede〔§E5.5〕）
//! - 行 ↔ canonical 类型映射（enum TEXT ↔ Rust enum · INTEGER 0/1 ↔ bool ·
//!   REAL ↔ f32）
//!
//! **不在本切片**：`session:*` IPC command/event（W2）· session 边界识别引擎
//! （W1-B `session_lifecycle.rs`）· 前端。canonical 类型单一真相 =
//! `crate::sessions`（W1-A.0 #366）· 本模块只 import 消费 · 不重定义、不扩展。
//!
//! enum ↔ DB 串以 `crate::sessions` 的 serde `rename_all = "camelCase"` 为准
//! （`session_status_*` / `link_state_*` helper + `enum_db_mapping_matches_serde`
//! drift guard 测试锁定）· 避免前后端 / DB 三处平行定义漂移。

use crate::db::{DbError, DbPool};
use crate::sessions::{AiSession, LinkState, SessionCommitLink, SessionError, SessionStatus};

/// 统一把底层 DB 错误折叠进 canonical `SessionError::DbError`（不扩展 canonical ·
/// 不加 `impl From` 到 canonical 类型 · HC-2 "只 import"）。
fn db_err(e: impl std::string::ToString) -> SessionError {
    SessionError::DbError(e.to_string())
}

// ── enum ↔ DB 串映射（canonical serde camelCase 单一真相 · 见模块 doc）──────

fn session_status_to_db(s: &SessionStatus) -> &'static str {
    match s {
        SessionStatus::Active => "active",
        SessionStatus::Ended => "ended",
        SessionStatus::IdleCutoff => "idleCutoff",
        SessionStatus::Archived => "archived",
    }
}

fn session_status_from_db(s: &str) -> Option<SessionStatus> {
    match s {
        "active" => Some(SessionStatus::Active),
        "ended" => Some(SessionStatus::Ended),
        "idleCutoff" => Some(SessionStatus::IdleCutoff),
        "archived" => Some(SessionStatus::Archived),
        _ => None,
    }
}

fn link_state_to_db(s: &LinkState) -> &'static str {
    match s {
        LinkState::Pending => "pending",
        LinkState::ConfirmedAuto => "confirmedAuto",
        LinkState::ConfirmedManual => "confirmedManual",
        LinkState::Unlinked => "unlinked",
        LinkState::Superseded => "superseded",
        LinkState::Stale => "stale",
    }
}

fn link_state_from_db(s: &str) -> Option<LinkState> {
    match s {
        "pending" => Some(LinkState::Pending),
        "confirmedAuto" => Some(LinkState::ConfirmedAuto),
        "confirmedManual" => Some(LinkState::ConfirmedManual),
        "unlinked" => Some(LinkState::Unlinked),
        "superseded" => Some(LinkState::Superseded),
        "stale" => Some(LinkState::Stale),
        _ => None,
    }
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// `ai_sessions` 表 DAO（over migrate_v9 · 镜像 `PaneLinkDao` idiom）。
pub struct AiSessionDao;

impl AiSessionDao {
    /// §G.2 列顺序（与 `AiSession` 字段顺序一致 · `row_to_session` 按 index 映射）。
    const COLS: &'static str = "id, workspace_id, cli_kind, source, title, started_at, \
         ended_at, end_reason, prompt_count, token_count, event_count, status, \
         parser_version, strategy_version, metadata_json, created_at, updated_at";

    fn row_to_session(row: &rusqlite::Row<'_>) -> Result<AiSession, rusqlite::Error> {
        let status_str: String = row.get(11)?;
        let status = session_status_from_db(&status_str).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                11,
                rusqlite::types::Type::Text,
                format!("unknown ai_sessions.status: {status_str}").into(),
            )
        })?;
        Ok(AiSession {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            cli_kind: row.get(2)?,
            source: row.get(3)?,
            title: row.get(4)?,
            started_at: row.get(5)?,
            ended_at: row.get(6)?,
            end_reason: row.get(7)?,
            prompt_count: row.get(8)?,
            token_count: row.get(9)?,
            event_count: row.get(10)?,
            status,
            parser_version: row.get(12)?,
            strategy_version: row.get(13)?,
            metadata_json: row.get(14)?,
            created_at: row.get(15)?,
            updated_at: row.get(16)?,
        })
    }

    pub fn insert(pool: &DbPool, session: &AiSession) -> Result<(), SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        conn.execute(
            "INSERT INTO ai_sessions
                (id, workspace_id, cli_kind, source, title, started_at, ended_at,
                 end_reason, prompt_count, token_count, event_count, status,
                 parser_version, strategy_version, metadata_json, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
            rusqlite::params![
                session.id,
                session.workspace_id,
                session.cli_kind,
                session.source,
                session.title,
                session.started_at,
                session.ended_at,
                session.end_reason,
                session.prompt_count,
                session.token_count,
                session.event_count,
                session_status_to_db(&session.status),
                session.parser_version,
                session.strategy_version,
                session.metadata_json,
                session.created_at,
                session.updated_at,
            ],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// workspace-scoped（§C.2 不跨 workspace）· 不存在 → `SessionNotFound`。
    pub fn get_by_id(
        pool: &DbPool,
        workspace_id: &str,
        id: &str,
    ) -> Result<AiSession, SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let sql = format!(
            "SELECT {} FROM ai_sessions WHERE id = ?1 AND workspace_id = ?2",
            Self::COLS
        );
        conn.query_row(
            &sql,
            rusqlite::params![id, workspace_id],
            Self::row_to_session,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => SessionError::SessionNotFound(id.to_string()),
            other => db_err(other),
        })
    }

    /// §G.2 idx_ai_sessions_workspace_started · 最新 session 优先。
    pub fn list_by_workspace(
        pool: &DbPool,
        workspace_id: &str,
    ) -> Result<Vec<AiSession>, SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let sql = format!(
            "SELECT {} FROM ai_sessions WHERE workspace_id = ?1 \
             ORDER BY started_at DESC, rowid ASC",
            Self::COLS
        );
        let mut stmt = conn.prepare(&sql).map_err(db_err)?;
        let rows = stmt
            .query_map([workspace_id], Self::row_to_session)
            .map_err(db_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(db_err)?);
        }
        Ok(out)
    }

    /// 更新可变元数据（id / workspace_id / cli_kind / source / started_at /
    /// created_at 不可变）· 不存在 → `SessionNotFound`。
    pub fn update(pool: &DbPool, session: &AiSession) -> Result<(), SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let n = conn
            .execute(
                "UPDATE ai_sessions SET
                    title = ?1, ended_at = ?2, end_reason = ?3, prompt_count = ?4,
                    token_count = ?5, event_count = ?6, status = ?7,
                    parser_version = ?8, strategy_version = ?9, metadata_json = ?10,
                    updated_at = ?11
                 WHERE id = ?12 AND workspace_id = ?13",
                rusqlite::params![
                    session.title,
                    session.ended_at,
                    session.end_reason,
                    session.prompt_count,
                    session.token_count,
                    session.event_count,
                    session_status_to_db(&session.status),
                    session.parser_version,
                    session.strategy_version,
                    session.metadata_json,
                    session.updated_at,
                    session.id,
                    session.workspace_id,
                ],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(SessionError::SessionNotFound(session.id.clone()));
        }
        Ok(())
    }

    /// §G.5：只 archive · 不 silent hard delete。不存在 → `SessionNotFound`。
    pub fn archive(pool: &DbPool, workspace_id: &str, id: &str) -> Result<(), SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let n = conn
            .execute(
                "UPDATE ai_sessions SET status = 'archived', updated_at = ?1
                 WHERE id = ?2 AND workspace_id = ?3",
                rusqlite::params![now_ms(), id, workspace_id],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(SessionError::SessionNotFound(id.to_string()));
        }
        Ok(())
    }
}

/// `session_commit_links` 表 DAO（over migrate_v9）。
pub struct SessionCommitLinkDao;

impl SessionCommitLinkDao {
    /// §G.3 列顺序（与 `SessionCommitLink` 字段顺序一致）。
    const COLS: &'static str = "id, workspace_id, session_id, commit_sha, is_primary, \
         link_state, auto_bound, confidence, confidence_reason, strategy_version, \
         source_event_id, linked_at, unlinked_at, unlinked_reason, \
         superseded_by_link_id, created_by, reviewed_by, created_at, updated_at";

    fn row_to_link(row: &rusqlite::Row<'_>) -> Result<SessionCommitLink, rusqlite::Error> {
        let state_str: String = row.get(5)?;
        let link_state = link_state_from_db(&state_str).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                format!("unknown session_commit_links.link_state: {state_str}").into(),
            )
        })?;
        let is_primary_int: i64 = row.get(4)?;
        let auto_bound_int: i64 = row.get(6)?;
        let confidence_f64: f64 = row.get(7)?;
        Ok(SessionCommitLink {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            session_id: row.get(2)?,
            commit_sha: row.get(3)?,
            is_primary: is_primary_int != 0,
            link_state,
            auto_bound: auto_bound_int != 0,
            confidence: confidence_f64 as f32,
            confidence_reason: row.get(8)?,
            strategy_version: row.get(9)?,
            source_event_id: row.get(10)?,
            linked_at: row.get(11)?,
            unlinked_at: row.get(12)?,
            unlinked_reason: row.get(13)?,
            superseded_by_link_id: row.get(14)?,
            created_by: row.get(15)?,
            reviewed_by: row.get(16)?,
            created_at: row.get(17)?,
            updated_at: row.get(18)?,
        })
    }

    /// 插入新 link。命中 `ux_session_commit_primary`（§H8 同 commit 仅 1 主关联）→
    /// canonical 无专用 variant（W1-A.0 最小集 · HC-2 不扩展）→ 折叠为
    /// `DbError`，message 显式 "primary link conflict" 供上层识别。
    pub fn insert(pool: &DbPool, link: &SessionCommitLink) -> Result<(), SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        conn.execute(
            "INSERT INTO session_commit_links
                (id, workspace_id, session_id, commit_sha, is_primary, link_state,
                 auto_bound, confidence, confidence_reason, strategy_version,
                 source_event_id, linked_at, unlinked_at, unlinked_reason,
                 superseded_by_link_id, created_by, reviewed_by, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
            rusqlite::params![
                link.id,
                link.workspace_id,
                link.session_id,
                link.commit_sha,
                link.is_primary as i64,
                link_state_to_db(&link.link_state),
                link.auto_bound as i64,
                link.confidence as f64,
                link.confidence_reason,
                link.strategy_version,
                link.source_event_id,
                link.linked_at,
                link.unlinked_at,
                link.unlinked_reason,
                link.superseded_by_link_id,
                link.created_by,
                link.reviewed_by,
                link.created_at,
                link.updated_at,
            ],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(ref err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                SessionError::DbError(format!(
                    "primary link conflict: {} already has an active primary link (§H8): {e}",
                    link.commit_sha
                ))
            }
            other => db_err(other),
        })?;
        Ok(())
    }

    /// workspace-scoped · 不存在 → `LinkNotFound`。
    pub fn get(
        pool: &DbPool,
        workspace_id: &str,
        id: &str,
    ) -> Result<SessionCommitLink, SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let sql = format!(
            "SELECT {} FROM session_commit_links WHERE id = ?1 AND workspace_id = ?2",
            Self::COLS
        );
        conn.query_row(&sql, rusqlite::params![id, workspace_id], Self::row_to_link)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => SessionError::LinkNotFound(id.to_string()),
                other => db_err(other),
            })
    }

    /// §G.3 idx_session_commit_links_session · 最近绑定优先。
    pub fn list_by_session(
        pool: &DbPool,
        workspace_id: &str,
        session_id: &str,
    ) -> Result<Vec<SessionCommitLink>, SessionError> {
        Self::list_where(
            pool,
            "WHERE workspace_id = ?1 AND session_id = ?2 \
             ORDER BY linked_at DESC, rowid ASC",
            rusqlite::params![workspace_id, session_id],
        )
    }

    /// §G.3 idx_session_commit_links_commit · 反查 commit 的所有 link。
    pub fn list_by_commit(
        pool: &DbPool,
        workspace_id: &str,
        commit_sha: &str,
    ) -> Result<Vec<SessionCommitLink>, SessionError> {
        Self::list_where(
            pool,
            "WHERE workspace_id = ?1 AND commit_sha = ?2 \
             ORDER BY linked_at DESC, rowid ASC",
            rusqlite::params![workspace_id, commit_sha],
        )
    }

    fn list_where(
        pool: &DbPool,
        where_order: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> Result<Vec<SessionCommitLink>, SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let sql = format!(
            "SELECT {} FROM session_commit_links {}",
            Self::COLS,
            where_order
        );
        let mut stmt = conn.prepare(&sql).map_err(db_err)?;
        let rows = stmt.query_map(params, Self::row_to_link).map_err(db_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(db_err)?);
        }
        Ok(out)
    }

    /// pending → confirmed*/unlinked/superseded/stale 等状态流转 · bump updated_at。
    /// 不存在 → `LinkNotFound`。
    pub fn update_state(
        pool: &DbPool,
        workspace_id: &str,
        id: &str,
        new_state: &LinkState,
    ) -> Result<(), SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let n = conn
            .execute(
                "UPDATE session_commit_links SET link_state = ?1, updated_at = ?2
                 WHERE id = ?3 AND workspace_id = ?4",
                rusqlite::params![link_state_to_db(new_state), now_ms(), id, workspace_id],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(SessionError::LinkNotFound(id.to_string()));
        }
        Ok(())
    }

    /// §H5：unlink = 软删（set `unlinked_at` + `unlinked_reason` + `link_state =
    /// unlinked`）· **绝不**物理 DELETE（审计 / 回滚依赖历史）。幂等：首次软删
    /// 返回 `true`；本就已 unlink / 不存在返回 `false`。软删后该行被
    /// `ux_session_commit_primary` partial index 排除 → 同 commit 可再建主 link。
    pub fn unlink(
        pool: &DbPool,
        workspace_id: &str,
        id: &str,
        reason: Option<&str>,
    ) -> Result<bool, SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let now = now_ms();
        let reason = reason.unwrap_or("manual correction");
        let n = conn
            .execute(
                "UPDATE session_commit_links SET
                    unlinked_at = ?1, unlinked_reason = ?2,
                    link_state = 'unlinked', updated_at = ?1
                 WHERE id = ?3 AND workspace_id = ?4 AND unlinked_at IS NULL",
                rusqlite::params![now, reason, id, workspace_id],
            )
            .map_err(db_err)?;
        Ok(n > 0)
    }

    /// §E5.5：改绑后旧 link → `superseded` · 记录 `superseded_by_link_id` 指向新
    /// link（保留审计 · 不删）。不存在 → `LinkNotFound`。
    pub fn supersede(
        pool: &DbPool,
        workspace_id: &str,
        old_id: &str,
        new_link_id: &str,
    ) -> Result<(), SessionError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_err)?;
        let n = conn
            .execute(
                "UPDATE session_commit_links SET
                    link_state = 'superseded', superseded_by_link_id = ?1,
                    updated_at = ?2
                 WHERE id = ?3 AND workspace_id = ?4",
                rusqlite::params![new_link_id, now_ms(), old_id, workspace_id],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(SessionError::LinkNotFound(old_id.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    fn setup() -> (TempDir, DbPool) {
        let dir = TempDir::new().unwrap();
        let pool = db::open_pool(&dir.path().join("session_dao_test.db")).unwrap();
        (dir, pool)
    }

    fn sample_session(id: &str, ws: &str, started_at: i64) -> AiSession {
        AiSession {
            id: id.to_string(),
            workspace_id: ws.to_string(),
            cli_kind: "claude".to_string(),
            source: "auto".to_string(),
            title: "fix bug".to_string(),
            started_at,
            ended_at: None,
            end_reason: None,
            prompt_count: 2,
            token_count: Some(1500),
            event_count: 7,
            status: SessionStatus::Active,
            parser_version: Some("p1".to_string()),
            strategy_version: Some("v1".to_string()),
            metadata_json: "{}".to_string(),
            created_at: started_at,
            updated_at: started_at,
        }
    }

    fn sample_link(id: &str, ws: &str, session_id: &str, sha: &str) -> SessionCommitLink {
        SessionCommitLink {
            id: id.to_string(),
            workspace_id: ws.to_string(),
            session_id: session_id.to_string(),
            commit_sha: sha.to_string(),
            is_primary: true,
            link_state: LinkState::Pending,
            auto_bound: true,
            confidence: 0.42,
            confidence_reason: "time-window + source".to_string(),
            strategy_version: "v1".to_string(),
            source_event_id: Some("ev-1".to_string()),
            linked_at: 1000,
            unlinked_at: None,
            unlinked_reason: None,
            superseded_by_link_id: None,
            created_by: "system".to_string(),
            reviewed_by: None,
            created_at: 1000,
            updated_at: 1000,
        }
    }

    // ── enum ↔ DB drift guard（canonical serde 单一真相）─────────────

    #[test]
    fn enum_db_mapping_matches_serde() {
        // helper 串必须与 canonical serde `rename_all = "camelCase"` 完全一致 ·
        // 否则 DB 写入与前端 binding / IPC payload 三处漂移。
        for s in [
            SessionStatus::Active,
            SessionStatus::Ended,
            SessionStatus::IdleCutoff,
            SessionStatus::Archived,
        ] {
            let serde_str = serde_json::to_string(&s).unwrap();
            let serde_str = serde_str.trim_matches('"');
            assert_eq!(session_status_to_db(&s), serde_str);
            assert_eq!(session_status_from_db(serde_str), Some(s));
        }
        for st in [
            LinkState::Pending,
            LinkState::ConfirmedAuto,
            LinkState::ConfirmedManual,
            LinkState::Unlinked,
            LinkState::Superseded,
            LinkState::Stale,
        ] {
            let serde_str = serde_json::to_string(&st).unwrap();
            let serde_str = serde_str.trim_matches('"');
            assert_eq!(link_state_to_db(&st), serde_str);
            assert_eq!(link_state_from_db(serde_str), Some(st));
        }
        assert_eq!(session_status_from_db("bogus"), None);
        assert_eq!(link_state_from_db("bogus"), None);
    }

    // ── AiSessionDao ────────────────────────────────────────────────

    #[test]
    fn ai_session_insert_get_roundtrip_equal() {
        let (_d, pool) = setup();
        let s = sample_session("s1", "w1", 5000);
        AiSessionDao::insert(&pool, &s).unwrap();
        let got = AiSessionDao::get_by_id(&pool, "w1", "s1").unwrap();
        assert_eq!(
            got, s,
            "insert→get must roundtrip the canonical struct verbatim"
        );
    }

    #[test]
    fn ai_session_get_missing_is_session_not_found() {
        let (_d, pool) = setup();
        assert_eq!(
            AiSessionDao::get_by_id(&pool, "w1", "nope"),
            Err(SessionError::SessionNotFound("nope".to_string()))
        );
    }

    #[test]
    fn ai_session_get_is_workspace_scoped() {
        // §C.2：不跨 workspace · w2 看不到 w1 的 session。
        let (_d, pool) = setup();
        AiSessionDao::insert(&pool, &sample_session("s1", "w1", 1)).unwrap();
        assert_eq!(
            AiSessionDao::get_by_id(&pool, "w2", "s1"),
            Err(SessionError::SessionNotFound("s1".to_string()))
        );
    }

    #[test]
    fn ai_session_list_by_workspace_orders_by_started_desc() {
        let (_d, pool) = setup();
        AiSessionDao::insert(&pool, &sample_session("old", "w1", 100)).unwrap();
        AiSessionDao::insert(&pool, &sample_session("new", "w1", 300)).unwrap();
        AiSessionDao::insert(&pool, &sample_session("mid", "w1", 200)).unwrap();
        AiSessionDao::insert(&pool, &sample_session("other", "w2", 999)).unwrap();

        let listed = AiSessionDao::list_by_workspace(&pool, "w1").unwrap();
        let ids: Vec<&str> = listed.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["new", "mid", "old"], "started_at DESC");
        assert!(AiSessionDao::list_by_workspace(&pool, "w-empty")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn ai_session_update_mutates_metadata_and_missing_is_not_found() {
        let (_d, pool) = setup();
        let mut s = sample_session("s1", "w1", 10);
        AiSessionDao::insert(&pool, &s).unwrap();

        s.status = SessionStatus::IdleCutoff;
        s.ended_at = Some(20);
        s.end_reason = Some("idle_cutoff".to_string());
        s.prompt_count = 9;
        s.event_count = 30;
        s.updated_at = 25;
        AiSessionDao::update(&pool, &s).unwrap();

        let got = AiSessionDao::get_by_id(&pool, "w1", "s1").unwrap();
        assert_eq!(got.status, SessionStatus::IdleCutoff);
        assert_eq!(got.ended_at, Some(20));
        assert_eq!(got.end_reason.as_deref(), Some("idle_cutoff"));
        assert_eq!(got.prompt_count, 9);
        assert_eq!(got.event_count, 30);
        assert_eq!(got.updated_at, 25);

        let ghost = sample_session("ghost", "w1", 1);
        assert_eq!(
            AiSessionDao::update(&pool, &ghost),
            Err(SessionError::SessionNotFound("ghost".to_string()))
        );
    }

    #[test]
    fn ai_session_archive_sets_status_and_missing_is_not_found() {
        let (_d, pool) = setup();
        AiSessionDao::insert(&pool, &sample_session("s1", "w1", 1)).unwrap();
        AiSessionDao::archive(&pool, "w1", "s1").unwrap();
        assert_eq!(
            AiSessionDao::get_by_id(&pool, "w1", "s1").unwrap().status,
            SessionStatus::Archived
        );
        assert_eq!(
            AiSessionDao::archive(&pool, "w1", "no-such"),
            Err(SessionError::SessionNotFound("no-such".to_string()))
        );
    }

    // ── SessionCommitLinkDao ────────────────────────────────────────

    fn seed_session(pool: &DbPool, id: &str, ws: &str) {
        AiSessionDao::insert(pool, &sample_session(id, ws, 1)).unwrap();
    }

    #[test]
    fn link_insert_get_roundtrip_equal() {
        let (_d, pool) = setup();
        seed_session(&pool, "s1", "w1");
        let l = sample_link("l1", "w1", "s1", "sha-abc");
        SessionCommitLinkDao::insert(&pool, &l).unwrap();
        let got = SessionCommitLinkDao::get(&pool, "w1", "l1").unwrap();
        assert_eq!(got, l, "link insert→get must roundtrip incl. bool/enum/f32");
    }

    #[test]
    fn link_fk_requires_existing_session() {
        // §G.3 FK：session_id 必须存在于 ai_sessions（PRAGMA foreign_keys=ON）。
        let (_d, pool) = setup();
        let l = sample_link("l1", "w1", "missing-session", "sha");
        let err = SessionCommitLinkDao::insert(&pool, &l).unwrap_err();
        assert!(
            matches!(err, SessionError::DbError(_)),
            "FK violation → DbError, got {err:?}"
        );
    }

    #[test]
    fn link_get_missing_is_link_not_found() {
        let (_d, pool) = setup();
        assert_eq!(
            SessionCommitLinkDao::get(&pool, "w1", "nope"),
            Err(SessionError::LinkNotFound("nope".to_string()))
        );
    }

    #[test]
    fn link_unique_active_primary_per_commit_h8() {
        // §H8：同 (workspace, commit) 至多 1 条 active is_primary link。
        let (_d, pool) = setup();
        seed_session(&pool, "s1", "w1");
        seed_session(&pool, "s2", "w1");
        SessionCommitLinkDao::insert(&pool, &sample_link("l1", "w1", "s1", "sha-x")).unwrap();
        let dup = sample_link("l2", "w1", "s2", "sha-x");
        let err = SessionCommitLinkDao::insert(&pool, &dup).unwrap_err();
        match err {
            SessionError::DbError(msg) => {
                assert!(msg.contains("primary link conflict"), "got {msg}")
            }
            other => panic!("expected primary link conflict DbError, got {other:?}"),
        }

        // 软删 l1 → partial index 排除 → 同 commit 可再建主 link
        assert!(SessionCommitLinkDao::unlink(&pool, "w1", "l1", None).unwrap());
        SessionCommitLinkDao::insert(&pool, &sample_link("l3", "w1", "s2", "sha-x"))
            .expect("after unlink the previous primary, a new primary must insert (§H8 partial)");
    }

    #[test]
    fn link_non_primary_duplicates_allowed() {
        // is_primary=0 不受 ux_session_commit_primary 约束 · 多候选可共存。
        let (_d, pool) = setup();
        seed_session(&pool, "s1", "w1");
        seed_session(&pool, "s2", "w1");
        let mut a = sample_link("l1", "w1", "s1", "sha-y");
        a.is_primary = false;
        let mut b = sample_link("l2", "w1", "s2", "sha-y");
        b.is_primary = false;
        SessionCommitLinkDao::insert(&pool, &a).unwrap();
        SessionCommitLinkDao::insert(&pool, &b)
            .expect("non-primary candidate links for same commit must coexist");
    }

    #[test]
    fn link_list_by_session_and_commit_order_linked_at_desc() {
        let (_d, pool) = setup();
        seed_session(&pool, "s1", "w1");
        let mut early = sample_link("l-early", "w1", "s1", "sha-1");
        early.is_primary = false;
        early.linked_at = 100;
        let mut late = sample_link("l-late", "w1", "s1", "sha-2");
        late.is_primary = false;
        late.linked_at = 300;
        SessionCommitLinkDao::insert(&pool, &early).unwrap();
        SessionCommitLinkDao::insert(&pool, &late).unwrap();

        let by_session = SessionCommitLinkDao::list_by_session(&pool, "w1", "s1").unwrap();
        let ids: Vec<&str> = by_session.iter().map(|l| l.id.as_str()).collect();
        assert_eq!(ids, vec!["l-late", "l-early"], "linked_at DESC");

        let by_commit = SessionCommitLinkDao::list_by_commit(&pool, "w1", "sha-1").unwrap();
        assert_eq!(by_commit.len(), 1);
        assert_eq!(by_commit[0].id, "l-early");
        assert!(
            SessionCommitLinkDao::list_by_commit(&pool, "w2", "sha-1")
                .unwrap()
                .is_empty(),
            "list is workspace-scoped"
        );
    }

    #[test]
    fn link_update_state_transitions_and_missing_is_not_found() {
        let (_d, pool) = setup();
        seed_session(&pool, "s1", "w1");
        SessionCommitLinkDao::insert(&pool, &sample_link("l1", "w1", "s1", "sha")).unwrap();
        SessionCommitLinkDao::update_state(&pool, "w1", "l1", &LinkState::ConfirmedManual).unwrap();
        assert_eq!(
            SessionCommitLinkDao::get(&pool, "w1", "l1")
                .unwrap()
                .link_state,
            LinkState::ConfirmedManual
        );
        assert_eq!(
            SessionCommitLinkDao::update_state(&pool, "w1", "ghost", &LinkState::Stale),
            Err(SessionError::LinkNotFound("ghost".to_string()))
        );
    }

    #[test]
    fn link_unlink_is_soft_idempotent_and_preserves_row_h5() {
        let (_d, pool) = setup();
        seed_session(&pool, "s1", "w1");
        SessionCommitLinkDao::insert(&pool, &sample_link("l1", "w1", "s1", "sha")).unwrap();

        let first = SessionCommitLinkDao::unlink(&pool, "w1", "l1", Some("wrong guess")).unwrap();
        assert!(first, "首次软删返回 true");

        let got = SessionCommitLinkDao::get(&pool, "w1", "l1").unwrap();
        assert_eq!(got.link_state, LinkState::Unlinked);
        assert!(
            got.unlinked_at.is_some(),
            "软删 set unlinked_at（保留物理行）"
        );
        assert_eq!(got.unlinked_reason.as_deref(), Some("wrong guess"));

        // 物理行仍在（§H5 审计 · 非 hard delete）
        let physical: i64 = pool
            .get()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM session_commit_links WHERE id='l1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(physical, 1, "soft unlink must keep the physical row");

        // 幂等：再 unlink 返回 false
        assert!(!SessionCommitLinkDao::unlink(&pool, "w1", "l1", None).unwrap());
        // 不存在也 false（不报错 · 幂等）
        assert!(!SessionCommitLinkDao::unlink(&pool, "w1", "no-such", None).unwrap());
    }

    #[test]
    fn link_unlink_defaults_reason_to_manual_correction() {
        let (_d, pool) = setup();
        seed_session(&pool, "s1", "w1");
        SessionCommitLinkDao::insert(&pool, &sample_link("l1", "w1", "s1", "sha")).unwrap();
        SessionCommitLinkDao::unlink(&pool, "w1", "l1", None).unwrap();
        assert_eq!(
            SessionCommitLinkDao::get(&pool, "w1", "l1")
                .unwrap()
                .unlinked_reason
                .as_deref(),
            Some("manual correction")
        );
    }

    #[test]
    fn link_supersede_marks_old_and_points_to_new_e5_5() {
        let (_d, pool) = setup();
        seed_session(&pool, "s1", "w1");
        seed_session(&pool, "s2", "w1");
        let mut old = sample_link("l-old", "w1", "s1", "sha");
        old.is_primary = false;
        SessionCommitLinkDao::insert(&pool, &old).unwrap();
        let mut new = sample_link("l-new", "w1", "s2", "sha");
        new.is_primary = false;
        SessionCommitLinkDao::insert(&pool, &new).unwrap();

        SessionCommitLinkDao::supersede(&pool, "w1", "l-old", "l-new").unwrap();
        let got = SessionCommitLinkDao::get(&pool, "w1", "l-old").unwrap();
        assert_eq!(got.link_state, LinkState::Superseded);
        assert_eq!(got.superseded_by_link_id.as_deref(), Some("l-new"));

        assert_eq!(
            SessionCommitLinkDao::supersede(&pool, "w1", "ghost", "l-new"),
            Err(SessionError::LinkNotFound("ghost".to_string()))
        );
    }
}
