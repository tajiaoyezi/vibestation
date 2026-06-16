---
id: BUG-001
status: done
---

# BUG-001 · Pane 右分屏 lifecycle 状态机异常

> **类型**：BUG（产品 GUI · pane lifecycle 状态机回滚不完整）
> **状态**：done（session 34 第 6 路尝试成功修复 · 详见 §K）
> **优先级**：中（影响核心 Terminal pane UX · 但非 ship blocker · 已知 workaround = 刷新 app）
> **发现**：2026-05-23 · session 34 · Arbiter 手动测试发现
> **依赖**：无前置（pane 模块独立可修）
> **估时**：~1-1.5d（详化后校准 · 含 TDD）

---

## §A · 复现步骤（Arbiter 实测 · session 34）

1. 启动应用（`pnpm tauri:dev`）· 至少有 1 个 terminal pane（默认）
2. 在该 pane 点击右上"右分屏 (⌘\\)"按钮 **第 1 次** → 成功分屏为左右 2 个 pane（预期）
3. 同一按钮点击 **第 2 次** → ❌ **无效果**（实际状态仍只有 2 个 pane · 等价于只执行了 1 次右分屏）
4. 关闭右侧 pane（X / close 按钮）→ ❌ **右侧出现一片空白**（不是预期的"左侧 pane 撑满"· 而是 splitter 留下空 placeholder）
5. 再次点击左侧 pane 的"右分屏"按钮 → ❌ **无任何反应**（state 已"假装已分屏" · early return）

---

## §B · 症状归因（3 个症状指向同一根因）

| 症状 | 表现 | 假设根因 |
|---|---|---|
| 症状 1 | 点 2 次只生成 1 屏 | split handler 有 idempotent guard / 已分屏状态检查 · 第二次 early return |
| 症状 2 | 关右侧留白 | close pane 没正确清理 splitter / split state · view tree 留 orphan container |
| 症状 3 | 再点击无反应 | 上一步残留的"幽灵分屏状态"让按钮误判 · 跳过创建逻辑 |

**统一根因假设**：split state（"该 pane 已被分屏"标志）与 view tree（DOM container / pane node 实际结构）**解耦** · close pane 时只清 view tree 没清 state · 留下"state 显示已分屏但 view 已无右侧"的不一致状态。

---

## §C · 代码 anchor（session 34 Phase 1 实证定位）

