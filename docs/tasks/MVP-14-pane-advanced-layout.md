---
id: MVP-14
type: mvp
title: Pane 高级布局（任意嵌套 + Dual AI / Triple / Quad + 导航 + 最大化）
status: ready
owner:
phase: v0.3
depends_on: ["MVP-05"]
blocks: ["MVP-17"]
blocked_by: []
blocked_from:
blocked_note:
estimate: 7d
plan_ref: implementation-plan.md §10.1 · §5.3 · §5.3.5 · §5.3.9
risk_ref: 本 spec §已知风险 R1-R5（递归渲染性能 / JSON 版本化 / 小屏拖拽 / PTY instance 复用 / 焦点几何算法）
reviewer: Codex CLI
---

# MVP-14: Pane 高级布局

> **状态**：`draft`（v0.2 ready-candidate · spec 已详化 · 等 Arbiter approve 后由主 agent 翻 `ready`）
> **依赖**：MVP-05（单层 Pane 分屏 · done · PR #141-#151 序列已落地 Pane storage / IPC / 前端递归渲染基础）
> **下游 blocks**：MVP-17（Pane Detach v0.3 · 需要 LayoutNode tree 与焦点/实例复用规则稳定）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md) v0.2 Pane 扩展 · [`§5.3`](../implementation-plan.md) Pane 系统 · [`§5.3.5`](../implementation-plan.md) 快捷键 · [`§5.3.9`](../implementation-plan.md) 实现风险
> **详化时间**：2026-05-06 session 24 · Codex CLI self-review（单人项目 v2-D.2 模式）

---

## 🎯 目标（Goal）

在 MVP-05 的单层 Pane 分屏基础上，解锁 v0.2 高级 Pane 布局：任意嵌套 LayoutNode tree（硬上限 5 层）、Dual AI / Triple Review / Quad 三个 Smart Layout 预设、方向键跳相邻 Pane、`⌘Enter` 临时最大化当前 Pane，并保证已有 PTY instance 不因预设切换或临时最大化被误销毁。

本 spec 对齐 `implementation-plan.md §10.1` 中从 v0.1 砍到 v0.2 的 Pane 项，落点是“升级现有 MVP-05 Pane 系统”，不是新建第二套布局引擎。未来实施 agent 的起点是 `crates/core/src/panes.rs`、`crates/core/src/pane_service.rs`、`web/src/panels/Terminal/PaneSplitView.tsx`、`PaneSplitter.tsx`、`SmartLayoutMenu.tsx`。

## 📖 背景（Context）

- **战略位置**：`implementation-plan.md §10.1` 明确 v0.2 增量包含 “Pane 任意嵌套 / Dual AI / Triple Review / Quad 预设 / 方向键跳邻居 / ⌘Enter 最大化”。MVP-14 是 v0.2 Pane 扩展主 task。
- **上游现状**：MVP-05 已落地 `LayoutNode` tagged union、`PaneState`、`PanesDao`、`pane_split` / `pane_close` / `pane_focus` / `pane_layout_apply` / `pane_split_ratio_update` IPC、Pane PTY 独立命名空间、SolidJS `PaneSplitView` 递归组件与 `PaneSplitter` 拖拽。
- **当前限制**：`crates/core/src/panes.rs` 当前硬编码 `MAX_LAYOUT_SPLIT_DEPTH = 2` 与 `MAX_LAYOUT_PANES = 4`，并通过 `validate_mvp_05()` 拒绝 3 横 / 3 竖等多层布局。MVP-14 负责放开为 5 层硬上限并补足 UI / 数据 / 性能验证。
- **CLAUDE.md 锁定**：Pane 系统必须沿用 Tauri 2 + SolidJS + rusqlite + ts-rs 的既有路径；禁止绕开 source-of-truth binding，也禁止把 v1.0 内部路线图项写成对外卖点。
- **设计锚点**：`design/directions/1-calm-studio.html` 已有 Pane system CSS、Smart Layouts radiogroup、Pane actions、splitter hover / hit-area、`⌘Enter` maximize button 图标与布局 header。MVP-14 复用视觉语言并扩展交互。
- **路线图位置**：`implementation-plan.md §11 W15` 写明 v0.2 Pane 扩展，目标是 “3 层嵌套不卡 60fps；预设切换 <100ms”。本 spec 把性能门槛扩展到深度 5 的自动化与手动 QA。

## 🛠 实施进度

MVP-14 估时 **7d**，拆 4 个 Phase 串行实施。Phase A 先放开 core contract 与数据迁移，Phase B 做前端递归 UI + 预设，Phase C 做导航/最大化/a11y，Phase D 交 runtime evidence 与性能量化。

| Phase | 范围 | 状态 | PR |
|-------|------|------|----|
| Phase A · LayoutNode v1 schema + core service | 放开 `MAX_LAYOUT_SPLIT_DEPTH` 到 5 · `LayoutEnvelope { version: 1, root }` · 迁移旧 `layout_state` / `tabs.layout` · 新增 advanced preset core pure functions · ts-rs bindings | ✅ done · PR #258 | — |
| Phase B · 递归 Pane UI + Smart Layouts 扩展 | `PaneSplitView` 递归渲染优化 · `SmartLayoutMenu` 增 Dual AI / Triple Review / Quad · 预设切换保留 Pane instance · nested splitter ratio 持久化 | ✅ done · PR #262 | — |
| Phase C · 键盘导航 + 临时最大化 + a11y | `⌘⌥ ←/→/↑/↓` / `Ctrl+Alt+Arrow` 几何相邻算法 · `⌘Enter` 临时最大化 · split divider keyboard resize · focus ring / ARIA | ⏳ todo | — |
| Phase D · runtime evidence + bench | 5 层嵌套性能 · 100 次 ratio drag · preset apply · keyboard nav · maximize restore · 4-7 张截图/录屏归档到 `docs/runtime-evidence/mvp-14/` | ⏳ todo | — |

**Phase A 起点 checklist**（让实施 agent 5 min 内启动）：

