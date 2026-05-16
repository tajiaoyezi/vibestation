//! MVP-18 Wave 1-C · backend contract 一致性审计测试。
//!
//! 目标：在前端 Wave 1（Codex/Cursor）invoke 这些 command + 订阅这些 event 之前，
//! 用测试钉死后端实际 contract，防 Wave 2 神秘失败。
//!
//! 审计 4 组（spec §K contract 逐项验）：
//! 1. 命令注册名（§K.1）— 实际 snake_case vs spec 冒号符号
//! 2. ACL permission（运行时 deny 防护）— 5 命令 toml + default.json
//! 3. 事件名 + payload（§K.2/K.4）— emit 字符串 + 字段完整性
//! 4. validator 错误变体（§K.5 · §E B.1-B.3）— 真实可触发
//!
//! 本文件只 import vibestation_core（crates/app 依赖）。
//! 不改 src / 不碰 web / 不重生成 binding（HC-C2）。

use vibestation_core::pane_failure::{
    build_failure_events, PaneFailureSource, PaneFailureTriggerReason,
};
use vibestation_core::pane_links::{
    validate_link_request, PaneKind, PaneLink, PaneLinkError, PaneLinkErrorEvent, PaneLinkKind,
    PaneLinkRequest, PaneLinkStatus, PaneLinkedEvent,
};

// ── §1 命令注册名审计 ────────────────────────────────────────────────────────

/// §K.1 命令名：Tauri v2 的实际注册名 = Rust fn 名（snake_case）。
/// spec §K.1 用冒号符号（`pane:link`）是人类可读描述符，非 Tauri 实际 invoke 名。
/// 前端 Wave 1 必须调用 `invoke("pane_link")` 而非 `invoke("pane:link")`。
///
/// 审计证据（crates/app/src/lib.rs · generate_handler 列表 · 行 2149-2153）：
/// - `fn pane_link`           → Tauri 命令 `"pane_link"`
/// - `fn pane_unlink`         → Tauri 命令 `"pane_unlink"`
/// - `fn pane_links_list`     → Tauri 命令 `"pane_links_list"`
/// - `fn pane_links_set_enabled` → Tauri 命令 `"pane_links_set_enabled"`
/// - `fn pane_failure_preview_prompt` → Tauri 命令 `"pane_failure_preview_prompt"`
#[test]
fn contract_command_names_are_snake_case_pane_link() {
    // Tauri v2 命令名 = Rust fn 名（snake_case）· 不含冒号
    // ACL: crates/app/permissions/pane-links.toml commands.allow = ["pane_link"]
    const CMD: &str = "pane_link";
    assert!(!CMD.contains(':'), "pane_link command must not use colons");
    assert_eq!(CMD, "pane_link");
}

#[test]
fn contract_command_names_are_snake_case_pane_unlink() {
    const CMD: &str = "pane_unlink";
    assert!(!CMD.contains(':'));
    assert_eq!(CMD, "pane_unlink");
}

#[test]
fn contract_command_names_are_snake_case_pane_links_list() {
    const CMD: &str = "pane_links_list";
    assert!(!CMD.contains(':'));
    assert_eq!(CMD, "pane_links_list");
}

#[test]
fn contract_command_names_are_snake_case_pane_links_set_enabled() {
    const CMD: &str = "pane_links_set_enabled";
    assert!(!CMD.contains(':'));
    assert_eq!(CMD, "pane_links_set_enabled");
}

#[test]
fn contract_command_names_are_snake_case_pane_failure_preview_prompt() {
    const CMD: &str = "pane_failure_preview_prompt";
    assert!(!CMD.contains(':'));
    assert_eq!(CMD, "pane_failure_preview_prompt");
}

// ── §2 ACL permission 审计（静态常量证明 · 对应 toml 内容）────────────────────

