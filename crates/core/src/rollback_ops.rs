//! MVP-20 Phase A · AI 一键回滚（session 级 revert）后端包装。
//!
//! 本模块独立于 `rebase_ops`（spec §I Phase A 起点 checklist：「新建独立模块 ·
//! 不和 rebase_ops 混」）。M1 里程碑聚焦 **revert plan 构建纯逻辑**
//! （spec §B.2 link_state/置信度过滤 + §J R4 newest-first 排序）；
//! M2 接 git2 revert 序列 / abort / crash recovery / 4 IPC / 8 ts-rs binding。
//!
//! 关键不变量（spec §B.2 + §J R4 + §E.2.3）：
//! - 仅 `confirmed_auto` / `confirmed_manual`，或 `pending && confidence ≥ 0.9
//!   && auto_bound` 的 link 进入 revert plan；`unlinked` / `superseded` /
//!   `stale` 永不包含。
//! - 低置信度（< 0.9）的 `pending` link **默认不 include**（用户可手动勾选），
//!   但仍出现在 plan 里并标记 `low_confidence` 供 UI 高亮（spec §C.Do.3）。
//! - revert 顺序 **newest-first**（commit timestamp 降序）：先 revert 最新
//!   commit 可最大程度减少不必要冲突（spec §J R4）。
//! - 本模块**永不**产生 `git reset --hard` 或任何不可逆 reset（spec §E.2.3 ·
//!   CLAUDE.md 禁区）—— M1 纯 plan 逻辑天然无此风险，M2 git2 调用链同样只用
//!   `Repository::revert`。

use crate::db::{DbError, DbPool};
use crate::session_dao::SessionCommitLinkDao;
use crate::sessions::{LinkState, SessionCommitLink};
use crate::workspace::WorkspaceStore;
use git2::{Oid, Repository, Status, StatusOptions};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;

/// 进入 revert plan 的最低自动置信度（spec §B.2）。低于此值的 `pending`
/// link 不自动包含（用户可在 UI 手动勾选）。
pub const MIN_AUTO_CONFIDENCE: f32 = 0.9;

const ROLLBACK_STATUS_IDLE: &str = "idle";
const ROLLBACK_STATUS_IN_PROGRESS: &str = "in_progress";
const ROLLBACK_STATUS_CONFLICT_PAUSED: &str = "conflict_paused";
const ROLLBACK_STATUS_COMPLETED: &str = "completed";
const ROLLBACK_STATUS_ABORTED: &str = "aborted";

/// `rollback:*` 操作的统一错误（spec §G.3 `RollbackError` · §I 起点 A7：
/// 仿 MVP-09 `CommitError` / MVP-16 `RebaseOpError` 结构化变体）。
///
/// M1 只构建 plan，实际触发的是 `EmptyPlan` / `SessionNotFound`；其余变体
/// 是 M2（revert 序列 / IPC）的契约前置声明（spec A7 要求 enum 完整 ·
/// enum 全列变体是契约定义，非投机代码）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RollbackError {
    /// working tree 有未提交改动（spec §E.2.4 · M2 `check_preconditions`）。
    DirtyWorkingTree {
        modified: Vec<String>,
        staged: Vec<String>,
    },
    /// 某 commit revert 产生冲突，转交 MVP-16（spec §E.4 · M2）。
    ConflictDetected {
        commit_sha: String,
        files: Vec<String>,
    },
    /// session 不存在或无可 revert 的 commit。
    SessionNotFound { session_id: String },
    /// 过滤后没有任何 commit 满足 revert 条件（全 unlinked/superseded/低置信）。
    EmptyPlan { session_id: String },
    /// 已有进行中的 rollback（spec §H.9 crash recovery · M2）。
    InProgress { session_id: String },
    /// 底层 git2 错误（spec §K · M2 revert 序列）。
    Git2Error {
        class: String,
        code: i32,
        message: String,
    },
    /// 底层 DB 错误（DAO 查询失败）。
    DbError { message: String },
}

impl std::fmt::Display for RollbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RollbackError {}

/// 单个候选 commit 进入 plan 构建的输入（M1 纯逻辑边界）。
///
/// M2 由 `SessionCommitLinkDao::list_by_session` + git2 commit 查询组装：
/// `link` 提供 link_state/confidence/auto_bound，`commit_timestamp` 来自
/// git2 commit object（spec §J R4 排序依据是 **commit time** 而非 link 时间）。
#[derive(Debug, Clone)]
pub struct RevertCandidate {
    pub link: SessionCommitLink,
    /// git2 commit author/committer time（Unix epoch 秒 · newest-first 排序键）。
    pub commit_timestamp: i64,
}

/// revert plan 里的单条目（spec §K `RollbackCommitEntry` 的 plan 阶段子集 ·
/// message/author/files_changed 等展示字段 M2 preview IPC 时 git2 enrich）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RevertPlanEntry {
    pub sha: String,
    pub confidence: f32,
    /// 默认 include = 高置信（≥ 0.9）或已确认绑定；低置信 pending 默认 false。
    pub include: bool,
    /// confidence < 0.9 → UI 高亮警告（spec §C.Do.3 · §D.2 ⚠ 标记）。
    pub low_confidence: bool,
    /// commit timestamp（epoch 秒 · 跨 IPC 用 number 防 bigint · 既有约定）。
    #[ts(type = "number")]
    pub commit_timestamp: i64,
}

/// `build_revert_plan` 输出（spec §B.2 过滤 + §J R4 排序后的最终计划）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RevertPlan {
    pub session_id: String,
    /// newest-first（commit_timestamp 降序）· 已过滤 unlinked/superseded/stale。
    pub entries: Vec<RevertPlanEntry>,
    /// 任一 entry low_confidence → true（UI 顶部 ⚠ 警告 · spec §D.2）。
    pub has_low_confidence: bool,
}

/// `rollback:preview` 单 commit 展示项（spec §K）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RollbackCommitEntry {
    pub sha: String,
    pub message: String,
    pub author: String,
    #[ts(type = "number")]
    pub timestamp: i64,
    pub confidence: f32,
    pub include: bool,
    #[ts(type = "number")]
    pub files_changed: i64,
}

/// `rollback:preview` 返回类型（spec §K）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RollbackPreview {
    pub session_id: String,
    pub commits: Vec<RollbackCommitEntry>,
    #[ts(type = "number")]
    pub total_files_changed: i64,
    #[ts(type = "number")]
    pub total_insertions: i64,
    #[ts(type = "number")]
    pub total_deletions: i64,
    pub has_low_confidence: bool,
}

