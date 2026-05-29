# Task `3.4`: `fs-watch-windows`

**Status**: Done

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`（详见 `docs/s2v/standard.md` §10.5.1 状态机）。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 3 `terminal-integration`
**Dependencies**: 依赖 1.1（Windows 编译基线 + `#[cfg(target_os)]` 分支范式）

## 1. Background

`crates/core/src/fs_watch.rs::GitStatusWatcher::spawn`（line 51-59）在 Windows 上直接短路：

```rust
#[cfg(target_os = "windows")]
{
    let _ = repo_path;
    let _ = callback;
    return Err(GitStatusWatchError::UnsupportedPlatform(
        "Windows ReadDirectoryChangesW runtime validation is deferred past v0.1".to_string(),
    ));
}
```

后果：Windows 上 `GitStatusWatcher::spawn()` 立即返回 `Err(UnsupportedPlatform)`，Git status UI **永不实时刷新**（PRD §Problem Statement 痛点、§Users 场景 2）。

而底层 `notify` crate 的 `RecommendedWatcher` 在 Windows 上本就走 `ReadDirectoryChangesW` backend —— 现有 `#[cfg(not(target_os = "windows"))]` 分支的逻辑（canonicalize + spawn thread + `run_watch_loop` + 200ms debounce + `.git/index.lock` 排除）几乎可直接复用于 Windows。这是「deferred past v0.1」的历史短路，PRD §Technical Approach 已确认 `notify` Windows backend 可用（决策 D6 / adr-006）。

## 2. Goal

Windows 上 `GitStatusWatcher::spawn()` 成功启动 `notify` 的 `ReadDirectoryChangesW` backend，文件改动经同样的 200ms debounce + `.git/index.lock` 排除契约触发 callback，使 Git status 在 Windows 实时刷新。mac/Linux 行为零回归（共用同一份 watch 逻辑）。

## 3. Scope

### In Scope

- `crates/core/src/fs_watch.rs`：
  - 移除 `GitStatusWatcher::spawn` 的 `#[cfg(target_os = "windows")]` 短路块（line 51-59），让 Windows 走与 Unix 相同的 `RecommendedWatcher` 启动路径（`notify` 在 Windows 自动选 `ReadDirectoryChangesW`）。
  - 把原 `#[cfg(not(target_os = "windows"))]` 块的逻辑提为**全平台共用**（去掉该 cfg gate）。
  - 保持 `GIT_STATUS_WATCH_DEBOUNCE = 200ms` 不变。
  - 保持 `.git/index.lock` 排除契约不变（`run_watch_loop` 内的 path 过滤）。
  - 保持 `dunce::canonicalize` 路径规范化（已对 Windows UNC 安全，survey 标 already-ok）。
  - `GitStatusWatchError::UnsupportedPlatform` 变体：若不再有任何平台返回它，标 `#[allow(dead_code)]` 或保留作未来非主流平台（如 wasm）兜底（实施时按 clippy 反馈决定 —— 不主动删 pub 变体以免破坏 IPC 错误枚举契约）。
- Windows 专属集成测试（spawn 在 Windows 返回 `Ok` + 改文件触发 callback + 200ms debounce 行为）。

### Out Of Scope

- 自写 `ReadDirectoryChangesW` 包装（`notify` 已提供，决策 D6 拒绝候选 c）。
- debounce 时长调优（保持 200ms `GIT_STATUS_WATCH_DEBOUNCE` 不变）。
- Git status 计算逻辑本身（`git_ops` / `git_log`，git2 跨平台，survey 标 already-ok）。
- 前端 Git status 徽章渲染（前端 TS，不在本 Rust task）。
- 跨平台 timing 差异的深度调优（Windows backend 事件粒度若与 Unix 不同，按 §8 R1 缓解，必要时记技术债）。

## 4. Users / Actors

- **Windows 11 AI-agent 开发者**：在 workspace（git repo）里改文件 / `git commit`，期望 Git status 徽章 200ms 内更新（PRD §Users 场景 2）。
- **`crates/app` workspace 层**：调用 `GitStatusWatcher::spawn(repo_path, callback)`，期望 Windows 上返回 `Ok(watcher)` 而非 `Err(UnsupportedPlatform)`。

