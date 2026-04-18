# Branch Protection · 分支保护配置指南

> **状态**：Phase 4 落地文档（2026-04-18）
> **范围**：main 分支保护规则的**推荐配置** · 需在 GitHub 仓库 Settings 里手动应用
> **背景**：`docs/tasks/README.md §原则 7`（state transition advisory · Codex PR #10 F1）· `CLAUDE.md §禁区`（禁止 push main）

---

## 🎯 目的

本文件说明应该在 **GitHub Repository Settings → Branches → Branch protection rules** 里为 `main` 分支配置哪些规则，以及每条规则消除什么具体风险。

GitHub 分支保护**不能写进 repo**（GitHub 产品设计），所以本文档是 repo admin 应用的 **checklist**，实际配置通过 GitHub web UI / `gh api` 完成。

---

## 🔒 推荐配置（main 分支）

### 1. Require a pull request before merging · 禁止直推 main

- ✅ **Require a pull request before merging**
  - ✅ Require approvals · **1 reviewer minimum**
  - ✅ **Dismiss stale pull request approvals when new commits are pushed**
    - 消除 Codex PR #6 F1 / PR #10 F1 的漏洞：作者在 approve 后私改 spec 再 merge
    - 等价于"翻转 gate (b)"：Author push 后 Reviewer re-approve 最新 HEAD
  - ✅ Require review from Code Owners（Phase 5 加 `.github/CODEOWNERS` 后启用）
  - ⛔ Require approval of the most recent reviewable push（**不勾** · 太严格会阻塞 bot PR · dependabot）

**消除风险**：
- `CLAUDE.md §禁区 1`：禁止 push 到 main
- `docs/tasks/README.md §原则 7`：state transition advisory → 变 enforced
- Codex PR #6 F1 / PR #10 F1：post-approval private flip

### 2. Require status checks to pass before merging

- ✅ **Require status checks to pass before merging**
  - ✅ **Require branches to be up to date before merging**
  - **Required status checks**（勾选以下全部）：
    - [ ] `Markdown lint · 文档一致性`（from `.github/workflows/ci.yml`）
    - [ ] `gitleaks`（from `.github/workflows/secret-scan.yml`）
    - [ ] `Guard · inline gitleaks:allow 检测`（from `.github/workflows/secret-scan.yml` · **Codex PR #11 F3** · 防 `# gitleaks:allow` 内联 bypass · 该 guard job 的 name 字段必须与此处一致）
    - [ ] `Validate task spec frontmatter`（from `.github/workflows/task-spec-validator.yml`）
    - [ ] `Pre-code status · 当前阶段说明`（always-green 指示器）
    - 以下 Spike W0 后启用：
      - [ ] `Rust · cargo check`
      - [ ] `Frontend · pnpm lint/typecheck`

> ⚠️ **Required check 选型关键** · Codex PR #11 F1 教训：
> - 所有 required check 的 workflow **必须无 `paths` 过滤 · 总是触发**
> - 带 `paths` 过滤 + 被设为 required → 无关 PR 永久 Pending 卡死合并
> - 正确 pattern：workflow 无 `paths` · job 内用 `git diff` 判断是否需实质校验 · 无关则 echo skip + `exit 0`
> - 当前所有 required workflow 已按此 pattern 改造（见 `.github/workflows/task-spec-validator.yml` / `secret-scan.yml`）

**消除风险**：
- SPIKE-06 A.5.3 F4：merge 前硬阻塞 gitleaks 扫描
- **Codex PR #11 F3**：防 gitleaks inline bypass（`# gitleaks:allow` 内联绕过被 CI 拒绝）
- **Codex PR #11 F1**：防 required check pending 死锁
- `docs/tasks/README.md §原则 7`：frontmatter validator enforced

### 3. Require conversation resolution before merging

- ✅ **Require conversation resolution before merging**

**消除风险**：评审意见被 silent merge

### 4. Require linear history

- ✅ **Require linear history**
  - 强制 squash / rebase merge · 不允许 merge commits（便于 git log 阅读）

### 5. Do not allow bypassing the above settings

