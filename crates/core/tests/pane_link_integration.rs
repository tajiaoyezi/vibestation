//! MVP-18 Phase A · §F.1 typed fixtures + §F.3/§E B integration tests.
//!
//! 集成测试层（`crates/core/tests/` binary · 非 src）。
//! 只 import A1 冻结公共 API（`vibestation_core::pane_links::*`）；
//! 不改 src / pane_links.rs / lib.rs（A1 文件域）。
//!
//! 覆盖范围：
//! - §F.1 全 6 fixture 函数
//! - §F.3 前 4 link fixture 场景（same_workspace / cross_workspace / duplicate / stale_child）
//! - §E B.1（cross-ws denied）· B.2（非 AI parent）· B.3（非执行型 child）· B.4（重复幂等不增行）
//! - §I.1（duplicate link insertion idempotency proof）

use tempfile::TempDir;
use vibestation_core::db::{self, DbPool};
use vibestation_core::pane_links::{
    validate_link_request, PaneKind, PaneLinkDao, PaneLinkError, PaneLinkKind, PaneLinkRequest,
};
use vibestation_core::workspace::WorkspaceStore;

// ── §F.1 Fixture 函数 ────────────────────────────────────────────────────────

/// §F.1 · 创建 in-memory/temp rusqlite workspace。
/// 返回 (TempDir, DbPool, workspace_id)。TempDir 必须被调用方持有以防 drop。
fn fixture_workspace() -> (TempDir, DbPool, String) {
    let dir = TempDir::new().unwrap();
    let pool = db::open_pool(&dir.path().join("pane_link_integration.db")).unwrap();
    let ws_dir = dir.path().join("ws");
    std::fs::create_dir_all(&ws_dir).unwrap();
    let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
    (dir, pool, ws.workspace_id)
}

/// §F.1 · 合成 AI Pane id（§F.1 注：pane registry 在 IPC 层 · 本层用 synthetic id）。
/// title 仅作 id 前缀以区分多个 AI pane。
fn fixture_ai_pane(workspace_id: &str, title: &str) -> String {
    format!("ai-pane-{workspace_id}-{title}")
}

/// §F.1 · 合成 Runner Pane id（执行型 · §E B.3 合法 child）。
fn fixture_runner_pane(workspace_id: &str, command: &str) -> String {
    format!("runner-pane-{workspace_id}-{command}")
}

/// §F.1 · 合成 Shell Pane id（执行型 · §E B.3 合法 child）。
fn fixture_shell_pane(workspace_id: &str, command: &str) -> String {
    format!("shell-pane-{workspace_id}-{command}")
}

/// §F.1 · 创建第二个 workspace + 合成 Pane id。
/// 用于跨 workspace 边界测试（§E B.1 / §F.3 cross_workspace）。
/// 返回 (TempDir2, DbPool2, workspace2_id, pane2_id)。
fn fixture_other_workspace_pane() -> (TempDir, DbPool, String, String) {
    let dir = TempDir::new().unwrap();
    let pool = db::open_pool(&dir.path().join("other_ws.db")).unwrap();
    let ws_dir = dir.path().join("other-ws");
    std::fs::create_dir_all(&ws_dir).unwrap();
    let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
    let pane_id = format!("runner-pane-{}", ws.workspace_id);
    (dir, pool, ws.workspace_id, pane_id)
}

/// §F.1 · 简单失败事件 struct（pane 失败信号 · 本切片不涉及 parser pipeline · 仅占位）。
struct PaneFailureEvent {
    pub exit_code: i32,
    pub stderr: String,
}

fn fixture_failure(exit_code: i32, stderr: &str) -> PaneFailureEvent {
    PaneFailureEvent {
        exit_code,
        stderr: stderr.to_string(),
    }
}

// ── 辅助：构造标准 PaneLinkRequest ──────────────────────────────────────────

fn link_req(workspace_id: &str, parent: &str, child: &str) -> PaneLinkRequest {
    PaneLinkRequest {
        workspace_id: workspace_id.to_string(),
        parent_pane_id: parent.to_string(),
        child_pane_id: child.to_string(),
        link_kind: PaneLinkKind::FailureFeedback,
    }
}

// ── §F.3 Catalog 场景 1：happy path · same workspace ────────────────────────

