---
id: MVP-05
type: mvp
title: Pane 分屏（单层 · 最多 4 Pane · Smart Layouts）
status: draft
owner:
phase: W6-W7
depends_on: ["MVP-04"]
blocks: []
blocked_by: []
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §5.3（Pane 系统）
risk_ref:
reviewer:
---

# MVP-05: Pane 分屏（单层）

> **状态**：`draft`
> **依赖**：MVP-04（Tab 终端存在才能分屏）
> **战略依据**：`§10.1` MVP B 折中方案 · `§5.3` Pane 系统

---

## 🎯 目标（Goal）

每个 Tab 内支持**单层**嵌套分屏（最多 4 Pane：1 次右分 + 1 次下分），提供 Smart Layouts 预设（Solo / AI+Runner），拖拽分隔条调整比例。

## 📖 背景（Context）

- `implementation-plan.md §10.1` MVP B 折中方案明确"Pane 最多 1 层嵌套（4 Pane）"
- 任意嵌套 / Dual AI / Triple Review / Quad 预设都砍到 v0.2+
- `CLAUDE.md` 锁定 #2（A 栏 MVP 范围）

---

## 🎨 功能范围（Scope）

**Do**：
- Pane 分屏快捷键：
  - `⌘\` 右分屏
  - `⌘⇧\` 下分屏
  - `⌘⌃W` 关当前 Pane
- 单层嵌套上限：右分一次 + 下分一次 = 最多 4 Pane（矩形 2×2 或 1×2 或 2×1）
- Smart Layouts 预设（一键切换）：
  - **Solo**：单 Pane 全占
  - **AI + Runner**：右分屏 50/50（左 AI CLI · 右 runner/shell）
- 分隔条拖拽调整比例，双击复位到默认比例（50/50）
- 比例持久化到 rusqlite per-Tab
- 每个 Pane 独立 PTY（复用 MVP-04 PTY 架构）
- 当前聚焦 Pane 高亮边框

**Don't**：
- 任意嵌套（v0.2+）
- 方向键跳邻居 Pane / ⌘Enter 最大化（v0.2+）
- Dual AI / Triple Review / Quad 预设（v0.2+）
- Pane Detach（v0.3+）

## 🖼 UI 引用

- `design/directions/1-calm-studio.html` 主区 Pane 分屏 demo（右分屏 + 下分屏 示例）
- 分隔条：1px 浅色，hover 时变主色 + 光标变 resize
- 当前 Pane 边框：主色 1px

## ✅ Acceptance

### A. 分屏操作

- [ ] `⌘\` 从当前 Pane 右分出一个新 Pane，新 Pane 默认运行同 shell（从父 Pane `PaneState.shell` 继承，cwd 默认继承父 Pane `cwd`；可选重置到 `$HOME`，由用户设置控制）
- [ ] `⌘⇧\` 从当前 Pane 下分出一个新 Pane（继承规则同上）
- [ ] `⌘⌃W` 关当前 Pane（若仅剩 1 个 Pane → 关整个 Tab）
- [ ] 已在右分状态再点 `⌘\` → 被拒绝（超单层上限），显示"Pane 已达单层上限"toast，持续 3s，含文案"v0.2 将支持任意嵌套"（纯中文，无 i18n 时不加链接）

### B. 单层嵌套规则

- [ ] 合法布局（✅）：Solo / 水平 2 Pane / 垂直 2 Pane / 2×2 (右分再下分)
- [ ] 非法布局（❌）：右分的右 Pane 再右分（= 3 横）、下分的下 Pane 再下分（= 3 竖）
- [ ] 布局上限逻辑用单元测试覆盖所有组合（至少 6 个用例起步：4 合法 + 2 非法）
- [ ] 非法操作拒绝时，layout tree 回滚到操作前状态，不留下脏数据（原子性见 §H）

### C. Smart Layouts

- [ ] 命令面板（临时用菜单）提供 "Apply Layout → Solo" 和 "Apply Layout → AI + Runner"
- [ ] Solo：关所有非当前 Pane；若任一待关闭 Pane 存在未保存终端编辑状态（editing state，如 vim/nano 运行中），弹二次确认对话框，默认取消，确认后才执行
- [ ] AI + Runner：强制右分屏 50/50；若当前已是 2×2 布局，先降级到单层（保留当前 focus Pane，关闭其余 Pane，二次确认同 Solo 规则），再执行右分；流程在命令面板文案中明示（"将关闭 N 个现有 Pane"）
- [ ] Smart Layouts 切换前显示 dry-run 预览：命令面板展示即将关闭的 Pane 数量及对应 shell/cwd 摘要，用户确认后执行

### D. 分隔条

- [ ] 拖拽水平分隔条调整左右比例
- [ ] 拖拽垂直分隔条调整上下比例
- [ ] 双击任一分隔条 → 复位到 50/50
- [ ] 比例持久化：持久化到 rusqlite `tabs.layout` 字段（JSON 序列化 `LayoutNode`，见 §H）；同 Tab 再打开保持上次比例
- [ ] 拖拽分隔条 60FPS：Chrome DevTools Performance 面板记录，帧时长 < 16ms，测 3 次取 P99

### E. Focus

- [ ] 点击 Pane → focus 切换，边框高亮（主色 1px solid）
- [ ] 输入直接到 focus 的 Pane；测：2 Pane 场景，focus Pane A 输入 `ls`，仅 Pane A 收到 keydown 事件，Pane B 不受干扰，测 3 次
- [ ] Focus 切换不打断其他 Pane 的 PTY 运行；测：Pane A 跑 `yes > /dev/null`，切 focus 到 Pane B，Pane A 的 `yes` 持续输出不间断（监控 PTY stdout rate 不降，DevTools Performance 验证）

### F. 性能

- [ ] 4 Pane 并存不额外显著增加内存：每 Pane ≈ MVP-04 单 Tab PTY 开销（SPIKE-05 单 Tab 10MB RSS 基准），4 Pane ≈ 40MB；总 10 Tab × 4 Pane = 40 个 PTY，RSS 上限 500MB（对齐 MVP-04 §E 性能目标）
- [ ] 拖拽分隔条 60FPS：同 D 条，帧时长 < 16ms，测 3 次取 P99
- [ ] 分屏 / 关 Pane 动画 < 150ms：从快捷键按下到 Pane DOM 绘制完成，`performance.now()` 差值，测 3 次取 P99；无动画时不测量动画时长，改为测量 "操作完成到 DOM 稳定" < 100ms

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | Pane 布局树模型（`LayoutNode` 递归，MVP 限深度 ≤ 2）+ 非法操作拒绝 + 原子回滚 |
| 视觉回归 | 4 种合法布局截图对比原型 |
| E2E | 完整流程：分屏 → 拖拽 → 关 → 应用 Layout → 二次确认取消/确认 |
| 手动 QA | 多屏不同 DPI 下分隔条精度 |

## 💾 数据模型变更

扩展 `tabs` table，加入 `layout`：
```rust
enum LayoutNode {
    Single(PaneId),
    Split { direction: SplitDir, ratio: f32, first: Box<LayoutNode>, second: Box<LayoutNode> },
}
// MVP-05 限制：LayoutNode 树深度 ≤ 2（根 + 一层 Split）
```

新 table `panes`（每个 Pane 独立 PTY）：
```rust
struct PaneState {
    pane_id: String,
    tab_id: String,               // FK
    shell: String,
    cwd: String,
    scroll_back: Vec<String>,
}
```

## ⚠️ 已知风险

- **深度限制规则易踩坑**：用户期望"最多 4 Pane 随便分" vs 现实"单层"——UI 错误提示要明确
- **Smart Layouts 覆盖用户布局**：应用预设前需二次确认（"这会关闭现有 X 个 Pane"）

## 📝 Notes

- LayoutNode 用递归 enum 实现，v0.2 扩展到多层时只需放开深度限制
- 当前 Pane focus 不跟随鼠标 hover（避免误触）

## 🔗 相关

- `implementation-plan.md` §5.3 Pane 系统
- 上游：MVP-04
- 下游：无直接（v0.2 多 Pane 预设 / 任意嵌套）

---

## §G. IPC Contract（ts-rs）

> 复用 MVP-04 §G 模式：所有 IPC struct 必须 derive `TS` + `serde`，前端禁手写 bindings，`build.rs` 自动生成。IPC 侧不包含 `scroll_back`，走独立拉取接口。

### G.1 预期 IPC struct 清单

| Struct / Enum | 用途 | 备注 |
|---|---|---|
| `PaneState` | Pane 状态同步 | 排除 `scroll_back`，同 MVP-04 `TabState` 排除模式 |
| `PaneCreateRequest` | 新建 Pane | `{ tab_id, parent_pane_id, direction: SplitDir, shell }` |
| `PaneCloseRequest` | 关闭 Pane | `{ pane_id }` |
| `LayoutNode` | 布局树节点 | 递归 tagged union（`Single` / `Split`） |
| `SplitDir` | 分割方向 | 简单 string union：`"horizontal"` / `"vertical"` |
| `LayoutApplyRequest` | 应用 Smart Layout | `{ tab_id, preset: "solo" \| "ai_runner", confirmed: bool }` |
| `SplitRatioUpdateRequest` | 更新分割比例 | `{ pane_id, new_ratio: f32 }` |
| `PaneFocusRequest` | 切换焦点 | `{ tab_id, focused_pane_id }` |
| `PaneListResponse` | Pane 列表 + 当前布局 | `{ panes: Vec<PaneState>, layout: LayoutNode }` |
| `PaneScrollbackFetchRequest` | 拉取 scrollback | `{ pane_id, offset, limit }`（独立接口，同 MVP-04 `tab_scrollback_fetch` 模式） |

### G.2 derive 模板

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LayoutNode {
    Single { pane_id: String },
    Split {
        direction: SplitDir,
        #[ts(type = "number")]
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum SplitDir {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneState {
    pub pane_id: String,
    pub tab_id: String,
    pub shell: String,
    pub cwd: String,
    // scroll_back 不在 IPC，独立 pane_scrollback_fetch
    #[ts(type = "number")]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneCreateRequest {
    pub tab_id: String,
    pub parent_pane_id: String,
    pub direction: SplitDir,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneCloseRequest {
    pub pane_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutApplyRequest {
    pub tab_id: String,
    pub preset: String, // "solo" | "ai_runner"
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SplitRatioUpdateRequest {
    pub pane_id: String,
    #[ts(type = "number")]
    pub new_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneFocusRequest {
    pub tab_id: String,
    pub focused_pane_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneListResponse {
    pub panes: Vec<PaneState>,
    pub layout: LayoutNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneScrollbackFetchRequest {
    pub pane_id: String,
    #[ts(type = "number")]
    pub offset: i64,
    #[ts(type = "number")]
    pub limit: i64,
}
```

