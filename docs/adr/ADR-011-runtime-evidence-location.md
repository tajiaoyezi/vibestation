# ADR-011: MVP / Feature runtime 证据存储位置标准化

**状态**：accepted
**日期**：2026-04-19（proposed） · 2026-04-19（accepted · session 10 末）
**决策者**：Claude Code (Opus 4.7 · session 10 末 · 提议) · User (Arbiter · 拍板选项 A · "按你的推荐来")
**对应 `CLAUDE.md` 决策表**：A 栏 #18（本 PR 新增）
**对应 Spike / MVP**：[MVP-02](../tasks/MVP-02-workspace-management.md)（触发本 ADR 的事件源）

---

## 背景与问题（Context and Problem Statement）

`.claude/rules/15-runtime-verification-gate.md` 和 dispatch-prompt-template §2.3 都要求 MVP / GUI / IPC 类 PR 必须交付 **runtime 证据**（截图 / 录屏 / 交互验证）· 证明"build 过 ≠ runtime 过"。但当前对**runtime 证据的存储位置**没有统一标准 · 在 MVP-02 PR #40 已经出现分歧：

- **Dispatch prompt 指定**：`spike-tmp/img/<task-id>/`（gitignored · 本机冷备 · PR comment 贴图）
- **OpenCode 实际做法**：自创 `docs/runtime-evidence/mvp-02/`（进 git · 1.3 MB × 3 jpg）· 未经授权
- **Arbiter 事后处理**（session 10）：Option C · 接受交付 · FU-2 follow up 决定最终路径

此分歧若不标准化 · 未来每个 MVP / feature PR 都可能出现"作者随机选一个路径"· 证据散落 · 未来 agent 难溯源。

**不决策的后果**：
- 每个 MVP 作者 / dispatch 接收方 各自选路径 · 碎片化
- runtime 证据可能分布在 `docs/runtime-evidence/` · `spike-tmp/img/` · PR comment · 某个未知目录
- 未来 agent clone repo 无法系统性复现 MVP 验收过程

## 决策驱动因素（Decision Drivers）

- **D1 · 可追溯性**：clone repo 能看完整证据链 · 不依赖 GitHub UI / 本机冷备
- **D2 · 仓库体积**：累积 20 个 MVP × 1-3 MB 证据 = ~40 MB · 可接受 · 但需评估 Git LFS 阈值
- **D3 · 形式合规**：对齐 rule 13（Decision-grade 代码即证据 · 必进 git）· Spike 已遵循（docs/spikes/code/ + raw/）
- **D4 · 作者负担**：作者本地产出 + commit · vs 上传 PR CDN 的工作量差别
- **D5 · PR 关闭后可访问性**：PR merge 后 closed · 评论里的图还能看 · 但 UI 查找困难
- **D6 · 和现有 dispatch prompt 一致**：目前 2.3 写的是 `spike-tmp/img/` · 需评估修改成本

## 考虑的选项（Considered Options）

### 选项 A · `docs/runtime-evidence/<task-id>/`（进 git）

**做法**：所有 MVP / feature 的 runtime 证据存到 `docs/runtime-evidence/<lowercase-task-id>/` · binary 直接 commit · 文件名语义化（`01-welcome-page.jpg` · `02-multi-workspace.jpg` 等）。

**样例**：MVP-02 的 `docs/runtime-evidence/mvp-02/` 即此形式（OpenCode 已实施）。

### 选项 B · `spike-tmp/img/<task-id>/`（gitignored · 本机）

**做法**：延续 dispatch prompt 2.3 现有指定 · 证据只在作者本机 · PR comment 贴图（走 GitHub user-attachments CDN）。

**样例**：OpenCode 早期 `spike-tmp/img/img.png` · `img_1.png`（session 9 OpenCode 自动化截图早期试水的残留）。

### 选项 C · 仅 PR comment（不进 repo · 靠 GitHub user-attachments CDN）

**做法**：证据完全不入本地目录 · 只拖到 PR comment 输入框 · GitHub 自动上传到 user-attachments CDN（格式 `https://github.com/user-attachments/assets/<uuid>`）。PR merge 后仍可访问 · 但需点进 PR 历史。

## 决策（Decision Outcome）

**最终选择**：**选项 A · `docs/runtime-evidence/<task-id>/`**（Arbiter 已拍板 · 2026-04-19 session 10 末）

**理由**：

1. **对齐 rule 13 精神**：SPIKE 的 code/ + raw/ 都在 git · runtime 证据逻辑等价 · 不应特殊处理
2. **可追溯性最强**：clone repo 一键见证据 · 未来 agent 不依赖 GitHub UI / 本机冷备
3. **体积可控**：20 个 MVP × 3 张图 × ~500 KB = ~30 MB · 远低于 Cargo.lock 2 × 143 KB 的同类问题 · 未到 Git LFS 阈值（GitHub 建议 > 50 MB 考虑 LFS · > 100 MB 强制 LFS）
4. **PR 关闭后无损**：(C) 方案 PR 关闭 git UI 搜图困难 · (A) 在 repo 内永久稳定
5. **OpenCode 已有实践**：MVP-02 已是此形式 · 选 (A) 零迁移成本