| 文件 | 行 | 内容 |
|---|---|---|
| `web/src/panels/Terminal/PaneTerminal.tsx` | L761-762 | 右分屏按钮 DOM（`title="右分屏 (⌘\\)"` · `aria-label="右分屏"`）· 入口 |
| `web/src/panels/Terminal/PaneTerminal.tsx` | L791-792 | 下分屏按钮（对照组 · 应同步修） |
| `web/src/panels/Terminal/Terminal.tsx` | L655 | `showToast("Pane 分屏失败：${msg}")` · 失败路径 toast · 但用户报告"无反应"= 静默失败（toast 没触发）· 需查 split handler 的早退路径 |
| `web/src/panels/Terminal/usePaneShortcuts.ts` | L5-6 | `⌘\` → split right · `⌘⇧\` → split down · 快捷键路径（验证：是否同样有 bug） |
| `web/src/panels/Terminal/PaneSplitView.tsx` | 全文 | split view 容器 · 关闭 pane 后 splitter / placeholder 清理逻辑 |
| `web/src/panels/Terminal/PaneSplitter.tsx` | 全文 | splitter 自身 · 可能持有 split state |
| `web/src/stores/sessions.ts` / `sessions-context.tsx` | 待 grep | pane 生命周期 store · split / close action 入口 |

---

## §D · Ready-gate 前置（详化时具体化）

ready-gate 翻 `ready` 前需补：

- [ ] §D.1 完整 Read 7 个 anchor 文件 · 确认 split state 持有方（pane store / splitter local / context）
- [ ] §D.2 验证症状是否同样出现在 "下分屏" 按钮（PaneTerminal L791）· 决定 fix scope
- [ ] §D.3 验证症状是否同样出现在 `⌘\` 快捷键路径（usePaneShortcuts L5）· 决定 fix scope
- [ ] §D.4 调研 split state 与 view tree 是否应合并（state machine 单 source of truth）· 还是保留 dual track + 加 reconciliation
- [ ] §D.5 TDD 失败 reproduce 测试（在 `web/...` vitest 范围 · 先写"点 2 次右分屏应有 3 个 pane"测试 · 红 → 绿）
- [ ] §D.6 验证 `pane:*` IPC 是否参与 split lifecycle（MVP-18 引入）· 是否需要后端配合改

---

## §E · 范围（详化时具体化）

- **IN**：右分屏（⌘\）多次点击 · close pane 后 splitter 清理 · 防"幽灵分屏状态"
- **IN-候选**：下分屏（⌘⇧\）镜像修（取决于 §D.2 验证）
- **IN-候选**：`⌘\` 快捷键路径同步修（取决于 §D.3）
- **OUT**：Smart Layout 预设（SmartLayoutMenu · 不同代码路径）· 单独 BUG 立项
- **OUT**：MVP-18 pane:link IPC（除非 §D.6 验证证实关联）

---

## §F · Acceptance（详化时具体化）

- [ ] TDD 失败 reproduce 测试先写（红）· 修复后绿（vitest 全套 exit 0 · 防 [[frontend-test-isolation-fullsuite-regression]]）
- [ ] 手动复现 3 症状都消除（Arbiter 本地 `pnpm tauri:dev` 实测）
- [ ] 点击 N 次右分屏（N ≥ 3）应正确生成 N+1 个 pane（或按设计 cap 例如 4 · spec 明确）
- [ ] 关闭中间 pane 后 view tree 自动 reflow · 无空 placeholder
- [ ] 关闭后再点击应正确创建新 pane
- [ ] 下分屏 / 快捷键路径（§D.2/§D.3 验证后定）同步修
- [ ] `cargo test --workspace` 不破坏（若 §D.6 涉及 backend）
- [ ] `pnpm lint && pnpm typecheck && pnpm vitest run` 全绿

---

## §G · 自审四问（CLAUDE.md 📝 触发器）

- **递归完备性**：本 spec 自己在 `docs/tasks/README.md` 索引吗？详化为 `ready` 时同步更新（含 BUG 类前缀 · 如 README 无 BUG 列 · 新增"已知 BUG"段）。✅
- **反向场景**：不修风险？答：核心 UX 阻塞（分屏是 Terminal pane 主功能）· 用户每次只能分一次 · 关后必须刷新 app 才能再分 · 影响 v0.1 GA 可用性。**应 prioritize 修**。
- **边界适用性**：本修适用所有 pane 类型吗？terminal / runner / log pane 是否共用 split 逻辑？详化时确认。
- **YAGNI**：现阶段真需要修吗？答：v0.1 GA 候选态 · 此 bug 是 P1（影响核心 UX）· 必修。✅

---

## §H · 历史

- 2026-05-23 · session 34 · Claude Code（主 agent · Arbiter 手动测试期间发现）
- Phase 1：定位 4 anchor · 3 症状归因
- Phase 2：实证 backend OK（panes.rs:1688 / 1720 测试覆盖 split→single collapse 与 roundtrip）· instrumentation log 实测确认 frontend root cause
- Phase 3：根因 = `RenderSplit` 内 `const split = props.layout;` 非 reactive capture · 嵌套 split 场景下内层递归 PaneSplitView 收 stale layout · 新 pane 不渲染
- Phase 4：**3 路 fix 全部失败** · per `~/.claude/rules/always/08-systematic-debugging.md` 红旗规则 STOP

## §J · session 34 失败尝试 audit + 下次方向

### 失败尝试 1：throw on stale read（用 createMemo + 显式 throw 标 invariant）

```ts
const split = createMemo(() => {
  if (props.layout.kind !== "split") {
    throw new Error("RenderSplit invoked with non-split layout");
  }
  return props.layout;
});
```

**实测失败**：dev webview 触发 toast `Pane 分屏失败：RenderSingle invoked with non-single layout`。SolidJS reactive batch ordering 中 · 外层 `<Show when={kind === "single"}>` 切换分支前 · 内层 memo 短暂 re-evaluate stale layout · throw 被 SolidJS error boundary 抓住转 toast。

### 失败尝试 2：嵌套 `<Show when={memo()}>` keyed render prop

```ts
const singlePaneId = createMemo<string | null>(() =>
  props.layout.kind === "single" ? props.layout.paneId : null,
);
return <Show when={singlePaneId()}>{(paneIdAccessor) => ...}</Show>;
```

**实测失败**：dev webview 触发 `Pane 分屏失败：null is not an object (evaluating 'node.owned[i]')`。SolidJS internals · 嵌套 `<Show>` + render prop 在 outer `<Show>` 切换分支时 · inner reactive owner 的 owned computations 数组遇 null entry · cleanup race。

### 失败尝试 3：`createMemo<T>((prev) => kind === expected ? props.layout : prev, initial)` prev-fallback

```ts
const splitLayout = createMemo<SplitLayout>(
  (prev) => (props.layout.kind === "split" ? props.layout : prev),
  props.layout as SplitLayout,
);
```

**实测失败**：dev webview 关闭 pane 时触发 `Pane 关闭失败：null is not an object (evaluating 'node.owned[i]')`。同 #2 同 SolidJS internal error · prev-fallback 没解决 owner cleanup race。

### 共同模式 · 为什么 vitest GREEN 但 dev FAIL

- vitest jsdom 环境的 SolidJS reactive scheduler 行为与 Tauri webview 不一致
- jsdom 中 reactive batch flush 完成后才 evaluate · 不会触发 owner cleanup race
- webview 中 batch 内 owner.owned 数组操作可能命中 SolidJS internal edge case
- 不能仅靠 vitest 通过判定 fix 工作 · 必须 dev 实测

### 下次 cold session 修复方向（架构级）

按 systematic-debugging Phase 4 红旗 · 不再补丁式修 · 考虑架构级方案：

1. **方案 A**：把 `RenderSingle` / `RenderSplit` 接收 narrowed prop type（`SingleLayout` / `SplitLayout`）· 在 PaneSplitView 顶层用 `<Switch><Match>` keyed 模式 narrow · 内部组件不再做 type guard
2. **方案 B**：用 `untrack` 包装 type guard · 阻断 reactive 追踪：`const split = untrack(() => props.layout as SplitLayout)` —— 但这又回 capture
3. **方案 C**：用 SolidJS `<Switch><Match keyed>` 强制 unmount+remount · 牺牲 xterm.js 状态保留换正确性
4. **方案 D**：升级 SolidJS 到最新版（看是否修了 `node.owned[i]` race）· 或开 issue 上报 SolidJS
5. **方案 E**：用 SolidJS `createDeferred` 或 `splitProps` 隔离 layout reactive scope · 减少 owner.owned 复杂度

**推荐起点**：方案 A · narrowed prop type · 最干净 · 不依赖 SolidJS internal 行为。

### 已交付的有效产出（保留进 git）

- 本 spec stub 含完整 anchor + 失败尝试 audit · 下次类似 SolidJS reactive bug 直接套用
- `web/tests/panels/Terminal/PaneSplitView.test.tsx` 新加 2 个测试 case：
  - PASS · `BUG-001 · collapses split → single when layout shrinks`（验证顶层 split→single 切换正常）
  - PASS · `BUG-001 · renders new pane when split nests deeper (real reproduce)`（嵌套 split 渲染新 pane）
- backend `apply_pane_close` 验证正确（panes.rs:1688 / 1720）· 排除一类假设

## §K · 实际成功修复（session 34 第 6 路尝试）

**方案**：plain getter function + JSX ternary 替代 createMemo + 嵌套 Show。

```tsx
// PaneSplitView.tsx · RenderSplit 关键代码
const splitNode = (): SplitLayout | undefined =>
  props.layout?.kind === "split" ? props.layout : undefined;
