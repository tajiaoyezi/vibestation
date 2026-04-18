---
id: MVP-09
type: mvp
title: Stage/Unstage + Commit 操作（git2 写）
status: draft
owner:
phase: W10-W11
depends_on: ["MVP-08", "SPIKE-04"]
blocks: []
blocked_by: []
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1
risk_ref:
reviewer:
---

# MVP-09: Stage/Unstage + Commit 操作

> **状态**：`draft`
> **依赖**：MVP-08（Status 面板）· SPIKE-04 §C（git2 写路径 smoke test）

---

## 🎯 目标（Goal）

在 Git Status 面板上支持 Stage / Unstage 单文件或整体操作，+ Commit UI（勾文件 + 输入 message + 可选 amend）。**不含 push/pull/fetch**（v0.2）。

## 📖 背景（Context）

- `implementation-plan.md §10.1` MVP B 折中方案：**保留 commit，砍 push/pull/fetch**
- `CLAUDE.md` #13（B 栏 → SPIKE-03 锁定）：Git 栈 **写路径用 git2 0.20**

---

## 🎨 功能范围（Scope）

**Do**：
- Stage 操作：
  - 单文件 Stage：Status 面板每行 ✓ 按钮
  - Stage All Unstaged：组标题 "Stage All" 按钮
  - Stage All Untracked：同上
- Unstage 操作：
  - 单文件 Unstage：Staged 组每行 ✗ 按钮
  - Unstage All Staged：组标题 "Unstage All" 按钮
- Commit UI：
  - Status 面板底部：消息框 + "Commit" 主 CTA + "Amend" 复选框
  - 多行消息支持（subject + body 分离，blank line 自动插入）
  - Commit 成功后 Status 刷新 + toast 提示 + Git Log 刷新
- Author 信息：从 git config 读取 `user.name` / `user.email`
- 快捷键：`⌘↵`（mac）/ `Ctrl+↵` 提交

**Don't**：
- Push / Pull / Fetch（v0.2）
- Branch operations（v0.2）
- Rebase / Merge / Cherry-pick（v0.3）
- Commit signing（GPG）（v0.2+）
- Partial staging（stage hunks）（v0.2）

## 🖼 UI 引用

- Status 面板底部：消息框 + CTA 按钮（参考 `design/directions/1-calm-studio.html` Bottom Panel）
- Commit 消息框：等宽字体（JetBrains Mono），72 字符宽度提示线

## ✅ Acceptance

### A. Stage / Unstage

- [ ] Unstaged 组每行有 ✓ 按钮 → 点击 stage 该文件
- [ ] Staged 组每行有 ✗ 按钮 → 点击 unstage 该文件
- [ ] 组标题有 "Stage All" / "Unstage All" 批量按钮
- [ ] 操作后 Status 面板立即刷新（乐观 UI + 实际 git call + 校正）

### B. Commit

- [ ] Status 面板底部有消息框（多行）
- [ ] "Commit" 按钮 disabled 状态：
  - No staged files → disabled + tooltip "No staged changes"
  - Empty message → disabled + tooltip "Commit message required"
- [ ] 点击 Commit：
  - 调用 git2 创建 commit 使用 staged tree
  - Author / committer 从 `git config` 读
  - Message 规范：第一行 subject（< 72 字符建议）+ blank + body
  - 支持中文 message（UTF-8）
- [ ] "Amend" 勾选：Commit 修改最后一个 commit（`git commit --amend`），message 自动填上 commit
- [ ] Commit 成功：toast "Committed {shortsha}" + Status 刷新 + Git Log 刷新（MVP-07）

### C. 错误处理

- [ ] git2 调用失败 → 明确错误提示 + 保留消息不清空
- [ ] 没有 identity（`user.name` 未设）→ 弹对话框让用户输入（写入 local git config）
- [ ] Detached HEAD → 允许 commit 但警告

### D. 性能

- [ ] Stage 单文件 < 100ms
- [ ] Commit < 500ms（典型仓库）
- [ ] Stage All 1000 文件 < 2s

### E. 测试 fixture

- [ ] 正常 commit（单文件 / 多文件）
- [ ] 空 staged 试 commit → 拒绝
- [ ] Amend
- [ ] 中文 message + 中文文件名
- [ ] `.gitignore` 外的 untracked 文件 stage
- [ ] 已 staged 后 working tree 又改 → Status 正确显示两份（staged 和 unstaged 同文件）

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | git2 wrapper（Repository / Index / Tree）|
| 集成 | Stage → Commit → Log 读新 commit 链路 |
| E2E | 完整 flow：改文件 → Status → Stage → Commit → 验证新 commit |
| 手动 QA | Amend / detached HEAD / 权限问题 |

## 💾 数据模型变更

无新 table。所有变更落在 git repo 本身。

## ⚠️ 已知风险

- **用户 git config 不完整**（user.name/email 未设）：弹窗引导用户填，写入 `<repo>/.git/config` local 而非 global（避免污染全局）
- **中文 commit message 编码**：SPIKE-04 §C 已验证 git2 UTF-8 支持
- **Pre-commit hooks**：若 repo 有 pre-commit hook 可能拖慢 commit → 不改 git2 行为，但 UI 显示"Committing..."转圈

## 📝 Notes

- MVP-09 的 commit **不签 GPG**（keychain 集成复杂，v0.2+）
- 不做 "Commit Verification"（v0.3+）
- 未来 push 按钮会出现在 Commit 成功 toast 上（v0.2 接入）

## 🔗 相关

- `CLAUDE.md` #13
- SPIKE-04 §C git2 写 smoke test
- 上游：MVP-08 · SPIKE-04
- 下游：v0.2 push/pull

---

**自审四问**：1. Stage + Commit + 错误 + 性能覆盖 ✅ · 2. user.name 缺失 / hooks graceful ✅ · 3. 中文 message / 文件名显式测 ✅ · 4. push/pull/signing/hunks 都在 v0.2+ ✅
