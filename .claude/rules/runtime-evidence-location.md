# Runtime 证据存储位置 · 项目级规则（ADR-011 落地）

> 本规则是 [ADR-011 · runtime evidence location](../../docs/adr/ADR-011-runtime-evidence-location.md)（accepted · 2026-04-19 session 10 末）在 Vibestation 项目的具体落地。
> 上位依据：[`~/.claude/rules/15-runtime-verification-gate.md`](~/.claude/rules/15-runtime-verification-gate.md)（全局 · Runtime 验证 Gate）。

## 触发场景

凡 PR 需要交付 runtime 证据（截图 / 录屏 / 交互验证）· 都由本规则决定**存储位置**：

- MVP / feature PR · 含 GUI / IPC / PTY / 外部进程 · 见 rule 15 定义
- Spike 不走本规则 · 走 [`.claude/rules/spike-delivery-checklist.md`](./spike-delivery-checklist.md)（"4 样齐全" 含独立 raw/ 目录）

## 硬规则

### R1 · 位置

MVP / feature 的所有 runtime 证据 **必须** 存到：

```
docs/runtime-evidence/<lowercase-task-id>/
```

示例：
- ✅ `docs/runtime-evidence/mvp-02/01-welcome-page.jpg`
- ✅ `docs/runtime-evidence/mvp-03/02-tool-window-sidebar.jpg`
- ✅ `docs/runtime-evidence/mvp-03/03-session-switch.mp4`
- ❌ `spike-tmp/img/mvp-03/...`（gitignored · 禁用 · 早期 dispatch prompt 指定 · 已 deprecated）
- ❌ 仅 PR comment（不进 repo · 禁用）
- ❌ 任何其他位置（禁止随机选目录）

### R2 · 进 git

本目录 **必须** 入 `git` · 不得 gitignore。

理由：
- 对齐 rule 13（Decision-grade = 证据 · 必进 git）
- clone repo 零依赖见完整证据链 · 未来 agent 可程序化扫描
- PR 关闭后证据路径稳定 · 不依赖 GitHub UI / 本机冷备

### R3 · 命名

文件名语义化 · 顺序前缀：

- `01-<name>.jpg` · `02-<name>.jpg` · `03-<name>.mp4` 等
- `<name>` 用英文小写 + 连字符 · 描述画面内容（`welcome-page` · `multi-workspace` · `delete-confirm-modal` 等）
- 视频 / 录屏用 `.mp4` 或 `.webm` · 图片用 `.jpg` / `.png`（压缩版不超过 500 KB / 原始版无硬上限但注意 repo 体积）

### R4 · 体积控制

- 单 MVP / feature 目录**推荐** ≤ 3 MB · **上限** 10 MB
- 录屏类超过 10 MB 时 · 强制压缩（HandBrake / ffmpeg · 目标 ≤ 5 MB）
- 累积 repo 体积预估：20 MVP × 3 MB = 60 MB 到 v1.0 · 远低于 Git LFS 阈值（GitHub 建议 > 50 MB 考虑 LFS · > 100 MB 强制）
- 若 v2.0+ 单 PR 证据超 10 MB · 触发 ADR 升级 → 引入 Git LFS 或迁移方案（见 ADR-011 §风险 R1）

### R5 · PR body 必引

PR body Test Plan 必须包含一行：

```markdown
- [x] Runtime 证据已提交到 `docs/runtime-evidence/<task-id>/` · 含 <N> 张截图 / 录屏
```

作者必须在 PR body 贴至少 1 张证据预览（GitHub 自动渲染图片 · 审查时可直接看）· 不能仅靠目录路径。

## 反模式

| 反模式 | 真正该做的 |
|---|---|
| PR comment 贴图 · 不进 repo | 图 commit 到 `docs/runtime-evidence/<task-id>/` · PR comment 可以引用该路径 |
| 证据放 `spike-tmp/img/` · 本机冷备 | 禁用 · 本规则 R1 明确 · 放 `docs/runtime-evidence/` |
| 证据放 `assets/` · `images/` · 或其他随手目录 | 禁用 · 统一路径只有 `docs/runtime-evidence/<task-id>/` |
| 图片 > 10 MB 不压缩 | 压缩（HandBrake / ffmpeg / imagemagick）· 下次 PR 必须满足 |
| PR body 不引用证据路径 | R5 明确要求 · 违反即退回 |

## 参考样例

MVP-02 PR #40（2026-04-19）· 是本规则 **accept 前**的实施 · 但恰好符合 R1-R4：

```
docs/runtime-evidence/mvp-02/
├── 01-welcome-page.jpg        (~450 KB · 欢迎页 create workspace CTA)
├── 02-multi-workspace.jpg     (~500 KB · sidebar 2 workspace + git badge)
└── 03-delete-confirm-modal.jpg (~400 KB · 删除二次确认 modal)
```

未来 MVP-03+ 按此结构 · 每 task 一个独立子目录。

## 关联

- [全局] `~/.claude/rules/13-cross-agent-delivery.md` · 交付物持久化（rule 13 的 runtime 证据专化）
- [全局] `~/.claude/rules/15-runtime-verification-gate.md` · Runtime 验证 Gate（上位 · 定义何时需 runtime 证据）
- [项目] `.claude/rules/dispatch-prompt-template.md` §2.3 · dispatch 时引用本规则
- [项目] `.claude/rules/spike-delivery-checklist.md` · Spike 走独立 "4 样齐全" 归档（不走本规则）
- [决策] `docs/adr/ADR-011-runtime-evidence-location.md` · 本规则的 ADR 源头
- [样例] `docs/runtime-evidence/mvp-02/` · 首个符合本规则的样例