### G.3 强制规范

1. **所有 IPC struct 必须** `#[derive(TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`，保持与 MVP-04 §G.3 一致。
2. **`LayoutNode`** 因递归 + payload，必须 tagged union（`#[serde(tag = "kind")]`），禁止用 plain string union；前端 TypeScript 自动推导为 discriminated union。
3. **`SplitDir`** 为简单 enum，用 string union（`#[serde(rename_all = "camelCase")]` + 无 `tag`），TypeScript 生成 `"horizontal" | "vertical"`。
4. **`PaneState.scroll_back`** 从 IPC 排除，独立 `pane_scrollback_fetch` 拉取；复用 MVP-04 `TabState` 排除模式，保持 IPC contract 全局一致。
5. **bindings 由 `build.rs` 生成**，前端禁止手写 TypeScript 类型；H2 regression proof 见 MVP-04 §G.3（临时重命名字段 → 期望 `pnpm typecheck` 失败）。

---

## §H. Pane 布局模型约束

> MVP-05 专有约束章节。与 MVP-07/08/09 的 §H（Git 栈约束）不同，本节约束 Pane 布局树模型、持久化与扩展路径。

### H.1 LayoutNode 深度约束

| 版本 | 最大深度 | 最大 Pane 数 | 说明 |
|---|---|---|---|
| MVP-05（v0.1） | ≤ 2 | 4 | 根节点 + 一层 Split：2×2 / 1×2 / 2×1 / Solo |
| v0.2 | ≤ 4 | 8+ | 放开深度限制，支持任意嵌套、Dual AI / Triple Review / Quad 预设 |
| v0.3 | 跨窗口 | 无上限 | Pane Detach，LayoutTree 分布多窗口，需 window_id 索引 |