/// `rollback:execute` progress payload（spec §G.3 / §G.4）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RollbackProgress {
    #[ts(type = "number")]
    pub done: i64,
    #[ts(type = "number")]
    pub total: i64,
    pub current_sha: Option<String>,
    pub status: String,
}

/// `rollback:abort` 返回类型（spec §G.3）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RollbackAbortResult {
    pub success: bool,
    pub head_sha: String,
    pub error: Option<String>,
}

/// `rollback:status` 返回类型（spec §K）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RollbackStatus {
    pub session_id: String,
    pub status: String,
    #[ts(type = "number")]
    pub current_idx: i64,
    #[ts(type = "number")]
    pub total: i64,
    pub current_sha: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RevertSequenceResult {
    pub progress: RollbackProgress,
    pub revert_shas: Vec<String>,
    pub head_sha: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackPlanRecordEntry {
    pub sha: String,
    pub include: bool,
    pub confidence: f32,
    pub status: String,
    pub revert_sha: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackOpRecord {
    pub id: i64,
    pub session_id: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub status: String,
    pub commit_plan: String,
    pub current_idx: i64,
    pub error_msg: Option<String>,
}

impl RollbackOpRecord {
    pub fn plan_entries(&self) -> Result<Vec<RollbackPlanRecordEntry>, RollbackError> {
        serde_json::from_str(&self.commit_plan).map_err(db_error)
    }
}

/// 是否允许该 link 进入 revert plan（spec §B.2 核心过滤规则）。
///
/// 允许：`confirmed_auto` / `confirmed_manual`（人工或高可信自动确认）·
/// 或 `pending` 且 `confidence ≥ 0.9` 且 `auto_bound`（高可信自动候选）。
/// 排除：`unlinked` / `superseded` / `stale`（spec §B.2 明确排除）·
/// 低置信 pending（出现在 plan 但 include=false）。
fn is_eligible(link: &SessionCommitLink) -> bool {
    match link.link_state {
        LinkState::ConfirmedAuto | LinkState::ConfirmedManual => true,
        LinkState::Pending => link.auto_bound, // 低置信 pending 仍入 plan（include=false），故只排除非 auto_bound
        LinkState::Unlinked | LinkState::Superseded | LinkState::Stale => false,
    }
}

/// 该 link 是否默认勾选 include（spec §H.2 · §C.Do.3）。
///
/// confirmed_* → 始终 include；pending → 仅当 confidence ≥ 0.9 且 auto_bound。
fn default_include(link: &SessionCommitLink) -> bool {
    match link.link_state {
        LinkState::ConfirmedAuto | LinkState::ConfirmedManual => true,
        LinkState::Pending => link.auto_bound && link.confidence >= MIN_AUTO_CONFIDENCE,
        LinkState::Unlinked | LinkState::Superseded | LinkState::Stale => false,
    }
}

/// 构建 session 的 revert plan（spec §B.2 过滤 + §J R4 newest-first 排序）。
///
/// 纯函数（无 git2 / DB 副作用）：输入候选列表，输出过滤排序后的 plan。
/// M2 由 IPC handler 用 `SessionCommitLinkDao` + git2 组装 `RevertCandidate`
/// 列表喂入。空 plan（无任何 eligible commit）返回 `EmptyPlan` 错误，避免
/// 前端展示空预览 modal（spec §D.2 预设至少 1 commit）。
pub fn build_revert_plan(
    session_id: &str,
    candidates: &[RevertCandidate],
) -> Result<RevertPlan, RollbackError> {
    let mut entries: Vec<RevertPlanEntry> = candidates
        .iter()
        .filter(|c| is_eligible(&c.link))
        .map(|c| RevertPlanEntry {
            sha: c.link.commit_sha.clone(),
            confidence: c.link.confidence,
            include: default_include(&c.link),
            low_confidence: c.link.confidence < MIN_AUTO_CONFIDENCE,
            commit_timestamp: c.commit_timestamp,
        })
        .collect();

    if entries.is_empty() {
        return Err(RollbackError::EmptyPlan {
            session_id: session_id.to_string(),
        });
    }

    // spec §J R4: newest-first（commit_timestamp 降序）· tie-break sha 保证确定性
    entries.sort_by(|a, b| {
        b.commit_timestamp
            .cmp(&a.commit_timestamp)
            .then_with(|| a.sha.cmp(&b.sha))
    });

    let has_low_confidence = entries.iter().any(|e| e.low_confidence);

    Ok(RevertPlan {
        session_id: session_id.to_string(),
        entries,
        has_low_confidence,
    })
}

pub struct RollbackOpDao;

impl RollbackOpDao {
    pub fn insert_in_progress(
        pool: &DbPool,
        session_id: &str,
        plan: &[RollbackPlanRecordEntry],
    ) -> Result<i64, RollbackError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_error)?;
        let commit_plan = serde_json::to_string(plan).map_err(db_error)?;
        conn.execute(
            "INSERT INTO rollback_ops
                (session_id, started_at, status, commit_plan, current_idx)
             VALUES (?1, ?2, 'in_progress', ?3, 0)",
            rusqlite::params![session_id, now_ms(), commit_plan],
        )
        .map_err(db_error)?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_in_progress(
        pool: &DbPool,
        session_id: &str,
    ) -> Result<Option<RollbackOpRecord>, RollbackError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_error)?;
        conn.query_row(
            "SELECT id, session_id, started_at, finished_at, status, commit_plan, current_idx, error_msg
             FROM rollback_ops
             WHERE session_id = ?1 AND status IN ('in_progress', 'conflict_paused')
             ORDER BY started_at DESC, id DESC
             LIMIT 1",
            [session_id],
            row_to_record,
        )
        .optional()
        .map_err(db_error)
    }

    pub fn latest_for_session(
        pool: &DbPool,
        session_id: &str,
    ) -> Result<Option<RollbackOpRecord>, RollbackError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_error)?;
        conn.query_row(
            "SELECT id, session_id, started_at, finished_at, status, commit_plan, current_idx, error_msg
             FROM rollback_ops
             WHERE session_id = ?1
             ORDER BY started_at DESC, id DESC
             LIMIT 1",
            [session_id],
            row_to_record,
        )
        .optional()
        .map_err(db_error)
    }

    pub fn update_progress(
        pool: &DbPool,
        session_id: &str,
        current_idx: i64,
        plan: &[RollbackPlanRecordEntry],
    ) -> Result<(), RollbackError> {
        Self::update_active(
            pool,
            session_id,
            ROLLBACK_STATUS_IN_PROGRESS,
            current_idx,
            plan,
            None,
        )
    }

    pub fn mark_conflict(
        pool: &DbPool,
        session_id: &str,
        current_idx: i64,
        plan: &[RollbackPlanRecordEntry],
        error_msg: &str,
    ) -> Result<(), RollbackError> {
        Self::update_active(
            pool,
            session_id,
            ROLLBACK_STATUS_CONFLICT_PAUSED,
            current_idx,
            plan,
            Some(error_msg),
        )
    }

    pub fn mark_completed(
        pool: &DbPool,
        session_id: &str,
        plan: &[RollbackPlanRecordEntry],
    ) -> Result<(), RollbackError> {
        Self::finish_active(pool, session_id, ROLLBACK_STATUS_COMPLETED, plan, None)
    }

    pub fn mark_aborted(
        pool: &DbPool,
        session_id: &str,
        plan: &[RollbackPlanRecordEntry],
        error_msg: Option<&str>,
    ) -> Result<(), RollbackError> {
        Self::finish_active(pool, session_id, ROLLBACK_STATUS_ABORTED, plan, error_msg)
    }

    fn update_active(
        pool: &DbPool,
        session_id: &str,
        status: &str,
        current_idx: i64,
        plan: &[RollbackPlanRecordEntry],
        error_msg: Option<&str>,
    ) -> Result<(), RollbackError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_error)?;
        let commit_plan = serde_json::to_string(plan).map_err(db_error)?;
        conn.execute(
            "UPDATE rollback_ops
             SET status = ?1, current_idx = ?2, commit_plan = ?3, error_msg = ?4
             WHERE id = (
                 SELECT id FROM rollback_ops
                 WHERE session_id = ?5 AND status IN ('in_progress', 'conflict_paused')
                 ORDER BY started_at DESC, id DESC
                 LIMIT 1
             )",
            rusqlite::params![status, current_idx, commit_plan, error_msg, session_id],
        )
        .map_err(db_error)?;
        Ok(())
    }

    fn finish_active(
        pool: &DbPool,
        session_id: &str,
        status: &str,
        plan: &[RollbackPlanRecordEntry],
        error_msg: Option<&str>,
    ) -> Result<(), RollbackError> {
        let conn = pool.get().map_err(DbError::from).map_err(db_error)?;
        let commit_plan = serde_json::to_string(plan).map_err(db_error)?;
        conn.execute(
            "UPDATE rollback_ops
             SET status = ?1, finished_at = ?2, commit_plan = ?3, error_msg = ?4
             WHERE id = (
                 SELECT id FROM rollback_ops
                 WHERE session_id = ?5 AND status IN ('in_progress', 'conflict_paused')
                 ORDER BY started_at DESC, id DESC
                 LIMIT 1
             )",
            rusqlite::params![status, now_ms(), commit_plan, error_msg, session_id],
        )
        .map_err(db_error)?;
        Ok(())
    }
}

