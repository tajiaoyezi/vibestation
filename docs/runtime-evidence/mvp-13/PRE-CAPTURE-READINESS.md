# MVP-13 Phase D · Pre-Capture 就绪体检

> **定位**：本文件是 MVP-13 spec §D Phase D Runtime 截图 / 录屏 + 性能量化的**前置体检**——主 agent（CLI）能程序化验证的代码侧前置已全跑，结论固化在此。
> **它不是 capture 本身**：spec §D Phase D 截图（CRUD 三大操作 / dirty tree 对话框 / Fuzzy Switcher 全键盘流程）+ 30s 录屏设计上就是 Arbiter 本人通过 `pnpm tauri:dev` 实跑 + screencapture 抓 GUI 截图 + 录屏，CLI agent 无法替代。
> **用途**：Arbiter 跑 Phase D capture 窗口时，先读本文件 —— 代码侧已 green 的 cargo 单元 / Criterion bench / vitest 不必重复跑；聚焦真正需要人的 GUI 截图 + 全键盘流程录屏。
>
> 体检执行：Claude Code 主 agent · 2026-05-20 · 镜像 MVP-19 #384 模式

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| 项                                     | 验证方式                                                   | 结果                                                                                                                                       |
| -------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| **branch_ops 单元测试**                | `cargo test -p vibestation-core --lib 'branch_ops::tests'` | **32 passed · 0 failed · 0 ignored**（含 branch_list detached_head / non_repo error / local_remote_tag / CRUD 全覆盖 · §B/§C/§D 验证）     |
| **Criterion bench Linux 基线已 done**  | `crates/core/benches/branch_bench.rs` + PR #226            | branch_list_10/list **1.06 ms** · 1000 branch fixture bench 已跑 · `docs/runtime-evidence/mvp-13/bench-output.txt` 已归档                  |
| **Phase A/B/C/D 全 4 phase 代码 done** | spec §I.0 · PR #220-#226 序列                              | A 后端 git2 + IPC done · B Primary Sidebar 分支树 done · C Fuzzy Switcher modal + `⌘B` keydown done · D 性能 bench + capture skeleton done |
| **既有 bench-output.txt evidence**     | `cat docs/runtime-evidence/mvp-13/bench-output.txt`        | branch_list_10 1.06ms / 1000 branch fixture · Linux Criterion `target/release` 数据完整                                                    |

---

## ⚠️ 关键 gap 预警

### gap-1 · Phase D GUI 截图 + 30s 录屏未捕获

**坐实**：`ls docs/runtime-evidence/mvp-13/` = 仅 `bench-output.txt`（**0 张截图 / 0 段录屏**）。

**影响**：spec §D Phase D 翻 done 判据 = 性能量化（done）+ GUI 截图 / 录屏（未到位）。当前 1/2 项就绪 · 仅缺 GUI capture。

**不是 gap 是 deferred**：当前索引描述「全 4 phase done · GUI capture deferred · 自动化 100%」与本体检一致。需 Arbiter 启 Phase D capture 窗口（预计 15-20 min · 4-5 张截图 CRUD + dirty tree + Fuzzy Switcher）。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 spec [`MVP-13 §D Phase D Runtime 证据`](../../tasks/MVP-13-branch-crud.md)：

1. **`pnpm tauri:dev`** 启动应用 · 准备 workspace 含 10/100 branch fixture
2. **建议 4-5 张 PNG**（spec §D 未硬 mandate 张数 · 推荐覆盖 §A/§B/§C/§D 全部接受标准）：
   - 01-branch-create.png · 新建分支 modal（name + from + create and checkout · §A.1）
   - 02-branch-checkout.png · 切换分支 · dirty tree 对话框（§B 流程）
   - 03-branch-delete.png · 删除分支二次确认（§C）
   - 04-fuzzy-switcher-open.png · `⌘B` 触发 Fuzzy Switcher modal · 输入过滤示例（`fpt` 匹配 `feat/pty-pool` · §D.3 subsequence + `<mark>` 高亮）
   - 05-fuzzy-switcher-keyboard.png · ↑↓ 选择 / Enter checkout 全键盘流程（§D.5）
3. **可选 30s 录屏**：完整 create → checkout → delete → fuzzy switcher 流程
4. **PR + R1-R5**：`docs/runtime-evidence/mvp-13/01-*.png` ... 顺序前缀 · 单文件 ≤ 500 KB · 总目录 ≤ 3 MB · PR body Test Plan 必含「Runtime 证据已提交到 `docs/runtime-evidence/mvp-13/` · 含 N 张截图」

### Multi-workspace 隔离 + 中文分支名（手动 QA）

- spec §E.3 多 workspace 隔离：切 workspace 时分支树状态 per-workspace（fuzzy switcher 历史也是 per-workspace）· capture 时可选验
- spec §F 测试矩阵手动 QA：macOS 中文分支名（`feat/中文-test`）· Linux 不同 fs（ext4 / btrfs）case-sensitivity · 推荐至少手测 1 个中文分支名 capture

---

## 结论

MVP-13 Phase D 验收项中：

- **代码侧 branch_ops 32 passed + Criterion Linux 基线已 done**（性能量化完整 · `bench-output.txt` 归档）✅
- **GUI 截图 / 录屏 0 张**（spec 明确 deferred · Arbiter 15-20 min capture 窗口）🔴

MVP-13 spec 维持 `ready`（Phase A/B/C/D 代码 done · 性能 bench done · 仅缺 GUI capture）。

**关联**：spec [`docs/tasks/MVP-13-branch-crud.md`](../../tasks/MVP-13-branch-crud.md) §A/§B/§C/§D/§E.3/§F · PR #220-#226 序列 · `crates/core/benches/branch_bench.rs` Linux 基线 · `bench-output.txt` 归档 · `.claude/rules/runtime-evidence-location.md` R1-R5 · 模板镜像 [MVP-19 PRE-CAPTURE-READINESS](../mvp-19/PRE-CAPTURE-READINESS.md) PR #384
