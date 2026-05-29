# ADR `006`: fs_watch 在 Windows 启用 notify backend

**Status**: Accepted
**Date**: 2026-05-29
**Category**: 协议接口
**Related**: PRD §Decisions Log D6

## Context

`crates/core/src/fs_watch.rs` 的 `GitStatusWatcher::spawn()` 在 Windows 上直接短路返回错误（确认于源码 行 51–59）：

```rust
#[cfg(target_os = "windows")]
{
    return Err(GitStatusWatchError::UnsupportedPlatform(
        "Windows ReadDirectoryChangesW runtime validation is deferred past v0.1".to_string(),
    ));
}
```

后果：Windows 上 Git status 永不实时刷新 —— 在 pwsh Tab `git commit` 后，Git Log / status 徽章不会自动更新（survey 标 high · PRD 用户场景 2 "Git 工作台日常" 的核心诉求落空）。

关键事实：

- 非 Windows 分支（行 61+）已用 `notify` crate 的 `RecommendedWatcher`（确认于源码），`notify` 本就跨平台、Windows backend 即 `ReadDirectoryChangesW`（survey + adapter `crates/core/Cargo.toml` 确认 notify 支持 Windows）。
- 现有契约：200ms debounce（`GIT_STATUS_WATCH_DEBOUNCE`）+ `.git/index.lock` 排除 + `dunce::canonicalize(&repo_path)` 路径规范化。
- 当初 Windows 禁用是 "ReadDirectoryChangesW runtime validation deferred past v0.1" —— 该 defer 前提（mac+Linux 主线打磨透、Windows 适配启动）现已成立。

本 ADR 的"协议接口"性质：`GitStatusWatcher::spawn()` 是 fs watch → Git status 刷新的事件接口，Windows 从"恒返回 `UnsupportedPlatform`"变为"返回真实 watcher"，是该接口对外行为的改变（错误契约 → 正常事件流）。

## Decision

**移除 `fs_watch.rs` 的 Windows `UnsupportedPlatform` 短路，启用 `notify` 的 Windows backend（`ReadDirectoryChangesW`）**，保持现有契约不变：

- 删除行 51–59 的 `#[cfg(target_os = "windows")]` 早返回，让 Windows 走与 Unix 同一条 `RecommendedWatcher` 路径（`notify` 自动选 Windows backend）。
- **契约保持不变**：200ms debounce（`GIT_STATUS_WATCH_DEBOUNCE`）+ `.git/index.lock` 排除 + `dunce::canonicalize` 路径规范化 —— 三平台一致。
- 复用成熟的 `notify` backend，不自写 `ReadDirectoryChangesW` 包装。
- `GitStatusWatchError::UnsupportedPlatform` 变体保留（其他失败路径仍可能用），仅不再在 Windows 默认返回它。

## Rationale

- **复用成熟 backend、契约不变**：`notify` 已跨平台封装 `ReadDirectoryChangesW`，启用即得，无需自写；200ms debounce + index.lock 排除契约三平台统一，行为可预期。
- **Windows 体验对等**：Git status 不实时 = Windows 体验降级（候选 a），与 PRD "与 mac/Linux 对等" 的愿景冲突。
- **不重复造轮子**：自写 `ReadDirectoryChangesW` 包装（候选 c）重复 `notify` 已做的事，徒增维护与 bug 面。

## Alternatives

- **(a) 保持 Windows 禁用**：拒绝 —— Git status 不实时刷新 = Windows 体验降级，违背"与 mac/Linux 对等"愿景。
- **(c) 自写 `ReadDirectoryChangesW` 包装**：拒绝 —— 重复造轮子（`notify` 已跨平台），增加维护负担与潜在 bug。
- **(b) 启用 `notify` Windows backend**（**选定**）：复用成熟 backend，契约（debounce / index.lock 排除）不变。

## Consequences

**正面**：

- Windows workspace 内改文件，Git status 徽章 200ms 内更新（PRD Success Metric "Git status 实时刷新" + 用户场景 2）。
- 三平台 fs watch 走同一 `notify` 路径，行为契约统一，减少平台分叉维护。
- mac/Linux 路径零变化（它们本就用 `RecommendedWatcher`，只是 Windows 之前被短路）。

**负面 / 风险**：

- `ReadDirectoryChangesW` 的事件语义 / 触发时机与 Unix inotify/FSEvents 可能有差异（重命名/批量写的事件粒度），可能影响 debounce 后的刷新及时性。缓解：本机 Windows 11 实跑 smoke（改文件 → 观察 200ms 内徽章刷新）；保留 `.git/index.lock` 排除避免 git 自身写入抖动。
- 路径分隔符 / UNC 网络盘 watch（PRD R4 + 边界场景）：`dunce::canonicalize` 已在 spawn 前规范化 repo_path（源码确认），UNC 去 verbatim 前缀；网络盘 `ReadDirectoryChangesW` 可能不可靠 —— 本机 smoke 优先验本地盘，UNC 作边界记录。
- 该接口行为变更（错误 → 事件流）需配套 BDD 场景（`test/features/fs-watch.feature`）+ Windows 专测，确认 Windows 真触发回调（关联 ADR-005）。

## Rollback Or Migration Plan

- **回滚**：恢复行 51–59 的 `#[cfg(target_os = "windows")]` 早返回即退回 "Windows 不支持 fs watch" 状态；前端已能处理 `GitStatusWatchError`（既有错误路径），回滚无破坏性。不改 DB schema、不改前端 IPC 事件名（仍是同一 Git status 刷新事件）。
- **迁移**：无数据迁移。Windows 用户从"无实时刷新"变为"有实时刷新"是纯增量行为，无既有状态需迁移；debounce/排除契约不变，已习惯 mac/Linux 行为的用户在 Windows 得到一致体验。

## Follow-ups

- task-3.4（fs-watch-windows）落地移除 Windows 短路 + 本机 smoke 验证（改文件 → 200ms 徽章刷新）。
- task-6.1（windows-test-gating）补 Windows fs_watch 专测（确认 Windows 触发回调，关联 ADR-005）。
- `test/features/fs-watch.feature` 补 Windows 场景（业务可读，Scenario ID 映射到测试）。
- UNC / 网络盘 watch 可靠性作边界记录，按反馈决定是否需特殊处理（关联 PRD R4）。
- 关联 ADR-001（PTY 编译解锁是 Windows 跑起来的前置）/ ADR-002（home_dir 解析正确的 workspace 根才能 watch 对路径）。
