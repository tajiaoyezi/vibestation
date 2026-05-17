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

use crate::sessions::{LinkState, SessionCommitLink};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 进入 revert plan 的最低自动置信度（spec §B.2）。低于此值的 `pending`
/// link 不自动包含（用户可在 UI 手动勾选）。
pub const MIN_AUTO_CONFIDENCE: f32 = 0.9;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