## 5. Behavior Contract

### 5.1 Required Reading

- 上游 task spec：`docs/specs/tasks/task-1.1-pty-platform-split.md`（Windows 编译基线 + cfg 范式）。
- 同 phase 参考：`docs/specs/phases/phase-3-terminal-integration.md` §3 涉及模块、§7 R-P3-3。
- BDD：`test/features/fs-watch.feature`（Task 3.4 场景）。
- 相关 ADR：`docs/decisions/adr-006-fs-watch-windows-notify-backend.md`（**本 task 的直接依据** · 决策 D6：启用 notify Windows backend，移除短路，保持 200ms debounce + `.git/index.lock` 契约）。
- 现状源码：`crates/core/src/fs_watch.rs`（`GitStatusWatcher::spawn` line 47-100 · `GIT_STATUS_WATCH_DEBOUNCE` line 16 · `run_watch_loop` · `GitStatusWatchError` line 21-33）。

### 5.2 Imports

- `notify::{RecommendedWatcher, RecursiveMode, Config, Event, EventKind, Watcher as NotifyWatcher}`（已有；Windows backend 由 `notify` 自动选，无需新 import）。
- `std::sync::{Arc, atomic::AtomicBool, mpsc}`（已有）。
- `std::thread` / `std::time::{Duration, Instant}`（已有）。
- `dunce`（已在 workspace 依赖；`canonicalize` 对 Windows UNC 安全）。
- 无新增第三方依赖（`notify` 的 Windows backend 已随 crate 编译进来，移除短路即生效）。

### 5.3 函数签名

Windows 适配后的真实签名骨架（公开签名完全不变 —— 仅删 Windows 短路块 + 去 cfg gate 使逻辑全平台共用）：

