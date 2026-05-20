# MVP-17 Phase D · Pre-Capture 就绪体检

> **定位**：本文件是 MVP-17 spec §D Phase D Runtime 证据（macOS + Linux 双平台各 5 张截图 + 30s 录屏 + 内存释放量化）的**前置体检**——主 agent（CLI）能程序化验证的代码侧前置已全跑，结论固化在此。
> **它不是 capture 本身**：spec §H Phase D Acceptance + macOS / Linux 双平台 GUI 截图 + 30s 录屏 + Activity Monitor / `ps -o rss=` 内存量化设计上就是 Arbiter 本人完成。CLI agent 无法替代 GUI、无 Linux 环境。
> **用途**：Arbiter 跑 Phase D capture 窗口时，先读本文件 —— 代码侧已 green 的 cargo 单元 + a11y 代码就位部分不必重复跑；聚焦真正需要人的双平台 GUI 截图 + Detach 拖窗口录屏 + 内存释放测量。
>
> 体检执行：Claude Code 主 agent · 2026-05-20 · 镜像 MVP-19 #384 模式

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| 项                                                          | 验证方式                                                                                     | 结果                                                                                                                                                                            |
| ----------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **external_term 单元测试**（§A 终端识别）                   | `cargo test -p vibestation-core --lib 'external_term'`                                       | **45 passed · 0 failed**（含 detect 5 终端 + launch + env_filter 全覆盖 · macOS / Linux 双平台 priority 排序）                                                                  |
| **pane_detach 单元测试**（§C/§D Detach 生命周期）           | `cargo test -p vibestation-core --lib 'pane_detach::tests'`                                  | **19 passed · 0 failed**（含 attached_event_null_window_label / bounds_default / clear_removes_all / generate_window_label_format / insert_duplicate_returns_already_detached） |
| **Phase A 代码 done**（PR #291 · Codex CLI）                | 16 文件 + 1430/-1 · 11 ts-rs binding · macOS runtime dry-run 验证                            | external_term detect + launch + env_filter + cwd/env API + IPC + permission/capability + build.rs                                                                               |
| **Phase B 代码 done**（PR #285 skeleton + session 30 完成） | session 30 worktree `/private/tmp/MVP-17-phase-B-work` HEAD `55b1642`                        | state.rs + window_manager close listener + 5 integration tests + 3 张 runtime evidence                                                                                          |
| **Phase C / E.4 代码 done**（PR #292 / #294 / #301 / #302） | `pnpm --filter @vibestation/web exec vitest run tests/panels/Settings/ExternalTerminalGroup` | UI + IPC wrapper + 6 test files 重写 33 vitest 全过 · settings UI + 右键菜单 + ⌘⇧O / ⌘⇧D / Reattach 全 wire                                                                     |
| **既有 evidence 部分归档**（Phase B/E.4）                   | `ls docs/runtime-evidence/mvp-17/phase-{b-lifecycle,e4}/`                                    | phase-b-lifecycle/ 含 dev-mode-blocker.raw.log + README · phase-e4/ 含 3 张 settings UI 截图（external-terminal-group expanded / preferred dropdown / dont-ask-again toggle）   |

### Phase A/B/C/E.4 代码完成度

