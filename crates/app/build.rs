//! Tauri build hook + ts-rs IPC contract codegen。
//!
//! 在每次 `cargo build` 时：
//! 1. 调 `tauri-build` 生成 Tauri config / capability schema
//! 2. 从 `vibestation-core` 导出 IPC struct 到
//!    `web/src/bindings/*.ts` · 供前端 `import` 消费
//!
//! 这是 SPIKE-08 §A（`docs/spikes/SPIKE-08-report.md`）的生产化落地 · 防止
//! MVP-02 H2 类 camelCase / snake_case drift 事故。

use std::{fs, path::PathBuf};

use ts_rs::{Config, TS};
use vibestation_core::{
    AppSettings, AppVersionInfo, AuthChallenge, AuthMethod, AuthRequest, BranchCheckoutRequest,
    BranchCreateRequest, BranchDeleteRequest, BranchError, BranchInfo, BranchKind,
    BranchListRequest, BranchListResponse, BranchSwitchResult, CherryPickRequest, CherryPickStatus,
    CommitAuthor, CommitDetail, CommitError, CommitParent, CommitRequest, CommitResponse,
    ConflictFile, ConflictHunk, ConflictHunkResolution, ConflictResolution,
    ConflictResolveFileRequest, ConflictedFile, CrashRecoveryState, CrashReportPayload, DiffHunk,
    DiffLine, DiffLineType, DiffRequest, DiffResponse, EnvEntry, EnvPreview, ExternalTerminalInfo,
    ExternalTerminalLaunchRequest, ExternalTerminalLaunchResult, FetchProgressEvent, FetchRequest,
    FetchResult, FileChange, FileStatusEvent, GitConfigIdentity, GitLogEntry, GitLogQueryRequest,
    GitLogQueryResponse, GitStatusCollapseRequest, GitStatusGroup, GitStatusPanelSettings,
    GitStatusRequest, GitStatusResponse, ImportApplyRequest, ImportApplyResult, ImportFieldType,
    ImportPreview, ImportScanResult, ImportSource, KeyBindingConflict, KeyBindingResolution,
    LayoutApplyAdvancedRequest, LayoutApplyRequest, LayoutApplyResult, LayoutEnvelope,
    LayoutHistoryEntry, LayoutNode, LayoutPresetKind, LayoutState, MergeConflictInfo, MergeRequest,
    MergeStatus, MergeStrategy, NetworkOpError, OperationDoneEvent, PaneCloseRequest,
    PaneCreateRequest, PaneDetachAction, PaneDetachCloseRequest, PaneDetachCloseResult,
    PaneDetachListEntry, PaneDetachOpenRequest, PaneDetachOpenResult, PaneDetachStateEvent,
    PaneFocusRequest, PaneInitRequest, PaneListResponse, PaneMaximizeRequest, PaneMaximizeResult,
    PaneNavDirection, PaneNavigateRequest, PaneNavigateResult, PanePtyExitedEvent,
    PanePtySpawnRequest, PanePtyStdoutEvent, PaneResizeStepRequest, PaneScrollbackFetchRequest,
    PaneState, PtyExitedEvent, PtySpawnRequest, PtyStdoutEvent, PullRequest, PullResult,
    PullStrategy, PushProgressEvent, PushRequest, PushResult, RailGraphBranchChangedPayload,
    RailGraphPerfSample, RailGraphRebaseStatePayload, RailGraphViewportSyncPayload,
    RebaseControlRequest, RebaseInteractivePlan, RebaseInteractiveStep, RebaseOp, RebaseOpError,
    RebaseStartRequest, RebaseStatus, RemoteInfo, RemoteListRequest, RemoteListResponse,
    SetGitIdentityRequest, SettingsUpdateRequest, ShellInfo, SpawnResult, SplitDir,
    SplitRatioUpdateRequest, StageFailedItem, StageRequest, StageResult, SwitcherMatch,
    SwitcherQueryRequest, SwitcherSearchResult, TabCloseRequest, TabCreateRequest, TabListResponse,
    TabRenameRequest, TabReorderRequest, TabState, TelemetryOptInRequest, TelemetryStatus,
    UnstageRequest, WorkspaceLayoutState, WorkspaceMetadata,
};