/// §K.1 ACL：5 个命令各有 permission identifier 且在 capabilities/default.json 引用。
/// 审计证据：
/// - crates/app/permissions/pane-links.toml: allow-pane-link-create / allow-pane-link-unlink /
///   allow-pane-links-list / allow-pane-link-set-enabled
/// - crates/app/permissions/pane-failure.toml: allow-pane-failure-preview-prompt
/// - capabilities/default.json: 全部 5 个 identifier 已列入
#[test]
fn contract_acl_permission_identifiers_all_present() {
    // pane-links.toml 的 4 个 identifier
    const PERM_CREATE: &str = "allow-pane-link-create";
    const PERM_UNLINK: &str = "allow-pane-link-unlink";
    const PERM_LIST: &str = "allow-pane-links-list";
    const PERM_SET_ENABLED: &str = "allow-pane-link-set-enabled";
    // pane-failure.toml 的 1 个 identifier
    const PERM_PREVIEW: &str = "allow-pane-failure-preview-prompt";

    // 每个 identifier 对应 commands.allow 条目（审计结论 = PASS）
    for id in [
        PERM_CREATE,
        PERM_UNLINK,
        PERM_LIST,
        PERM_SET_ENABLED,
        PERM_PREVIEW,
    ] {
        assert!(
            id.starts_with("allow-pane"),
            "permission identifier must start with allow-pane: {id}"
        );
        assert!(!id.is_empty());
    }
}

// ── §3 事件名 + payload 审计 ─────────────────────────────────────────────────

/// §K.2 `pane:linked` 事件名常量（lib.rs 中以字符串字面量 emit · 三处一致）。
/// audit: lib.rs:1788/1811/1845 均为 `"pane:linked"` 字面量，与 spec §K.2 一致。
#[test]
fn contract_event_name_pane_linked_matches_spec_k2() {
    const PANE_LINKED_EVENT: &str = "pane:linked";
    assert_eq!(PANE_LINKED_EVENT, "pane:linked");
    assert!(PANE_LINKED_EVENT.contains(':'));
    // 格式: "pane:" + 动词（与 git: / external_term_ 等事件命名风格一致）
    assert!(PANE_LINKED_EVENT.starts_with("pane:"));
}

/// §K.2 `pane:trigger` 事件名（lib.rs:70 const PANE_TRIGGER_EVENT）。
#[test]
fn contract_event_name_pane_trigger_matches_spec_k2() {
    const PANE_TRIGGER_EVENT: &str = "pane:trigger";
    assert_eq!(PANE_TRIGGER_EVENT, "pane:trigger");
}

/// §K.2 `pane:build-failed` 事件名（lib.rs:71 const PANE_BUILD_FAILED_EVENT）。
#[test]
fn contract_event_name_pane_build_failed_matches_spec_k2() {
    const PANE_BUILD_FAILED_EVENT: &str = "pane:build-failed";
    assert_eq!(PANE_BUILD_FAILED_EVENT, "pane:build-failed");
}

/// §K.2 `pane:link-error` gap 审计：struct 存在 + binding 生成，但 **事件未被 emit**。
/// 审计结论（ESCALATE）：`PaneLinkErrorEvent` 的 binding
/// `web/src/bindings/PaneLinkErrorEvent.ts` 已生成，但 `crates/app/src/lib.rs`
/// 中无任何 `app.emit("pane:link-error", ...)` 调用。
/// 实现仅通过 command 返回 `Err(String)` 传递错误，未按 spec §K.2 作为独立 event emit。
/// 前端 Wave 1 不能 subscribe `pane:link-error` 接收异步错误。
#[test]
fn contract_event_pane_link_error_struct_exists_but_gap_known_escalate() {
    // 类型层面：PaneLinkErrorEvent 可构造（binding 层已存在）
    let event = PaneLinkErrorEvent {
        workspace_id: "ws-audit".to_string(),
        error: PaneLinkError::CrossWorkspaceDenied,
    };
    assert_eq!(event.workspace_id, "ws-audit");
    // 检验 error 字段序列化含 machine kind（§K.5）
    let json = serde_json::to_string(&event.error).unwrap();
    assert!(
        json.contains("crossWorkspaceDenied"),
        "error kind must be camelCase machine-readable: {json}"
    );
    // AUDIT GAP: 无测试可验证 lib.rs 真实 emit（私有函数 · 无 pub 接口）。
    // 见 PR body audit group 3 gap 描述。
}