- ✅ **Do not allow bypassing the above settings**
  - **但允许 administrators bypass**（glich 场景下 maintainer 紧急操作）· GitHub 默认

**风险权衡**：完全不允许 bypass 可能让紧急 hotfix 不可用；允许 admin bypass 是业界惯例，需配合"admin bypass 后立刻补 post-mortem"（写入 `docs/session-history/`）

### 6. Restrict who can push to matching branches

- ✅ **Restrict who can push to matching branches**
  - Allowed: 仓库 admin + maintainer team（Phase 5 组织化后细化）

### 7. Rules applied to everyone including administrators

> ⚠️ **不推荐全打开**（会让 maintainer 紧急 revert 无法执行）。保留 admin bypass。

- ⛔ Require signed commits（Phase 5+ 按需启用 · 门槛较高）
- ⛔ Require deployments to succeed before merging（暂无 deployment）
- ⛔ Lock branch（仅冻结旧版本用）

---

## 🛠 如何应用（两种方式）

### 方式 A · GitHub Web UI（推荐 · 一次性操作）

1. 打开 `https://github.com/tajiaoyezi/vibestation/settings/branches`
2. 点击 `Add branch protection rule`
3. Branch name pattern: `main`
4. 按上面 §1-6 逐项勾选
5. **Required status checks** 需要先让本 PR merge 进 main 让 workflow 在 GitHub 注册 → 之后才能从下拉菜单选到
6. `Create`

### 方式 B · `gh api`（可版本化 · 但参数复杂）

```bash
# 示例（参数需按实际 status check 名字调整）
gh api --method PUT \
  -H "Accept: application/vnd.github+json" \
  /repos/tajiaoyezi/vibestation/branches/main/protection \
  -f required_status_checks.strict=true \
  -f required_status_checks.contexts[]="Markdown lint · 文档一致性" \
  -f required_status_checks.contexts[]="gitleaks" \
  -f required_status_checks.contexts[]="Validate task spec frontmatter" \
  -f enforce_admins=false \
  -f required_pull_request_reviews.required_approving_review_count=1 \
  -f required_pull_request_reviews.dismiss_stale_reviews=true \
  -f required_conversation_resolution=true \
  -f required_linear_history=true \
  -f allow_force_pushes=false \
  -f allow_deletions=false
```

> 完整 API 参数：https://docs.github.com/en/rest/branches/branch-protection#update-branch-protection

---

## ✅ 应用后验证清单

- [ ] `git push` 直推 main 被拒绝（非 admin 用户）
- [ ] 新 PR 未 approve 时 `merge` 按钮 disabled
- [ ] 任一 required status check 失败时 `merge` 按钮 disabled
- [ ] PR 在 approve 后 push 新 commit → approval 自动 dismiss（测 Codex PR #6 F1 修复效果）
- [ ] gitleaks 扫到 secret 时 PR 阻塞 · merge 不了（测 SPIKE-06 A.5.3 F4 修复效果）
- [ ] task spec frontmatter 错误时 PR 阻塞（测 README §原则 7 修复效果）

---

## 📝 与其他治理文档的关系

- `CLAUDE.md §禁区`：本文档落地"禁止 push main"规则
- `docs/tasks/README.md §原则 7`：本文档落地 state transition enforcement（原 advisory → Phase 4 enforced）
- `docs/tasks/SPIKE-06 §A.5.3`：本文档引用 `.github/workflows/secret-scan.yml` 落地 gitleaks 硬阻塞
- `.github/PULL_REQUEST_TEMPLATE.md`：本文档配套的 PR schema · 确保 PR body 含关键字段
- `.github/workflows/ci.yml` / `secret-scan.yml` / `task-spec-validator.yml`：本文档引用的 required status checks 来源

---

## 🔄 变更流程

修改 branch protection 属于 **治理决策**，必须：
1. 开 issue 讨论变更理由
2. 更新本文件 + 对应 workflow 文件
3. 独立评审通过
4. 同时应用到 GitHub web UI 和本文档（保持一致）

---

**本文件 Phase 4 建立（2026-04-18）· admin 首次应用日期：<待填>**