fn row_to_record(row: &rusqlite::Row<'_>) -> Result<RollbackOpRecord, rusqlite::Error> {
    Ok(RollbackOpRecord {
        id: row.get(0)?,
        session_id: row.get(1)?,
        started_at: row.get(2)?,
        finished_at: row.get(3)?,
        status: row.get(4)?,
        commit_plan: row.get(5)?,
        current_idx: row.get(6)?,
        error_msg: row.get(7)?,
    })
}

pub fn rollback_preview(pool: &DbPool, session_id: &str) -> Result<RollbackPreview, RollbackError> {
    let (workspace_id, repo_path) = session_workspace_and_repo(pool, session_id)?;
    let repo = Repository::open(&repo_path).map_err(map_git_error)?;
    let links =
        SessionCommitLinkDao::list_by_session(pool, &workspace_id, session_id).map_err(|e| {
            RollbackError::DbError {
                message: e.to_string(),
            }
        })?;

    let mut candidates = Vec::new();
    for link in links {
        let commit = find_commit(&repo, &link.commit_sha)?;
        candidates.push(RevertCandidate {
            link,
            commit_timestamp: commit.time().seconds(),
        });
    }
    let plan = build_revert_plan(session_id, &candidates)?;
    let mut commits = Vec::new();
    let mut total_files_changed = 0;
    let mut total_insertions = 0;
    let mut total_deletions = 0;

    for entry in &plan.entries {
        let commit = find_commit(&repo, &entry.sha)?;
        let summary = commit_summary(&commit);
        let author = commit.author().name().unwrap_or("").to_string();
        let (files_changed, insertions, deletions) = diff_stats(&repo, &commit)?;
        total_files_changed += files_changed;
        total_insertions += insertions;
        total_deletions += deletions;
        commits.push(RollbackCommitEntry {
            sha: entry.sha.clone(),
            message: summary,
            author,
            timestamp: commit.time().seconds(),
            confidence: entry.confidence,
            include: entry.include,
            files_changed,
        });
    }

    Ok(RollbackPreview {
        session_id: session_id.to_string(),
        commits,
        total_files_changed,
        total_insertions,
        total_deletions,
        has_low_confidence: plan.has_low_confidence,
    })
}

pub fn rollback_execute(
    pool: &DbPool,
    session_id: &str,
    include_shas: Vec<String>,
) -> Result<RollbackProgress, RollbackError> {
    Ok(rollback_execute_with_progress(pool, session_id, include_shas, |_| {})?.progress)
}