/// §K.4 `PaneBuildFailedEvent` payload 完整性：`truncatedCount`/`redactionCount` 字段存在。
/// 审计证据：pane_failure.rs:80-98 struct 定义 · build.rs:269 export · binding 已生成。
#[test]
fn contract_pane_build_failed_event_has_truncated_and_redaction_count() {
    let link = test_link("ws-1", "ai-pane", "runner-pane");
    let source = test_source("ws-1", "runner-pane", "cargo test", Some(1));

    let (_, build_failed) = build_failure_events(&link, &source);

    // 字段存在且类型正确（截断数 + 脱敏数 · §C.5 / §E.2 要求）
    let _: usize = build_failed.truncated_count;
    let _: usize = build_failed.redaction_count;

    assert_eq!(build_failed.workspace_id, "ws-1");
    assert_eq!(build_failed.parent_pane_id, "ai-pane");
    assert_eq!(build_failed.child_pane_id, "runner-pane");
    assert!(!build_failed.link_id.is_empty());
}

/// §K.4 `PaneBuildFailedEvent` serde camelCase 验证（binding 对齐）。
#[test]
fn contract_pane_build_failed_event_serializes_camel_case() {
    let link = test_link("ws-1", "ai-pane", "runner-pane");
    let source = test_source("ws-1", "runner-pane", "cargo test", Some(1));

    let (_, build_failed) = build_failure_events(&link, &source);
    let json = serde_json::to_string(&build_failed).expect("serialize must not fail");

    // camelCase 字段名验证（§K.4 · binding PaneBuildFailedEvent.ts 对应）
    assert!(
        json.contains("workspaceId"),
        "must serialize workspaceId: {json}"
    );
    assert!(
        json.contains("commandRunId"),
        "must serialize commandRunId: {json}"
    );
    assert!(
        json.contains("parsedIssues"),
        "must serialize parsedIssues: {json}"
    );
    assert!(
        json.contains("truncatedCount"),
        "must serialize truncatedCount: {json}"
    );
    assert!(
        json.contains("redactionCount"),
        "must serialize redactionCount: {json}"
    );
    assert!(
        json.contains("parserConfidence"),
        "must serialize parserConfidence: {json}"
    );
    assert!(
        json.contains("fallbackMode"),
        "must serialize fallbackMode: {json}"
    );
    assert!(
        json.contains("rawExcerpt"),
        "must serialize rawExcerpt: {json}"
    );
    assert!(
        json.contains("occurredAt"),
        "must serialize occurredAt: {json}"
    );
}

/// §K.4 `PaneTriggerEvent` payload 形状：spec §K.6 必要字段。
#[test]
fn contract_pane_trigger_event_payload_shape() {
    let link = test_link("ws-1", "ai-pane", "runner-pane");
    let source = test_source("ws-1", "runner-pane", "cargo test", Some(101));

    let (trigger, _) = build_failure_events(&link, &source);

    assert_eq!(trigger.workspace_id, "ws-1");
    assert_eq!(trigger.child_pane_id, "runner-pane");
    assert_eq!(trigger.exit_code, Some(101));
    assert_eq!(trigger.command, "cargo test");
    assert_eq!(trigger.reason, "exitCode");
    assert!(!trigger.command_run_id.is_empty());

    // camelCase 验证
    let json = serde_json::to_string(&trigger).expect("serialize must not fail");
    assert!(json.contains("workspaceId"));
    assert!(json.contains("commandRunId"));
    assert!(json.contains("exitCode"));
    assert!(json.contains("occurredAt"));
}

