# Windows 端到端 Smoke 证据索引（task-6.2)

> Windows 适配工作流 `feat/windows-support` · Phase 6 integration-matrix · task-6.2 windows-smoke-matrix。
> 本目录归档 Windows 本机（H:\ Windows 11）`pnpm tauri:dev` critical UX path 的 runtime-smoke 证据（截图 / 录屏 / 日志）。
> 机械可验证部分（tauri.conf bundle target / 三平台 cargo test / 跨平台基元）由 `crates/app/tests/windows_smoke_matrix.rs` 聚合断言；
> GUI critical UX path（需窗口交互）按 §2.14 本机实跑并在此归档。

---

## Critical UX Path Smoke Checklist（Phase 6 §6 端到端 smoke 第 3 条）

本机 H:\ Windows 11 · `pnpm tauri:dev` 起窗后逐条跑：

- [ ] **S1 · 起窗**：`pnpm tauri:dev` 起窗成功 · 无 panic / 白屏。
- [ ] **S2 · 新建 Tab + ConPTY**：新建 Tab → 经 ConPTY 拉起默认 shell（pwsh→powershell→cmd 探测链 · ADR-003）→ prompt 可见。
- [ ] **S3 · 命令回显**：在 Tab 内输入 `git --version`（或 `echo`）→ 回显正确 · 无乱码 / 残帧。
- [ ] **S4 · Git status 200ms 刷新**：在 workspace 内改一个文件 → Git status 徽章 200ms 内刷新（fs_watch Windows backend · `GIT_STATUS_WATCH_DEBOUNCE`）。

证据归档：每条 path 的截图 / 录屏 / 控制台日志放本目录 · 并在对应 task-6.2 §10 runtime-smoke 引用。

---

## 自动化已覆盖（无需手动 · 进 CI / 本机 cargo test）

ConPTY spawn + echo + exit + signal-terminate 的进程级语义已由 task-2.2 集成测试
`crates/core/tests/pty_windows_conpty_integration.rs`（`#![cfg(windows)]` · 真 spawn `cmd.exe`）自动化证明：

- `test_2_2_1_conpty_spawn_cmd_reads_prompt`：spawn → 读到非空 prompt（不 hang）。
- `test_2_2_2_conpty_echo_roundtrip`：写 echo → 读到回显（= S3 命令回显的进程级证据）。
- `test_2_2_3_conpty_detects_process_exit_no_hang`：写 `exit` → 检测退出 emit Exited（不 hang）。
- `test_2_2_4_signal_terminate_kills_conpty_child`：signal/kill 终止子进程 emit Exited。

故 S2/S3 的"ConPTY 拉起 + 回显"链路已有自动化兜底；本目录手动证据补充 GUI 渲染层（xterm.js 显示 / Git 徽章视觉刷新）这一 headless 测不到的部分。

---

## 三平台测试矩阵状态（task-6.2 §10 汇总）

| 平台 | cargo test --workspace | vitest run | 验证主体 |
|---|---|---|---|
| Windows 11（本机 H:\）| ✅ 0 panicked（Unix-only ignored）| ✅ | 本机实跑（task-6.1 门控基线）|
| macOS | ⏳ defer CI matrix / reviewer | ⏳ defer | reviewer / CI（task-5.2 windows-latest 矩阵 + mac/Linux leg）|
| Ubuntu | ⏳ defer CI matrix / reviewer | ⏳ defer | reviewer / CI |

> mac/Linux 本机在 Windows 开发机上跑不了 · 三平台零回归（AC4 / AC-P6.4）defer 到 CI matrix（task-5.2）/ reviewer 复跑 · 与 Phase 前基线比对。