- MVP-05 实施时，**深度检查必须在 Rust 侧**（core crate）硬编码 `MAX_LAYOUT_DEPTH = 2`，前端仅做乐观预检；任何超出深度的 split 请求返回 `Err(LayoutError::MaxDepthExceeded)`。

### H.2 合法 / 非法布局矩阵（单元测试必覆盖）

| 布局 | depth | pane_count | 状态 |
|---|---|---|---|
| Solo | 1 | 1 | ✅ 合法 |
| 水平 2 Pane（右分） | 2 | 2 | ✅ 合法 |
| 垂直 2 Pane（下分） | 2 | 2 | ✅ 合法 |
| 2×2（右分后再下分） | 2 | 4 | ✅ 合法 |
| 3 横（右分后，右 Pane 再右分） | 3 | 3 | ❌ **MVP-05 拒绝** |
| 3 竖（下分后，下 Pane 再下分） | 3 | 3 | ❌ **MVP-05 拒绝** |

- 单元测试至少覆盖上表 6 种组合 + 边界回滚测试（非法操作后 layout tree 不变）。

### H.3 布局变更原子性

- 每次 `split` / `close` / `layout_apply` 操作，必须**原子更新**数据库状态：
  - 更新 `tabs.layout` JSON 字段
  - 新建 / 删除 `panes` 表记录
  - 更新 `tabs.focused_pane_id`