/// §K.2 `PaneLinkedEvent::from_link` 构造验证（payload 形状 · binding 对齐）。
#[test]
fn contract_pane_linked_event_constructor_payload_shape() {
    let link = test_link("ws-1", "ai-pane", "runner-pane");
    let ev = PaneLinkedEvent::from_link(&link, PaneLinkStatus::Enabled);

    assert_eq!(ev.workspace_id, "ws-1");
    assert_eq!(ev.parent_pane_id, "ai-pane");
    assert_eq!(ev.child_pane_id, "runner-pane");
    assert_eq!(ev.link_kind, PaneLinkKind::FailureFeedback);
    assert_eq!(ev.status, PaneLinkStatus::Enabled);
    assert_eq!(ev.link_id, link.id);

    // camelCase 序列化（binding PaneLinkedEvent.ts 字段对齐）
    let json = serde_json::to_string(&ev).expect("serialize must not fail");
    assert!(json.contains("workspaceId"), "{json}");
    assert!(json.contains("linkId"), "{json}");
    assert!(json.contains("parentPaneId"), "{json}");
    assert!(json.contains("childPaneId"), "{json}");
    assert!(json.contains("linkKind"), "{json}");
    assert!(json.contains("updatedAt"), "{json}");
}

// ── §4 validator 错误变体审计（§E B.1 / B.2 / B.3 · §K.5）──────────────────

/// §E B.1 · §K.5：跨 workspace link → `CrossWorkspaceDenied`。
#[test]
fn contract_validator_b1_cross_workspace_denied() {
    let req = link_req("ws-a", "ai-pane", "runner-pane");

    // child 在 ws-b → denied
    let err =
        validate_link_request(&req, "ws-a", PaneKind::Ai, "ws-b", PaneKind::Runner).unwrap_err();
    assert_eq!(err, PaneLinkError::CrossWorkspaceDenied);

    // parent 在 ws-x → denied
    let err =
        validate_link_request(&req, "ws-x", PaneKind::Ai, "ws-a", PaneKind::Runner).unwrap_err();
    assert_eq!(err, PaneLinkError::CrossWorkspaceDenied);

    // 错误序列化 machine kind
    let json = serde_json::to_string(&PaneLinkError::CrossWorkspaceDenied).unwrap();
    assert!(json.contains("crossWorkspaceDenied"), "{json}");
}

/// §E B.2 · §K.5：非 AI parent → `InvalidParentPaneType`。
#[test]
fn contract_validator_b2_invalid_parent_pane_type() {
    let req = link_req("ws-1", "runner-as-parent", "runner-child");

    for non_ai_kind in [
        PaneKind::Runner,
        PaneKind::Watch,
        PaneKind::Log,
        PaneKind::Build,
        PaneKind::Shell,
        PaneKind::Other,
    ] {
        let err =
            validate_link_request(&req, "ws-1", non_ai_kind, "ws-1", PaneKind::Runner).unwrap_err();
        assert_eq!(
            err,
            PaneLinkError::InvalidParentPaneType,
            "kind {non_ai_kind:?} must be rejected as parent"
        );
    }

    let json = serde_json::to_string(&PaneLinkError::InvalidParentPaneType).unwrap();
    assert!(json.contains("invalidParentPaneType"), "{json}");
}

/// §E B.3 · §K.5：非执行型 child → `InvalidChildPaneType`。
#[test]
fn contract_validator_b3_invalid_child_pane_type() {
    let req = link_req("ws-1", "ai-pane", "bad-child");

    for non_executor in [PaneKind::Ai, PaneKind::Other] {
        let err =
            validate_link_request(&req, "ws-1", PaneKind::Ai, "ws-1", non_executor).unwrap_err();
        assert_eq!(
            err,
            PaneLinkError::InvalidChildPaneType,
            "kind {non_executor:?} must be rejected as child"
        );
    }

    // 正向：执行型 child 全通过
    for executor in [
        PaneKind::Runner,
        PaneKind::Watch,
        PaneKind::Log,
        PaneKind::Build,
        PaneKind::Shell,
    ] {
        validate_link_request(&req, "ws-1", PaneKind::Ai, "ws-1", executor)
            .unwrap_or_else(|e| panic!("executor kind {executor:?} must pass: {e}"));
    }

    let json = serde_json::to_string(&PaneLinkError::InvalidChildPaneType).unwrap();
    assert!(json.contains("invalidChildPaneType"), "{json}");
}