- [ ] 读 `crates/core/src/panes.rs` 当前 `LayoutNode` / `SplitDir` / `SmartLayoutKind` / `validate_mvp_05()` / `split_layout()` / `close_pane_in_layout()` / `update_split_ratio()` / `apply_smart_layout()`。
- [ ] 读 `crates/core/src/pane_service.rs` 当前 transaction 写法，保留 `tabs.layout + tabs.focused_pane_id + panes` 原子更新模式。
- [ ] 读 `crates/core/src/db.rs`，确认 `workspaces.layout_state` 与 `tabs.layout` 当前列名；本 spec 的 “workspace.layout” 指用户语义，实施层映射为 `workspaces.layout_state`。
- [ ] 新增 `LayoutEnvelope` v1，不破坏现有 `LayoutNode.ts` 生成；旧 `{"kind":"single","paneId":""}` 与 preset string 必须可迁移。
- [ ] 修改 core validator：`MAX_LAYOUT_SPLIT_DEPTH = 5`，ratio clamp 范围锁定 `[0.05, 0.95]`，pane count 从固定 4 改为由深度计算的安全上限。
- [ ] 扩展 `SmartLayoutKind`：`Solo` / `AiAndRunner` / `DualAi` / `TripleReview` / `Quad`，并保证 preset apply 优先复用已有 Pane instance。
- [ ] 新增或扩展 IPC commands：`pane_layout_apply` 支持新 preset；新增 `pane_navigate` / `pane_maximize` / `pane_resize_step`（详见 §G）。
- [ ] ts-rs binding 由 `crates/app/build.rs` 自动生成；前端只能 `import type`，禁止手写平行 interface。
- [ ] 单元 fixture 全在测试内生成，不依赖本地真实 workspace；沿用 MVP-05 `tempfile + rusqlite` 模式。
- [ ] Phase A 结束前跑：`cargo test --package vibestation-core panes::`、`cargo test --package vibestation-core pane_service::`、`cargo build -p vibestation-app`、`pnpm typecheck`。

## 🎨 功能范围（Scope）

**Do**：

- **任意嵌套 LayoutNode tree**：以现有 `LayoutNode::Single / Split` 为基础，支持最多 5 层 split depth；超过 5 层 backend 拒绝，frontend toast `Pane 嵌套已达 5 层上限`。
- **3 个新 Smart Layout 预设**：
  - Dual AI：`H(0.5, ClaudeCli, CodexCli)`
  - Triple Review：`H(0.5, ClaudeCli, V(0.5, Runner, Log))`
  - Quad：`H(0.5, V(0.5, A, B), V(0.5, C, D))`
- **预设切换保留内容**：按 Pane content identity 匹配复用现有 ClaudeCli / CodexCli / Shell / Runner / Log pane；只有缺失的 PaneType 才 spawn 新 pane。
- **多层 splitter ratio 调整**：每个 Split 节点独立持久化 ratio，鼠标拖拽用 RAF 节流，键盘 resize 用 `⌘⌃ Arrow` / `Ctrl+Alt+Shift Arrow` 5% 步进。
- **方向键导航**：`⌘⌥ ←/→/↑/↓`（mac）与 `Ctrl+Alt+←/→/↑/↓`（Linux）从当前 focus leaf 跳到几何相邻 leaf。
- **临时最大化**：`⌘Enter` / `Ctrl+Enter` 把当前 focus Pane 临时全屏，其他 Pane 隐藏但不 unmount、不 kill PTY；再次按键恢复原 layout 与 ratio。
- **持久化与恢复**：layout 变化 debounce 500ms 写 sqlite；workspace / tab 切换恢复 layout、focused pane、maximized 状态（maximized 为 session state，不写 DB）。
- **无障碍**：所有 splitter 可 keyboard focus；Pane header button 有 aria-label；focus ring 与 200ms transition 可见；reduced-motion 下禁用 scale 动画。

**Don't**（明确不做）：

- **Pane Detach 独立窗口**：v0.3 MVP-17 范围；MVP-14 只预留 LayoutNode placeholder，不创建新 Window。
- **自定义预设保存到 TOML**：v0.3+；本 task 只做内置 5 个 preset（Solo / AI+Runner / Dual AI / Triple Review / Quad）。
- **v1.0 内部 Pane 联动项**：不实现、不在 UI 文案宣传；只保留现有 Runner pane，禁止把它扩展成跨 pane 自动反哺工作流。
- **重写 Terminal / PTY 架构**：不改 `pane_pty_*` 命名，不把 Pane PTY 合回 Tab PTY。
- **引入第三方布局库**：不引 `react-grid-layout` / `react-mosaic` / `golden-layout` / `split.js`；递归渲染和拖拽自实现。
- **多 Tab 之间拖拽 Pane**：跨 tab move / copy 推后；MVP-14 只在当前 tab 的 LayoutNode tree 内 split / close / preset。
- **无限嵌套**：5 层是硬上限；不提供 “高级用户关闭限制” 开关。
- **复杂非 CLI Pane 内容**：DiffViewer / LogFollower 若当前仓库还未完整落地，MVP-14 spec 只定义 PaneType 位置，不强行实现新内容面板。

## 🖼 UI 引用（UI Reference）

- **Pane system CSS**：`design/directions/1-calm-studio.html` line 390-526
  - `.pane-grid` 当前是单层 `grid-template-columns: 1fr 4px 1fr`；MVP-14 需改为递归 flex/grid 容器而不是固定三列。
  - `.pane.active` 用主色 top border；MVP-14 保留并扩展为 focus ring + temporary maximize ring。
  - `.pane-splitter::before` 已提供 8px 左右 hit-area；MVP-14 继续沿用，并增加垂直 splitter 的上下 hit-area。
  - `.layout-switch` 是 Smart Layouts 控件基座；MVP-14 添加 Dual AI / Triple Review / Quad active state。
- **Center header + Smart Layouts**：`design/directions/1-calm-studio.html` line 843-880
  - header chip 显示 `layout` 和当前 preset 名。
  - `layout-switch` radiogroup title 是 `Smart Layouts — 一键预设`。
  - 图标按钮已有 Solo / AI+Watch / AI+Test / Triple Review / Quad 形态；MVP-14 将 AI+Watch / AI+Test 合并为 AI+Runner，并补 Dual AI。