/// §F.3 `pane_link_same_workspace`：validate ok + create 成功 + 行入库。
#[test]
fn pane_link_same_workspace() {
    let (_dir, pool, ws) = fixture_workspace();
    let ai = fixture_ai_pane(&ws, "claude");
    let runner = fixture_runner_pane(&ws, "cargo-test");

    // validate_link_request 必须 ok（§E B.1-B.3 全通）
    let req = link_req(&ws, &ai, &runner);
    validate_link_request(&req, &ws, PaneKind::Ai, &ws, PaneKind::Runner)
        .expect("same-workspace AI→Runner link must validate ok");

    // DAO create 成功 + 行入库
    let link = PaneLinkDao::create(&pool, &req).expect("create must succeed");
    assert!(!link.id.is_empty());
    assert_eq!(link.workspace_id, ws);
    assert_eq!(link.parent_pane_id, ai);
    assert_eq!(link.child_pane_id, runner);
    assert_eq!(link.link_kind, PaneLinkKind::FailureFeedback);
    assert!(link.enabled);
    assert_eq!(link.fallback_mode, "structured");
    assert_eq!(link.created_by, "user");
    assert!(
        link.created_at > 1_000_000_000_000,
        "created_at must be unix millis"
    );

    // DB row 存在
    let count: i64 = pool
        .get()
        .unwrap()
        .query_row("SELECT count(*) FROM pane_links", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

// ── §F.3 Catalog 场景 2：cross workspace → denied (§E B.1) ──────────────────

/// §F.3 `pane_link_cross_workspace`：parent/child 不同 workspace → CrossWorkspaceDenied。
#[test]
fn pane_link_cross_workspace() {
    let (_dir1, _pool1, ws1) = fixture_workspace();
    let (_dir2, _pool2, ws2, runner_ws2) = fixture_other_workspace_pane();

    let ai = fixture_ai_pane(&ws1, "claude");
    let req = link_req(&ws1, &ai, &runner_ws2);

    // child 在 ws2 → CrossWorkspaceDenied（§E B.1）
    let err = validate_link_request(&req, &ws1, PaneKind::Ai, &ws2, PaneKind::Runner).unwrap_err();
    assert_eq!(
        err,
        PaneLinkError::CrossWorkspaceDenied,
        "cross-workspace link must be denied"
    );
}

// ── §F.3 Catalog 场景 3：duplicate → 幂等不增行 (§E B.4 / §I.1) ─────────────

/// §F.3 `pane_link_duplicate`：同 tuple 建两次 → 同 id + count==1。
/// §I.1 Prove duplicate link insertion is idempotent。
#[test]
fn pane_link_duplicate() {
    let (_dir, pool, ws) = fixture_workspace();
    let ai = fixture_ai_pane(&ws, "claude");
    let runner = fixture_runner_pane(&ws, "cargo-watch");
    let req = link_req(&ws, &ai, &runner);

    let first = PaneLinkDao::create(&pool, &req).expect("first create must succeed");
    let second = PaneLinkDao::create(&pool, &req).expect("second create (duplicate) must succeed");

    assert_eq!(
        first.id, second.id,
        "§B.4: duplicate must return same link id"
    );

    let count: i64 = pool
        .get()
        .unwrap()
        .query_row("SELECT count(*) FROM pane_links", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        count, 1,
        "§B.4/§I.1: duplicate must NOT create a second DB row"
    );
}

// ── §F.3 Catalog 场景 4：stale child → link row persists ────────────────────

/// §F.3 `pane_link_stale_child`：child pane 概念上关闭后，link 行仍在（不级联删）。
/// 本切片：create 后 link 行存在；不实现 unlink/stale DAO（§F.2 TODO 待后续切片）。
#[test]
fn pane_link_stale_child() {
    let (_dir, pool, ws) = fixture_workspace();
    let ai = fixture_ai_pane(&ws, "claude");
    let runner = fixture_runner_pane(&ws, "pytest");
    let req = link_req(&ws, &ai, &runner);

    let link = PaneLinkDao::create(&pool, &req).expect("create must succeed");

    // 模拟 child pane 关闭：本切片只验证 link 行仍在（no cascade delete · §F.2 TODO）
    // deleted_at 由 unlink DAO（后续切片）管理；此处 deleted_at IS NULL 即 active
    let count: i64 = pool
        .get()
        .unwrap()
        .query_row(
            "SELECT count(*) FROM pane_links WHERE id = ?1 AND deleted_at IS NULL",
            [&link.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "stale child: link row must remain active (no cascade delete in this slice)"
    );
}

// ── §E B.2：非 AI parent → InvalidParentPaneType ────────────────────────────

/// §E B.2：parent 是 Runner（非 AI）→ InvalidParentPaneType。
#[test]
fn non_ai_parent_rejected_b2() {
    let (_dir, _pool, ws) = fixture_workspace();
    let runner_parent = fixture_runner_pane(&ws, "cargo-build");
    let runner_child = fixture_runner_pane(&ws, "cargo-test");
    let req = link_req(&ws, &runner_parent, &runner_child);

    let err =
        validate_link_request(&req, &ws, PaneKind::Runner, &ws, PaneKind::Runner).unwrap_err();
    assert_eq!(
        err,
        PaneLinkError::InvalidParentPaneType,
        "non-AI parent must be rejected with InvalidParentPaneType"
    );
}

// ── §E B.3：非执行型 child → InvalidChildPaneType ───────────────────────────

/// §E B.3：child 是 AI Pane（非执行型）→ InvalidChildPaneType。
#[test]
fn non_executor_child_rejected_b3() {
    let (_dir, _pool, ws) = fixture_workspace();
    let ai = fixture_ai_pane(&ws, "claude");
    let ai_child = fixture_ai_pane(&ws, "codex");
    let req = link_req(&ws, &ai, &ai_child);

    let err = validate_link_request(&req, &ws, PaneKind::Ai, &ws, PaneKind::Ai).unwrap_err();
    assert_eq!(
        err,
        PaneLinkError::InvalidChildPaneType,
        "non-executor child (AI) must be rejected with InvalidChildPaneType"
    );
}

/// §E B.3：child 是 Other（纯 UI Pane）→ InvalidChildPaneType。
#[test]
fn pure_ui_child_rejected_b3() {
    let (_dir, _pool, ws) = fixture_workspace();
    let ai = fixture_ai_pane(&ws, "claude");
    let ui_pane = format!("ui-pane-{ws}");
    let req = link_req(&ws, &ai, &ui_pane);

    let err = validate_link_request(&req, &ws, PaneKind::Ai, &ws, PaneKind::Other).unwrap_err();
    assert_eq!(
        err,
        PaneLinkError::InvalidChildPaneType,
        "pure UI (Other) child must be rejected with InvalidChildPaneType"
    );
}

// ── §F.1 fixture 完备性：所有 child PaneKind 枚举 ──────────────────────────

/// §E B.3 正向：Runner / Watch / Log / Build / Shell 均为合法 child。
#[test]
fn all_executor_child_kinds_accepted() {
    let (_dir, _pool, ws) = fixture_workspace();
    let ai = fixture_ai_pane(&ws, "claude");
    let child = "some-child-pane";
    let req = link_req(&ws, &ai, child);

    for kind in [
        PaneKind::Runner,
        PaneKind::Watch,
        PaneKind::Log,
        PaneKind::Build,
        PaneKind::Shell,
    ] {
        validate_link_request(&req, &ws, PaneKind::Ai, &ws, kind)
            .unwrap_or_else(|e| panic!("executor kind {kind:?} must be accepted, got {e}"));
    }
}

// ── §F.1 fixture_failure smoke ───────────────────────────────────────────────

/// §F.1 `fixture_failure`：简单 struct 构造 smoke test。
/// parser pipeline 在 Phase C 实现；本切片仅验证 struct 可构造、字段正确。
#[test]
fn fixture_failure_struct_smoke() {
    let ev = fixture_failure(1, "error: cannot find value `foo`");
    assert_eq!(ev.exit_code, 1);
    assert!(ev.stderr.contains("foo"));

    let ok = fixture_failure(0, "");
    assert_eq!(ok.exit_code, 0);
    assert!(ok.stderr.is_empty());
}

// ── §F.1 fixture_shell_pane smoke ───────────────────────────────────────────

/// §F.1 `fixture_shell_pane` 合成 id 可与 validate_link_request 组合。
#[test]
fn fixture_shell_pane_validates_as_child() {
    let (_dir, _pool, ws) = fixture_workspace();
    let ai = fixture_ai_pane(&ws, "claude");
    let shell = fixture_shell_pane(&ws, "bash");
    let req = link_req(&ws, &ai, &shell);

    validate_link_request(&req, &ws, PaneKind::Ai, &ws, PaneKind::Shell)
        .expect("shell pane must be valid child");
}
