<!--
  Task Spec Template · 复制本文件并改名为 <TYPE>-NN-<slug>.md
  命名 / 状态 / 字段 / 新建流程详见 docs/tasks/README.md
  下方 frontmatter 字段 common schema 所有 TYPE 共用；正文 section 按 TYPE 填写差异化内容
-->

---

id: TYPE-NN # SPIKE-01 / MVP-03 / BUG-001 / FEAT-01
type: spike # spike | mvp | bug | feat
title: <中文标题，≤ 30 字>
status: draft # draft | ready | in-progress | blocked | done
owner: # 认领者标识（留空 = 未认领）
phase: W0-D1 # W0-D1 / W1 / W5 / v0.2 / ...
depends_on: [] # ["SPIKE-01", "MVP-03"]
blocks: [] # 该 task 完成后解锁哪些 task
blocked_by: [] # 仅 status: blocked 时必填：["SPIKE-02", "apple-dev-program-approval"]
blocked_from: # 仅 status: blocked 时必填：ready | in-progress（进入 blocked 前的状态，解除时自动恢复）
blocked_note: # 可选：人类可读的阻塞原因（1-2 句）
estimate: 1d # 0.5d / 1d / 3d / 1w
plan_ref: implementation-plan.md §N.N # 引用的战略计划章节
risk_ref: # R1 / R12 / R27（implementation-plan §9 风险 ID，可选）
reviewer: # 独立评审者（PR merge 前填写，必须 ≠ owner）

---

# <TYPE-NN>: <中文标题>

> **状态**：`draft` → `ready` → `in-progress` → `done`
> **依赖**：`depends_on` / **阻塞**：`blocks`
> **战略依据**：[`implementation-plan.md §N.N`](../implementation-plan.md)

---

## 🎯 目标（Goal）

一句话：这个 task 要做什么。不写"怎么做"。

## 📖 背景（Context）

2-4 句：

- 为什么现在做？
- 上游 / 下游依赖是什么？
- 历史上有过什么尝试？（如果有）

---

<!-- ============================================== -->
<!-- SPIKE 专属 section（type: spike 填以下）      -->
<!-- ============================================== -->

## ✅ 通过标准（Pass Criteria）

**必须全部可量化**。示例：

- [ ] 冷启动耗时 macOS < 2s、Ubuntu Wayland < 3s、Ubuntu X11 < 3s
- [ ] IME（中文输入）在三平台均能正常工作（截图或录屏证据）
- [ ] 主线程阻塞事件数 ≤ 0 / 分钟
- [ ] tauri-plugin-clipboard / fs / updater 三个 plugin smoke test 通过

## ❌ 失败信号（Fail Signals）

触发 fallback 的具体条件（any of）：

- 冷启动 > 5s
- Wayland 崩溃率 > 10%
- 任一 plugin 无法加载

## 🔀 Fallback 方案

**通过** → 锁定 `<默认方案>`，写 ADR-XXX
**失败** → 启动 `<fallback 方案>`（1 天 spike），通过则切换，同步 `CLAUDE.md` 决策表从 B 栏 → A 栏

对应 `CLAUDE.md` 决策表 #N。

## 📦 产出（Deliverables）

> **强制**：4 样齐全 · accept 前主 agent 必须逐项归档完成。详细规则见 [`.claude/rules/spike-delivery-checklist.md`](../../.claude/rules/spike-delivery-checklist.md)。
>
> 原则：**accept 动作和归档动作是原子的**。`/tmp` 产出的代码 / raw 在 review 通过瞬间必须迁到 repo · 不允许跨 session 遗留。

- [ ] **(1) 决策文档**：`docs/spikes/<id>-report.md`（必进 git · 含结论 + 数据 + raw 索引）
- [ ] **(2) 实测源码**：`docs/spikes/code/<id>/`（必进 git · 含 `src/` + `Cargo.toml` + `Cargo.lock` + `README.md`）
  - Cargo.lock 进 git（版本冻结 · 保证可复现 · `.gitignore` 已加白名单 `!docs/spikes/code/**/Cargo.lock`）
  - README.md 必含：来源 agent / 产出时间 / 复现命令 / report 数据溯源
- [ ] **(3) Raw 数据**：`docs/spikes/raw/<id>/`（必进 git · JSON / log / txt + README 索引）
  - Report 里每个具体数字必须能在此溯源