- **Pane action buttons**：`design/directions/1-calm-studio.html` line 885-973
  - Pane header actions 包含 split right、split down、maximize、close。
  - `Maximize · ⌘Enter` 图标已存在；MVP-14 负责把按钮接到真实 `pane_maximize`/frontend state。
- **Splitter visual**：`design/directions/1-calm-studio.html` line 977
  - `<div class="pane-splitter" title="拖拽调整 · 双击复位"></div>` 是现有语义；多层 splitter title 保持一致，并追加 keyboard hint tooltip。
- **Keyboard hint**：`design/directions/1-calm-studio.html` line 855-858
  - 当前原型仍写 `⌘D` / `⌘⇧D`；实施时必须按 `implementation-plan.md §5.3.5` 修订为 `⌘\` / `⌘⇧\` / `⌘⌃W`，避免和 Diff `⌘D` 冲突。
- **视觉反馈**：
  - focus pane：主色边框 + 200ms transition。
  - selected layout preset：主色文字 + `var(--bg-2)` 背景。
  - maximized pane：header 右侧显示 `maximized` chip；reduced-motion 时不做 scale 动画。

## ✅ Acceptance

### A. LayoutNode v1 + 任意嵌套

- [x] A.1 `LayoutNode` 继续使用 tagged union（`Single` / `Split`），新增 `LayoutEnvelope { version: 1, root: LayoutNode }` 后，旧 `LayoutNode` JSON 可无损包装为 v1 envelope；单元测试覆盖旧 JSON → 新 envelope round-trip。
- [x] A.2 backend validator 接受 1 / 2 / 3 / 4 / 5 层合法嵌套；fixture `layout_depth_5_alternating()` validate PASS，`layout_depth_6_alternating()` 返回 `PaneLayoutError::MaxDepthExceeded { max_depth: 5 }`。
- [x] A.3 ratio 统一 clamp 到 `[0.05, 0.95]`；`0.049` 与 `0.951` 在 backend 返回 `InvalidRatio`，frontend 拖拽时先 clamp 且 toast 不出现。
- [x] A.4 关闭 Pane 时父 Split 只剩 1 child → 自动折叠为 sibling subtree；测试 `H(A, V(B, C))` 删除 `B` 后得到 `H(A, C)`，ratio 保留父节点比例。
- [x] A.5 同向连续 split 不再因 MVP-05 单层限制被拒绝；测试 `H(A, H(B, C))` 在 v0.2 validator PASS，但深度仍计入 5 层上限。
- [x] A.6 空 layout / 缺失 pane id / duplicated pane id 均被 backend 拒绝，错误含具体 pane id，transaction 回滚后 `tabs.layout` 与 `panes` 行数不变。

### B. Smart Layouts 预设

- [ ] B.1 Smart Layouts 菜单显示 5 个 preset：Solo / AI+Runner / Dual AI / Triple Review / Quad；当前 preset 按 `layout-switch button.on` 高亮，tooltip 写清布局语法。
- [ ] B.2 Dual AI 应用后 layout 等价 `H(0.5, ClaudeCli, CodexCli)`；若已有 ClaudeCli 或 CodexCli Pane，复用原 pane id 和 PTY session id，测试 `pane_pty_stdout` 不重连。
- [ ] B.3 Triple Review 应用后 layout 等价 `H(0.5, ClaudeCli, V(0.5, Runner, Log))`；右侧上下 splitter 可独立拖动并持久化。
- [ ] B.4 Quad 应用后 layout 等价 `H(0.5, V(0.5, A, B), V(0.5, C, D))`；不足 4 个 Pane 时按当前 focus、现有 DFS 顺序、缺失默认 shell 的顺序补齐。
- [ ] B.5 预设切换前显示 dry-run：列出 reused / created / closed pane 数量；若将关闭正在运行的 Runner / Shell Pane，确认按钮为 destructive 样式，默认 Cancel。
- [ ] B.6 预设切换成功后 500ms 内写入 sqlite；重新打开同 workspace + tab，layout 与 focused pane 恢复为切换后的状态。

### C. 递归渲染与 splitter ratio

- [ ] C.1 `PaneSplitView` 渲染 5 层 nested fixture 时不产生 React/Solid key 警告；每个 leaf pane 的 DOM 有稳定 `data-pane-id`。
- [ ] C.2 递归渲染使用 `<For>` 渲染 child 列表，node props 用 `createMemo` 派生；DevTools Solid update 记录中拖动一层 splitter 时未触发无关 sibling pane body 重渲染。
- [ ] C.3 鼠标拖动任意层级 splitter 时，只有目标 Split ratio 更新；测试 `H(A, V(B, C))` 拖右侧垂直 splitter 不改变 root horizontal ratio。
- [ ] C.4 splitter 双击复位当前 Split 到 50/50；测试嵌套布局中双击内层 splitter 后仅内层 ratio 变为 `0.5`。
- [ ] C.5 小屏 `< 1024px` 下 splitter hit-area 仍为 8px；Playwright mobile viewport 点击 hit-area 中线 ±3px 均能触发 drag start。
- [ ] C.6 拖拽过程中用 RAF 节流，`pointermove` 100 次不会触发 100 次 sqlite 写；最终 `pointerup` 后 debounce 500ms 只写一次。

### D. 方向键导航

- [ ] D.1 macOS `⌘⌥ ←/→/↑/↓` 与 Linux `Ctrl+Alt+←/→/↑/↓` 均触发 `pane_navigate` 或 frontend geometry navigate；在 input/terminal 捕获组合键时不向 PTY 写入 escape sequence。
- [ ] D.2 几何相邻算法以当前 focused leaf 的 bounding box 为锚，向目标方向找重叠投影最大的 pane；测试 2×2 中从左上按 ↓ 到左下，不跳到右下。
- [ ] D.3 不跨越非相邻 splitter 误跳；测试 `H(A, V(B, C))` 中 A 按 ↓ 无目标则 no-op，A 按 → 到 B 或 C 中与 A 垂直中心更接近者。
- [ ] D.4 no-op 场景有 150ms acknowledged 闪动，不发 toast；测试最左 pane 按 ← 后 focused pane id 不变。
- [ ] D.5 focus 切换后 pane border 高亮 200ms transition，输入仍只进入新 focused pane；E2E 在目标 pane 输入 `pwd`，其他 pane 无新增 stdin。
- [ ] D.6 workspace / tab 切换后 active pane 恢复，方向键导航以新 tab 的 geometry 重新计算，不复用旧 tab DOMRect cache。

### E. `⌘Enter` 临时最大化

- [ ] E.1 当前 focused pane 按 `⌘Enter` / `Ctrl+Enter` 后进入临时最大化；其他 panes 隐藏但不 unmount，PTY stdout 事件仍继续进入 store。
- [ ] E.2 再按一次恢复原 LayoutNode、focused pane、所有 split ratio；测试 maximize 前后 `serde_json(layout)` 完全一致。
- [ ] E.3 最大化动画 < 200ms；reduced-motion 设置开启时跳过 scale transition 但状态切换仍正确。
- [ ] E.4 最大化期间 split / close / preset 按钮 disabled 或转为先恢复再执行，避免 hidden pane layout 被修改；点击 disabled 按钮 tooltip 说明 `退出最大化后可调整布局`。
- [ ] E.5 最大化期间 workspace switch / tab close 会清理 session-only maximized state；回到原 workspace 不保持 maximize，避免跨 workspace 残留。

### F. 持久化 / 迁移 / 多 workspace

- [x] F.1 `workspaces.layout_state` 中旧值 `solo` / `aiAndRunner` / `null` 自动迁移到 `LayoutEnvelope { version: 1 }`；迁移测试逐项断言输出 JSON。
- [x] F.2 `tabs.layout` 中 MVP-05 旧 tagged union（`kind: "single"` / `kind: "split"`）继续被读取；写回时统一输出 v1 envelope 或当前锁定格式，PR body 写明选择。
- [ ] F.3 layout 写入 debounce 500ms；连续拖拽 20 次 ratio 只产生 ≤ 2 次 DB write，测试用 fake timer 或 mock DAO 计数。（Phase C 范围）
- [ ] F.4 workspace A 和 workspace B 的 layout / focus / history 独立；切换 A→B→A 后 A 恢复原 nested layout。
- [ ] F.5 Layout history LRU 5 条：每次 preset / split / close / ratio pointerup 后记录摘要；超过 5 条丢最旧；不提供 UI 恢复入口，仅为 v0.3 debug / support 使用。
- [ ] F.6 JSON version 不认识（如 `version: 99`）时 fallback 到 Solo + toast `布局版本过新，已恢复默认布局`，原 JSON 备份到 log，不覆盖直到用户产生新 layout。

### G. 错误处理 / 边界

- [ ] G.1 超 5 层 split：backend 返回 `MaxDepthExceeded`，frontend toast 3s，layout tree 和 panes 表不变。
- [ ] G.2 预设切换无法复用必需 PaneType 且 spawn 失败：整个 operation 回滚，UI 显示具体失败 pane type，现有 PTY 不被关闭。
- [ ] G.3 关闭最后一个 Pane：仍走现有语义（关闭 Tab 或提示），MVP-14 不允许产生空 LayoutNode。
- [ ] G.4 split ratio 拖到 5% 以下：frontend clamp，backend 二次校验；视觉上 Pane 最小宽高不小于 160px x 120px，低于阈值时 splitter 停止移动。
- [ ] G.5 非 git workspace / 无 PTY workspace 下 Smart Layouts 仍可用于 Shell panes，不依赖 Git 模块。
- [ ] G.6 corrupted layout JSON：读取失败时 UI 进入 recoverable empty state，提供 `Reset layout` 按钮，点击后写 Solo。

### H. 性能 / runtime evidence

- [ ] H.1 5 层嵌套、16 leaf panes fixture 下首屏 render 到 pane headers 全部可见 < 100ms（Playwright + `performance.now()`，3 次取 P99）。
- [ ] H.2 5 层嵌套拖动最深层 splitter 1s，Chrome Performance 录制帧时长 P99 < 16ms。
- [ ] H.3 Smart Layout preset apply（Quad → Triple Review）DOM commit < 100ms，PTY session id 复用率按 expected 断言。
- [ ] H.4 `⌘⌥ Arrow` navigation 从 keydown 到 focus ring visible < 50ms，3 次取 P99。
- [ ] H.5 `⌘Enter` maximize enter / exit 各 < 200ms，maximized 期间 PTY stdout 不丢事件。
- [ ] H.6 Runtime evidence 放 `docs/runtime-evidence/mvp-14/`：至少 4 张 PNG（Dual AI / Triple Review / Quad / maximize）+ 1 段 10s drag/navigation 录屏。

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元（core） | LayoutNode depth validator、ratio clamp、pane id uniqueness、close collapse、preset apply instance reuse、v1 migration | `cargo test --package vibestation-core panes:: pane_service::` |
| 集成（app IPC） | `pane_layout_apply` 新 preset、`pane_navigate`、`pane_maximize`、`pane_resize_step`、DB round-trip | `cargo test --workspace pane_advanced::` 或 app integration fixture |
| Criterion | validator / preset apply / navigation geometry pure function / close collapse / serialize v1 envelope | `crates/core/benches/pane_advanced_bench.rs` |
| E2E（Playwright） | split 到 5 层、拖 ratio、apply preset、direction nav、maximize restore、workspace switch restore | `web/tests/e2e/pane-advanced.spec.ts` |
| 视觉回归 | Dual AI / Triple Review / Quad / 5 层 nested / maximized / reduced-motion | Playwright screenshot diff |
| 手动 QA | macOS / Linux keyboard modifier、Retina + 1x DPI、1024px 小屏、外接键盘方向键、长期 PTY stdout 不丢 | `docs/runtime-evidence/mvp-14/manual-qa.md` |

### C.1 · fixture 准备

所有 fixture 用 `tempfile::TempDir` + `rusqlite` test DB + 当前 `PanesDao` / `TabsDao` 初始化，不依赖真实用户 workspace：

```rust
fn layout_solo(pane: &str) -> LayoutNode { /* Single */ }
fn layout_dual_ai(a: &str, b: &str) -> LayoutNode { /* H(a, b) */ }
fn layout_triple_review(ai: &str, runner: &str, log: &str) -> LayoutNode { /* H(ai, V(runner, log)) */ }
fn layout_quad(a: &str, b: &str, c: &str, d: &str) -> LayoutNode { /* H(V(a,b), V(c,d)) */ }
fn layout_depth_5_alternating() -> LayoutNode { /* H/V 交替 · 5 split depth */ }
fn layout_depth_6_alternating() -> LayoutNode { /* should fail */ }
fn fixture_workspace_with_legacy_layout_state(value: &str) -> (TempDir, DbPool) { /* migration */ }
fn fixture_running_pty_instances(count: usize) -> (TempDir, DbPool, FakePtyManager) { /* instance reuse */ }
```

### C.2 · Criterion bench 模板

新建 `crates/core/benches/pane_advanced_bench.rs`：

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_validate_depth_5(c: &mut Criterion) {
    c.bench_function("pane_validate_depth_5", |b| {
        let layout = layout_depth_5_alternating();
        b.iter(|| vibestation_core::panes::validate_layout_v1(&layout));
    });
}

fn bench_apply_quad_from_solo(c: &mut Criterion) { /* setup panes · apply Quad */ }
fn bench_apply_triple_from_quad_reuse(c: &mut Criterion) { /* verify reused ids */ }
fn bench_close_nested_collapse(c: &mut Criterion) { /* H(A,V(B,C)) close B */ }
fn bench_navigation_geometry_16(c: &mut Criterion) { /* 16 leaf boxes · find neighbor */ }
fn bench_layout_envelope_serde(c: &mut Criterion) { /* serde_json round-trip */ }

criterion_group!(
    benches,
    bench_validate_depth_5,
    bench_apply_quad_from_solo,
    bench_apply_triple_from_quad_reuse,
    bench_close_nested_collapse,
    bench_navigation_geometry_16,
    bench_layout_envelope_serde,
);
criterion_main!(benches);
```

