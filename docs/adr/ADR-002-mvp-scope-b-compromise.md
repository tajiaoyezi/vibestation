# ADR-002: MVP v0.1 范围 = B 折中方案

**状态**：accepted
**日期**：2026-04-18（Phase 1 锁定 · Phase 3 ADR 建立）
**决策者**：项目发起人 · 多 agent 评审
**对应 `CLAUDE.md` 决策表**：#2

---

## 背景与问题

Vibestation v1.0 的完整 vision 包含：
- 多 Tab 终端 + AI CLI 集成
- JetBrains 级 Git 工作台（含 push/pull/fetch / rebase / cherry-pick）
- 自绘 commit rail graph
- AI-Aware Pane 联动（session ↔ commit 自动绑定 · 一键回滚）

全量实施预估 **24-25 周**（`implementation-plan.md §0.1 v1 估算`）。

**问题**：v1 规划没考虑个人开发者的真实 capacity：
- 每周投入 15 小时（兼职项目）
- 跨平台（mac + linux + windows）同步支持会拖长 2-3 倍
- AI-Aware 部分依赖 R1（Claude/Codex CLI 协议解析）· 未实机验证前估时不可靠

需要缩紧 v0.1 范围到"**用户可实际用 + 工期可控 + 不走死胡同**"的子集。

## 决策驱动因素

- **D1 · 工期可控**：v0.1 GA 目标 **12-14 周**（比 v1 减半）
- **D2 · 用户实际可用**：砍哪些不影响"能替代 Ghostty + VSCode Git UI 的 80% 日常"
- **D3 · 不走死胡同**：砍的部分 v0.2/v0.3 可以补 · 不造成架构债
- **D4 · 叙事合规**：v0.1 对外不提 AI-Aware · `CLAUDE.md` #3 禁区硬约束

## 考虑的选项

- **选项 A · 全量（v1 原计划）**：24-25 周 · 工期超过个人 capacity · **否**
- **选项 B · 折中方案**：保留核心 + 砍高成本 features · 12-14 周 · 见下方具体清单
- **选项 C · 极简核心**：仅多 Tab 终端 + Git Log 只读 · 6-8 周 · 用户端"不够用"（无 commit / 无 diff）· **否**
- **选项 D · v0 伪发布**：不发布 GA · 仅技术 preview · 不对齐 GA 承诺 · **否**

## 决策

**选择**：选项 B · **B 折中方案**

**v0.1 保留**（13 项）：
1. Tauri 应用骨架 + 启动 / 崩溃恢复
2. Workspace 管理 + 项目识别
3. Tool Windows 布局（Calm Studio 视觉）
4. 多 Tab 终端（PTY + xterm + Shell/CLI）
5. **Pane 分屏**（单层嵌套 · 最多 4 Pane · 2 预设：Solo / AI+Runner）
6. 配置导入（Ghostty + iTerm2 + Alacritty）
7. Git Log 只读视图（含分支 / tag 标签贴 · **无 rail graph**）
8. Diff 基础视图（自绘 · 行级对比 · **无语法高亮**）
9. Git Status 只读面板
10. Stage / Unstage + Commit（本地写 · **不含 push/pull/fetch**）
11. 多 workspace 同时打开
12. 双平台打包 + 签名（macOS dmg 公证 + Linux AppImage）
13. 终端正确性矩阵 100% 通过（`implementation-plan.md §10.6` · 15 项）

**v0.1 砍到 v0.2**（5 项）：Push/Pull/Fetch · 自绘 rail graph · 分支 CRUD · Pane 任意嵌套 · 方向键导航

**v0.1 砍到 v0.3**（4 项）：Diff 语法高亮 · Rebase/Merge/Cherry-pick · Pop to External · Pane Detach

**v0.1 砍到 v1.0**（3 项 · 严格不宣传）：AI-Aware 联动 · session ↔ commit 绑定 · AI 一键回滚

## 后果

### 正面

- **工期现实化**：12-14 周 vs 24-25 周 · 兼职个人开发者可行
- **用户实际可用**：保留 13 项核心覆盖 "Ghostty + VSCode Git UI 的 80% 日常"
- **架构前置**：Pane / PTY / 数据模型 v0.1 就到位 · v0.2/v0.3 是"补功能"不是"重写"
- **叙事合规**：v0.1 对外**不**提 AI-Aware · 避免"做一半被喷"

### 负面

- **功能竞争力弱**：v0.1 vs Fork / Tower · 功能少 40% · 卖点偏终端 + Calm Studio 视觉
- **v0.2 压力**：砍到 v0.2 的 5 项加起来约 5-6 周 · v0.2 到 v0.3 间隔约 2 个月

### 风险

- **scope creep**：实施中临时加 "v0.2 的某项小功能" → 工期溢出 · 对策：PR 提案触发"是否属于 v0.1 范围"合规 check（`implementation-plan.md §10.1` 是唯一依据）
- **descoping tree 触发**：若周投入下降 → 按 `§10.5 descoping tree` 进一步砍（仅 Ghostty 不导入 iTerm2/Alacritty · 砍 Pane 分屏 · macOS-only 等）

## 与 `implementation-plan.md` 的映射

- 对应章节：§10.1 功能清单 · §10.5 descoping tree · §10.6 终端正确性矩阵
- 对应风险：R31（功能 vs 工期失衡 · 本 ADR 前置缓解）

## 相关

- `CLAUDE.md` 决策表：#2
- 详细 spec：[MVP-01..10](../tasks/) 详细 · [MVP-11..20](../tasks/) 占位
- 相关 ADR：ADR-009（AI-Aware v1.0 vision · 定义"不宣传"范围）

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code