pub fn rollback_execute_with_progress<F>(
    pool: &DbPool,
    session_id: &str,
    include_shas: Vec<String>,
    mut on_progress: F,
) -> Result<RevertSequenceResult, RollbackError>
where
    F: FnMut(RollbackProgress),
{
    let (_workspace_id, repo_path) = session_workspace_and_repo(pool, session_id)?;
    let repo = Repository::open(&repo_path).map_err(map_git_error)?;
    if RollbackOpDao::get_in_progress(pool, session_id)?.is_some() {
        return Err(RollbackError::InProgress {
            session_id: session_id.to_string(),
        });
    }
    check_preconditions(&repo)?;

    let preview = rollback_preview(pool, session_id)?;
    let mut plan_entries: Vec<RollbackPlanRecordEntry> = preview
        .commits
        .iter()
        .filter(|commit| include_shas.iter().any(|sha| sha == &commit.sha))
        .map(|commit| RollbackPlanRecordEntry {
            sha: commit.sha.clone(),
            include: true,
            confidence: commit.confidence,
            status: "pending".to_string(),
            revert_sha: None,
        })
        .collect();

    if plan_entries.is_empty() {
        return Err(RollbackError::EmptyPlan {
            session_id: session_id.to_string(),
        });
    }

    RollbackOpDao::insert_in_progress(pool, session_id, &plan_entries)?;
    let total = plan_entries.len() as i64;
    let mut revert_shas = Vec::new();

    for idx in 0..plan_entries.len() {
        let sha = plan_entries[idx].sha.clone();
        match revert_commit(&repo, &sha, session_id) {
            Ok(revert_sha) => {
                plan_entries[idx].status = "reverted".to_string();
                plan_entries[idx].revert_sha = Some(revert_sha.clone());
                revert_shas.push(revert_sha);
                RollbackOpDao::update_progress(pool, session_id, idx as i64 + 1, &plan_entries)?;
                on_progress(RollbackProgress {
                    done: idx as i64 + 1,
                    total,
                    current_sha: Some(sha),
                    status: ROLLBACK_STATUS_IN_PROGRESS.to_string(),
                });
            }
            Err(err @ RollbackError::ConflictDetected { .. }) => {
                plan_entries[idx].status = "conflict".to_string();
                RollbackOpDao::mark_conflict(
                    pool,
                    session_id,
                    idx as i64,
                    &plan_entries,
                    &err.to_string(),
                )?;
                return Err(err);
            }
            Err(err) => return Err(err),
        }
    }

    RollbackOpDao::mark_completed(pool, session_id, &plan_entries)?;
    mark_session_rolled_back(pool, session_id, &revert_shas)?;

    Ok(RevertSequenceResult {
        progress: RollbackProgress {
            done: total,
            total,
            current_sha: None,
            status: ROLLBACK_STATUS_COMPLETED.to_string(),
        },
        revert_shas,
        head_sha: head_sha(&repo)?,
    })
}

pub fn rollback_abort(
    pool: &DbPool,
    session_id: &str,
) -> Result<RollbackAbortResult, RollbackError> {
    let (_workspace_id, repo_path) = session_workspace_and_repo(pool, session_id)?;
    let repo = Repository::open(&repo_path).map_err(map_git_error)?;
    let Some(record) = RollbackOpDao::get_in_progress(pool, session_id)? else {
        return Ok(RollbackAbortResult {
            success: false,
            head_sha: head_sha(&repo).unwrap_or_default(),
            error: Some("no rollback in progress".to_string()),
        });
    };
    let plan = record.plan_entries()?;
    let completed: Vec<String> = plan
        .iter()
        .filter_map(|entry| entry.revert_sha.clone())
        .collect();
    let result = abort_revert_completed(&repo, session_id, &completed)?;
    RollbackOpDao::mark_aborted(pool, session_id, &plan, result.error.as_deref())?;
    Ok(result)
}

pub fn rollback_status(pool: &DbPool, session_id: &str) -> Result<RollbackStatus, RollbackError> {
    let (_workspace_id, repo_path) = session_workspace_and_repo(pool, session_id)?;
    let repo = Repository::open(&repo_path).map_err(map_git_error)?;
    if let Some(status) = detect_in_progress(&repo, pool, session_id)? {
        return Ok(status);
    }
    let Some(record) = RollbackOpDao::latest_for_session(pool, session_id)? else {
        return Ok(RollbackStatus {
            session_id: session_id.to_string(),
            status: ROLLBACK_STATUS_IDLE.to_string(),
            current_idx: 0,
            total: 0,
            current_sha: None,
        });
    };
    status_from_record(&record)
}

pub fn detect_in_progress(
    repo: &Repository,
    pool: &DbPool,
    session_id: &str,
) -> Result<Option<RollbackStatus>, RollbackError> {
    if !repo.path().join("REVERT_HEAD").exists() {
        return Ok(None);
    }
    let Some(record) = RollbackOpDao::get_in_progress(pool, session_id)? else {
        return Ok(None);
    };
    status_from_record(&record).map(Some)
}

pub fn check_preconditions(repo: &Repository) -> Result<(), RollbackError> {
    let dirty = dirty_working_tree(repo)?;
    if dirty.modified.is_empty() && dirty.staged.is_empty() {
        Ok(())
    } else {
        Err(RollbackError::DirtyWorkingTree {
            modified: dirty.modified,
            staged: dirty.staged,
        })
    }
}

pub fn build_revert_message(original_message: &str, session_id: &str) -> String {
    let summary = original_message.lines().next().unwrap_or("").trim();
    format!("Revert \"{summary}\" [AI session rollback: {session_id}]")
}

pub fn revert_sequence(
    repo: &Repository,
    plan: &RevertPlan,
    session_id: &str,
) -> Result<RevertSequenceResult, RollbackError> {
    check_preconditions(repo)?;
    let selected: Vec<&RevertPlanEntry> =
        plan.entries.iter().filter(|entry| entry.include).collect();
    if selected.is_empty() {
        return Err(RollbackError::EmptyPlan {
            session_id: session_id.to_string(),
        });
    }
    let total = selected.len() as i64;
    let mut revert_shas = Vec::new();
    for entry in selected {
        let revert_sha = revert_commit(repo, &entry.sha, session_id)?;
        revert_shas.push(revert_sha);
    }
    Ok(RevertSequenceResult {
        progress: RollbackProgress {
            done: total,
            total,
            current_sha: None,
            status: ROLLBACK_STATUS_COMPLETED.to_string(),
        },
        revert_shas,
        head_sha: head_sha(repo)?,
    })
}

pub fn revert_commit(
    repo: &Repository,
    commit_sha: &str,
    session_id: &str,
) -> Result<String, RollbackError> {
    let commit = find_commit(repo, commit_sha)?;
    let message = build_revert_message(&commit_summary(&commit), session_id);
    let head_before = repo
        .head()
        .ok()
        .and_then(|head| head.peel_to_commit().ok())
        .map(|c| c.id());

    repo.revert(&commit, None).map_err(map_git_error)?;
    let index = repo.index().map_err(map_git_error)?;
    if index.has_conflicts() {
        return Err(RollbackError::ConflictDetected {
            commit_sha: commit_sha.to_string(),
            files: conflict_files(repo)?,
        });
    }
    drop(index);
    let oid = commit_current_index(repo, &message)?;
    repo.cleanup_state().map_err(map_git_error)?;
    if head_before == Some(oid) {
        return Err(RollbackError::Git2Error {
            class: "Revert".to_string(),
            code: -1,
            message: "revert did not advance HEAD".to_string(),
        });
    }
    Ok(oid.to_string())
}

pub fn abort_revert(
    repo: &Repository,
    session_id: &str,
) -> Result<RollbackAbortResult, RollbackError> {
    abort_revert_completed(repo, session_id, &[])
}