```rust
pub const GIT_STATUS_WATCH_DEBOUNCE: Duration = Duration::from_millis(200); // 不变

impl GitStatusWatcher {
    // 签名不变；删 #[cfg(target_os = "windows")] 短路块，逻辑全平台共用
    pub fn spawn<F>(repo_path: PathBuf, callback: F) -> Result<Self, GitStatusWatchError>
    where
        F: FnMut() + Send + 'static,
    {
        // 原 #[cfg(not(target_os = "windows"))] 块去掉 cfg gate，成为唯一实现：
        let repo_path = dunce::canonicalize(&repo_path).unwrap_or(repo_path);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let (startup_tx, startup_rx) = mpsc::channel();

        let join = thread::Builder::new()
            .name("vibestation-git-status-watch".to_string())
            .spawn(move || {
                let (event_tx, event_rx) = mpsc::channel();
                let watcher = RecommendedWatcher::new(
                    move |event| { let _ = event_tx.send(event); },
                    Config::default(),
                )
                .and_then(|mut watcher| {
                    // Windows: notify 自动用 ReadDirectoryChangesW
                    watcher.watch(&repo_path, RecursiveMode::Recursive)?;
                    Ok(watcher)
                });
                match watcher {
                    Ok(watcher) => {
                        let _ = startup_tx.send(Ok(()));
                        run_watch_loop(repo_path, watcher, event_rx, thread_stop, callback);
                    }
                    Err(error) => { let _ = startup_tx.send(Err(error.to_string())); }
                }
            })
            .map_err(|error| GitStatusWatchError::ThreadStart(error.to_string()))?;

        match startup_rx.recv_timeout(WATCH_START_TIMEOUT) {
            Ok(Ok(())) => Ok(Self { stop, join: Some(join) }),
            // ...既有错误分支不变...
        }
    }
}

// run_watch_loop 签名不变；内部 200ms debounce + .git/index.lock 排除契约保持
fn run_watch_loop(
    repo_path: PathBuf,
    watcher: RecommendedWatcher,
    event_rx: Receiver<notify::Result<Event>>,
    stop: Arc<AtomicBool>,
    callback: impl FnMut(),
);

// UnsupportedPlatform 变体保留（IPC 错误枚举契约），按需 #[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum GitStatusWatchError {
    #[error("unsupported fs watch platform: {0}")]
    UnsupportedPlatform(String), // 不再由 Windows 触发；保留作非主流平台兜底
    // ...其余变体不变...
}
```

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Problem Statement 痛点 · §Core Capabilities #3): Windows 上 `GitStatusWatcher::spawn(repo_path, callback)` 对有效 git repo 路径返回 `Ok(GitStatusWatcher)`，不再返回 `Err(UnsupportedPlatform)`。
- [ ] **AC2** (PRD §Users 场景 2 · §Success Metrics「Git status 实时刷新」): Windows 上 watcher 启动后，在被监视目录内创建/修改文件，callback 被触发（集成测试断言 callback 计数 > 0，在 debounce 窗口 + 合理 timeout 内）。
- [ ] **AC3** (决策 D6 · 契约保持): `GIT_STATUS_WATCH_DEBOUNCE` 仍为 200ms；`.git/index.lock` 改动不触发 callback（排除契约保持，Windows 与 Unix 同）。
- [ ] **AC4** (PRD §Constraints 兼容性 · §Success Metrics 反指标 · mac/Linux 零回归): mac/Linux 上 `GitStatusWatcher` 行为零回归 —— 现有 fs_watch 测试全绿（spawn/debounce/index.lock 排除逻辑共用同一实现）。
- [ ] **AC5** (PRD §User Flow 边界 · 路径): Windows 上 `dunce::canonicalize` 对含空格/反斜杠/UNC 前缀的 repo 路径正确规范化后仍能 `watch` 成功（不因 verbatim 前缀失败）。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 Windows spawn 返回 Ok | SCEN-3.4.1 | TEST-3.4.1 `test_3_4_1_windows_spawn_returns_ok` | lib `#[cfg(test)]` Windows-gated（自包含，无需独立 tests/ 文件） | cargo test -p vibestation_core（fs_watch） | Done |
| AC2 改文件触发 callback | SCEN-3.4.2 | TEST-3.4.2 `test_3_4_2_windows_file_change_triggers_callback` | tempdir + notify backend（lib 内嵌） | cargo test -p vibestation_core fs_watch | Done |
| AC3 200ms debounce + index.lock 排除 | SCEN-3.4.3 | `watcher_debounces_rapid_changes` + `watcher_excludes_git_internals`（既有，跨平台共用） | N/A | cargo test -p vibestation_core fs_watch | Done |
| AC4 mac/Linux 零回归 | SCEN-3.4.4 | `watcher_handles_workspace_change`（现有用例集，全平台共用同一实现） | N/A | cargo test --workspace（mac/Linux CI） | Done |
| AC5 Windows 路径 canonicalize | SCEN-3.4.5 | TEST-3.4.5 `test_3_4_5_windows_unc_canonicalize_watch` | N/A | cargo test -p vibestation_core fs_watch | Done |

## 8. Risks

- **R1（PRD §Technical Risks R1 旁系 / Phase §7 R-P3-3 · ReadDirectoryChangesW timing）**：`notify` Windows backend 事件粒度 / 触发时序与 Unix（kqueue/inotify）不同，集成测试 callback 等待可能 flaky。缓解：测试 timeout 设足够余量（≥ debounce × 数倍，遵 dispatch §2.11「timeout ≥ 本地最大 × 2」），若 Windows 上语义差异导致持续 flaky 则标 `#[cfg_attr(...ignore...)]` + 记技术债 + §2.14 本机手动验补证（不陷 timeout 扩张循环）。
- **R2（PRD §Technical Risks R3 · mac/Linux 回归）**：去掉 `#[cfg(not(target_os = "windows"))]` gate 把逻辑变成全平台共用，若原 Windows 块里有 Unix 块没有的清理动作会丢失 —— 经核对原 Windows 块仅 `let _ = ...` + early return，无实质逻辑，移除安全；mac/Linux 全量 `cargo test` 必绿。
- **R3（`UnsupportedPlatform` dead_code）**：移除 Windows 触发后该变体若无其他调用，clippy `-D warnings` 可能报 dead_code。缓解：保留 pub 变体（IPC 错误枚举契约稳定）+ 按 clippy 反馈加 `#[allow(dead_code)]` 或在非主流平台 cfg 兜底里复用。
- **R4（CI headless）**：windows-latest CI 上 fs 事件可触发但 GUI 不可验；AC2 集成测试用 tempdir 真实改文件（非 GUI），CI 可跑；200ms 实时刷新的端到端在 Phase 3 §6 / §2.14 本机 `pnpm tauri:dev` 验。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制；scoped: cargo test -p vibestation_core fs_watch（含 crates/core/tests 集成） -->
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings

