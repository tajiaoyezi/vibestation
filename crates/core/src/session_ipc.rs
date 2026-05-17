//! MVP-19 W2-A.0 · session↔commit 绑定的 IPC 契约类型（§K.1 命令 / §K.2 事件 /
//! §K.4 payload）。
//!
//! 角色 = Wave 1 的 `sessions.rs`（W1-A.0 #366）之于 DAO/引擎：**本模块是
//! Phase B IPC wire 形状的单一真相**。W2-B（app 层命令 handler · `crates/app`）
//! 与 W2-C（前端 session API client · `web/src`）并行消费本契约 · 不平行重定义。
//!
//! 约定（与 canonical `crate::sessions` 一致 · 防 H2 类 camelCase drift）：
//! - 所有跨 IPC 的 i64（epoch `*_at` / `*_count` / `*_size` …）一律
//!   `#[ts(type = "number")]` —— serde 把 i64 序列化成 JSON number ·
//!   ts-rs 默认 `bigint` 是运行时谎言 · 对齐 project 既有全 `*Count`/`*_at`
//!   number 约定（见 `crate::sessions::AiSession`）。
//! - `#[serde(rename_all = "camelCase")]` 全员 · wire 形状 = 前端 binding 形状。
//! - 复合字段复用 canonical `AiSession` / `SessionCommitLink` / `SessionError`
//!   （只引用 · 不重定义 · 不扩展 canonical）。
//!
//! 不在本切片：命令 handler / ACL / 事件 emit（W2-B `crates/app`）· session
//! 关联评分算法（W2-B `session_service`）· 前端 store（W2-C `web/src`）。

use crate::sessions::{AiSession, SessionCommitLink, SessionError};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// §K.4 `SessionBindCommitRequest.mode` · 自动推断 vs 用户手动指派。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum SessionBindMode {
    /// 系统依时间窗口 + 来源上下文自动推断候选（低置信落 pending · §C.1.3/§H4）。
    Auto,
    /// 用户在 UI 显式把 commit 指派给某 session（§C.1.5 人工路径）。
    Manual,
}

// ── §K.1 命令 request / result ──────────────────────────────────────────────

/// `session:start` · 显式或自动开启 session（§C.1.1 boundary）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartRequest {
    pub workspace_id: String,
    pub cli_kind: String,
    /// `auto`（进程驱动）| `manual`（用户驱动）· §G.2 `source`。
    pub source: String,
    /// 触发 session 的 pane（边界识别上下文 · 可空 = 非 pane 驱动）。
    pub pane_id: Option<String>,
    pub title: Option<String>,
    /// 起始时刻 epoch millis（前端不传时 handler 取 now · §K.4 风格）。
    #[ts(type = "number | null")]
    pub started_at: Option<i64>,
}

/// `session:start` 响应 · 返回新建（或复用 active）的 session 行。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartResult {
    pub session: AiSession,
    /// true = 命中已有 active session（幂等 · 未新建）· false = 新建。
    pub already_active: bool,
}

/// `session:end` · 结束 session，写 `end_reason`（§G.2 status 流转）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionEndRequest {
    pub workspace_id: String,
    pub session_id: String,
    /// `manual_end` | `idle_cutoff` | `process_exit` | `clear` …（§C.1.1）。
    pub end_reason: Option<String>,
}

/// `session:end` 响应 · 返回结束后的 session 行。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionEndResult {
    pub session: AiSession,
}

/// §K.4 `SessionBindCommitRequest` verbatim · 自动/手动绑定 commit。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionBindCommitRequest {
    pub workspace_id: String,
    pub commit_sha: String,
    /// `None` = 让后端按策略推断候选 session；`Some` = 显式指派（manual）。
    pub session_id: Option<String>,
    pub mode: SessionBindMode,
    pub reason: Option<String>,
}

/// `session:bind-commit` 响应 · 返回新建的 link（含置信度 / pending 状态）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionBindCommitResult {
    pub link: SessionCommitLink,
    /// §H4：低置信自动绑定落 `pending`（true = 待人工确认 · 不当 confirmed）。
    pub requires_confirmation: bool,
}

