# MVP-04 Phase D · Pre-Capture 就绪体检

> **定位**：本文件是 MVP-04 spec §I 22 用例 + 22 张截图 + 2 段录屏的**前置体检**——主 agent（CLI）能程序化验证的代码侧前置已全跑，结论固化在此。
> **它不是 capture 本身**：spec §I.4 的 22 用例中 15 个 ignored 是 spec 明确归类「Tab 补全交互 / xterm 灰字渲染 / CLI login + LLM endpoint」类（cargo test 层无法稳定模拟），设计上就是 Arbiter 本人通过 `pnpm tauri:dev` 实跑 + `screencapture` / `screencapture -V` 实际抓 22 张 JPG + 2 段 MP4，归档到 `docs/runtime-evidence/mvp-04/phase-d/<case>.{jpg,mp4}`。CLI agent 无法替代，也不得编造。
> **用途**：Arbiter 跑 Phase D capture 窗口时，先读本文件 —— 代码侧已 green 的 cargo test 不必重复跑；聚焦真正需要人的 15 个运行态截图 + Phase F 录屏。
>
> 体检执行：Claude Code 主 agent · 2026-05-20 · 镜像 MVP-19 #384 模式

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| 项                                                            | 验证方式                                                               | 结果                                                                                                                                                                                                                                                                                                            |
| ------------------------------------------------------------- | ---------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **§I shell_compat 22 用例 cargo 化**                          | `cargo test --manifest-path crates/app/Cargo.toml --test shell_compat` | **7 passed · 15 ignored · 0 failed**（active 部分 = §I.1 macOS shell resolve / IPC roundtrip / §I.2 CLI 默认 args 验证）                                                                                                                                                                                        |
| **Phase A/B/C/E/F 代码全 done**                               | git log + spec §I.0                                                    | Phase A code（migration v5 + tabs + IPC）· Phase B PR #82（PtyManager + tab*pty*\* IPC）· Phase C PR #91（前端 xterm tab）· Phase E PR #95（scrollback_append/fetch）· Phase F PR #99（5 PNG + metrics-phase-f.md 已归档）                                                                                      |
| **既有 evidence 已归档（5 PNG + metrics + 多 phase 子目录）** | `ls docs/runtime-evidence/mvp-04*`                                     | 主目录 5 PNG（01-create-loading-card / 02-rename-tab / 03-switch-scrollback / 04-close-tab / 05-performance-overlay）+ metrics-phase-f.md · 子目录 phase-b（cargo-test.log + tauri-dev-smoke.gif）+ phase-c（5 PNG）+ phase-e（4 PNG）+ storage-prep（migration-idempotency-proof + schema-diff + test-output） |

### 15 ignored 用例的归类（**不是漏验** · spec §I.4 明确归到运行态范畴）

按 spec §I.4 说明，15 ignored 用例 cargo test 层无法稳定模拟（Tab 补全交互需 PTY readline · xterm 灰字渲染需 webview · CLI login + LLM endpoint 需真实 API key），**必须**靠 `pnpm tauri:dev` 手动验证 + 截图归档：

- §I.1 默认 shell 矩阵 ignored 部分：3 shell × 「Tab 补全」交互验证（zsh / bash / fish 各 1 case · fish 缺失时 silent skip）
- §I.2 CLI 矩阵 ignored 部分：Claude CLI + Codex CLI 各 4-5 case 涉及 xterm 渲染（ANSI / OSC52 / 灰字 placeholder）+ 真实 LLM endpoint（需 API key · CI 环境无）

**Linux ignored 沿袭**（spec §I.3 + Phase D Ubuntu blocked）：所有真实 PTY case 在 Linux 标 `#[cfg_attr(target_os = "linux", ignore)]`（根因 PR #82 / #86 实证：GitHub Actions Ubuntu runner mio epoll 对 PTY close event timing 不稳定 · macOS kqueue 稳定）· Ubuntu Phase D 后续 runtime 验证窗口统一深挖。

---

## ⚠️ 关键 gap 预警

### gap-1 · Phase D 22 PNG + 2 MOV 完全未归档