- 使用 rusqlite `transaction()` 包裹；任何步骤失败 → 完整回滚，禁止出现 "layout 改了一半、panes 没删" 的脏状态。
- 前端状态（SolidJS store）与后端状态同步：操作成功后通过 Tauri event 推送 `PaneListResponse`，前端以服务端状态为准覆写本地。

### H.4 布局持久化存储

- **`tabs.layout`** TEXT 字段：存储 JSON 序列化的 `LayoutNode`（tagged union 格式）。
- 若 MVP-04 storage prep（PR #72）的 migration v5 仅定义 `tabs` 表且无 `layout` 字段，MVP-05 实施需 **migration v6** 扩展：
  ```sql
  ALTER TABLE tabs ADD COLUMN layout TEXT DEFAULT '{"kind":"single","paneId":""}';
  ALTER TABLE tabs ADD COLUMN focused_pane_id TEXT DEFAULT NULL;
  ```
  - `focused_pane_id` 替代原 `tabs` 表隐式单 Pane 假设，支持多 Pane 下 focus 持久化。
- **`panes` 表**：新表，schema 与 `PaneState` 对齐（不含 `scroll_back`）；`scroll_back` 存 JSON TEXT（同 MVP-04 `tabs` 表 `scroll_back` 模式）。
- **外键约束**：`panes.tab_id → tabs.id ON DELETE CASCADE`，Tab 关闭时自动级联删除所属 Pane。

### H.5 扩展路径

- **v0.2 深度放开**：`LayoutNode` enum 本身无需改动；只需将 `MAX_LAYOUT_DEPTH` 从 2 改为 4，前端 resize 逻辑适配多层级分隔条。Smart Layouts 新增预设枚举值即可。
- **v0.3 Pane Detach**：涉及 IPC contract 大改（`LayoutNode` 需跨窗口），必须走 ADR 流程；当前 §G 的 `PaneState` / `LayoutNode` 暂不预留 `window_id` 字段，避免 YAGNI 污染。

---

**自审四问**（2026-04-20）：
1. 合法/非法 + 视觉覆盖 ✅ · §H.2 矩阵 6 用例起步 + 回滚测试
2. 非法操作 graceful 拒绝 ✅ · toast 3s + 原子回滚 + 深度检查 Rust 侧硬编码
3. 多 DPI 显式测 ✅ · 手动 QA 项保留
4. 任意嵌套 / 方向键 / Dual AI / Pane Detach 都在 v0.2+/v0.3+ ✅ · §H.1/H.5 明确扩展路径
5. §G ts-rs contract（LayoutNode tagged union + SplitDir string union）✅ · 9 个 IPC struct 全清单
6. §H 布局深度约束 + 持久化迁移路径 ✅ · migration v6 方案 + 原子性约束