pub fn abort_revert_completed(
    repo: &Repository,
    session_id: &str,
    completed_revert_shas: &[String],
) -> Result<RollbackAbortResult, RollbackError> {
    repo.cleanup_state().map_err(map_git_error)?;
    for revert_sha in completed_revert_shas.iter().rev() {
        revert_commit(repo, revert_sha, session_id)?;
    }
    Ok(RollbackAbortResult {
        success: true,
        head_sha: head_sha(repo)?,
        error: None,
    })
}

fn status_from_record(record: &RollbackOpRecord) -> Result<RollbackStatus, RollbackError> {
    let plan = record.plan_entries()?;
    let current_sha = plan
        .get(record.current_idx as usize)
        .map(|entry| entry.sha.clone());
    Ok(RollbackStatus {
        session_id: record.session_id.clone(),
        status: record.status.clone(),
        current_idx: record.current_idx,
        total: plan.len() as i64,
        current_sha,
    })
}

fn mark_session_rolled_back(
    pool: &DbPool,
    session_id: &str,
    revert_shas: &[String],
) -> Result<(), RollbackError> {
    let conn = pool.get().map_err(DbError::from).map_err(db_error)?;
    let shas_json = serde_json::to_string(revert_shas).map_err(db_error)?;
    conn.execute(
        "UPDATE ai_sessions
         SET rolled_back_at = ?1, rollback_commit_shas = ?2, rollback_session_id = ?3
         WHERE id = ?3",
        rusqlite::params![now_ms(), shas_json, session_id],
    )
    .map_err(db_error)?;
    Ok(())
}

fn session_workspace_and_repo(
    pool: &DbPool,
    session_id: &str,
) -> Result<(String, PathBuf), RollbackError> {
    let conn = pool.get().map_err(DbError::from).map_err(db_error)?;
    let workspace_id: String = conn
        .query_row(
            "SELECT workspace_id FROM ai_sessions WHERE id = ?1",
            [session_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => RollbackError::SessionNotFound {
                session_id: session_id.to_string(),
            },
            other => db_error(other),
        })?;
    let workspace = WorkspaceStore::get_by_id(pool, &workspace_id).map_err(db_error)?;
    let repo_path = workspace
        .repo_root
        .clone()
        .unwrap_or_else(|| workspace.path.clone());
    Ok((workspace_id, PathBuf::from(repo_path)))
}

fn find_commit<'repo>(
    repo: &'repo Repository,
    sha: &str,
) -> Result<git2::Commit<'repo>, RollbackError> {
    let oid = Oid::from_str(sha).map_err(map_git_error)?;
    repo.find_commit(oid).map_err(map_git_error)
}

fn commit_summary(commit: &git2::Commit<'_>) -> String {
    commit
        .summary()
        .map(ToOwned::to_owned)
        .or_else(|| {
            commit
                .message()
                .map(|msg| msg.lines().next().unwrap_or("").to_string())
        })
        .unwrap_or_default()
}

fn diff_stats(
    repo: &Repository,
    commit: &git2::Commit<'_>,
) -> Result<(i64, i64, i64), RollbackError> {
    let tree = commit.tree().map_err(map_git_error)?;
    let parent_tree = if commit.parent_count() == 0 {
        None
    } else {
        Some(
            commit
                .parent(0)
                .map_err(map_git_error)?
                .tree()
                .map_err(map_git_error)?,
        )
    };
    let diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)
        .map_err(map_git_error)?;
    let stats = diff.stats().map_err(map_git_error)?;
    Ok((
        stats.files_changed() as i64,
        stats.insertions() as i64,
        stats.deletions() as i64,
    ))
}

fn commit_current_index(repo: &Repository, message: &str) -> Result<Oid, RollbackError> {
    let mut index = repo.index().map_err(map_git_error)?;
    let tree_oid = index.write_tree().map_err(map_git_error)?;
    let tree = repo.find_tree(tree_oid).map_err(map_git_error)?;
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let sig = repo
        .signature()
        .or_else(|_| git2::Signature::now("Codex CLI", "noreply@openai.com"))
        .map_err(map_git_error)?;
    let parents = parent.iter().collect::<Vec<_>>();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .map_err(map_git_error)
}

#[derive(Default)]
struct DirtyState {
    modified: Vec<String>,
    staged: Vec<String>,
}

fn dirty_working_tree(repo: &Repository) -> Result<DirtyState, RollbackError> {
    let mut options = StatusOptions::new();
    options.include_untracked(true).renames_head_to_index(true);
    let statuses = repo.statuses(Some(&mut options)).map_err(map_git_error)?;
    let mut dirty = DirtyState::default();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();
        if status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            dirty.staged.push(path.clone());
        }
        if status.intersects(
            Status::WT_NEW
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_TYPECHANGE,
        ) {
            dirty.modified.push(path);
        }
    }
    Ok(dirty)
}

fn conflict_files(repo: &Repository) -> Result<Vec<String>, RollbackError> {
    let mut out = Vec::new();
    let index = repo.index().map_err(map_git_error)?;
    if let Ok(conflicts) = index.conflicts() {
        for conflict in conflicts {
            let conflict = conflict.map_err(map_git_error)?;
            for entry in [conflict.ancestor, conflict.our, conflict.their]
                .into_iter()
                .flatten()
            {
                let path = String::from_utf8_lossy(&entry.path).to_string();
                if !out.contains(&path) {
                    out.push(path);
                }
            }
        }
    }
    Ok(out)
}