- [ ] **(4) 冷备**：`spike-tmp/archive/<id>/`（gitignored · 本地保留 · 含 target/ build 产物 + 大测试文件 + v1 失败版本）
- [ ] **(5) 录屏 / 截图**（仅需 UI / IME 类 Spike）：`spike-artifacts/<id>/`（gitignored · 不入 repo）
- [ ] **(6) ADR**（仅有决策锁定的 Spike）：`docs/adr/ADR-XXX-<slug>.md` 新建或从 proposed → accepted

**独立评审必查项**（Arbiter / reviewer 对照 PR Test Plan 逐项 verify · 不通过即 block merge）：

- [ ] 4 样物料（1-4）全部在各自目录
- [ ] clone repo 后 `cd docs/spikes/code/<id> && cargo build` 能复现
- [ ] Report 所有数字可在 raw 文件溯源

## 🛠 依赖资源（Resources Needed）

- 硬件：macOS 15 / Ubuntu 24 Wayland / Ubuntu 24 X11 各 1 台
- 账号：（如 Apple Developer Program）
- 数据集：（如 linux kernel git 仓库 clone）

## ⚠️ 已知风险

- **Rn**（`implementation-plan.md §9`）：描述 + 影响
- 未知风险：Spike 过程中记录到 `docs/spikes/<id>-report.md`（per-task）

---

<!-- ============================================== -->
<!-- MVP / FEAT 专属 section（type: mvp | feat）    -->
<!-- ============================================== -->

## 🎨 功能范围（Scope）

**Do**：

- 列 1
- 列 2

**Don't**（显式排除，避免 scope creep）：

- 列 1
- 列 2

## 🖼 UI 引用（UI Reference）

- 原型：`design/directions/1-calm-studio.html` 区块 `#<anchor>` 或坐标描述
- 截图：`docs/tasks/assets/<id>/*.png`（Phase 3 建立 assets 目录）
- 关键交互：鼠标 hover / 快捷键 / 空状态

## ✅ Acceptance

evaluator 按此逐项对照 diff：

- [ ] 功能 A 在场景 X 下正确渲染
- [ ] 快捷键 `<Cmd+K>` 打开命令面板（macOS）/ `<Ctrl+K>`（Linux）
- [ ] 空数据状态显示引导文案
- [ ] 键盘导航（Tab / Shift+Tab / Enter / Esc）全部可达
- [ ] a11y：所有交互有 aria-label

## 🧪 测试策略

| 层次     | 范围                | 覆盖路径                            |
| -------- | ------------------- | ----------------------------------- |
| 单元     | `core/` Rust 逻辑   | `cargo test`                        |
| 集成     | IPC + rusqlite 交互 | `cargo test --features integration` |
| E2E      | 用户流程            | Tauri webdriver + Playwright        |
| 视觉回归 | Calm Studio 主视觉  | Playwright screenshot diff          |

## 💾 数据模型变更（如有）

- rusqlite table：`<name>`，key: `<type>`，value: `<type>`
- 迁移策略：backward-compat / break with migration script

---

<!-- ============================================== -->
<!-- BUG 专属 section（type: bug）                  -->
<!-- ============================================== -->

## 🐛 复现步骤（Reproduction Steps）

1. 在 macOS 15 启动 app
2. 打开 2 个 Tab
3. 在 Tab 2 输入 `yes` 并回车
4. ...

## 🎯 期望行为 vs ❌ 实际行为

**期望**：Tab 切换流畅，Tab 1 不受 Tab 2 `yes` 影响
**实际**：Tab 1 渲染卡顿 500ms+

## 🔬 根因分析（Root Cause）

- 初步假设：...
- 验证方式：...
- 实际根因：...（修复后补）

## ✔️ 修复验证（Fix Verification）

- [ ] 复现步骤不再触发 bug
- [ ] 新增回归测试 `<path>`
- [ ] `cargo test` + `pnpm test` 全通过

---

## 📝 Notes / 讨论

（实施过程中的关键决策、踩坑、与其他 task 的联动，自由填写）

## 🔗 相关

- ADR：`docs/adr/ADR-XXX-<slug>.md`（如有）
- 对应 `CLAUDE.md` 决策表：#N
- 相关 PR：#NN
- 前置讨论：`docs/internal/session-history/<date>-<topic>.md`（Phase 3 后）

---

**填写完毕后自审**（CLAUDE.md "📝 写规则/清单前的自审四问"）：

1. **递归完备性**：字段都填了吗？Acceptance 可量化吗？
2. **反向场景**：做不到怎么办？有 Fallback 吗？
3. **边界适用性**：单机 / 多 agent / 所有平台都适用吗？
4. **YAGNI**：当前 Phase 真需要这么细吗？还是占位即可？

任一条答不清楚 → **spec 回到 draft，继续打磨**。