### C.3 · H2 regression proof

复用 ADR-014 模式，任选 `LayoutApplyAdvancedRequest`：

1. 临时给 `preset` 字段加 `#[ts(rename = "xxxProof")]`。
2. 运行 `cargo build -p vibestation-app` 生成 binding。
3. 运行 `pnpm typecheck`。
4. 预期 TypeScript 报现有前端访问 `preset` 失败。
5. 回滚 rename，重新 `cargo build -p vibestation-app && pnpm typecheck` PASS。

### C.4 · runtime evidence 要求

Phase D 才提交 runtime evidence；spec 详化 PR 豁免截图。实施完成时最少归档：

- `01-dual-ai-layout.png`
- `02-triple-review-layout.png`
- `03-quad-layout.png`
- `04-maximized-pane.png`
- `05-keyboard-navigation.mov` 或 10s 录屏
- `bench-output.txt`（Criterion raw output，≤ 50KB）

## 💾 数据模型变更

MVP-05 已有两条布局存储路径：

- `tabs.layout`：当前 tab 的 LayoutNode JSON，是 Pane 渲染的直接来源。
- `workspaces.layout_state`：workspace 级布局状态字段，当前语义较松散；用户层面常称 `workspace.layout`。

MVP-14 锁定为 **LayoutEnvelope v1**：

