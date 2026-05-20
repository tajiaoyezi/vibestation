# MVP-09 Phase D · Pre-Capture 就绪体检

> **定位**：本文件是 MVP-09 spec §D Phase D Runtime 截图 / 录屏的**前置体检**——主 agent（CLI）能程序化验证的代码侧前置已全跑，结论固化在此。
> **它不是 capture 本身**：spec §I.5 Phase D 截图 / 录屏 + §D Criterion bench 性能量化（已 done）+ §E 集成测试（已 done）设计上就是 Arbiter 本人通过 `pnpm tauri:dev` 实跑 + screencapture 抓 stage/unstage/commit/amend/错误流截图，CLI agent 无法替代，也不得编造。
> **用途**：Arbiter 跑 Phase D capture 窗口时，先读本文件 —— 代码侧已 green 的 cargo/vitest + 性能 bench 不必重复跑；聚焦真正需要人的 stage/unstage/commit GUI 截图 + 错误流（pre-commit hook fail / DetachedHead / IdentityMissing）人眼验证。
>
> 体检执行：Claude Code 主 agent · 2026-05-20 · 镜像 MVP-19 #384 模式

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| 项                                       | 验证方式                                                  | 结果                                                                                                                                                                                                              |
| ---------------------------------------- | --------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **git_ops 单元测试**                     | `cargo test -p vibestation-core --lib 'git_ops::tests::'` | **12 passed · 0 failed · 0 ignored**（含 commit_creates_new_sha · commit_fails_no_staged_files · amend_modifies_last · stage/unstage 路径）                                                                       |
| **Phase A/B/C 代码全 done**              | git log + spec §I.0                                       | A PR #116 后端 git2 写路径 + IPC + ts-rs binding · B PR #118 Status 面板 stage/unstage + CommitBar · C PR #159 消息 composer + amend + identity dialog + detached HEAD + pre-commit hook stderr + Git Log refresh |
| **Phase D · §D Criterion bench 已 done** | PR #156 · `crates/core/benches/git_ops_bench.rs`          | Linux 基线 stage **0.26ms** / commit **0.35ms** / stage_1k **31.5ms** · 远低于 spec §D 性能要求（stage < 100ms · commit < 500ms · stage_1k < 2000ms · **余量 380× / 1400× / 63×**）                               |
| **Phase D · §E 集成测试已 done**         | PR #156                                                   | 含 pre-commit hook fail · DetachedHead · IdentityMissing 全覆盖（错误流 3 类全验证）                                                                                                                              |
| **既有 evidence 部分归档**               | `ls docs/runtime-evidence/mvp-09/`                        | `linux/` 子目录存在（推测含 Linux Criterion bench raw output · 验 #156 性能数字溯源）                                                                                                                             |

---

## ⚠️ 关键 gap 预警

### gap-1 · Phase D 截图 / 录屏完全未捕获

**坐实**：`ls docs/runtime-evidence/mvp-09/` = 仅 `linux/` 子目录（**0 张截图 / 0 段录屏 in 主目录**）。

**影响**：spec §D Phase D 翻 done 判据 = 性能量化（done）+ §E 集成测试（done）+ **runtime 截图 / 录屏**（未到位）。当前 2/3 项已就绪 · 仅缺 GUI capture。

**不是 gap 是 deferred**：spec §I.0 明确「Phase A/B/C 全 done · Phase D 性能量化 done（PR #156）· 仅缺 Phase D runtime 截图 / 录屏（GUI capture · Arbiter 本地）。所有 spec acceptance 项除 GUI evidence 外都已满足。spec status 翻 done 需等 GUI evidence 补齐后由 Arbiter approve。」需 Arbiter 启 Phase D capture 窗口（预计 15-20 min · 4-6 张截图 stage/unstage/commit/amend/错误流）。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 spec [`MVP-09 §D Phase D Runtime 证据`](../../tasks/MVP-09-stage-unstage-commit.md)：

1. **`pnpm tauri:dev`** 启动应用 · 准备 workspace 含 unstaged changes / staged changes / no identity / detached HEAD 等 fixture
2. **建议 4-6 张 PNG**（spec §D 未硬 mandate 张数 · 推荐覆盖 spec §C 全部接受标准）：
   - 01-stage-files.png · Status 面板单文件 stage（§B.1）+ 批量 stage（§B.2）
   - 02-unstage-files.png · 单文件 unstage（§B.1）+ 批量 unstage（§B.2）
   - 03-commit-bar.png · CommitBar 消息框 + 提交按钮 enable / disable（§C.1）
   - 04-amend-toggle.png · Amend toggle 状态 + previous commit message 预填（§C.2）
   - 05-identity-dialog.png · IdentityMissing 弹 dialog 提示设置 user.name/email（§C.3）
   - 06-precommit-hook-fail.png · pre-commit hook 失败 stderr 显示 + "Copy" 按钮（§C.4）+ detached HEAD warning（§C.3）
3. **可选 30s 录屏**：完整 stage → commit → Git Log refresh 流程
4. **PR + R1-R5**：`docs/runtime-evidence/mvp-09/01-*.png` ... 顺序前缀 · 单文件 ≤ 500KB · 总目录 ≤ 3 MB · PR body Test Plan 必含「Runtime 证据已提交到 `docs/runtime-evidence/mvp-09/` · 含 N 张截图」

### v0.2 规划项不阻塞当前 capture

- spec §C.3 标 **Detached HEAD commit v0.1 不支持**：Phase A 后端 `git_ops.rs` 检测 `!head.is_branch()` 直接返回 `CommitError::DetachedHead` · `CommitRequest` 无 `allow_detached` 字段 · v0.1 前端降级为单按钮提示 · v0.2 规划：后端加 `allow_detached: bool` + `Repository::commit` 跳过 HEAD 分支检查（需权衡 · detached HEAD commit 会 orphan）。本 Phase D 不验 v0.2 路径。

---

## 结论

MVP-09 Phase D 验收项中：

- **代码侧 git_ops 12 passed · 0 failed · §D Criterion bench Linux 基线已 done · §E 集成测试已 done**（性能量化 + 错误流 3 类全 ready）✅
- **GUI 截图 / 录屏 0 张**（spec 明确 deferred · Arbiter 15-20 min capture 窗口）🔴

MVP-09 spec 维持 `ready`（Phase A/B/C 代码 done · Phase D 性能 + §E 集成测试 done · 仅缺 GUI capture）。

**关联**：spec [`docs/tasks/MVP-09-stage-unstage-commit.md`](../../tasks/MVP-09-stage-unstage-commit.md) §B/§C/§D/§E · PR #116 Phase A · PR #118 Phase B · PR #159 Phase C · PR #156 Phase D 性能 + §E 集成 · `crates/core/benches/git_ops_bench.rs` Linux 基线 · `.claude/rules/runtime-evidence-location.md` R1-R5 · 模板镜像 [MVP-19 PRE-CAPTURE-READINESS](../mvp-19/PRE-CAPTURE-READINESS.md) PR #384