**与选项 B / C 的关键差别**：

| 维度 | A 进 git | B gitignored | C PR comment |
|---|---|---|---|
| clone 可见 | ✅ | ❌（本机） | ❌（需 GitHub UI） |
| PR 关闭后查找 | ✅（文件路径稳定） | ❌（本机丢 = 失证据） | ⚠（可访问但 UI 差） |
| repo 体积 | +1-3 MB/MVP | 0 | 0 |
| 未来 agent 重现 | ✅（零依赖） | ❌（依赖本机） | ⚠（依赖 GitHub 服务） |
| 和 Spike 归档一致 | ✅ | ❌ | ❌ |

## 后果（Consequences）

### 正面

- 所有 MVP / feature PR runtime 证据位置统一 · 未来 agent 可程序化扫描
- 和 Spike 归档一致 · 减少认知负担
- PR merge 后证据在 main 永久保留 · 不依赖任何外部服务

### 负面

- Repo 体积增长 1-3 MB/MVP（约 30-60 MB 到 v1.0）
- 历史文件层级变深（`docs/runtime-evidence/mvp-02/01-welcome-page.jpg` 这种路径）
- OpenCode 早期 `spike-tmp/img/` 残留（2 张小图 52 KB）需要清理 · 或以"FU-2 决议前的历史"形式保留

### 风险

- **R1** · 如果某 MVP 需要大量视频证据（录屏 > 10 MB）· 单 PR 就推动 repo 体积 · 需引入 Git LFS · 但概率低（MVP 一般 3 截图足够）
- **R2** · binary 进 git 导致 `git log --all` 变慢（文件对 git object db 开销）· 阈值大约 100 MB · 当前方案到 v1.0 仍远低于阈值
- **Fallback**：若 repo 体积确实爆（v2.0 后）· 引入 Git LFS 或迁移到 `C` 方案 · 迁移成本评估 YAGNI 当前阶段

## 实施项（Implementation Checklist · 已落地）

ADR accepted 后 · 已同步更新以下（本 PR 完成 ✅）：

- [x] **dispatch-prompt-template §2.3**：把 `spike-tmp/img/<task-id>/` 改为 `docs/runtime-evidence/<task-id>/` · 加 ADR-011 引用
- [x] **`.claude/rules/runtime-evidence-location.md`** · 新建项目级规则文件 · 引用本 ADR + 全局 `~/.claude/rules/15-runtime-verification-gate.md`（不直接修改全局 rule · 留待用户自决是否同步）
- [x] **`CLAUDE.md` 决策表 A 栏**：新增 #18 row · 锁定 runtime evidence 路径 = `docs/runtime-evidence/<task-id>/`
- [x] **清理 `spike-tmp/img/`** · OpenCode 早期 52 KB 残留 · 已 `rm -rf`（gitignored 目录 · 不在本 PR diff · 但 PR body 注明）
- [x] **MVP-02 `docs/runtime-evidence/mvp-02/`**：保留不动 · 作为本 ADR 的 reference 样例
- [x] **MVP-03+ dispatch prompt**：按新路径 `docs/runtime-evidence/<task-id>/` 分发（dispatch prompt template 已更新 · 后续 dispatch 自动遵循）

## 与 `implementation-plan.md` 的映射

- 对应章节：§5.1（v0.1 MVP 开发流程）· §R27 修复侧（MVP 交付验收流程补强）
- 对应风险：R27（MVP accept 流程不统一）

## 相关（Links）

- `CLAUDE.md` 决策表：待补（accepted 后加入 A 栏）
- 触发事件：[PR #40 `15649bc`](https://github.com/tajiaoyezi/vibestation/pull/40)（MVP-02 · OpenCode 自创 `docs/runtime-evidence/`）
- 关联规则：
  - [`.claude/rules/15-runtime-verification-gate.md`](../../.claude/rules/15-runtime-verification-gate.md)
  - [`.claude/rules/dispatch-prompt-template.md` §2.3](../../.claude/rules/dispatch-prompt-template.md)
  - [`~/.claude/rules/13-cross-agent-delivery.md`](~/.claude/rules/13-cross-agent-delivery.md)
- Session 10 协作记录：FU-2 归档（session save file 标注）

---

## Reviewer gate 历史（已闭合）

- 主 agent（Claude Code Opus 4.7）是**提议者** · 不能自行 accept
- ✅ Arbiter (User) 在 dialogue 明确批准选项 A · 时间 2026-04-19 session 10 末
- ✅ Accept 后已同步：本 ADR 状态 accepted · CLAUDE.md A 栏 #18 新 row · 实施项 6 步 · 翻转 PR 走 (a) 路径（本 PR）

---

**修订历史**：
- 2026-04-19 · 初版 · Claude Code (Opus 4.7 · session 10 末 · FU-2 draft · 提议选项 A) · PR #44 merged commit `025371d`
- 2026-04-19 · accepted · User 拍板选项 A（"按你的推荐来"）· 翻转 PR · 同步实施 6 步