```json
{
  "version": 1,
  "root": {
    "kind": "split",
    "direction": "horizontal",
    "ratio": 0.5,
    "first": { "kind": "single", "paneId": "pane-claude" },
    "second": { "kind": "single", "paneId": "pane-codex" }
  },
  "focusedPaneId": "pane-claude",
  "updatedAt": 1760000000000
}
```

**迁移策略**：

- `workspaces.layout_state IS NULL` → 写入 Solo envelope，`root` 指向当前 active tab 的 focused pane；若无 pane，则等待 `pane_init_for_tab` 创建后写。
- `workspaces.layout_state = "solo"` → Solo envelope。
- `workspaces.layout_state = "aiAndRunner"` / `"ai_runner"` → AI+Runner envelope。
- `tabs.layout` 旧 `LayoutNode` tagged union → 运行时包装为 `LayoutEnvelope { version: 1, root }`；写回格式由 Phase A 选择，但必须全局一致。
- `version > 1` → 不迁移，fallback Solo + warning；避免旧客户端覆盖新客户端布局。

**持久化规则**：

- split / close / preset apply：操作成功后立即在 transaction 内写 `tabs.layout` + `focused_pane_id`。
- ratio drag：pointermove 只更新 frontend transient state；pointerup 后 debounce 500ms 写 `pane_split_ratio_update`。
- keyboard resize：每次 step 更新 frontend state，连续 key repeat debounce 500ms 写。
- temporary maximize：session-only，不写 DB。
- layout history：`workspaces.layout_state` envelope 外，追加 app setting key `layout_history_{workspace_id}`，LRU 5 条，v0.2 无 UI 入口。

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)。MVP-14 所有新增 / 扩展 IPC struct 继续以 Rust struct 为 source of truth，禁止前端手写对偶 TypeScript interface。

### G.1 本 MVP 涉及的 IPC struct 清单（明确 12 个新增 / 扩展 binding）