fn head_sha(repo: &Repository) -> Result<String, RollbackError> {
    Ok(repo
        .head()
        .map_err(map_git_error)?
        .peel_to_commit()
        .map_err(map_git_error)?
        .id()
        .to_string())
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn map_git_error(error: git2::Error) -> RollbackError {
    RollbackError::Git2Error {
        class: format!("{:?}", error.class()),
        code: error.code() as i32,
        message: error.message().to_string(),
    }
}

fn db_error(error: impl std::string::ToString) -> RollbackError {
    RollbackError::DbError {
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::sessions::{AiSession, SessionStatus};
    use git2::{Oid, Repository, Signature};
    use std::{fs, path::PathBuf};
    use tempfile::TempDir;

    /// 构造测试用 SessionCommitLink（只设过滤相关字段 · 其余默认）。
    fn link(sha: &str, state: LinkState, confidence: f32, auto_bound: bool) -> SessionCommitLink {
        SessionCommitLink {
            id: format!("link-{sha}"),
            workspace_id: "ws1".into(),
            session_id: "sess1".into(),
            commit_sha: sha.into(),
            is_primary: true,
            link_state: state,
            auto_bound,
            confidence,
            confidence_reason: String::new(),
            strategy_version: "v1".into(),
            source_event_id: None,
            linked_at: 0,
            unlinked_at: None,
            unlinked_reason: None,
            superseded_by_link_id: None,
            created_by: "system".into(),
            reviewed_by: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn cand(sha: &str, state: LinkState, conf: f32, auto: bool, ts: i64) -> RevertCandidate {
        RevertCandidate {
            link: link(sha, state, conf, auto),
            commit_timestamp: ts,
        }
    }

    #[test]
    fn confirmed_links_always_included() {
        let cands = vec![
            cand("a", LinkState::ConfirmedAuto, 0.5, true, 100),
            cand("b", LinkState::ConfirmedManual, 0.1, false, 200),
        ];
        let plan = build_revert_plan("sess1", &cands).unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert!(plan.entries.iter().all(|e| e.include));
    }

    #[test]
    fn pending_high_confidence_auto_bound_included() {
        let cands = vec![cand("a", LinkState::Pending, 0.95, true, 100)];
        let plan = build_revert_plan("sess1", &cands).unwrap();
        assert_eq!(plan.entries.len(), 1);
        assert!(plan.entries[0].include);
        assert!(!plan.entries[0].low_confidence);
    }

    #[test]
    fn pending_low_confidence_present_but_not_included() {
        // spec §C.Do.3 + §H.2: 低置信 pending 出现在 plan（供 UI 高亮）但默认不勾选
        let cands = vec![cand("a", LinkState::Pending, 0.72, true, 100)];
        let plan = build_revert_plan("sess1", &cands).unwrap();
        assert_eq!(plan.entries.len(), 1);
        assert!(!plan.entries[0].include);
        assert!(plan.entries[0].low_confidence);
        assert!(plan.has_low_confidence);
    }

    #[test]
    fn pending_non_auto_bound_excluded() {
        let cands = vec![cand("a", LinkState::Pending, 0.99, false, 100)];
        let err = build_revert_plan("sess1", &cands).unwrap_err();
        assert_eq!(
            err,
            RollbackError::EmptyPlan {
                session_id: "sess1".into()
            }
        );
    }

    #[test]
    fn unlinked_superseded_stale_always_excluded() {
        // spec §B.2: 这三态永不进 plan（即使高置信）
        let cands = vec![
            cand("a", LinkState::Unlinked, 0.99, true, 100),
            cand("b", LinkState::Superseded, 0.99, true, 200),
            cand("c", LinkState::Stale, 0.99, true, 300),
        ];
        let err = build_revert_plan("sess1", &cands).unwrap_err();
        assert_eq!(
            err,
            RollbackError::EmptyPlan {
                session_id: "sess1".into()
            }
        );
    }

    #[test]
    fn plan_ordered_newest_first() {
        // spec §J R4: commit_timestamp 降序（先 revert 最新 commit）
        let cands = vec![
            cand("old", LinkState::ConfirmedAuto, 0.9, true, 100),
            cand("new", LinkState::ConfirmedAuto, 0.9, true, 300),
            cand("mid", LinkState::ConfirmedAuto, 0.9, true, 200),
        ];
        let plan = build_revert_plan("sess1", &cands).unwrap();
        let shas: Vec<&str> = plan.entries.iter().map(|e| e.sha.as_str()).collect();
        assert_eq!(shas, vec!["new", "mid", "old"]);
    }

    #[test]
    fn same_timestamp_tie_break_by_sha_deterministic() {
        let cands = vec![
            cand("zzz", LinkState::ConfirmedAuto, 0.9, true, 100),
            cand("aaa", LinkState::ConfirmedAuto, 0.9, true, 100),
        ];
        let plan = build_revert_plan("sess1", &cands).unwrap();
        let shas: Vec<&str> = plan.entries.iter().map(|e| e.sha.as_str()).collect();
        assert_eq!(shas, vec!["aaa", "zzz"]); // 确定性 tie-break
    }

    #[test]
    fn mixed_states_filter_and_order_combined() {
        let cands = vec![
            cand("keep_new", LinkState::ConfirmedManual, 0.3, false, 500),
            cand("drop_unlinked", LinkState::Unlinked, 0.99, true, 600),
            cand("keep_pending_hi", LinkState::Pending, 0.92, true, 400),
            cand("keep_low", LinkState::Pending, 0.5, true, 700),
            cand("drop_superseded", LinkState::Superseded, 0.99, true, 800),
        ];
        let plan = build_revert_plan("sess1", &cands).unwrap();
        let shas: Vec<&str> = plan.entries.iter().map(|e| e.sha.as_str()).collect();
        // 过滤掉 unlinked/superseded · 剩 3 个按 ts 降序：keep_low(700) keep_new(500) keep_pending_hi(400)
        assert_eq!(shas, vec!["keep_low", "keep_new", "keep_pending_hi"]);
        assert!(plan.has_low_confidence); // keep_low 0.5 < 0.9
                                          // keep_low include=false（低置信），keep_new/keep_pending_hi include=true
        let low = plan.entries.iter().find(|e| e.sha == "keep_low").unwrap();
        assert!(!low.include && low.low_confidence);
    }

    // §E.2.3「严禁 reset --hard」的验证**不在此处做 source-introspection 自检**：
    // 单文件模块里测试与实现同文件，`include_str!` 必然把测试自身/注释/panic
    // message 对禁区符号的引用一起读入，contains 校验结构性自我误伤（RED + GREEN
    // 两阶段已实证 false-fail · CLAUDE.md 自审四问#1 递归完备性命中）。
    // spec §E.2.3 原文即「代码层面用 cargo grep 验证」—— 属 PR-level 外部检查，
    // 非单元测试。M1 是纯 plan 逻辑、零 git2 调用，§E.2.3 风险为零；真实防护点
    // 在 M2（revert_sequence 接 git2 时）· 届时 PR check：
    //   grep -nE 'ResetType::Hard|\.reset\(' crates/core/src/rollback_ops.rs \
    //     （排除 #[cfg(test)] 与注释后）必须为空。

    struct GitFixture {
        _dir: TempDir,
        path: PathBuf,
        repo: Repository,
        base_tree: Oid,
    }

    impl GitFixture {
        fn new() -> Self {
            let dir = TempDir::new().unwrap();
            let repo = Repository::init(dir.path()).unwrap();
            let mut fixture = Self {
                _dir: dir,
                path: repo.workdir().unwrap().to_path_buf(),
                repo,
                base_tree: Oid::zero(),
            };
            let base = fixture.commit_file("app.txt", "base\n", "base");
            fixture.base_tree = fixture.repo.find_commit(base).unwrap().tree_id();
            fixture
        }

        fn commit_file(&self, path: &str, content: &str, message: &str) -> Oid {
            let full_path = self.path.join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&full_path, content).unwrap();
            let mut index = self.repo.index().unwrap();
            index.add_path(std::path::Path::new(path)).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = self.repo.find_tree(tree_oid).unwrap();
            let sig = Signature::now("Codex CLI", "noreply@openai.com").unwrap();
            let parent = self
                .repo
                .head()
                .ok()
                .and_then(|head| head.peel_to_commit().ok());
            let parents = parent.iter().collect::<Vec<_>>();
            self.repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
                .unwrap()
        }

        fn head(&self) -> Oid {
            self.repo.head().unwrap().peel_to_commit().unwrap().id()
        }

        fn head_tree(&self) -> Oid {
            self.repo
                .head()
                .unwrap()
                .peel_to_commit()
                .unwrap()
                .tree_id()
        }
    }

    fn plan_from_shas(session_id: &str, shas_newest_first: &[String]) -> RevertPlan {
        RevertPlan {
            session_id: session_id.to_string(),
            entries: shas_newest_first
                .iter()
                .enumerate()
                .map(|(idx, sha)| RevertPlanEntry {
                    sha: sha.clone(),
                    confidence: 1.0,
                    include: true,
                    low_confidence: false,
                    commit_timestamp: 1000 - idx as i64,
                })
                .collect(),
            has_low_confidence: false,
        }
    }

    #[test]
    fn test_message_suffix_format() {
        assert_eq!(
            build_revert_message("fix: 修复 stage 路径异常\n\nbody", "session-123"),
            "Revert \"fix: 修复 stage 路径异常\" [AI session rollback: session-123]"
        );
    }

    #[test]
    fn test_dirty_working_tree_guard() {
        let fixture = GitFixture::new();
        fs::write(fixture.path.join("app.txt"), "dirty\n").unwrap();

        let err = check_preconditions(&fixture.repo).unwrap_err();
        match err {
            RollbackError::DirtyWorkingTree { modified, staged } => {
                assert!(modified.contains(&"app.txt".to_string()));
                assert!(staged.is_empty());
            }
            other => panic!("expected dirty working tree, got {other:?}"),
        }
    }

    #[test]
    fn test_revert_single_commit() {
        let fixture = GitFixture::new();
        let target = fixture
            .commit_file("app.txt", "base\none\n", "feat: one")
            .to_string();

        let reverted = revert_commit(&fixture.repo, &target, "session-1").unwrap();
        let commit = fixture
            .repo
            .find_commit(Oid::from_str(&reverted).unwrap())
            .unwrap();

        assert!(commit
            .message()
            .unwrap()
            .contains("[AI session rollback: session-1]"));
        assert_eq!(fixture.head_tree(), fixture.base_tree);
    }

    #[test]
    fn test_revert_sequence_5_commits() {
        let fixture = GitFixture::new();
        let mut shas = Vec::new();
        for idx in 1..=5 {
            shas.push(
                fixture
                    .commit_file(
                        &format!("file-{idx}.txt"),
                        &format!("content {idx}\n"),
                        &format!("feat: {idx}"),
                    )
                    .to_string(),
            );
        }
        shas.reverse();
        let plan = plan_from_shas("session-5", &shas);

        let result = revert_sequence(&fixture.repo, &plan, "session-5").unwrap();

        assert_eq!(result.revert_shas.len(), 5);
        assert_eq!(fixture.head_tree(), fixture.base_tree);
        let head = fixture.repo.head().unwrap().peel_to_commit().unwrap();
        assert!(head
            .message()
            .unwrap()
            .contains("[AI session rollback: session-5]"));
    }

    #[test]
    fn test_revert_conflict_detection() {
        let fixture = GitFixture::new();
        let older = fixture.commit_file("app.txt", "base\nolder\n", "feat: older");
        let before = fixture.commit_file("app.txt", "base\nnewer\n", "feat: newer");

        let err = revert_commit(&fixture.repo, &older.to_string(), "session-conflict").unwrap_err();

        match err {
            RollbackError::ConflictDetected { commit_sha, files } => {
                assert_eq!(commit_sha, older.to_string());
                assert!(files.contains(&"app.txt".to_string()));
            }
            other => panic!("expected conflict, got {other:?}"),
        }
        assert_eq!(fixture.head(), before);
    }

    #[test]
    fn test_abort_cleanup() {
        let fixture = GitFixture::new();
        let target = fixture
            .commit_file("app.txt", "base\nabort\n", "feat: abort")
            .to_string();
        let reverted = revert_commit(&fixture.repo, &target, "session-abort").unwrap();

        let result = abort_revert_completed(&fixture.repo, "session-abort", &[reverted]).unwrap();

        assert!(result.success);
        assert!(!fixture.repo.path().join("REVERT_HEAD").exists());
        assert_eq!(
            fs::read_to_string(fixture.path.join("app.txt")).unwrap(),
            "base\nabort\n"
        );
    }

    fn active_session(id: &str, workspace_id: &str) -> AiSession {
        AiSession {
            id: id.to_string(),
            workspace_id: workspace_id.to_string(),
            cli_kind: "codex".to_string(),
            source: "auto".to_string(),
            title: format!("session {id}"),
            started_at: 1,
            ended_at: None,
            end_reason: None,
            prompt_count: 0,
            token_count: None,
            event_count: 1,
            status: SessionStatus::Active,
            parser_version: None,
            strategy_version: Some("v1".to_string()),
            metadata_json: "{}".to_string(),
            created_at: 1,
            updated_at: 1,
        }
    }

    #[test]
    fn test_rollback_state_persisted() {
        let dir = TempDir::new().unwrap();
        let pool = db::open_pool(&dir.path().join("rollback-state.db")).unwrap();
        crate::AiSessionDao::insert(&pool, &active_session("session-state", "ws1")).unwrap();
        let plan = vec![RollbackPlanRecordEntry {
            sha: "abc123".to_string(),
            include: true,
            confidence: 1.0,
            status: "pending".to_string(),
            revert_sha: None,
        }];

        let id = RollbackOpDao::insert_in_progress(&pool, "session-state", &plan).unwrap();
        let record = RollbackOpDao::get_in_progress(&pool, "session-state")
            .unwrap()
            .expect("in-progress rollback record");

        assert_eq!(record.id, id);
        assert_eq!(record.status, "in_progress");
        assert_eq!(record.current_idx, 0);
        assert_eq!(record.plan_entries().unwrap()[0].sha, "abc123");
    }

    #[cfg(feature = "integration")]
    #[test]
    fn integration_full_session_revert_5commits() {
        let fixture = GitFixture::new();
        let dir = TempDir::new().unwrap();
        let pool = db::open_pool(&dir.path().join("rollback-full.db")).unwrap();
        insert_workspace_and_session(&pool, "ws-full", "session-full", &fixture.path);
        let mut shas = Vec::new();
        for idx in 1..=5 {
            let sha = fixture
                .commit_file(
                    &format!("full-{idx}.txt"),
                    &format!("full {idx}\n"),
                    &format!("feat: full {idx}"),
                )
                .to_string();
            insert_link(&pool, "ws-full", "session-full", &sha, 1.0);
            shas.push(sha);
        }

        let progress = rollback_execute(&pool, "session-full", shas.clone()).unwrap();

        assert_eq!(progress.done, 5);
        assert_eq!(progress.total, 5);
        assert_eq!(fixture.head_tree(), fixture.base_tree);
        assert_eq!(
            RollbackOpDao::latest_for_session(&pool, "session-full")
                .unwrap()
                .unwrap()
                .status,
            "completed"
        );
    }

    #[cfg(feature = "integration")]
    #[test]
    fn integration_abort_mid_revert() {
        let fixture = GitFixture::new();
        let dir = TempDir::new().unwrap();
        let pool = db::open_pool(&dir.path().join("rollback-abort.db")).unwrap();
        insert_workspace_and_session(&pool, "ws-abort", "session-abort-int", &fixture.path);
        let sha = fixture
            .commit_file("abort-int.txt", "abort\n", "feat: abort integration")
            .to_string();
        insert_link(&pool, "ws-abort", "session-abort-int", &sha, 1.0);
        let plan = vec![RollbackPlanRecordEntry {
            sha: sha.clone(),
            include: true,
            confidence: 1.0,
            status: "pending".to_string(),
            revert_sha: None,
        }];
        RollbackOpDao::insert_in_progress(&pool, "session-abort-int", &plan).unwrap();
        let reverted = revert_commit(&fixture.repo, &sha, "session-abort-int").unwrap();
        let mut updated = plan;
        updated[0].status = "reverted".to_string();
        updated[0].revert_sha = Some(reverted);
        RollbackOpDao::update_progress(&pool, "session-abort-int", 1, &updated).unwrap();

        let result = rollback_abort(&pool, "session-abort-int").unwrap();

        assert!(result.success);
        assert_eq!(
            RollbackOpDao::latest_for_session(&pool, "session-abort-int")
                .unwrap()
                .unwrap()
                .status,
            "aborted"
        );
        assert_eq!(
            fs::read_to_string(fixture.path.join("abort-int.txt")).unwrap(),
            "abort\n"
        );
    }

    #[cfg(feature = "integration")]
    #[test]
    fn integration_conflict_resume() {
        let fixture = GitFixture::new();
        let dir = TempDir::new().unwrap();
        let pool = db::open_pool(&dir.path().join("rollback-conflict.db")).unwrap();
        insert_workspace_and_session(&pool, "ws-conflict", "session-conflict-int", &fixture.path);
        let older = fixture
            .commit_file("app.txt", "base\nolder\n", "feat: conflict older")
            .to_string();
        fixture.commit_file("app.txt", "base\nnewer\n", "feat: conflict newer");
        insert_link(&pool, "ws-conflict", "session-conflict-int", &older, 1.0);

        let err = rollback_execute(&pool, "session-conflict-int", vec![older.clone()]).unwrap_err();

        assert!(matches!(err, RollbackError::ConflictDetected { .. }));
        assert_eq!(
            RollbackOpDao::latest_for_session(&pool, "session-conflict-int")
                .unwrap()
                .unwrap()
                .status,
            "conflict_paused"
        );
    }

    #[cfg(feature = "integration")]
    #[test]
    fn integration_crash_recovery() {
        let fixture = GitFixture::new();
        let dir = TempDir::new().unwrap();
        let pool = db::open_pool(&dir.path().join("rollback-crash.db")).unwrap();
        insert_workspace_and_session(&pool, "ws-crash", "session-crash", &fixture.path);
        let plan = vec![RollbackPlanRecordEntry {
            sha: "abc123".to_string(),
            include: true,
            confidence: 1.0,
            status: "pending".to_string(),
            revert_sha: None,
        }];
        RollbackOpDao::insert_in_progress(&pool, "session-crash", &plan).unwrap();
        fs::write(fixture.repo.path().join("REVERT_HEAD"), "abc123\n").unwrap();

        let status = detect_in_progress(&fixture.repo, &pool, "session-crash").unwrap();

        assert!(status.is_some());
        assert_eq!(status.unwrap().status, "in_progress");
    }

    #[cfg(feature = "integration")]
    fn insert_workspace_and_session(
        pool: &db::DbPool,
        workspace_id: &str,
        session_id: &str,
        repo_path: &std::path::Path,
    ) {
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO workspaces
                (workspace_id, name, path, has_git, repo_root, created_at, last_opened)
             VALUES (?1, 'WS', ?2, 1, ?2, 1, 1)",
            rusqlite::params![workspace_id, repo_path.to_string_lossy().to_string()],
        )
        .unwrap();
        drop(conn);
        crate::AiSessionDao::insert(pool, &active_session(session_id, workspace_id)).unwrap();
    }

    #[cfg(feature = "integration")]
    fn insert_link(
        pool: &db::DbPool,
        workspace_id: &str,
        session_id: &str,
        sha: &str,
        confidence: f32,
    ) {
        crate::SessionCommitLinkDao::insert(
            pool,
            &SessionCommitLink {
                id: format!("link-{sha}"),
                workspace_id: workspace_id.to_string(),
                session_id: session_id.to_string(),
                commit_sha: sha.to_string(),
                is_primary: true,
                link_state: LinkState::ConfirmedAuto,
                auto_bound: true,
                confidence,
                confidence_reason: "test".to_string(),
                strategy_version: "v1".to_string(),
                source_event_id: None,
                linked_at: 1,
                unlinked_at: None,
                unlinked_reason: None,
                superseded_by_link_id: None,
                created_by: "system".to_string(),
                reviewed_by: None,
                created_at: 1,
                updated_at: 1,
            },
        )
        .unwrap();
    }
}
