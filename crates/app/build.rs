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
    AppSettings, AppVersionInfo, CommitAuthor, CommitDetail, CommitError, CommitParent,
    CommitRequest, CommitResponse, CrashReportPayload, DiffHunk, DiffLine, DiffLineType,
    DiffRequest, DiffResponse, FileChange, FileStatusEvent, GitConfigIdentity, GitLogEntry,
    GitLogQueryRequest, GitLogQueryResponse, GitStatusCollapseRequest, GitStatusGroup,
    GitStatusPanelSettings, GitStatusRequest, GitStatusResponse, LayoutApplyRequest, LayoutNode,
    LayoutState, PaneCloseRequest, PaneCreateRequest, PaneFocusRequest, PaneInitRequest,
    PaneListResponse, PanePtyExitedEvent, PanePtySpawnRequest, PanePtyStdoutEvent,
    PaneScrollbackFetchRequest, PaneState, PtyExitedEvent, PtySpawnRequest, PtyStdoutEvent,
    SetGitIdentityRequest, SettingsUpdateRequest, ShellInfo, SplitDir, SplitRatioUpdateRequest,
    StageFailedItem, StageRequest, StageResult, TabCloseRequest, TabCreateRequest, TabListResponse,
    TabRenameRequest, TabState, TelemetryOptInRequest, TelemetryStatus, UnstageRequest,
    WorkspaceMetadata,
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
    println!("cargo:rerun-if-changed=../core/src/app_settings.rs");
    println!("cargo:rerun-if-changed=../core/src/telemetry.rs");

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

    AppSettings::export_all(&config).expect("export AppSettings");
    SettingsUpdateRequest::export_all(&config).expect("export SettingsUpdateRequest");
    ShellInfo::export_all(&config).expect("export ShellInfo");

    // MVP-10 Phase B · Telemetry contract（ADR-015）
    CrashReportPayload::export_all(&config).expect("export CrashReportPayload");
    TelemetryOptInRequest::export_all(&config).expect("export TelemetryOptInRequest");
    TelemetryStatus::export_all(&config).expect("export TelemetryStatus");
    AppVersionInfo::export_all(&config).expect("export AppVersionInfo");

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
            "export type { TabListResponse } from \"./TabListResponse\";",
            "export type { PtyStdoutEvent } from \"./PtyStdoutEvent\";",
            "export type { PtyExitedEvent } from \"./PtyExitedEvent\";",
            "export type { PtySpawnRequest } from \"./PtySpawnRequest\";",
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
            "export type { AppSettings } from \"./AppSettings\";",
            "export type { SettingsUpdateRequest } from \"./SettingsUpdateRequest\";",
            "export type { ShellInfo } from \"./ShellInfo\";",
            "export type { CrashReportPayload } from \"./CrashReportPayload\";",
            "export type { TelemetryOptInRequest } from \"./TelemetryOptInRequest\";",
            "export type { TelemetryStatus } from \"./TelemetryStatus\";",
            "export type { AppVersionInfo } from \"./AppVersionInfo\";",
            "",
        ]
        .join("\n"),
    )
    .expect("write bindings index");
}