const direction = (): SplitDir => splitNode()?.direction ?? "horizontal";
const first = (): LayoutNode | undefined => splitNode()?.first;
const second = (): LayoutNode | undefined => splitNode()?.second;
// ...

return (
  <div class={`vs-pane-split vs-pane-split-${direction()}`} style={styleProp()}>
    <div class="vs-pane-split-first">
      {first() && (
        <PaneSplitView layout={first()!} ...其他 props />
      )}
    </div>
    <PaneSplitter direction={direction()} ratio={ratio()} ... />
    <div class="vs-pane-split-second">
      {second() && (
        <PaneSplitView layout={second()!} ...其他 props />
      )}
    </div>
  </div>
);
```

**关键设计原则**：

1. **Plain getter function 不是 createMemo**：每次 JSX 内访问都 evaluate `props.layout?.X`· SolidJS props proxy 保证 reactive · 但**不创建额外 owner.owned scope**（`createMemo` 会创建 owned computation · 嵌套递归 PaneSplitView 时 owner.owned 数组复杂度爆炸 · 触发 SolidJS internal cleanup race）
2. **Optional chaining + nullable fallback**：`props.layout?.kind === "split" ? props.layout : undefined`· `<Match>` / `<Show>` 切换分支前的 short-window stale read 返回 undefined 而非 throw
3. **JSX ternary `{cond && <X />}` 替代嵌套 `<Show>` render prop**：ternary 是 SolidJS 编译时识别的 reactive 模式 · 不创建额外 reactive scope · 不嵌套 owner

**为什么前 5 路全失败 · 第 6 路成功**：

| 路径 | createMemo | 嵌套 Show | 结果 |
|---|---|---|---|
| 1 · throw on stale | ✓ | ✗ | toast: invoked with non-X layout |
| 2 · 嵌套 Show + nullable accessor | ✓ | ✓✓ | toast: `node.owned[i]` |
| 3 · prev-fallback memo | ✓ | ✗ | toast: `node.owned[i]` |
| 4 · 方案 A narrowed prop + Switch/Match | ✓ | ✗ | toast: `props.layout undefined` |
| 5 · 方案 A + defensive memo | ✓✓ | ✓ | toast: `node.owned[i]` + 全空白 |
| **6 · plain getter + JSX ternary** | **✗** | **✗** | **✅ 修复成功** |

共同模式：`createMemo` 用得越多 / Show 嵌套越深 · owner.owned 复杂度越高 · 越易触发 SolidJS internal cleanup race。**plain getter 是 SolidJS reactive primitive · 不增加 reactive scope · 避开整个问题**。

**经验沉淀**（写入 vibestation memory）：

- SolidJS 中**递归组件**渲染嵌套结构时 · 内部派生字段**首选 plain getter function**· 避免 createMemo（除非性能 critical 且确定不嵌套）
- 优先 `{cond && <X />}` ternary · 避免 `<Show when={cond}>{(x) => ...}</Show>` keyed render prop（嵌套 reactive scope 易触发 SolidJS internal cleanup race）
- Optional chaining `props.X?.Y` 是 SolidJS unmount race 的标准防御 · 不是 type safety 弱化

---

> **本 BUG 是 v0.1 GA 候选阶段发现 · 影响核心 UX · 应 Arbiter 自定 prioritize 窗口修复**。
>
> 关联 memory：[[diffline-shiki-flaky-investigation-stub]]（同 session 终止模式 · 但本 BUG P1 vs DiffLine flaky 🟠 audit polish · 本 BUG 优先级高）。