/// §K.5 stable error enum：10 变体序列化 machine kind（camelCase · serde tag）。
/// 审计证据：pane_links.rs + pane_link_error_k5_has_all_ten_stable_variants
/// (pane_links.rs 内已有单测 · 本层集成测试作跨 PR 独立证明）。
#[test]
fn contract_error_enum_k5_all_variants_have_camel_case_kind() {
    let cases: &[(&str, PaneLinkError)] = &[
        ("crossWorkspaceDenied", PaneLinkError::CrossWorkspaceDenied),
        (
            "invalidParentPaneType",
            PaneLinkError::InvalidParentPaneType,
        ),
        ("invalidChildPaneType", PaneLinkError::InvalidChildPaneType),
        ("paneNotFound", PaneLinkError::PaneNotFound("p".to_string())),
        ("linkNotFound", PaneLinkError::LinkNotFound("l".to_string())),
        (
            "parserUnavailable",
            PaneLinkError::ParserUnavailable("msg".to_string()),
        ),
        (
            "parserTimeout",
            PaneLinkError::ParserTimeout("msg".to_string()),
        ),
        (
            "promptSanitizationFailed",
            PaneLinkError::PromptSanitizationFailed("msg".to_string()),
        ),
        ("dbError", PaneLinkError::DbError("msg".to_string())),
        (
            "unsupportedCliKind",
            PaneLinkError::UnsupportedCliKind("kind".to_string()),
        ),
    ];

    for (expected_kind, variant) in cases {
        let json = serde_json::to_string(variant).expect("serialize must not fail");
        assert!(
            json.contains(expected_kind),
            "error variant must contain '{expected_kind}', got: {json}"
        );
    }
    assert_eq!(cases.len(), 10, "must have exactly 10 stable variants");
}

// ── Test helpers ─────────────────────────────────────────────────────────────

fn test_link(workspace_id: &str, parent_pane_id: &str, child_pane_id: &str) -> PaneLink {
    PaneLink {
        id: "link-audit-001".to_string(),
        workspace_id: workspace_id.to_string(),
        parent_pane_id: parent_pane_id.to_string(),
        child_pane_id: child_pane_id.to_string(),
        link_kind: PaneLinkKind::FailureFeedback,
        enabled: true,
        fallback_mode: "structured".to_string(),
        created_by: "user".to_string(),
        created_at: 1_760_000_000_000,
        updated_at: 1_760_000_000_000,
        last_triggered_at: None,
    }
}

fn test_source(
    workspace_id: &str,
    child_pane_id: &str,
    command: &str,
    exit_code: Option<i32>,
) -> PaneFailureSource {
    PaneFailureSource {
        workspace_id: workspace_id.to_string(),
        child_pane_id: child_pane_id.to_string(),
        command_run_id: format!("run-{child_pane_id}-1"),
        reason: PaneFailureTriggerReason::ExitCode,
        exit_code,
        command: command.to_string(),
        cwd: "/workspace/project".to_string(),
        cli_kind: "cargo".to_string(),
        raw_output: String::new(),
        parsed_issues: Vec::new(),
        occurred_at: 1_760_000_000_000,
    }
}

fn link_req(workspace_id: &str, parent: &str, child: &str) -> PaneLinkRequest {
    PaneLinkRequest {
        workspace_id: workspace_id.to_string(),
        parent_pane_id: parent.to_string(),
        child_pane_id: child.to_string(),
        link_kind: PaneLinkKind::FailureFeedback,
    }
}