| Phase     | 范围                                                    | 状态                                                        | PR                                                                                                                      |
| --------- | ------------------------------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Phase A   | Pop to External 终端识别 + 启动                         | ✅ done                                                     | [#291](https://github.com/tajiaoyezi/vibestation/pull/291) Codex CLI                                                    |
| Phase B   | Pane Detach WebviewWindow 生命周期                      | ✅ done（skeleton #285 + session 30 完成）                  | [#285](https://github.com/tajiaoyezi/vibestation/pull/285) skeleton + session 30                                        |
| Phase C   | UI + 快捷键 + 右键菜单 + 集成                           | ✅ done（#292 源码 + #294 fix-up + session 30 vitest 重写） | [#292](https://github.com/tajiaoyezi/vibestation/pull/292) · [#294](https://github.com/tajiaoyezi/vibestation/pull/294) |
| Phase E.4 | Settings UI follow-up + 6 test files describe.skip 重写 | ✅ done                                                     | [#301](https://github.com/tajiaoyezi/vibestation/pull/301) · [#302](https://github.com/tajiaoyezi/vibestation/pull/302) |
| Phase D   | runtime 证据 + GUI capture                              | 🟡 **本文件**待 Arbiter                                     | —                                                                                                                       |

---

## ⚠️ 关键 gap 预警

### gap-1 · Phase D 双平台 5+5 截图 + 30s 录屏 + 内存量化未捕获

**坐实**：`ls docs/runtime-evidence/mvp-17/` = 仅 `phase-b-lifecycle/` + `phase-e4/`（**主目录 0 张 Phase D 截图 / 0 段录屏 / 0 内存数据**）。

**影响**：spec §H Phase D Acceptance 翻 done 判据 = macOS + Linux 双平台各 5 张截图 + 30s 录屏 + 内存释放量化 · 当前 0/3 项到位。

**不是 gap 是 deferred**：spec 明确「Phase D · runtime 证据 + GUI capture · ⏳ deferred（Arbiter 自定时机 · 类似其他 v0.3 sprint MVP）」。需 Arbiter 启 Phase D capture 窗口（macOS 单平台预计 30 min · Linux + 双平台预计 60 min · 取决于是否本会话覆盖 Linux）。

### gap-2 · Linux 平台 capture CLI agent 无环境

CLI agent 在 macOS 本机运行 · 无法在本会话内完成 Linux 平台 5 张截图。需 Arbiter Ubuntu VM 或 GitHub Actions runner 跑 · 或 defer 至 v0.3 sprint Phase D batch 同步 Linux verification 窗口。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 spec [`MVP-17 §H Phase D Acceptance`](../../tasks/MVP-17-external-terminal-pane-detach.md)：

1. **macOS 5 张截图**（Phase A + B + C 覆盖）：
   - 01-pop-external-dialog.png · Phase A 终端选择对话框 + env 预览（5 终端 priority 列表 · 含 Ghostty / iTerm2 / Terminal.app / Alacritty / 自定义）
   - 02-detached-window.png · Phase B detached WebviewWindow 显示 + mini-toolbar（pane_id + workspace 名 + Reattach 按钮）
   - 03-context-menu.png · Phase C 右键菜单 "Pop to External" / "Detach Pane" 两项
   - 04-detached-placeholder.png · 原位置 "Detached · click to bring back" placeholder
   - 05-multi-detached.png · 多 detached pane 共存 + reattach 还原

2. **Linux 5 张截图**（同 macOS 5 张 + Linux 终端 detect 验证）：
   - 同 macOS 列表 · 但 Phase A 终端 priority = Ghostty / Alacritty / GNOME Terminal / Konsole / 自定义

3. **30s 录屏**（macOS 优先）：
   - Detach → 拖窗口到另一屏 → 关闭还原（spec §H Acceptance 关键流程）

4. **内存释放量化**（spec §H runtime evidence）：
   - 关闭 detached 后 ≤ 10MB 残留（通过 Activity Monitor 或 `ps -o rss= -p <pid>` 测）
   - 测量方法：detach 前 baseline · detach 后 + close · 30s 后 diff

5. **PR + R1-R5**：`docs/runtime-evidence/mvp-17/phase-d/01-*.png` ... `docs/runtime-evidence/mvp-17/phase-d/{macos,linux}/` 分平台子目录 · `rollback-flow.mov` · 单文件 ≤ 500 KB · 总目录 ≤ 10 MB

---

## 结论

MVP-17 Phase D 验收项中：

- **代码侧 external_term 45 + pane_detach 19 = 64 passed · 0 failed**（Phase A/B/C/E.4 全 done · 6 test files 重写 33 vitest 全过）✅
- **既有 Phase B-lifecycle + E.4 evidence 部分归档**（3 张 settings UI 截图 + lifecycle log + README）✅
- **Phase D 双平台 10 张截图 + 30s 录屏 + 内存量化 0 项到位**（spec 明确 deferred · Arbiter 30-60 min capture 窗口）🔴
- **Linux 平台 CLI agent 无环境**（需 Arbiter Ubuntu VM 或 defer 至 v0.3 sprint batch · 同 MVP-12/14/15/16 一起）🔴

MVP-17 spec 维持 `ready`（Phase A/B/C/E.4 代码 done · Phase D capture 待 Arbiter）。

**关联**：spec [`docs/tasks/MVP-17-external-terminal-pane-detach.md`](../../tasks/MVP-17-external-terminal-pane-detach.md) §A/§B/§C/§D/§E/§F/§H · PR #291 Phase A · #285 Phase B skeleton · #292/#294 Phase C · #301/#302 Phase E.4 · `.claude/rules/runtime-evidence-location.md` R1-R5 · 模板镜像 [MVP-19 PRE-CAPTURE-READINESS](../mvp-19/PRE-CAPTURE-READINESS.md) PR #384