/// `session:unbind` · 软解绑 + 审计（§H5 不硬删）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionUnbindRequest {
    pub workspace_id: String,
    pub link_id: String,
    pub reason: Option<String>,
}

/// `session:unbind` 响应 · 幂等（首次软删 true · 已删/不存在 false）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionUnbindResult {
    pub link_id: String,
    pub unlinked: bool,
}

/// `session:list` · 按 workspace 列 session（§D.1 列表视图）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionListRequest {
    pub workspace_id: String,
}

/// `session:list` 响应 · `started_at DESC`（最新优先 · §G.2 索引）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionListResult {
    pub sessions: Vec<AiSession>,
}

/// `session:get-detail` · 详情页数据（§D.1 Session 详情视图）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetailRequest {
    pub workspace_id: String,
    pub session_id: String,
}

/// `session:get-detail` 响应 · session 主体 + 关联 commit links + 基础统计
/// （§D.1 Summary strip：commit 数 / 置信均值）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetailResult {
    pub session: AiSession,
    pub links: Vec<SessionCommitLink>,
    /// 关联 commit 数（= 未软删 links 计数 · §D.1 Summary）。
    #[ts(type = "number")]
    pub commit_count: i64,
    /// 置信均值（无 link 时 0.0 · §D.1 Summary "绑定置信均值"）。
    pub avg_confidence: f32,
}

/// `session:rebind` · 改绑 commit 到同 workspace 另一 session（§E5.4/§E5.5）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionRebindRequest {
    pub workspace_id: String,
    pub link_id: String,
    pub target_session_id: String,
    pub reason: Option<String>,
}

/// `session:rebind` 响应 · 旧 link → superseded · 返回新建 link。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionRebindResult {
    pub superseded_link_id: String,
    pub new_link: SessionCommitLink,
}

/// `session:recalculate-candidates` · 重算某 commit 的候选关联（§D.3
/// "Unbind and recalc" / §C.1.14 最小回填）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecalcRequest {
    pub workspace_id: String,
    pub commit_sha: String,
}

/// `session:recalculate-candidates` 响应 · 返回该 commit 当前候选 link 集
/// （含 pending · 供 UI 让用户挑选确认）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecalcResult {
    pub candidates: Vec<SessionCommitLink>,
}

// ── §K.2 事件 payload ───────────────────────────────────────────────────────

/// `session:started` · session 启动通知（前端刷新列表 / 徽章 pending 点）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartedEvent {
    pub workspace_id: String,
    pub session: AiSession,
}

/// `session:ended` · session 结束通知（含 end_reason · §D.1 状态切换）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionEndedEvent {
    pub workspace_id: String,
    pub session_id: String,
    pub end_reason: Option<String>,
}

/// `session:commit-bound` · commit 完成绑定（Git Log 徽章异步刷新 · §D.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionCommitBoundEvent {
    pub workspace_id: String,
    pub link: SessionCommitLink,
}

/// `session:commit-unbound` · commit 完成解绑（徽章变 stale/移除 · §D.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionCommitUnboundEvent {
    pub workspace_id: String,
    pub link_id: String,
    pub commit_sha: String,
}

/// `session:link-updated` · pending → confirmed 等状态变更（徽章弱化样式
/// 切换 · §D.2 置信度样式）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionLinkUpdatedEvent {
    pub workspace_id: String,
    pub link: SessionCommitLink,
}