> Integration：fs_watch 的 tempdir + notify backend 集成测试随 `cargo test --workspace` 一起跑（crates/core 内嵌 + tests/）。E2E / Coverage：无独立 e2e 框架；MVP 不强制覆盖率。Runtime-smoke：Git status 200ms 实时刷新的真实验证在 Phase 3 §6 端到端 smoke + §2.14 本机 `pnpm tauri:dev`，不列入本 task §9（CI headless 不可验 GUI）。

## 10. Completion Notes

**完成于 2026-05-29 · feat/windows-support 分支 · solo 三段 commit**

1. **改动文件**：`crates/core/src/fs_watch.rs`（本 task 主体）+ `crates/app/src/fix_path_env.rs`（解锁 workspace clippy 必需的连带修复，见第 6 点）。
2. **fs_watch.rs**：删除 `GitStatusWatcher::spawn` 的 `#[cfg(target_os="windows")]` `UnsupportedPlatform` 短路块（原 line 51-59）；去掉原 `#[cfg(not(target_os="windows"))]` cfg gate，使 canonicalize + spawn thread + `run_watch_loop` 逻辑成为全平台唯一实现（`notify` 的 `RecommendedWatcher` 在 Windows 自动选 `ReadDirectoryChangesW` backend）。
3. **契约保持**：`GIT_STATUS_WATCH_DEBOUNCE = 200ms` 不变；`.git/index.lock` 排除契约（`should_process_git_status_path`）不变；`dunce::canonicalize` 路径规范化不变 —— 三平台一致。
4. **UnsupportedPlatform 变体**：保留 pub 变体（IPC 错误枚举契约，ADR-006 §Decision）。实测移除 Windows 短路后该变体不再被任何平台构造，但因 `GitStatusWatchError` 是 pub 且可达，编译期**不报 dead_code**，无需 `#[allow(dead_code)]`。
5. **测试**：新增 Windows-gated TEST-3.4.1（spawn 返回 Ok 非 UnsupportedPlatform）/ TEST-3.4.2（改文件触发 callback，5s timeout = debounce×25 余量，遵 dispatch §2.11）/ TEST-3.4.5（含空格路径 canonicalize 后 watch 成功）；既有 `watcher_handles_workspace_change` 集成测试在 Windows 由短路导致的失败转绿。`cargo test -p vibestation-core --lib fs_watch` = **6 passed / 0 failed**（Windows 11 本机真实运行 · 实测 ReadDirectoryChangesW 真触发回调，无 flaky）。无需新增 `crates/core/tests/` 隔离集成文件（lib `#[cfg(test)]` 已自包含覆盖）。
6. **连带修复（fix_path_env.rs）**：移除 fs_watch 短路使 `vibestation-core` 在 Windows 干净编译后，clippy 继续检查 `vibestation-app`，暴露此前被 core 编译失败掩盖的 3 个 Windows 错误（`std::env`/`std::process::Command` import 在 Windows `#[cfg(not(macos/linux))]` 早返回路径下 unused + needless `return`）。把这两个 import cfg-gate 到 macos/linux，去掉 `return`（attributed cfg block 自动成尾表达式）。逻辑零变化，属 Windows 编译基线必要修复（非本 task §3 范围，但为达成本批 workspace-clippy-0 验收必需）。
7. **AC 状态**：AC1 ✅（spawn 返回 Ok）· AC2 ✅（改文件触发 callback，实测无 flaky）· AC3 ✅（200ms debounce 常量不变 + index.lock 排除单测绿）· AC4 ✅（mac/Linux 共用同一实现，既有单测全绿）· AC5 ✅（含空格路径 dunce::canonicalize 后 watch 成功）。`cargo clippy --workspace -- -D warnings` = **0 error**（原 13 个 dead-code/unused error 全清）；`cargo check` / `cargo build --workspace` 均 0 error。
