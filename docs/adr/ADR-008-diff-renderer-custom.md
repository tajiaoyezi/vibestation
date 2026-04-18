# ADR-008: Diff 渲染 = 自建（`diff` crate + Canvas/HTML）· 不用 Monaco

**状态**：accepted
**日期**：2026-04-18（Phase 1 锁定 · Phase 3 ADR 建立）
**决策者**：项目发起人 · 多 agent 评审
**对应 `CLAUDE.md` 决策表**：#7

---

## 背景与问题

Git 工作台的 Diff 视图是核心用户触点（点 commit → 看改了什么）。

候选实现：
- **Monaco Editor**：VSCode 核心 · 功能完备 · 但巨型（~3MB gzipped）
- **CodeMirror 6**：轻量但功能差 Monaco 一档
- **自建**：基于 `diff` crate 算法 + Canvas / HTML 自绘

MVP v0.1 明确范围（`implementation-plan.md §10.1`）：
- "**Diff 基础视图（自绘 · 行级对比 · 无语法高亮）**"
- 语法高亮推到 v0.3（MVP-15）

## 决策驱动因素

- **D1 · Bundle 约束**：Monaco 3MB >> v0.1 总前端预算 150kb
- **D2 · 视觉一致性**：Calm Studio 审美 · Monaco / CodeMirror 有强主题偏见
- **D3 · 性能可控**：大文件 Diff（10MB 级）自建可做虚拟滚动 · Monaco 在此边界会卡
- **D4 · 未来扩展**：v0.3 加 tree-sitter 高亮 · 自建路径比改 Monaco plugin 简单

## 考虑的选项

- **A · Monaco Editor**：功能满 · Bundle 爆炸（3MB+）· 主题样式偏执 · **拒绝**
- **B · CodeMirror 6**：轻量（~200kb）· 功能中等 · 但仍是"外部编辑器 API" · 不契合"只读 Diff 视图" · 拒绝
- **C · shiki / prism.js 高亮 + 自绘 diff layout**：复杂 · v0.1 用不上
- **D · `diff` crate + Canvas 自绘**：MVP v0.1 目标 · bundle 最小 · 视觉完全受控
- **E · `diff` crate + HTML + CSS**：DOM 节点数 · 1k 行 diff = 1k+ div · SolidJS 虚拟化可解

## 决策

**选择**：选项 D + 选项 E 混合（**自建**）
- **Diff 算法**：Rust 侧 `similar` crate（或 `diff` crate）生成 unified diff + word-level diff
- **渲染**：
  - **默认**：HTML + CSS · SolidJS virtualized list（行数 < 10k 场景）
  - **大文件路径**：Canvas 渲染（行数 > 10k · v0.2+ 优化）
- **语法高亮**：v0.1 **无**（`implementation-plan.md §10.1` 明确砍）· v0.3 MVP-15 加 tree-sitter
- **禁区**：`CLAUDE.md §禁区` 明确不用 Monaco

**理由**：
1. **Bundle 预算**：Monaco 3MB >> v0.1 整体预算 · 直接排除
2. **MVP 范围已锁定**：`§10.1` 明确 "自绘 · 行级对比 · 无高亮" · 自建路径与 scope 完美匹配
3. **视觉一致性**：Calm Studio 主题 · 无需和 Monaco / CodeMirror 的主题 API 对抗
4. **扩展明确**：v0.3 加高亮走 MVP-15（tree-sitter）· 路径清晰

## 后果

### 正面

- **Bundle 极小**：`similar` crate ~50kb（Rust 侧编译进 Tauri 后端）· 前端 Diff 组件 < 20kb · 远低于 Monaco 3MB
- **视觉完全受控**：SolidJS 组件 · Calm Studio token 直接应用
- **性能可控**：virtualized list · 10 万行 diff 不崩
- **学习成本低**：贡献者不需要学 Monaco API

### 负面

- **功能少 Monaco 一档**：v0.1 无高亮 / 无 inline comment / 无 minimap · 对齐 `§10.1` 已砍清单
- **实现工期**：Diff 视图 MVP-08 估时 5 天 · 用 Monaco 应该 2 天 · 取舍：工期 vs bundle
- **v0.3 加高亮成本**：tree-sitter 集成估 6 天（MVP-15）

### 风险

- **大文件性能**：10MB 级 diff 不做 virtualization 会崩 · 对策：SolidJS `<For>` + IntersectionObserver 虚拟化
- **word-level diff 复杂度**：word 粒度对比比行粒度难做对 · v0.1 先做行级 · v0.3 加 word
- **代码 review 期望落差**：用户期待"VSCode 级" Diff UI · v0.1 基础 UI 可能被吐槽 · 已预期并通过 `§10.1` 锁定

## 与 `implementation-plan.md` 的映射

- 对应章节：§3.1（前端栈不用 Monaco）· §10.1（Diff 基础视图砍到 v0.3 高亮）
- 对应风险：无专属 R 号 · 若 Diff 视图用户反馈差 → descoping tree `§10.5` 有"砍 Diff 视图"路径

## 相关

- `CLAUDE.md` 决策表：#7
- 详细 spec：[MVP-08 Diff 基础视图（自绘）+ Git Status 只读面板](../tasks/MVP-08-diff-and-git-status.md)
- v0.3 高亮 spec：[MVP-15 Diff 语法高亮（tree-sitter）](../tasks/MVP-15-diff-syntax-highlight.md)
- 相关 ADR：ADR-004（SolidJS · Diff 视图的前端框架）· ADR-007（Git 栈 · Diff 数据来源）

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code（Phase 3 · 把 Phase 1 锁定决策正式化为 ADR）
