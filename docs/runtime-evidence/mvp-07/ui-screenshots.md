# MVP-07 UI Screenshots · Technical Debt 索引

## ⚠️ 降级声明（不 block merge · GA gate 补齐）

本 PR 交付 agent（OpenCode）在 **CLI 自动化会话** · **无 display server / 无 macOS 桌面**·
按 prompt §Runtime 证据应提交的 3 张 PNG 截图未能生成。

主 agent（Claude Code）review 时考察了两条路径：

1. **主 agent 代跑 `pnpm tauri:dev` + 手工截图**：需占用用户 macOS 桌面焦点 · 用户手工操作
   UI（创建 workspace / 展开 Secondary Sidebar / 输入搜索关键词）· 预计 10 min
2. **降级为 technical debt 索引 + GA gate 补齐**：本文件模式 · Arbiter 批准（2026-04-21）

**Arbiter 选定方案 2**· 理由：
- 代码质量由 **92 cargo tests**（含 **26 git_log 模块测试**）保证 · 含 query/pagination/filter/
  error/branch labels/commit detail 全路径
- H2 regression proof 已做（ts-rs drift 防御 · 见 `H2-regression-proof.md`）
- 和 PR #82（Codex MVP-04 Phase B）降级模式**对齐**：Codex 也只给静态 smoke gif + Rust 集成
  测试双覆盖 · 同一严谨度级别
- GA 前开 app 补齐 3 张图成本低（5-10 min · 任一后续 session）

---

## GA gate · 需补的 3 张截图（按 dispatch prompt §Runtime 证据要求）

### 01-log-list.png · Git Log 列表状态

**复现步骤**：
1. `pnpm tauri:dev` 启动 app
2. 创建 workspace 指向**本 vibestation repo**（含 `.git` · 140+ commits · 够展示 pagination）
3. 展开 Secondary Sidebar（MVP-03 layout · 快捷键或 toggle button）
4. **预期看到**：
   - 搜索 / 筛选栏（两个输入框：message · author）
   - Paginated commit 列表 · 每行显示 short_sha · 相对时间 · message 首行 · author name
   - 分支 / tag 标签贴（如 main → 当前 HEAD · 彩色区分）
   - "Load more" 按钮（若 `hasMore=true`）

**截图工具**：macOS `Cmd+Shift+4` · 框选 Secondary Sidebar 区域 · 保存到
`docs/runtime-evidence/mvp-07/01-log-list.png`

### 02-commit-detail.png · Commit 详情展开

**复现步骤**：
1. 续 01 · 点任一 commit 行
2. **预期看到**：
   - 详情区 slide-up（或另一 pane / 另一 Tab · 取决于前端实现）
   - 显示：full SHA · author email · author/commit date · parents(s) · message 全文
   - 文件变更列表：每行 "path +additions -deletions" + 状态（M/A/D/R · 彩色角标）
3. 截 Cmd+Shift+4 · 保存 `docs/runtime-evidence/mvp-07/02-commit-detail.png`

### 03-filter-search.png · 搜索过滤结果

**复现步骤**：
1. 续 02 · 在搜索栏输入 `feat` 或 `fix` 或 `docs`
2. **预期看到**：
   - 列表立即过滤（debounce 300ms）· 只显示 message 含关键词的 commit
   - 空结果时显示 "no matching commits" placeholder
3. 截图 · 保存 `docs/runtime-evidence/mvp-07/03-filter-search.png`

---

## 前端 CSS / design token 参考（已实施 · 无需补证）

所有 panel 样式走 Calm Studio design tokens · 无硬编码色值：
- `--bg-0/1/2` · 背景层级
- `--text-1/2/3/4` · 文字层级
- `--accent` / `--accent-soft` · 交互高亮
- `--font-mono` / `--font-ui` · 字体
- `--space-*` · 间距节奏
- `--r-*` · border radius
- `--dur-*` / `--ease` · 动画时长/缓动

这部分已在 `web/src/styles.css` +285 行 + `web/src/panels/GitLog/GitLogPanel.tsx` 321 行实施 ·
代码审查 review 已过。

---

## 责任人 / 触发时机

| 场景 | 责任人 | 动作 |
|---|---|---|
| 下一次 `pnpm tauri:dev` 任务（如 MVP-04 Phase C xterm 前端实施）| 该 executor 顺便补 | 3 张截图 + 覆盖本文件 |
| v0.1.0 GA 发布前 PR | Arbiter 指定的 agent | 专门 PR `chore(mvp-07): 补 UI 截图证据` |
| 最迟节点 | v0.1.0-alpha 发布前 · 含 MVP-07 feature 对外公开前 | block alpha 发布 |

**不作为 v0.1 GA blocker 的条件**：本文件存在 + spec §Acceptance 已被 cargo tests 覆盖 +
PR body 在 Arbiter approval 段显式授权降级。

---

## 相关

- PR #83（本 PR · OpenCode MVP-07 · 主 agent 代修 rebase + reset-author）
- PR #82（Codex MVP-04 Phase B · 同级降级 · 静态 gif + Rust 集成测试覆盖）
- ADR-011（runtime evidence location · R1-R5）
- Dispatch prompt：[`spike-tmp/dispatch/MVP-07-git-log-opencode-prompt.md`](../../../spike-tmp/dispatch/MVP-07-git-log-opencode-prompt.md)