fn main() {
    tauri_build::build();

    // 监控 core crate 的 IPC contract 源文件 · 改动时重跑 build.rs。
    println!("cargo:rerun-if-changed=../core/src/workspace.rs");
    println!("cargo:rerun-if-changed=../core/src/layout.rs");
    println!("cargo:rerun-if-changed=../core/src/tabs.rs");
    println!("cargo:rerun-if-changed=../core/src/panes.rs");
    println!("cargo:rerun-if-changed=../core/src/pty.rs");
    println!("cargo:rerun-if-changed=../core/src/git_log.rs");
    println!("cargo:rerun-if-changed=../core/src/diff.rs");
    println!("cargo:rerun-if-changed=../core/src/git_status.rs");
    println!("cargo:rerun-if-changed=../core/src/git_ops.rs");
    println!("cargo:rerun-if-changed=../core/src/branch_ops.rs");
    println!("cargo:rerun-if-changed=../core/src/git_sync.rs");
    println!("cargo:rerun-if-changed=../core/src/rebase_ops.rs");
    println!("cargo:rerun-if-changed=../core/src/app_settings.rs");
    println!("cargo:rerun-if-changed=../core/src/telemetry.rs");
    println!("cargo:rerun-if-changed=../core/src/pty_pool.rs");
    println!("cargo:rerun-if-changed=../core/src/external_term/mod.rs");
    println!("cargo:rerun-if-changed=../core/src/external_term/detect.rs");
    println!("cargo:rerun-if-changed=../core/src/external_term/env_filter.rs");
    println!("cargo:rerun-if-changed=../core/src/external_term/launch.rs");
    println!("cargo:rerun-if-changed=../core/src/config_import/mod.rs");
    println!("cargo:rerun-if-changed=../core/src/config_import/ipc.rs");
    println!("cargo:rerun-if-changed=../core/src/config_import/keybinding.rs");
    println!("cargo:rerun-if-changed=../core/src/rail_graph_events.rs");
    println!("cargo:rerun-if-changed=../core/src/pane_detach.rs");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_dir = manifest_dir.join("../../web/src/bindings");

    fs::create_dir_all(&output_dir).expect("create web/src/bindings");

    // ts-rs 12 · `Config::with_out_dir` 指定输出根目录 · `export_all` 递归
    // 导出 struct 及其间接引用的类型。
    let config = Config::new().with_out_dir(&output_dir);
    WorkspaceMetadata::export_all(&config).expect("export WorkspaceMetadata");
    LayoutState::export_all(&config).expect("export LayoutState");
    TabState::export_all(&config).expect("export TabState");
    TabCreateRequest::export_all(&config).expect("export TabCreateRequest");
    TabCloseRequest::export_all(&config).expect("export TabCloseRequest");
    TabRenameRequest::export_all(&config).expect("export TabRenameRequest");
    TabReorderRequest::export_all(&config).expect("export TabReorderRequest");
    TabListResponse::export_all(&config).expect("export TabListResponse");
    PtyStdoutEvent::export_all(&config).expect("export PtyStdoutEvent");
    PtyExitedEvent::export_all(&config).expect("export PtyExitedEvent");
    PtySpawnRequest::export_all(&config).expect("export PtySpawnRequest");
    PaneState::export_all(&config).expect("export PaneState");
    PaneCreateRequest::export_all(&config).expect("export PaneCreateRequest");
    PaneCloseRequest::export_all(&config).expect("export PaneCloseRequest");
    PaneInitRequest::export_all(&config).expect("export PaneInitRequest");
    LayoutNode::export_all(&config).expect("export LayoutNode");
    SplitDir::export_all(&config).expect("export SplitDir");
    LayoutApplyRequest::export_all(&config).expect("export LayoutApplyRequest");
    SplitRatioUpdateRequest::export_all(&config).expect("export SplitRatioUpdateRequest");
    PaneFocusRequest::export_all(&config).expect("export PaneFocusRequest");
    PaneListResponse::export_all(&config).expect("export PaneListResponse");
    PaneScrollbackFetchRequest::export_all(&config).expect("export PaneScrollbackFetchRequest");
    PanePtySpawnRequest::export_all(&config).expect("export PanePtySpawnRequest");
    PanePtyStdoutEvent::export_all(&config).expect("export PanePtyStdoutEvent");
    PanePtyExitedEvent::export_all(&config).expect("export PanePtyExitedEvent");
    // MVP-14 Phase A · Advanced layout bindings
    LayoutEnvelope::export_all(&config).expect("export LayoutEnvelope");
    LayoutPresetKind::export_all(&config).expect("export LayoutPresetKind");
    LayoutApplyAdvancedRequest::export_all(&config).expect("export LayoutApplyAdvancedRequest");
    LayoutApplyResult::export_all(&config).expect("export LayoutApplyResult");
    PaneNavDirection::export_all(&config).expect("export PaneNavDirection");
    PaneNavigateRequest::export_all(&config).expect("export PaneNavigateRequest");
    PaneNavigateResult::export_all(&config).expect("export PaneNavigateResult");
    PaneMaximizeRequest::export_all(&config).expect("export PaneMaximizeRequest");
    PaneMaximizeResult::export_all(&config).expect("export PaneMaximizeResult");
    PaneResizeStepRequest::export_all(&config).expect("export PaneResizeStepRequest");
    LayoutHistoryEntry::export_all(&config).expect("export LayoutHistoryEntry");
    WorkspaceLayoutState::export_all(&config).expect("export WorkspaceLayoutState");

    GitLogEntry::export_all(&config).expect("export GitLogEntry");
    CommitAuthor::export_all(&config).expect("export CommitAuthor");
    CommitParent::export_all(&config).expect("export CommitParent");
    FileChange::export_all(&config).expect("export FileChange");
    CommitDetail::export_all(&config).expect("export CommitDetail");
    GitLogQueryRequest::export_all(&config).expect("export GitLogQueryRequest");
    GitLogQueryResponse::export_all(&config).expect("export GitLogQueryResponse");
    DiffRequest::export_all(&config).expect("export DiffRequest");
    DiffResponse::export_all(&config).expect("export DiffResponse");
    DiffHunk::export_all(&config).expect("export DiffHunk");
    DiffLine::export_all(&config).expect("export DiffLine");
    DiffLineType::export_all(&config).expect("export DiffLineType");
    GitStatusRequest::export_all(&config).expect("export GitStatusRequest");
    GitStatusResponse::export_all(&config).expect("export GitStatusResponse");
    FileStatusEvent::export_all(&config).expect("export FileStatusEvent");
    GitStatusPanelSettings::export_all(&config).expect("export GitStatusPanelSettings");
    GitStatusCollapseRequest::export_all(&config).expect("export GitStatusCollapseRequest");
    GitStatusGroup::export_all(&config).expect("export GitStatusGroup");

    StageRequest::export_all(&config).expect("export StageRequest");
    UnstageRequest::export_all(&config).expect("export UnstageRequest");
    CommitRequest::export_all(&config).expect("export CommitRequest");
    CommitResponse::export_all(&config).expect("export CommitResponse");
    StageResult::export_all(&config).expect("export StageResult");
    StageFailedItem::export_all(&config).expect("export StageFailedItem");
    CommitError::export_all(&config).expect("export CommitError");
    GitConfigIdentity::export_all(&config).expect("export GitConfigIdentity");
    SetGitIdentityRequest::export_all(&config).expect("export SetGitIdentityRequest");

    BranchKind::export_all(&config).expect("export BranchKind");
    BranchInfo::export_all(&config).expect("export BranchInfo");
    BranchListRequest::export_all(&config).expect("export BranchListRequest");
    BranchListResponse::export_all(&config).expect("export BranchListResponse");
    BranchCreateRequest::export_all(&config).expect("export BranchCreateRequest");
    BranchCheckoutRequest::export_all(&config).expect("export BranchCheckoutRequest");
    BranchDeleteRequest::export_all(&config).expect("export BranchDeleteRequest");
    BranchSwitchResult::export_all(&config).expect("export BranchSwitchResult");
    BranchError::export_all(&config).expect("export BranchError");
    SwitcherQueryRequest::export_all(&config).expect("export SwitcherQueryRequest");
    SwitcherMatch::export_all(&config).expect("export SwitcherMatch");
    SwitcherSearchResult::export_all(&config).expect("export SwitcherSearchResult");

    RemoteInfo::export_all(&config).expect("export RemoteInfo");
    RemoteListRequest::export_all(&config).expect("export RemoteListRequest");
    RemoteListResponse::export_all(&config).expect("export RemoteListResponse");
    PushRequest::export_all(&config).expect("export PushRequest");
    PushResult::export_all(&config).expect("export PushResult");
    PullRequest::export_all(&config).expect("export PullRequest");
    PullResult::export_all(&config).expect("export PullResult");
    PullStrategy::export_all(&config).expect("export PullStrategy");
    FetchRequest::export_all(&config).expect("export FetchRequest");
    FetchResult::export_all(&config).expect("export FetchResult");
    AuthMethod::export_all(&config).expect("export AuthMethod");
    AuthRequest::export_all(&config).expect("export AuthRequest");
    AuthChallenge::export_all(&config).expect("export AuthChallenge");
    NetworkOpError::export_all(&config).expect("export NetworkOpError");
    MergeConflictInfo::export_all(&config).expect("export MergeConflictInfo");
    ConflictFile::export_all(&config).expect("export ConflictFile");
    PushProgressEvent::export_all(&config).expect("export PushProgressEvent");
    FetchProgressEvent::export_all(&config).expect("export FetchProgressEvent");
    OperationDoneEvent::export_all(&config).expect("export OperationDoneEvent");

    RebaseStartRequest::export_all(&config).expect("export RebaseStartRequest");
    RebaseInteractivePlan::export_all(&config).expect("export RebaseInteractivePlan");
    RebaseInteractiveStep::export_all(&config).expect("export RebaseInteractiveStep");
    RebaseOp::export_all(&config).expect("export RebaseOp");
    RebaseStatus::export_all(&config).expect("export RebaseStatus");
    RebaseControlRequest::export_all(&config).expect("export RebaseControlRequest");
    RebaseOpError::export_all(&config).expect("export RebaseOpError");
    MergeRequest::export_all(&config).expect("export MergeRequest");
    MergeStrategy::export_all(&config).expect("export MergeStrategy");
    MergeStatus::export_all(&config).expect("export MergeStatus");
    CherryPickRequest::export_all(&config).expect("export CherryPickRequest");
    CherryPickStatus::export_all(&config).expect("export CherryPickStatus");
    ConflictedFile::export_all(&config).expect("export ConflictedFile");
    ConflictHunk::export_all(&config).expect("export ConflictHunk");
    ConflictResolution::export_all(&config).expect("export ConflictResolution");
    ConflictResolveFileRequest::export_all(&config).expect("export ConflictResolveFileRequest");
    ConflictHunkResolution::export_all(&config).expect("export ConflictHunkResolution");
    CrashRecoveryState::export_all(&config).expect("export CrashRecoveryState");

    AppSettings::export_all(&config).expect("export AppSettings");
    SettingsUpdateRequest::export_all(&config).expect("export SettingsUpdateRequest");
    ShellInfo::export_all(&config).expect("export ShellInfo");

    // MVP-20 BUG-001 · cd echo flash 修复 · IPC 返回类型
    SpawnResult::export_all(&config).expect("export SpawnResult");

    // MVP-17 Phase A · Pop to External IPC contract
    ExternalTerminalInfo::export_all(&config).expect("export ExternalTerminalInfo");
    ExternalTerminalLaunchRequest::export_all(&config)
        .expect("export ExternalTerminalLaunchRequest");
    ExternalTerminalLaunchResult::export_all(&config).expect("export ExternalTerminalLaunchResult");
    EnvPreview::export_all(&config).expect("export EnvPreview");
    EnvEntry::export_all(&config).expect("export EnvEntry");

    // MVP-10 Phase B · Telemetry contract（ADR-015）
    CrashReportPayload::export_all(&config).expect("export CrashReportPayload");
    TelemetryOptInRequest::export_all(&config).expect("export TelemetryOptInRequest");
    TelemetryStatus::export_all(&config).expect("export TelemetryStatus");
    AppVersionInfo::export_all(&config).expect("export AppVersionInfo");

    // MVP-06 Phase B · Config Import IPC contract（spec §G.1 · 7 struct + 1 enum）
    ImportSource::export_all(&config).expect("export ImportSource");
    ImportFieldType::export_all(&config).expect("export ImportFieldType");
    ImportScanResult::export_all(&config).expect("export ImportScanResult");
    ImportPreview::export_all(&config).expect("export ImportPreview");
    ImportApplyRequest::export_all(&config).expect("export ImportApplyRequest");
    ImportApplyResult::export_all(&config).expect("export ImportApplyResult");
    KeyBindingConflict::export_all(&config).expect("export KeyBindingConflict");
    KeyBindingResolution::export_all(&config).expect("export KeyBindingResolution");

    // MVP-12 Phase A · Rail graph IPC contract（4 payload struct · ts-rs binding）
    RailGraphViewportSyncPayload::export_all(&config).expect("export RailGraphViewportSyncPayload");
    RailGraphBranchChangedPayload::export_all(&config)
        .expect("export RailGraphBranchChangedPayload");
    RailGraphRebaseStatePayload::export_all(&config).expect("export RailGraphRebaseStatePayload");
    RailGraphPerfSample::export_all(&config).expect("export RailGraphPerfSample");

    // MVP-17 Phase B · Pane Detach IPC contract（5 IPC struct + 1 event + 2 nested = 8 binding）
    PaneDetachOpenRequest::export_all(&config).expect("export PaneDetachOpenRequest");
    PaneDetachOpenResult::export_all(&config).expect("export PaneDetachOpenResult");
    PaneDetachCloseRequest::export_all(&config).expect("export PaneDetachCloseRequest");
    PaneDetachCloseResult::export_all(&config).expect("export PaneDetachCloseResult");
    PaneDetachListEntry::export_all(&config).expect("export PaneDetachListEntry");
    PaneDetachStateEvent::export_all(&config).expect("export PaneDetachStateEvent");
    PaneDetachAction::export_all(&config).expect("export PaneDetachAction");

    // 前端统一 import 入口（手工维护 · 防缺文件 · SPIKE-08 POC pattern）。
    fs::write(
        output_dir.join("index.ts"),
        [
            "// Generated by crates/app/build.rs from ts-rs derives in vibestation-core.",
            "// Do NOT edit manually. Run `cargo build` to regenerate.",
            "// Source of truth: crates/core/src/workspace.rs · layout.rs · tabs.rs · panes.rs · pty.rs · git_log.rs · diff.rs · git_status.rs.",
            "export type { WorkspaceMetadata } from \"./WorkspaceMetadata\";",
            "export type { LayoutState } from \"./LayoutState\";",
            "export type { TabState } from \"./TabState\";",
            "export type { TabCreateRequest } from \"./TabCreateRequest\";",
            "export type { TabCloseRequest } from \"./TabCloseRequest\";",
            "export type { TabRenameRequest } from \"./TabRenameRequest\";",
            "export type { TabReorderRequest } from \"./TabReorderRequest\";",
            "export type { TabListResponse } from \"./TabListResponse\";",
            "export type { PtyStdoutEvent } from \"./PtyStdoutEvent\";",
            "export type { PtyExitedEvent } from \"./PtyExitedEvent\";",
            "export type { PtySpawnRequest } from \"./PtySpawnRequest\";",
            "export type { SpawnResult } from \"./SpawnResult\";",
            "export type { PaneState } from \"./PaneState\";",
            "export type { PaneCreateRequest } from \"./PaneCreateRequest\";",
            "export type { PaneInitRequest } from \"./PaneInitRequest\";",
            "export type { PaneCloseRequest } from \"./PaneCloseRequest\";",
            "export type { LayoutNode } from \"./LayoutNode\";",
            "export type { SplitDir } from \"./SplitDir\";",
            "export type { LayoutApplyRequest } from \"./LayoutApplyRequest\";",
            "export type { SplitRatioUpdateRequest } from \"./SplitRatioUpdateRequest\";",
            "export type { PaneFocusRequest } from \"./PaneFocusRequest\";",
            "export type { PaneListResponse } from \"./PaneListResponse\";",
            "export type { PaneScrollbackFetchRequest } from \"./PaneScrollbackFetchRequest\";",
            "export type { PanePtySpawnRequest } from \"./PanePtySpawnRequest\";",
            "export type { PanePtyStdoutEvent } from \"./PanePtyStdoutEvent\";",
            "export type { PanePtyExitedEvent } from \"./PanePtyExitedEvent\";",
            // MVP-14 Phase A · Advanced layout bindings
            "export type { LayoutEnvelope } from \"./LayoutEnvelope\";",
            "export type { LayoutPresetKind } from \"./LayoutPresetKind\";",
            "export type { LayoutApplyAdvancedRequest } from \"./LayoutApplyAdvancedRequest\";",
            "export type { LayoutApplyResult } from \"./LayoutApplyResult\";",
            "export type { PaneNavDirection } from \"./PaneNavDirection\";",
            "export type { PaneNavigateRequest } from \"./PaneNavigateRequest\";",
            "export type { PaneNavigateResult } from \"./PaneNavigateResult\";",
            "export type { PaneMaximizeRequest } from \"./PaneMaximizeRequest\";",
            "export type { PaneMaximizeResult } from \"./PaneMaximizeResult\";",
            "export type { PaneResizeStepRequest } from \"./PaneResizeStepRequest\";",
            "export type { LayoutHistoryEntry } from \"./LayoutHistoryEntry\";",
            "export type { WorkspaceLayoutState } from \"./WorkspaceLayoutState\";",
            "export type { GitLogEntry } from \"./GitLogEntry\";",
            "export type { CommitAuthor } from \"./CommitAuthor\";",
            "export type { CommitParent } from \"./CommitParent\";",
            "export type { FileChange } from \"./FileChange\";",
            "export type { CommitDetail } from \"./CommitDetail\";",
            "export type { GitLogQueryRequest } from \"./GitLogQueryRequest\";",
            "export type { GitLogQueryResponse } from \"./GitLogQueryResponse\";",
            "export type { DiffRequest } from \"./DiffRequest\";",
            "export type { DiffResponse } from \"./DiffResponse\";",
            "export type { DiffHunk } from \"./DiffHunk\";",
            "export type { DiffLine } from \"./DiffLine\";",
            "export type { DiffLineType } from \"./DiffLineType\";",
            "export type { GitStatusRequest } from \"./GitStatusRequest\";",
            "export type { GitStatusResponse } from \"./GitStatusResponse\";",
            "export type { FileStatusEvent } from \"./FileStatusEvent\";",
            "export type { GitStatusPanelSettings } from \"./GitStatusPanelSettings\";",
            "export type { GitStatusCollapseRequest } from \"./GitStatusCollapseRequest\";",
            "export type { GitStatusGroup } from \"./GitStatusGroup\";",
            "export type { StageRequest } from \"./StageRequest\";",
            "export type { UnstageRequest } from \"./UnstageRequest\";",
            "export type { CommitRequest } from \"./CommitRequest\";",
            "export type { CommitResponse } from \"./CommitResponse\";",
            "export type { StageResult } from \"./StageResult\";",
            "export type { StageFailedItem } from \"./StageFailedItem\";",
            "export type { CommitError } from \"./CommitError\";",
            "export type { GitConfigIdentity } from \"./GitConfigIdentity\";",
            "export type { SetGitIdentityRequest } from \"./SetGitIdentityRequest\";",
            "export type { BranchKind } from \"./BranchKind\";",
            "export type { BranchInfo } from \"./BranchInfo\";",
            "export type { BranchListRequest } from \"./BranchListRequest\";",
            "export type { BranchListResponse } from \"./BranchListResponse\";",
            "export type { BranchCreateRequest } from \"./BranchCreateRequest\";",
            "export type { BranchCheckoutRequest } from \"./BranchCheckoutRequest\";",
            "export type { BranchDeleteRequest } from \"./BranchDeleteRequest\";",
            "export type { BranchSwitchResult } from \"./BranchSwitchResult\";",
            "export type { BranchError } from \"./BranchError\";",
            "export type { SwitcherQueryRequest } from \"./SwitcherQueryRequest\";",
            "export type { SwitcherMatch } from \"./SwitcherMatch\";",
            "export type { SwitcherSearchResult } from \"./SwitcherSearchResult\";",
            "export type { RemoteInfo } from \"./RemoteInfo\";",
            "export type { RemoteListRequest } from \"./RemoteListRequest\";",
            "export type { RemoteListResponse } from \"./RemoteListResponse\";",
            "export type { PushRequest } from \"./PushRequest\";",
            "export type { PushResult } from \"./PushResult\";",
            "export type { PullRequest } from \"./PullRequest\";",
            "export type { PullResult } from \"./PullResult\";",
            "export type { PullStrategy } from \"./PullStrategy\";",
            "export type { FetchRequest } from \"./FetchRequest\";",
            "export type { FetchResult } from \"./FetchResult\";",
            "export type { AuthMethod } from \"./AuthMethod\";",
            "export type { AuthRequest } from \"./AuthRequest\";",
            "export type { AuthChallenge } from \"./AuthChallenge\";",
            "export type { NetworkOpError } from \"./NetworkOpError\";",
            "export type { MergeConflictInfo } from \"./MergeConflictInfo\";",
            "export type { ConflictFile } from \"./ConflictFile\";",
            "export type { PushProgressEvent } from \"./PushProgressEvent\";",
            "export type { FetchProgressEvent } from \"./FetchProgressEvent\";",
            "export type { OperationDoneEvent } from \"./OperationDoneEvent\";",
            "export type { RebaseStartRequest } from \"./RebaseStartRequest\";",
            "export type { RebaseInteractivePlan } from \"./RebaseInteractivePlan\";",
            "export type { RebaseInteractiveStep } from \"./RebaseInteractiveStep\";",
            "export type { RebaseOp } from \"./RebaseOp\";",
            "export type { RebaseStatus } from \"./RebaseStatus\";",
            "export type { RebaseControlRequest } from \"./RebaseControlRequest\";",
            "export type { RebaseOpError } from \"./RebaseOpError\";",
            "export type { MergeRequest } from \"./MergeRequest\";",
            "export type { MergeStrategy } from \"./MergeStrategy\";",
            "export type { MergeStatus } from \"./MergeStatus\";",
            "export type { CherryPickRequest } from \"./CherryPickRequest\";",
            "export type { CherryPickStatus } from \"./CherryPickStatus\";",
            "export type { ConflictedFile } from \"./ConflictedFile\";",
            "export type { ConflictHunk } from \"./ConflictHunk\";",
            "export type { ConflictResolution } from \"./ConflictResolution\";",
            "export type { ConflictResolveFileRequest } from \"./ConflictResolveFileRequest\";",
            "export type { ConflictHunkResolution } from \"./ConflictHunkResolution\";",
            "export type { CrashRecoveryState } from \"./CrashRecoveryState\";",
            "export type { AppSettings } from \"./AppSettings\";",
            "export type { SettingsUpdateRequest } from \"./SettingsUpdateRequest\";",
            "export type { ShellInfo } from \"./ShellInfo\";",
            // MVP-17 Phase A · Pop to External contract
            "export type { ExternalTerminalInfo } from \"./ExternalTerminalInfo\";",
            "export type { ExternalTerminalLaunchRequest } from \"./ExternalTerminalLaunchRequest\";",
            "export type { ExternalTerminalLaunchResult } from \"./ExternalTerminalLaunchResult\";",
            "export type { EnvPreview } from \"./EnvPreview\";",
            "export type { EnvEntry } from \"./EnvEntry\";",
            "export type { CrashReportPayload } from \"./CrashReportPayload\";",
            "export type { TelemetryOptInRequest } from \"./TelemetryOptInRequest\";",
            "export type { TelemetryStatus } from \"./TelemetryStatus\";",
            "export type { AppVersionInfo } from \"./AppVersionInfo\";",
            // MVP-06 Phase B · Config Import contract
            "export type { ImportSource } from \"./ImportSource\";",
            "export type { ImportFieldType } from \"./ImportFieldType\";",
            "export type { ImportScanResult } from \"./ImportScanResult\";",
            "export type { ImportPreview } from \"./ImportPreview\";",
            "export type { ImportApplyRequest } from \"./ImportApplyRequest\";",
            "export type { ImportApplyResult } from \"./ImportApplyResult\";",
            "export type { KeyBindingConflict } from \"./KeyBindingConflict\";",
            "export type { KeyBindingResolution } from \"./KeyBindingResolution\";",
            // MVP-12 Phase A · Rail graph IPC contract
            "export type { RailGraphViewportSyncPayload } from \"./RailGraphViewportSyncPayload\";",
            "export type { RailGraphBranchChangedPayload } from \"./RailGraphBranchChangedPayload\";",
            "export type { RailGraphRebaseStatePayload } from \"./RailGraphRebaseStatePayload\";",
            "export type { RailGraphPerfSample } from \"./RailGraphPerfSample\";",
            // MVP-17 Phase B · Pane Detach IPC contract
            "export type { PaneDetachOpenRequest } from \"./PaneDetachOpenRequest\";",
            "export type { PaneDetachOpenResult } from \"./PaneDetachOpenResult\";",
            "export type { PaneDetachCloseRequest } from \"./PaneDetachCloseRequest\";",
            "export type { PaneDetachCloseResult } from \"./PaneDetachCloseResult\";",
            "export type { PaneDetachListEntry } from \"./PaneDetachListEntry\";",
            "export type { PaneDetachStateEvent } from \"./PaneDetachStateEvent\";",
            "export type { PaneDetachAction } from \"./PaneDetachAction\";",
            "export type { DetachedWindowBounds } from \"./DetachedWindowBounds\";",
            "",
        ]
        .join("\n"),
    )
    .expect("write bindings index");
}