| Rust struct / enum | 类型 | 用途 | 前端 import 路径 |
|---|---|---|---|
| `LayoutEnvelope` | 新增 | v1 layout JSON 顶层 envelope | `import type { LayoutEnvelope } from "../bindings/LayoutEnvelope"` |
| `WorkspaceLayoutState` | 新增 | workspace 级 layout_state round-trip | `import type { WorkspaceLayoutState } from "../bindings/WorkspaceLayoutState"` |
| `LayoutPresetKind` | 新增 | `solo / aiAndRunner / dualAi / tripleReview / quad` | `import type { LayoutPresetKind } from "../bindings/LayoutPresetKind"` |
| `LayoutApplyAdvancedRequest` | 新增 | preset apply 输入，含 preserveInstances / confirmed | `import type { LayoutApplyAdvancedRequest } from "../bindings/LayoutApplyAdvancedRequest"` |
| `LayoutApplyResult` | 新增 | preset apply 输出，含 reused / created / closed ids | `import type { LayoutApplyResult } from "../bindings/LayoutApplyResult"` |
| `PaneNavigateRequest` | 新增 | 方向键跳邻居输入 | `import type { PaneNavigateRequest } from "../bindings/PaneNavigateRequest"` |
| `PaneNavigateResult` | 新增 | 方向键跳邻居输出 | `import type { PaneNavigateResult } from "../bindings/PaneNavigateResult"` |
| `PaneMaximizeRequest` | 新增 | 临时最大化 toggle 输入 | `import type { PaneMaximizeRequest } from "../bindings/PaneMaximizeRequest"` |
| `PaneMaximizeResult` | 新增 | 临时最大化输出 | `import type { PaneMaximizeResult } from "../bindings/PaneMaximizeResult"` |
| `PaneResizeStepRequest` | 新增 | keyboard resize 5% step 输入 | `import type { PaneResizeStepRequest } from "../bindings/PaneResizeStepRequest"` |
| `LayoutHistoryEntry` | 新增 | LRU 5 layout history | `import type { LayoutHistoryEntry } from "../bindings/LayoutHistoryEntry"` |
| `PaneLayoutError` | 新增 | advanced layout 错误 tagged union | `import type { PaneLayoutError } from "../bindings/PaneLayoutError"` |

**复用现有 binding（不重新定义）**：

- `LayoutNode`
- `SplitDir`
- `PaneState`
- `PaneListResponse`
- `PaneFocusRequest`
- `PaneCloseRequest`
- `SplitRatioUpdateRequest`

