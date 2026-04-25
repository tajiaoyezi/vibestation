---
id: MVP-05
type: mvp
title: Pane 分屏（单层 · 最多 4 Pane · Smart Layouts）
status: ready
owner:
phase: W6-W7
depends_on: ["MVP-04"]
blocks: []
blocked_by: []
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §5.3（Pane 系统）
risk_ref:
reviewer: Kimi
---

# MVP-05: Pane 分屏（单层）

> **状态**：`ready`
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
- Pane Detach（v0.3+)

## 🛠 实施进度

| Phase | 范围 | 状态 | PR |
|-------|------|------|----|
| Phase A · storage prep | migration v6（`CREATE TABLE panes` + `tabs` 加 `layout` / `focused_pane_id` 列）+ `PanesDao` CRUD + 单元测试 + ts-rs bindings 生成 | ✅ done | spec 早期 PR |
| Phase B Step 2 · layout pure functions | 4 pure functions（split_layout / close_pane_in_layout / update_split_ratio / apply_smart_layout）+ 17 单元测试 + 7 micro-bench（48-210 ns）· panes.rs +647 行 | ✅ done | [#141](https://github.com/tajiaoyezi/vibestation/pull/141) |
| Phase B Step 1 · pane_pty IPC | `pane_pty_*` 5 IPC commands · §H.6 锁 A 独立命名空间 · 独立 `PtyManager` 实例 · 反向映射 PtyEvent → PanePtyEvent · 3 unit tests | ✅ done | [#142](https://github.com/tajiaoyezi/vibestation/pull/142) |
| Phase B Step 2 · IPC layer | 5 layout IPC commands（pane_split / close / focus / layout_apply / split_ratio_update）· transactional pane_service 500 行 · §H.3 atomicity（rusqlite Transaction · 任意一步 fail 全 rollback）· 13 unit tests | ✅ done | [#143](https://github.com/tajiaoyezi/vibestation/pull/143) |
| Phase C · 前端分屏 UI scaffolding | `pane_init_for_tab` IPC（idempotent · 2 tests）+ 3 SolidJS 组件（PaneTerminal · PaneSplitView · PaneSplitter）+ usePaneShortcuts hook + CSS · **0 集成 Terminal.tsx** · 独立 typecheck/lint 通过 | 🟡 partial done | [#144](https://github.com/tajiaoyezi/vibestation/pull/144) |
| Phase C · 集成完整版 | Terminal.tsx 集成 PaneSplitView · 快捷键 wire（pane_split/close）· 拖拽 splitter（rAF + transform · 60FPS · §D + §F.2/F.3）· Smart Layouts 命令面板 + dry-run + 二次确认 dialog（§C）· pane_focus IPC wire（§E）· §F 6 条 P99 性能测量 | ⏳ todo · 估 2-3h | — |
| Phase D · runtime 证据 | ≥ 5 张截图 / 30s 录屏 · 覆盖 Solo / 水平 2 Pane / 垂直 2 Pane / 2×2 / Smart Layouts apply · 放 `docs/runtime-evidence/mvp-05/` | ⏳ todo · 0.5h（接 Phase C 完整版） | — |

**Phase A 实施起点 checklist**（让 agent 接 spec 后 5 min 内启动）：

- [ ] `crates/core/Cargo.toml` 已含 `git2` / `rusqlite` / `serde` / `ts-rs`（继承 MVP-04 · 不需要新增依赖）
- [ ] migration v6 路径锁定（§H.4 SQL 已写好）：`crates/core/src/db.rs` 加 `migrate_v6` 函数 · 复用 `migrate_v5` 模式（PR #72）
- [ ] `PanesDao` CRUD（仿 MVP-04 `TabsDao` 模式 · PR #72 line 119 起）：
  - `insert(pane: PaneState) -> Result<()>`
  - `update(pane: PaneState) -> Result<()>`
  - `delete(pane_id: &str) -> Result<()>`
  - `list_by_tab(tab_id: &str) -> Result<Vec<PaneState>>`
  - `get(pane_id: &str) -> Result<Option<PaneState>>`
- [ ] `LayoutNode` 序列化 / 反序列化测试（`serde_json` + ts-rs tagged union）
- [ ] IPC commands 注册顺序（`crates/app/src/lib.rs` `invoke_handler!`）：
  - `pane_split` / `pane_close` / `pane_focus` / `pane_layout_apply` / `pane_split_ratio_update`
  - `pane_pty_spawn` / `pane_pty_stdin` / `pane_pty_resize` / `pane_pty_signal` / `pane_pty_kill`
  - 总 **10 个新 IPC commands**（5 layout + 5 pty）
- [ ] permission toml：`crates/app/permissions/panes.toml` + `pane-pty.toml` 新建（10 个 `allow-{name}`）
- [ ] capability `default.json` 引用上述 permission
- [ ] ts-rs binding 自动生成到 `web/src/bindings/`（`build.rs` 触发 · 13 个 struct 见 §G.5）
- [ ] fixture：用 `tempfile` crate 运行时生成 sqlite + tabs 行 · 不要硬编码本地路径（仿 MVP-09 §C.1）

**下次 agent 起点**（session 19 末更新）：**Phase C 完整版** · 修改 `web/src/panels/Terminal/Terminal.tsx`（853 行 · 重构风险）集成 PaneSplitView 替换 TerminalPane 渲染路径 · wire 快捷键 hook 调 pane_split/close · 实现拖拽 splitter（rAF + transform · 60FPS）· 实现 Smart Layouts 命令面板 + dry-run 预览 + 二次确认 dialog · pane_focus IPC wire 让 click 切焦持久化 · 完成 §F 6 条 P99 性能测量 · capture Phase D 5 截图 + 30s 录屏 · 估 2-3h 集中工作。Phase B/C scaffolding 全部建材已落地（PR #141-#144 · 5 IPC backend commands + 3 SolidJS components + 1 hook 已 typecheck 通过 · 待 import 集成）。

**依赖关系说明**：MVP-05 Phase A/B 可以和 MVP-04 Phase C/D/E/F **并行**启动（文件域物理隔离）· Phase C 前端分屏 UI 必须等 MVP-04 Phase C xterm 前端 done（共享 Terminal 组件基础）。

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

- [ ] F.1 4 Pane 内存：每 Pane ≈ MVP-04 单 Tab PTY 开销（SPIKE-05 单 Tab 10MB RSS 基准）· 4 Pane ≈ 40MB · 总 10 Tab × 4 Pane = 40 PTY < 500MB · 用 `ps -o rss` + 4 Pane fixture · 测 3 次取 P99
- [ ] F.2 拖拽水平分隔条 60FPS：DevTools Performance 录 1s 水平拖拽 · 帧时长 < 16ms · 测 3 次取 P99
- [ ] F.3 拖拽垂直分隔条 60FPS：同 F.2 · 垂直方向独立测一次 · 帧时长 < 16ms · 测 3 次取 P99
- [ ] F.4 分屏快捷键 → DOM 绘制完成 < 150ms：`performance.now()` 测 `⌘\` keydown 到 SolidJS 新 Pane DOM commit · 测 3 次取 P99
- [ ] F.5 关 Pane 快捷键 → 剩余 Pane 重排完成 < 100ms：`performance.now()` 测 `⌘⌃W` keydown 到剩余 Pane 重排 DOM commit · 测 3 次取 P99
- [ ] F.6 Smart Layouts apply（Solo / AI+Runner）→ 关闭 N Pane + 新 Pane 完成 < 200ms：fixture 4 Pane → Solo · `performance.now()` 测命令面板确认到最终布局 DOM commit · 测 3 次取 P99

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | Pane 布局树模型（`LayoutNode` 递归，MVP 限深度 ≤ 2）+ 非法操作拒绝 + 原子回滚 |
| 视觉回归 | 4 种合法布局截图对比原型 |
| E2E | 完整流程：分屏 → 拖拽 → 关 → 应用 Layout → 二次确认取消/确认 |
| 手动 QA | 多屏不同 DPI 下分隔条精度 |

#### C.1 · fixture 准备脚本 + Criterion bench 模板

所有 fixture 用 `tempfile::TempDir` + `rusqlite::Connection` in-memory · 不依赖本地文件系统：

```rust
// tests/fixtures/mvp_05_helpers.rs（新建）
use rusqlite::Connection;
use tempfile::TempDir;

fn create_fixture_solo_layout() -> (TempDir, Connection) {
    let dir = tempfile::tempdir().unwrap();
    let conn = Connection::open(dir.path().join("test.db")).unwrap();
    crates_core::db::migrate(&conn).unwrap();  // 跑到 v6
    // 插入 1 tab + 1 pane（Solo 布局）
    (dir, conn)
}

fn create_fixture_horizontal_2pane() -> (TempDir, Connection) { /* 1 tab + 2 panes 水平 */ }
fn create_fixture_vertical_2pane() -> (TempDir, Connection) { /* 1 tab + 2 panes 垂直 */ }
fn create_fixture_2x2_layout() -> (TempDir, Connection) { /* 1 tab + 4 panes 2×2 */ }
fn create_fixture_invalid_3horizontal() -> Vec<LayoutNode> { /* 用于测 §H.2 非法布局拒绝 */ }
fn create_fixture_invalid_3vertical() -> Vec<LayoutNode> { /* 同上 */ }
```

每个 helper 返回 `(TempDir, Connection)` 元组 · 测试用 `let (_dir, conn) = create_fixture_solo_layout();` 持有 · 测试结束 `dir` drop 自动清理。

**Criterion bench 模板**（`crates/core/benches/pane_bench.rs`）：

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_split_pane(c: &mut Criterion) {
    c.bench_function("split_solo_to_horizontal", |b| {
        b.iter(|| {
            let (_dir, conn) = create_fixture_solo_layout();
            // call vibestation_core::pane::split(...)
        });
    });
}

fn bench_layout_apply_solo(c: &mut Criterion) {
    c.bench_function("layout_apply_2x2_to_solo", |b| {
        b.iter(|| {
            let (_dir, conn) = create_fixture_2x2_layout();
            // call vibestation_core::pane::apply_layout(Solo, ...)
        });
    });
}

fn bench_close_pane_atomic(c: &mut Criterion) {
    c.bench_function("close_pane_atomic", |b| {
        b.iter(|| {
            let (_dir, conn) = create_fixture_horizontal_2pane();
            // call vibestation_core::pane::close(...)
        });
    });
}

criterion_group!(benches, bench_split_pane, bench_layout_apply_solo, bench_close_pane_atomic);
criterion_main!(benches);
```

跑 `cargo bench --bench pane_bench` 验证 P99 数字。

## 💾 数据模型变更

详见：

- **§G.2 IPC struct derive 模板** · `LayoutNode` / `PaneState` 等 Rust struct 完整定义（ts-rs 注解齐全）
- **§H.4 布局持久化存储** · migration v6 完整 SQL（`CREATE TABLE panes` + `ALTER TABLE tabs`）

本段仅列 storage 层高层摘要：

- 新表：`panes`（每个 Pane 独立 PTY · FK → `tabs(tab_id)` ON DELETE CASCADE）
- `tabs` 扩展：`layout` TEXT（JSON 序列化 `LayoutNode`）+ `focused_pane_id` TEXT（当前聚焦 Pane）
- migration 版本：v6（接 MVP-04 Phase A v5）· 由 MVP-05 Phase A 实施

## ⚠️ 已知风险

- **深度限制规则易踩坑**：用户期望"最多 4 Pane 随便分" vs 现实"单层"——UI 错误提示要明确
- **Smart Layouts 覆盖用户布局**：应用预设前需二次确认（"这会关闭现有 X 个 Pane"）

## 📝 Notes

- LayoutNode 用递归 enum 实现，v0.2 扩展到多层时只需放开深度限制
- 当前 Pane focus 不跟随鼠标 hover（避免误触）
- **session 15（2026-04-22）reviewer 对齐确认**：Kimi 第 10 次协作交付 · 5 gap 修复 + 自审第 7 条新增 · Claude Code reviewer self-push 翻转 gate (a) · 3 个对齐锚点 verified：
  - FK `panes.tab_id REFERENCES tabs(tab_id)` vs `crates/core/src/db.rs` `migrate_v5` 主键 `tab_id TEXT PRIMARY KEY` ✅
  - `PRAGMA user_version = 6` vs MVP-04 Phase A 已占 v5 ✅
  - §H.6 锁 A 选项（`pane_pty_*` 独立）vs MVP-04 Phase B PR #82 已落地 `tab_pty_*` 5 IPC + `PtySpawnRequest` ts-rs binding ✅

## 🔗 相关

- `implementation-plan.md` §5.3 Pane 系统
- 上游：MVP-04
- 下游：无直接（v0.2 多 Pane 预设 / 任意嵌套）

---

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)（规范源头）。复用 MVP-04 §G 模式：所有 IPC struct 必须 derive `TS` + `serde`，前端禁手写 bindings，`build.rs` 自动生成。IPC 侧不包含 `scroll_back`，走独立拉取接口。

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

### G.4 · 与 MVP-04 已落地 binding 的复用决策

MVP-05 实施前必须明确复用 / 新增边界，避免和 MVP-04 Phase A/B 已生成 binding 冲突：

| 已有 binding | MVP-05 §G.1 涉及 | 决策 | 理由 |
|---|---|---|---|
| `TabState`（MVP-04 Phase A 已生成）| §G.1 `PaneListResponse` 的 `tab_id` 字段 | ⛔ 不复用为输入 · 仅引用 `tab_id` | `TabState` 含 `scroll_back` · 不适合作为 Pane 上下文 · MVP-05 只需要 `tab_id` 引用 |
| `PtySpawnRequest`（MVP-04 Phase B 已生成）| §H.6 锁 A 选项独立 `pane_pty_*` | ⛔ 不复用 · 新建 `PanePtySpawnRequest` | 改 `PtySpawnRequest` 会破坏 MVP-04 Phase B 已落地 binding（5 IPC + 前端调用）· 独立命名空间避免 |
| `TabsDao`（MVP-04 Phase A）| MVP-05 Phase A `PanesDao` | ⛔ 不复用 · 新建 `PanesDao` 但仿 `TabsDao` 模式 | `TabsDao` 操作 `tabs` 表 · `PanesDao` 操作 `panes` 表 · 表不同 DAO 不混 |
| `migrate_v5`（MVP-04 Phase A）| MVP-05 Phase A `migrate_v6` | ⛔ 不复用 · 新建 `migrate_v6` | migration 单调递增 · v5 已锁 `tabs` 表 · v6 加 `panes` 表 + `tabs` 2 列（§H.4 SQL 已写）|

### G.5 · MVP-05 新增 binding 清单（明确数量）

以下 **13 个 binding** 为 MVP-05 **新增** · 实施时 `web/src/bindings/` 应新增 13 个 `.ts` 文件：

| Rust struct / enum | 用途 | 前端 import 路径 |
|---|---|---|
| `PaneState` | Pane 状态同步 · 排除 `scroll_back` | `import type { PaneState } from "../bindings/PaneState"` |
| `PaneCreateRequest` | 新建 Pane | `import type { PaneCreateRequest } from "../bindings/PaneCreateRequest"` |
| `PaneCloseRequest` | 关闭 Pane | `import type { PaneCloseRequest } from "../bindings/PaneCloseRequest"` |
| `LayoutNode` | 布局树节点 · 递归 tagged union | `import type { LayoutNode } from "../bindings/LayoutNode"` |
| `SplitDir` | 分割方向 · string union | `import type { SplitDir } from "../bindings/SplitDir"` |
| `LayoutApplyRequest` | 应用 Smart Layout | `import type { LayoutApplyRequest } from "../bindings/LayoutApplyRequest"` |
| `SplitRatioUpdateRequest` | 更新分割比例 | `import type { SplitRatioUpdateRequest } from "../bindings/SplitRatioUpdateRequest"` |
| `PaneFocusRequest` | 切换焦点 | `import type { PaneFocusRequest } from "../bindings/PaneFocusRequest"` |
| `PaneListResponse` | Pane 列表 + 当前布局 | `import type { PaneListResponse } from "../bindings/PaneListResponse"` |
| `PaneScrollbackFetchRequest` | 拉取 scrollback | `import type { PaneScrollbackFetchRequest } from "../bindings/PaneScrollbackFetchRequest"` |
| `PanePtySpawnRequest` | Pane PTY spawn · 独立命名 | `import type { PanePtySpawnRequest } from "../bindings/PanePtySpawnRequest"` |
| `PanePtyStdoutEvent` | Pane PTY stdout event | `import type { PanePtyStdoutEvent } from "../bindings/PanePtyStdoutEvent"` |
| `PanePtyExitedEvent` | Pane PTY exited event | `import type { PanePtyExitedEvent } from "../bindings/PanePtyExitedEvent"` |

> 加上引用 MVP-04 的 `TabState`（不重新生成）= 实施时 bindings 目录共新增 **13 个 `.ts` 文件**。

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

#### H.3.1 · 原子性测试 case（Phase A 实施时必加）

3 类操作 × 2 类失败注入 = **6 个测试 case**，验证 transaction 回滚不留脏数据：

```rust
#[test]
fn split_atomicity_fails_during_panes_insert() {
    let (_dir, conn) = create_fixture_solo_layout();
    // 注入失败：mock PanesDao::insert 返回 Err
    let result = pane::split_with_mock_failure(&conn, "panes_insert");
    assert!(result.is_err());
    // 验证回滚：tabs.layout 仍是 Solo · panes 表无新行
    let layout_json: String = conn.query_row(
        "SELECT layout FROM tabs WHERE tab_id=?", ["t1"], |r| r.get(0)
    ).unwrap();
    let layout: LayoutNode = serde_json::from_str(&layout_json).unwrap();
    assert!(matches!(layout, LayoutNode::Single { .. }));
    let panes_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM panes WHERE tab_id=?", ["t1"], |r| r.get(0)
    ).unwrap();
    assert_eq!(panes_count, 1);  // 原 1 pane · 没新增
}

#[test]
fn split_atomicity_fails_during_layout_update() {
    // 注入 tabs.layout UPDATE 失败 · 验证 panes 表无新行 + layout 不变
}

#[test]
fn close_atomicity_fails_during_panes_delete() {
    // 注入 panes DELETE 失败 · 验证原 pane 仍在 + layout 不变
}

#[test]
fn close_atomicity_fails_during_layout_update() {
    // 注入 tabs.layout UPDATE 失败 · 验证 panes 表行数不变
}

#[test]
fn layout_apply_atomicity_fails_during_panes_batch_delete() {
    // Smart Layout 批量关闭中途失败 · 验证所有 pane 仍在 + layout 不变
}

#[test]
fn layout_apply_atomicity_fails_during_focused_pane_update() {
    // focused_pane_id 写入失败 · 验证 panes + layout 均不变
}
```

每个 case 验证：**操作前后** `tabs.layout` JSON / `panes` 表行数 / `focused_pane_id` 三者状态完全一致（不留脏数据）。

### H.4 布局持久化存储

- **`tabs.layout`** TEXT 字段：存储 JSON 序列化的 `LayoutNode`（tagged union 格式）。
- **`panes`** 表：新表，schema 与 `PaneState` 对齐（不含 `scroll_back`）；`scroll_back` 存 JSON TEXT（同 MVP-04 `tabs` 表 `scroll_back` 模式）。
- `focused_pane_id` 替代原 `tabs` 表隐式单 Pane 假设，支持多 Pane 下 focus 持久化。
- **外键约束**：`panes.tab_id REFERENCES tabs(tab_id) ON DELETE CASCADE`，Tab 关闭时自动级联删除所属 Pane。

MVP-05 Phase A 实施 migration v6 完整 SQL（仿 MVP-04 `migrate_v5` 模式）：

```sql
-- migration v6 · MVP-05 Phase A
CREATE TABLE IF NOT EXISTS panes (
    pane_id      TEXT PRIMARY KEY,
    tab_id       TEXT NOT NULL,
    shell        TEXT NOT NULL,
    cwd          TEXT NOT NULL,
    scroll_back  TEXT NOT NULL DEFAULT '[]',
    created_at   INTEGER NOT NULL,
    FOREIGN KEY (tab_id) REFERENCES tabs(tab_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_panes_tab_created ON panes(tab_id, created_at DESC);

-- 同 migration · tabs 表扩展 layout + focused_pane_id
ALTER TABLE tabs ADD COLUMN layout TEXT NOT NULL DEFAULT '{"kind":"single","paneId":""}';
ALTER TABLE tabs ADD COLUMN focused_pane_id TEXT;

PRAGMA user_version = 6;
```

### H.5 扩展路径

- **v0.2 深度放开**：`LayoutNode` enum 本身无需改动；只需将 `MAX_LAYOUT_DEPTH` 从 2 改为 4，前端 resize 逻辑适配多层级分隔条。Smart Layouts 新增预设枚举值即可。
- **v0.3 Pane Detach**：涉及 IPC contract 大改（`LayoutNode` 需跨窗口），必须走 ADR 流程；当前 §G 的 `PaneState` / `LayoutNode` 暂不预留 `window_id` 字段，避免 YAGNI 污染。

### H.6 · Pane PTY IPC 命名决策

MVP-04 Phase B（PR #82）已落地 5 个 `tab_pty_*` IPC commands（spawn/stdin/resize/signal/kill · payload 用 `tab_id`）。MVP-05 Pane 独立 PTY · IPC 层有两条路径：

| 选项 | IPC 命名 | 和 MVP-04 Phase B 关系 | 前端改动 | 推荐度 |
|---|---|---|---|---|
| A · 新建 `pane_pty_*` 5 commands | `pane_pty_spawn` / `stdin` / `resize` / `signal` / `kill` | 独立命名空间 · MVP-04 不改 | 前端增加 pane 分支 · `tab_pty_*` 保留 | ✅ 推荐 |
| B · 复用 `tab_pty_*` + 加 paneId 参数 | `tab_pty_spawn({ tabId, paneId? })` | 破坏 `PtySpawnRequest` struct（已有 ts-rs binding） | 前端所有 `tab_pty_*` 调用加 `paneId` | ❌ 不推荐 |

**锁定 A**（独立命名）· 理由：

- MVP-04 Phase B 的 `PtySpawnRequest` 已生成 ts-rs binding（`web/src/bindings/PtySpawnRequest.ts`）· 改 struct 会触发前端全量 refactor
- Pane PTY 生命周期和 Tab PTY 独立（Pane 可以关但 Tab 存在 · 反之亦然）· 独立 IPC 表达更清晰
- 未来 v0.3 Pane Detach 时 · `pane_pty_*` 可以独立升级（如加 `window_id` 参数）· 不影响 `tab_pty_*`

**实施约定**（Phase B）：

- `crates/core/src/pane_pty.rs` 新建（仿 `pty.rs` 架构 · 复用 `PtyManager` 基础设施）
- 5 IPC commands：`pane_pty_spawn` / `pane_pty_stdin` / `pane_pty_resize` / `pane_pty_signal` / `pane_pty_kill` · payload 用 `pane_id`
- 2 events：`pane_pty_stdout` / `pane_pty_exited` · payload `{ paneId, ... }`
- Tauri permission：`allow-pane-pty-spawn` / `allow-pane-pty-stdin` / `allow-pane-pty-resize` / `allow-pane-pty-signal` / `allow-pane-pty-kill`（5 permission · 新建 `crates/app/permissions/pane-pty.toml`）
- ts-rs struct：`PanePtySpawnRequest` · `PanePtyStdoutEvent` · `PanePtyExitedEvent`（独立命名 · 不复用 `Pty*`）

---

**自审四问**（2026-04-20 · 2026-04-22 session 15 补第 7 条）：

1. 合法/非法 + 视觉覆盖 ✅ · §H.2 矩阵 6 用例起步 + 回滚测试
2. 非法操作 graceful 拒绝 ✅ · toast 3s + 原子回滚 + 深度检查 Rust 侧硬编码
3. 多 DPI 显式测 ✅ · 手动 QA 项保留
4. 任意嵌套 / 方向键 / Dual AI / Pane Detach 都在 v0.2+/v0.3+ ✅ · §H.1/H.5 明确扩展路径
5. §G ts-rs contract（LayoutNode tagged union + SplitDir string union）✅ · 10 个 IPC struct 全清单
6. §H 布局深度约束 + 持久化迁移路径 ✅ · migration v6 方案 + 原子性约束
7. **对齐 MVP-04 Phase A/B 实施现状 ✅** · FK `tabs(tab_id)` + `PRAGMA user_version = 6` + `pane_pty_*` 独立命名锁定 A 选项