**坐实**：`ls docs/runtime-evidence/mvp-04/phase-d/` = **目录不存在**。Phase D 22 张 JPG + 2 段 MP4 完全未捕获（vs Phase F 5 PNG 已归档 · 这是不同的子集）。

**影响**：spec §I.0 列明「Phase D 整体 done 翻转判据：本 §I.0 + 22 张截图 + 2 录屏 全到位」· 当前 §I.0 cargo 7 passed/15 ignored 已 OK · 但 22 张截图 + 2 录屏未到位 · Phase D 仍不能翻 done。

**不是 gap 是 deferred**：这是 spec **明确设计**的 deferred capture（§I.4 描述了"必须靠 `pnpm tauri:dev` 手动验证 + 截图归档"），不是 bug 或遗漏。需 Arbiter 启 Phase D capture 窗口（预计 60-90 min · 22 case × 2-3 min/case + 2 段录屏）。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 spec [`MVP-04 §I.4 测试执行流程`](../../tasks/MVP-04-multi-tab-terminal.md)：

1. **`pnpm tauri:dev`** 启动应用 · 准备 zsh + bash + fish（若本机有）shell 环境
2. **§I.1 默认 shell 矩阵**（12 case · 实跑 3 shell × 4 测试项 · 每 case 独立 Tab）：
   - resolve / shell-pid / Tab 补全 / 灰字渲染（Tab 补全 + 灰字渲染 = 15 ignored 中 6 个 · 必手捕）
3. **§I.2 CLI 实机矩阵**（10 case · Claude CLI + Codex CLI 各 5 测试项 · 每 case 独立 Tab）：
   - default args / login flow / LLM endpoint / OSC52 强制 / ANSI 渲染（剩余 15 ignored 中 9 个 · 必手捕）
4. **22 张 JPG**：`screencapture -V <wid> docs/runtime-evidence/mvp-04/phase-d/<shell>_<NN>_<case>.jpg`
5. **2 段 MP4**：`screencapture -V 30 -x docs/runtime-evidence/mvp-04/phase-d/<flow>.mov`（覆盖 §I.4 Tab 补全交互流 + CLI login 流）
6. **fish 缺失策略**：本机无 fish 时 cases 09 / 12 走 `eprintln! + return` silent skip · 不阻塞（spec §I.5）· 但截图描述应注明「fish skipped on local machine」

### Ubuntu Phase D 后续补（spec §I.3）

所有 §I.1 / §I.2 用例 · Ubuntu 平台标 **deferred**（v0.1 macOS-first GA 后再补）· 触发条件：Ubuntu VM / GitHub Actions runner 上深挖 mio epoll PTY close event timing 不稳定根因（PR #82 / #86 历史 workaround 移除时机）· 不阻塞 macOS-first v0.1.0-alpha 发布。

---

## 结论

MVP-04 Phase D 验收项中：

- **代码侧 22 用例全 active 部分 PASS**（7 passed + 15 ignored 明确归到运行态范畴 · 0 failed）✅
- **既有 evidence 完整归档**（Phase B/C/E/F 全有截图 / log / 数据）✅
- **22 张 JPG + 2 段 MP4 完全未捕获**（spec 明确 deferred · 不是 gap · 需 Arbiter 60-90 min capture 窗口）🔴
- **Ubuntu Phase D 推 v0.1 GA 后**（spec §I.3 明确）🟡

MVP-04 spec 维持 `ready`（capture 待 Arbiter）。当前索引状态描述「A/B/C/E/F done；D shell 兼容历史已收口；§I 22 PNG + 2 MOV 属 deferred capture」与本体检结论一致。

**关联**：spec [`docs/tasks/MVP-04-multi-tab-terminal.md`](../../tasks/MVP-04-multi-tab-terminal.md) §I.0/§I.1/§I.2/§I.3/§I.4 · `crates/app/tests/shell_compat.rs`（22 case 1:1 映射）· `.claude/rules/runtime-evidence-location.md` R1-R5 · 模板镜像 [MVP-19 PRE-CAPTURE-READINESS](../mvp-19/PRE-CAPTURE-READINESS.md) PR #384