### G.2 derive 模板

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutEnvelope {
    #[ts(type = "number")]
    pub version: u32,
    pub root: LayoutNode,
    pub focused_pane_id: Option<String>,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum LayoutPresetKind {
    Solo,
    AiAndRunner,
    DualAi,
    TripleReview,
    Quad,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutApplyAdvancedRequest {
    pub tab_id: String,
    pub preset: LayoutPresetKind,
    pub preserve_instances: bool,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutApplyResult {
    pub response: PaneListResponse,
    pub reused_pane_ids: Vec<String>,
    pub created_pane_ids: Vec<String>,
    pub closed_pane_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PaneNavDirection { Left, Right, Up, Down }

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneNavigateRequest {
    pub tab_id: String,
    pub from_pane_id: String,
    pub direction: PaneNavDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneMaximizeRequest {
    pub tab_id: String,
    pub pane_id: String,
    pub toggle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PaneLayoutError {
    MaxDepthExceeded { max_depth: u32, attempted_depth: u32 },
    InvalidRatio { ratio: f32, min: f32, max: f32 },
    PaneNotFound { pane_id: String },
    DuplicatePane { pane_id: String },
    PresetApplyFailed { preset: LayoutPresetKind, reason: String },
    MigrationFailed { version: Option<u32>, message: String },
    DbError { message: String },
}
```

### G.3 强制规范

- [x] 新增 struct / enum 必须 `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]`。
- [x] struct 字段统一 `#[serde(rename_all = "camelCase")]`；递归 / payload enum 用 `#[serde(tag = "kind", rename_all = "camelCase")]`。
- [x] `f32` / `f64` / integer timestamp 字段必须用 `#[ts(type = "number")]`，防止生成不可用类型。
- [x] `LayoutNode` 继续保持现有 tagged union，不允许 Phase A 改成 ad-hoc JSON value。
- [x] 前端所有类型从 `web/src/bindings/*` import；禁止在 `web/src/panels/Terminal/` 手写 `type LayoutNode = ...`。
- [x] bindings 由 `cargo build -p vibestation-app` 触发生成；`web/src/bindings/` 不手写。

### G.4 H2 regression proof

- [x] 1. 临时在 `LayoutApplyAdvancedRequest.preset` 加 `#[ts(rename = "xxxProof")]`。
- [x] 2. 运行 `cargo build -p vibestation-app`。
- [x] 3. 运行 `pnpm typecheck`。
- [x] 4. TypeScript 报 `preset` 字段不存在（`Type '"preset"' is not assignable to type 'keyof LayoutApplyAdvancedRequest'`），证明前端依赖生成 binding。
- [x] 5. 回滚临时 rename。
- [x] 6. 重新 `cargo build -p vibestation-app && pnpm typecheck`，PASS。

### G.5 复用决策

| 现有项 | MVP-14 是否复用 | 决策 | 理由 |
|---|---:|---|---|
| `LayoutNode` | ✅ | 复用并保留 tagged union | MVP-05 已落地，递归结构可直接承载 5 层；避免破坏 PaneSplitView |
| `SplitDir` | ✅ | 复用 | Horizontal / Vertical 足够表达新 preset |
| `PaneState` | ✅ | 复用 | instance reuse 以 pane id 为核心，不需要新 PaneState |
| `PaneListResponse` | ✅ | 复用，但可扩展 focusedPaneId | 前端已有更新路径 |
| `LayoutApplyRequest` | ⚠️ | 不直接扩展旧 request，新增 `LayoutApplyAdvancedRequest` 或保持旧命令兼容后新增 command | 避免破坏 `SmartLayoutMenu` 现有调用；Phase B 可逐步迁移 |
| `pane_pty_*` | ✅ | 不改命名 | Pane PTY 生命周期已经独立，MVP-14 只要求不重启 |
| `workspaces.layout_state` | ✅ | 作为 workspace-level envelope 存储 | 当前 DB 已有列，避免新增表 |
| 第三方 layout lib | ⛔ | 不复用 / 不引入 | 现有递归组件足够，第三方会拖入 React 假设或复杂状态模型 |

### G.6 新增 binding 清单

MVP-14 预期新增 **12 个 `.ts` binding 文件**：

1. `LayoutEnvelope.ts`
2. `WorkspaceLayoutState.ts`
3. `LayoutPresetKind.ts`
4. `LayoutApplyAdvancedRequest.ts`
5. `LayoutApplyResult.ts`
6. `PaneNavigateRequest.ts`
7. `PaneNavigateResult.ts`
8. `PaneMaximizeRequest.ts`
9. `PaneMaximizeResult.ts`
10. `PaneResizeStepRequest.ts`
11. `LayoutHistoryEntry.ts`
12. `PaneLayoutError.ts`

## §H. 决策锁定（MVP-14 专有）

### H.1 技术栈：SolidJS 递归组件 + LayoutNode tree

**决策**：沿用现有 `LayoutNode` tree 与 SolidJS `PaneSplitView` 递归组件，不引第三方布局引擎。

| 选项 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| A · 复用 LayoutNode + 自实现递归 | 与 MVP-05 完全兼容；代码量可控；ts-rs 已有 binding | 需要自己维护 geometry / resize | ✅ 选定 |
| B · 引 react-mosaic / golden-layout | 功能完整 | React 假设重；Tauri/Solid 集成成本高；样式难贴合 Calm Studio | ❌ 禁止 |
| C · CSS grid 固定模板 | 简单 | 不能表达任意嵌套和 close collapse | ❌ 不满足目标 |

### H.2 不碰列表：不引第三方布局库

**决策**：MVP-14 不引 `react-grid-layout`、`react-mosaic`、`golden-layout`、`split.js`、`interact.js`。

| 替代方案 | 为什么不选 |
|---|---|
| `split.js` | 只解决 splitter，不解决 LayoutNode persistence / pane identity / nested focus |
| `golden-layout` | 框架重、状态模型不匹配、窗口 detach 诱惑大 |
| `react-grid-layout` | 网格布局不等于 tree split，且 React 依赖不适合 SolidJS |

### H.3 嵌套深度上限：5 层

**决策**：MVP-14 最大 split depth = 5；第 6 层 hard reject。

| 上限 | 评估 | 结论 |
|---|---|---|
| 3 层 | 性能稳，但 Triple + 用户再 split 空间不足 | 太保守 |
| 5 层 | 覆盖高级用户；仍可用 fixture 验证 60fps | ✅ 选定 |
| 无限 | 支持故事好听，但 geometry / resize / a11y 风险不可控 | ❌ 禁止 |

说明：`implementation-plan.md §5.3.9` 已指出超过 3 层可能触发递归重渲染风险。本 spec 把 5 层作为 v0.2 上限，并要求 Phase D 用 5 层 fixture 证明性能。

### H.4 LayoutNode JSON schema v1 + 迁移

**决策**：顶层加 `version: 1`，保留 `root: LayoutNode`，以 envelope 方式版本化；旧 enum / 旧 LayoutNode 自动迁移。

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| A · envelope version | 后续 v2 可演进；错误恢复清晰 | 需要迁移包装 | ✅ 选定 |
| B · 直接改 LayoutNode enum | 少一层 JSON | 未来 schema 版本无法判断 | ❌ 不选 |
| C · 新表存 layout | 查询清晰 | MVP-14 范围过大；现有列足够 | ❌ YAGNI |

### H.5 渲染优化：`<For>` + `createMemo` + 避免无关重渲染

**决策**：递归层使用稳定 child array + `<For>`；每个 node 的 orientation / style / ratio 用 `createMemo`；Pane body 不随 sibling ratio 更新重建。

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| A · `<For>` + memoized props | SolidJS 细粒度更新；易测 | 需要整理现有组件 props | ✅ 选定 |
| B · 直接递归 JSX + `<Show>` 嵌套 | 写法快 | 深层条件嵌套易误触整棵 subtree | ❌ 不选 |
| C · canvas 绘制 layout | 性能强 | 终端 DOM / xterm 无法 canvas 化 | ❌ 不适用 |

### H.6 拖拽 split divider：原生 pointer events + RAF

**决策**：继续用原生 pointer events；pointermove 中只写 transient signal，RAF 合帧；pointerup debounce 500ms 持久化。

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| A · pointer events + RAF | 无依赖；MVP-05 已验证 | 需要处理 nested coordinate | ✅ 选定 |
| B · mouse events | 兼容老浏览器 | pointer capture 弱，触摸板/笔支持差 | ❌ 不选 |
| C · 第三方 drag lib | 快 | 依赖膨胀且不理解 LayoutNode | ❌ 不选 |

无障碍要求：splitter `tabindex=0`，`role="separator"`，`aria-orientation` 与 SplitDir 对齐，方向键调整 ratio 5% step，Home/End 可调到 50% / 最近 clamp 边界。

### H.7 持久化策略：500ms debounce + workspace 隔离 + LRU 5

**决策**：高频 ratio 更新 debounce 500ms；workspace 隔离用 `workspace_id`；layout history LRU 5 仅作为内部恢复/调试数据。

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| A · debounce 写 sqlite | 降低 DB write；体验稳定 | 崩溃前 500ms 可能丢最后一次 drag | ✅ 选定 |
| B · 每 move 写 DB | 恢复最精确 | 写放大严重，拖拽掉帧 | ❌ 禁止 |
| C · 只内存保存 | 性能好 | workspace reopen 丢 layout | ❌ 不满足 |

### H.8 与 MVP-17 Pane Detach 边界

**决策**：MVP-14 不做 detach；若未来 detached pane 出现，LayoutNode v1 中该 leaf 可替换为 placeholder，但本 task 不创建窗口、不跨窗口同步 geometry。

| 场景 | MVP-14 责任 | MVP-17 责任 |
|---|---|---|
| 当前 tab 内 nested split | ✅ | ❌ |
| 当前 tab 内 maximize | ✅ | ❌ |
| 跨窗口 detach | ❌ | ✅ |
| detached pane 回插 | ❌ | ✅ |
| LayoutNode placeholder 设计说明 | ✅ 边界说明 | ✅ 实施 |

## ⚠️ 已知风险

- **R1 · 5 层嵌套递归渲染 60fps**：深层 Split 可能导致 xterm fit / ResizeObserver 级联。缓解：`<For>` + `createMemo` + 5 层硬上限 + Phase D 5 层 fixture P99 <16ms。
- **R2 · LayoutNode JSON 格式版本化**：无 version 会让后续 detach / placeholder 迁移无法判断。缓解：LayoutEnvelope v1 顶层 `version`，未知版本 fallback Solo，避免旧客户端覆盖新格式。
- **R3 · 小屏拖拽误操作**：`<1024px` 下 splitter 视觉宽度窄。缓解：8px hit-area + visible handle + min pane size 160x120 + keyboard resize。
- **R4 · 预设切换时 PTY instance 复用**：错误实现会关闭用户正在跑的 shell。缓解：PaneType identity + pane id 优先复用；dry-run 列 reused / closed；单测 fake PTY 断言未 kill。
- **R5 · 几何相邻算法在非均匀嵌套中误跳**：只按 DFS 或 DOM 顺序会在 `H(A,V(B,C))` 等布局跳错。缓解：基于 DOMRect 投影重叠 + 距离排序；E2E 覆盖不规则 nested fixture；no target 时 no-op。

## 📝 Notes

- 本 spec 保留 `status: draft` 是有意行为；ready 翻转由 Arbiter approve 后主 agent 处理。
- MVP-14 是 MVP-05 的扩展，不应改写 MVP-05 已验证的 Pane PTY 命名与 transaction 原子性。
- 实施 PR 若发现 `workspaces.layout_state` 与 `tabs.layout` source-of-truth 冲突，必须在 PR body 写明选择；默认 `tabs.layout` 是渲染权威，`workspaces.layout_state` 是 workspace 恢复摘要。
- `LogFollower` / `Runner` PaneType 若在实施时仍未完整 UI 化，Triple Review / Quad 可用 Shell/Runner placeholder 承载，但 preset 语法和 LayoutNode 必须稳定。

## 🔗 相关

- `implementation-plan.md` §5.3 Pane 系统 · §5.3.5 快捷键 · §5.3.9 实现风险 · §10.1 v0.2 功能清单 · §11 W15 路线图
- `design/directions/1-calm-studio.html` line 390-526（Pane system CSS）· line 843-880（Smart Layouts）· line 885-973（Pane actions）· line 977（splitter）
- 上游：MVP-05 Pane 分屏（单层）· `docs/tasks/MVP-05-pane-split-single-level.md`
- 代码入口：`crates/core/src/panes.rs` · `crates/core/src/pane_service.rs` · `web/src/panels/Terminal/PaneSplitView.tsx` · `PaneSplitter.tsx` · `SmartLayoutMenu.tsx`
- 下游：MVP-17 Pane Detach（v0.3）· v1.0 内部 Pane 联动项（本 spec 不实现、不宣传）

---

## 自审四问

1. **递归完备性**：spec 覆盖 LayoutNode v1、5 层 validator、close collapse、ratio、preset、navigation、maximize、persistence、IPC、testing、runtime evidence；同时把 spec 自己的 scope 限制写入 Don't / §H，避免“任意嵌套”滑向无限嵌套。
2. **反向场景**：超 5 层、预设缺 PaneType、PTY spawn 失败、ratio <5%、关闭最后 pane、未知 JSON version、最大化期间继续 split、非均匀布局误跳均有 acceptance 或风险项。
3. **边界适用性**：Solo / 2 pane / 4 pane / Triple / Quad / 5 层 nested / 小屏 / reduced-motion / macOS 与 Linux shortcut / 多 workspace 恢复均覆盖。
4. **YAGNI**：自定义 TOML preset、Pane Detach、跨 tab move、无限嵌套、第三方布局库、v1.0 内部联动项均明确推后或禁止，MVP-14 只升级现有 Pane layout。

## 详化完成度评估

| 12 段必含 | 状态 | 备注 |
|----------|------|------|
| 1. frontmatter | ✅ | id / type / title / status:draft / depends_on / blocks / estimate / plan_ref / risk_ref / reviewer: Codex CLI |
| 2. 顶部状态说明 | ✅ | 状态 / 依赖 / 下游 blocks / 战略依据 / 详化时间 5 行齐 |
| 3. 🎯 目标 Goal | ✅ | 2 段 · 含 implementation-plan 链接与实施入口 |
| 4. 📖 背景 Context | ✅ | plan_ref + CLAUDE.md + 路线图 + MVP-05 已落地 |
| 5. 🛠 实施进度表 | ✅ | Phase A/B/C/D + Phase A 起点 checklist · 7d |
| 6. 🎨 功能范围 Scope | ✅ | Do 8 项 / Don't 8 项 |
| 7. 🖼 UI 引用 | ✅ | design line 锚点 + 6 类 UI 元素 |
| 8. ✅ Acceptance | ✅ | A-H 8 组 · 48 个 checkbox · 每项可独立验证 |
| 9. 🧪 测试策略 | ✅ | 单元 / 集成 / Criterion / E2E / 视觉回归 / 手动 QA + fixture + bench 模板 |
| 10. 💾 数据模型变更 | ✅ | `workspaces.layout_state` / `tabs.layout` · LayoutEnvelope v1 · enum/旧 JSON 迁移 |
| 11. §G IPC Contract | ✅ | G.1-G.6 齐全 · 12 个新增/扩展 binding 数字明确 |
| 12. §H 决策锁定 + 风险/Notes/相关/自审 | ✅ | H.1-H.8 每段含决策 + 替代方案 + 理由 · R1-R5 mitigation |

**完成度**：12/12 = **100%**（建议 Arbiter review 通过后翻 `status: ready`）。

**遗留问题**：无阻塞项；只有实施期需要在 PR body 记录 `tabs.layout` 与 `workspaces.layout_state` 的最终 source-of-truth 选择。
