//! Vibestation 核心业务逻辑
//!
//! 本 crate 包含与 UI / 桌面框架无关的业务核心：workspace 管理、git 操作、PTY、AI-aware
//! pane 联动（v1.0 vision）等。保持纯 Rust · 可独立 `cargo test` · 不依赖 Tauri。

pub mod app_settings;
pub mod branch_ops;
pub mod config_import;
pub mod db;
pub mod diff;
pub mod fs_watch;
pub mod git_log;
pub mod git_ops;
pub mod git_status;
pub mod git_sync;
pub mod layout;
pub mod pane_pty;
pub mod pane_service;
pub mod panes;
pub mod pty;
pub mod pty_pool;
pub mod rail_graph_events;
pub mod rebase_ops;
pub mod tabs;
pub mod telemetry;
pub mod workspace;

pub use app_settings::{AppSettings, AppSettingsStore, SettingsUpdateRequest};
pub use branch_ops::{
    branch_checkout, branch_create, branch_delete, branch_list, branch_switcher_query,
    BranchCheckoutRequest, BranchCreateRequest, BranchDeleteRequest, BranchError, BranchInfo,
    BranchKind, BranchListRequest, BranchListResponse, BranchSwitchResult, SwitcherMatch,
    SwitcherQueryRequest, SwitcherSearchResult,
};
pub use config_import::ipc::{
    apply as config_import_apply, build_preview as config_import_build_preview,
    detect_conflicts_ipc as config_import_detect_conflicts,
    scan_all_sources_ipc as config_import_scan, ImportApplyRequest, ImportApplyResult,
    ImportFieldType, ImportPreview, ImportScanResult, KeyBindingConflict, KeyBindingResolution,
};
pub use config_import::ImportSource;
pub use diff::{
    DiffError, DiffHunk, DiffLine, DiffLineType, DiffRequest, DiffResponse, DiffService,
};
pub use fs_watch::{GitStatusWatchError, GitStatusWatcher, GIT_STATUS_WATCH_DEBOUNCE};
pub use git_log::{
    CommitAuthor, CommitDetail, CommitParent, FileChange, GitLogEntry, GitLogError,
    GitLogQueryRequest, GitLogQueryResponse, GitLogReader,
};
pub use git_ops::{
    CommitError, CommitRequest, CommitResponse, GitConfigIdentity, GitOpsService,
    SetGitIdentityRequest, StageFailedItem, StageRequest, StageResult, UnstageRequest,
};
pub use git_status::{
    FileStatusEvent, GitStatusCollapseRequest, GitStatusGroup, GitStatusPanelSettings,
    GitStatusRequest, GitStatusResponse, GitStatusService,
};
pub use git_sync::{
    git_auth_provide, git_fetch, git_fetch_with_events, git_merge_abort, git_pull,
    git_pull_with_events, git_push, git_push_with_events, git_remote_list, AuthChallenge,
    AuthMethod, AuthRequest, ConflictFile, FetchProgressEvent, FetchRequest, FetchResult,
    GitSyncEventHandlers, MergeConflictInfo, NetworkOpError, OperationDoneEvent, PullRequest,
    PullResult, PullStrategy, PushProgressEvent, PushRequest, PushResult, RemoteInfo,
    RemoteListRequest, RemoteListResponse,
};
pub use layout::{LayoutState, LayoutStore};
pub use pane_pty::{map_event as map_pane_pty_event, PanePtyEvent};
pub use pane_service::{
    apply_layout_preset, apply_pane_close, apply_pane_focus, apply_pane_init_for_tab,
    apply_pane_split, apply_split_ratio_update, PaneInitRequest,
};
pub use panes::{
    LayoutApplyRequest, LayoutNode, PaneCloseRequest, PaneCreateRequest, PaneError,
    PaneFocusRequest, PaneListResponse, PanePtyExitedEvent, PanePtySpawnRequest,
    PanePtyStdoutEvent, PaneScrollbackFetchRequest, PaneState, PanesDao, SmartLayoutKind, SplitDir,
    SplitRatioUpdateRequest,
};
pub use pty::{
    check_shell_exists, list_available_shells, resolve_default_shell, PtyError, PtyEvent,
    PtyEventReceiver, PtyExitedEvent, PtyManager, PtySpawnRequest, PtyStdoutEvent, ShellInfo,
    PTY_EVENT_QUEUE_CAPACITY,
};
pub use pty_pool::SpawnResult;
pub use rail_graph_events::{
    RailGraphBranchChangedPayload, RailGraphPerfSample, RailGraphRebaseStatePayload,
    RailGraphViewportSyncPayload,
};
pub use rebase_ops::{
    cherrypick_abort, cherrypick_continue, cherrypick_start, conflict_resolve_file,
    conflict_status, detect_in_progress, merge_abort as rebase_merge_abort, merge_start,
    rebase_abort, rebase_continue, rebase_interactive_apply, rebase_interactive_plan, rebase_skip,
    rebase_start, CherryPickRequest, CherryPickStatus, ConflictHunk, ConflictHunkResolution,
    ConflictResolution, ConflictResolveFileRequest, ConflictedFile, CrashRecoveryState,
    MergeRequest, MergeStatus, MergeStrategy, RebaseControlRequest, RebaseInteractivePlan,
    RebaseInteractiveStep, RebaseOp, RebaseOpError, RebaseStartRequest, RebaseStatus,
};
pub use tabs::{
    TabCloseRequest, TabCreateRequest, TabListResponse, TabRenameRequest, TabReorderRequest,
    TabState, TabsDao,
};
pub use telemetry::{
    build_status as telemetry_build_status, build_version_info as telemetry_build_version_info,
    capture_crash_report as telemetry_capture_crash_report,
    capture_panic as telemetry_capture_panic, init_sentry as telemetry_init_sentry,
    is_initialized as telemetry_is_initialized, runtime_opt_in as telemetry_runtime_opt_in,
    set_runtime_opt_in as telemetry_set_runtime_opt_in,
    should_send_telemetry as telemetry_should_send, AppVersionInfo, CrashReportPayload,
    TelemetryError, TelemetryOptInRequest, TelemetryStatus,
};
pub use workspace::{WorkspaceMetadata, WorkspaceStore};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn greet() -> &'static str {
    "Vibestation core online"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_returns_fixed_message() {
        assert_eq!(greet(), "Vibestation core online");
    }

    #[test]
    fn version_is_non_empty() {
        assert!(
            !VERSION.is_empty(),
            "CARGO_PKG_VERSION should be injected by Cargo"
        );
    }
}