/// `session:error` · 可恢复错误通知前端（§K.2 · 复用 canonical
/// `SessionError` tagged enum · 不另造错误形状）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionErrorEvent {
    pub workspace_id: String,
    pub error: SessionError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::{LinkState, SessionStatus};

    fn sample_session() -> AiSession {
        AiSession {
            id: "s1".into(),
            workspace_id: "w1".into(),
            cli_kind: "claude".into(),
            source: "auto".into(),
            title: "t".into(),
            started_at: 1_700_000_000_000,
            ended_at: None,
            end_reason: None,
            prompt_count: 3,
            token_count: Some(1200),
            event_count: 9,
            status: SessionStatus::Active,
            parser_version: None,
            strategy_version: None,
            metadata_json: "{}".into(),
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
        }
    }

    fn sample_link() -> SessionCommitLink {
        SessionCommitLink {
            id: "l1".into(),
            workspace_id: "w1".into(),
            session_id: "s1".into(),
            commit_sha: "abc".into(),
            is_primary: true,
            link_state: LinkState::Pending,
            auto_bound: true,
            confidence: 0.5,
            confidence_reason: "tw".into(),
            strategy_version: "v1".into(),
            source_event_id: None,
            linked_at: 1_700_000_000_000,
            unlinked_at: None,
            unlinked_reason: None,
            superseded_by_link_id: None,
            created_by: "system".into(),
            reviewed_by: None,
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
        }
    }

    #[test]
    fn bind_mode_serializes_camelcase() {
        assert_eq!(
            serde_json::to_string(&SessionBindMode::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(
            serde_json::to_string(&SessionBindMode::Manual).unwrap(),
            "\"manual\""
        );
        // roundtrip
        let m: SessionBindMode = serde_json::from_str("\"manual\"").unwrap();
        assert_eq!(m, SessionBindMode::Manual);
    }

    #[test]
    fn requests_use_camelcase_wire_keys() {
        // 防 H2 类 snake_case/camelCase drift：wire 形状必须 camelCase。
        let req = SessionBindCommitRequest {
            workspace_id: "w1".into(),
            commit_sha: "abc".into(),
            session_id: Some("s1".into()),
            mode: SessionBindMode::Auto,
            reason: None,
        };
        let j = serde_json::to_string(&req).unwrap();
        assert!(j.contains("\"workspaceId\""), "got {j}");
        assert!(j.contains("\"commitSha\""), "got {j}");
        assert!(j.contains("\"sessionId\""), "got {j}");
        assert!(!j.contains("workspace_id"), "snake_case leaked: {j}");
    }

    #[test]
    fn i64_epoch_fields_serialize_as_json_number_not_string() {
        // i64 跨 IPC 必须是 JSON number（ts-rs binding 侧 #[ts(type="number")]
        // 对齐此 wire 行为 · bigint/string 都是谎言）。
        let res = SessionDetailResult {
            session: sample_session(),
            links: vec![sample_link()],
            commit_count: 1,
            avg_confidence: 0.5,
        };
        let v: serde_json::Value = serde_json::to_value(&res).unwrap();
        assert!(
            v["commitCount"].is_number(),
            "commitCount must be JSON number, got {:?}",
            v["commitCount"]
        );
        assert!(
            v["session"]["startedAt"].is_number(),
            "nested AiSession.startedAt must stay JSON number"
        );
        assert_eq!(v["commitCount"], serde_json::json!(1));
    }

    #[test]
    fn result_types_roundtrip() {
        let r = SessionStartResult {
            session: sample_session(),
            already_active: false,
        };
        let s = serde_json::to_string(&r).unwrap();
        let back: SessionStartResult = serde_json::from_str(&s).unwrap();
        assert_eq!(back, r);

        let rb = SessionRebindResult {
            superseded_link_id: "old".into(),
            new_link: sample_link(),
        };
        let back2: SessionRebindResult =
            serde_json::from_str(&serde_json::to_string(&rb).unwrap()).unwrap();
        assert_eq!(back2, rb);
    }

    #[test]
    fn error_event_reuses_canonical_tagged_error() {
        // §K.2 · SessionErrorEvent 复用 canonical SessionError tagged enum ·
        // 不另造错误形状（machine 可读 kind 透传）。
        let ev = SessionErrorEvent {
            workspace_id: "w1".into(),
            error: SessionError::LinkNotFound("l9".into()),
        };
        let j = serde_json::to_string(&ev).unwrap();
        assert!(j.contains("\"workspaceId\""));
        assert!(
            j.contains("linkNotFound"),
            "must carry canonical machine kind, got {j}"
        );
    }
}
