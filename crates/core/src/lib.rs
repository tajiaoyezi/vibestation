//! Vibestation 核心业务逻辑
//!
//! 本 crate 包含与 UI / 桌面框架无关的业务核心：workspace 管理、git 操作、PTY、AI-aware
//! pane 联动（v1.0 vision）等。保持纯 Rust · 可独立 `cargo test` · 不依赖 Tauri。

pub mod app_settings;
pub mod config_import;
pub mod db;
pub mod diff;
pub mod fs_watch;
pub mod git_log;
pub mod git_ops;
pub mod git_status;
pub mod layout;
pub mod pane_pty;
pub mod pane_service;
pub mod panes;
pub mod pty;
pub mod tabs;
pub mod telemetry;
pub mod workspace;

pub use app_settings::{AppSettings, AppSettingsStore, SettingsUpdateRequest};
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
    check_shell_exists, resolve_default_shell, PtyError, PtyEvent, PtyEventReceiver,
    PtyExitedEvent, PtyManager, PtySpawnRequest, PtyStdoutEvent, PTY_EVENT_QUEUE_CAPACITY,
};
pub use tabs::{
    TabCloseRequest, TabCreateRequest, TabListResponse, TabRenameRequest, TabState, TabsDao,
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
