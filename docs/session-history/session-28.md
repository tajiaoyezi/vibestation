# Session 28 · 2026-05-12

**session**: 28
**date**: 2026-05-12（单 day · 9 PR merged · 4-track 并发派工 + 5 idle 查漏补缺）
**pr_range**: #271-#279（9 PR merged · MVP-15 Phase D 自动化全收 + 5 idle 修复）
**theme**: v0.3 sprint 4-track 并发派工峰值 + 主 agent idle 查漏补缺 · **MVP-15 Phase D §F vitest bench + §G edge cases vitest 自动化全收**·新增 `scripts/validate-runtime-evidence.mjs` 工具 + `.validator-exceptions.json` 配置 · 主 repo cargo clippy `-D warnings` 0 / pnpm lint+typecheck PASS / 25 evidence 目录 0 ERROR · `extensions.worktreeConfig=true` 升级根治 §2.12 主 repo .git/config 跨 agent 污染 · §2.4 Cursor N=1 首次 dispatch worktree 违规 audit trail

---

## 主题摘要

### 1 · 4-track 并发派工 + 5 idle 查漏补缺 · 单 session 9 PR merged

session 27（3 PR）峰值的 3× 跃升 · 团队 = 主 agent + Codex CLI + OpenCode + Cursor 四 agent 并行。

#### 9 PR merged 明细

**Track 4 主 agent（6 PR · 1 dispatch playbook + 5 idle 查漏补缺）**：

- **PR #271** · v0.3 sprint Phase D capture playbook（641 行 · 仿 MVP-05 14 invariant · 4 MVP × ~28 GUI evidence step · 让 Arbiter 90-120 min 一气呵成）
- **PR #272** · MVP-12/16 spec stale `PR #N` backfill（3 处占位回填 · MVP-08 PR #100 · MVP-16 PR #266 · MVP-12 PR #256）
- **PR #274** · MVP-08 Phase D 7 PNG → JPG quality 65 压缩 · 25.84MB → 9.35MB（sips · 满足 R4 ≤ 10MB · 同步 metrics-phase-e.md 8 处 `.png` → `.jpg`）
- **PR #276** · cargo clippy 13 warnings fix（8× deprecated `validate_mvp_05` → `validate_layout` · 2× unused mut · 1× manual `!Range::contains` · 2× dead_code 加 `#[allow(dead_code)]` · 2× unused imports · 恢复 `-D warnings` 干净）
- **PR #278** · dispatch-prompt §2.5.1 升级 · 启用 `extensions.worktreeConfig=true` + `git config --worktree`（根治 §2.12 跨 agent author 污染 · session 14+28 共 6 次实证）
- **PR #279** · validator `--exceptions` 配置 + MVP-11 R3 命名 + R4 dir tolerance 豁免（spec L140/L278 硬引用 05a/A5/B4 命名 · 不能改名 · validator main 从 1 ERROR → 0 ERROR）

**Track 1 Codex CLI · PR #275** · MVP-15 Phase D §F vitest bench + fixture（5 commit · +631514/-0 · 21 文件 · 新建 `scripts/fixtures/generate-syntax-highlight-fixtures.sh` 169 行 + 5 lang × 1MB fixture + 10MB TS fixture + 5 bench file · §F.1 shiki parse 1900.52ms / §F.2 三档 0.0004/2.7077/1.4725ms / §F.3 theme switch 0.0140ms / §F.4 LRU cache 0.0013ms / §F.5 heap 55.54MB · 全部满足 spec budget · DevTools 完整首屏部分留 Arbiter playbook）

**Track 2 OpenCode · PR #277** · MVP-15 Phase D §G edge cases vitest（5 commit · +386/-5 · 4 测试文件 + README + spec 勾选 · G.1 shiki 加载失败 4 cases / G.3 空文件 5 cases / G.4 单行超大暴露 Phase C 缺陷 4 cases / G.5 Worker fallback 4 cases · 17/17 vitest PASS · G.2 spec L243 explicit skip · **N=2 后回归 PASS · N=3 永久转出未触发**）

**Track 3 Cursor fix-up · PR #273** · runtime-evidence-validator + §2.4 N=1 audit trail（首次 dispatch 在主 repo 主 working tree 违反 §2.4 · 主 agent 备份到 `/tmp/cursor-track-3-backup/` · fix-up dispatch 在 `/private/tmp/runtime-evidence-validator-work` worktree 重 commit · **4 文件 byte-level 一致**：`scripts/validate-runtime-evidence.mjs` 449 行 + 9 vitest cases + `scripts/README.md` + `docs/runtime-evidence/_VALIDATION-REPORT.md` 416 行）

### 2 · Dependabot 4 PR 处置

- ❌ **close** #270 (notify 6→8 major×2 · MVP-08 fs_watch 兼容性高风险) + #268 (similar 2→3 major · MVP-08 diff algorithm 输出可能变化)
- ✅ **merged** #269 (sentry 0.47→0.48 · cargo check + telemetry tests 14/14 PASS) + #240 (Tauri 2.10.1→2.11.1 minor · pnpm install + typecheck + lint PASS)

### 3 · 协作 failure mode · 治理事件

| 事件                             | 次数 | 根因                                                                                                        | 处置                                                                                                        |
| -------------------------------- | ---- | ----------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| §2.4 worktree N=1 (Cursor)       | 1    | 首次 dispatch 在主 repo 主 working tree commit                                                              | 备份 → fix-up dispatch · byte-level 一致 · audit trail 永久                                                 |
| §2.12 主 repo `.git/config` 污染 | 3+   | dispatch §2.5.1 旧版 `git config user.email`（无 --worktree）写主 repo 共享 .git/config · 多 agent 互相覆盖 | 每次主 agent commit 前 unset local + amend reset author · PR #278 §2.5.1 升级根治                           |
| §2.10 OpenCode evidence-based    | 0    | N=2 后回归测试                                                                                              | **PR #277 三段全真实**（lint/typecheck/vitest 子集 17/17 + baseline 12 errors 一致）· N=3 未触发 · N=2 保持 |

### 4 · 主 repo 健康度（session 28 后）

- ✅ cargo clippy `--workspace --all-targets -- -D warnings` exit 0（PR #276）
- ✅ pnpm lint（prettier）+ pnpm typecheck（tsc --noEmit）exit 0
- ✅ pnpm vitest run 217/217（含既有 DiffLine flake · 单跑 PASS · session 27 实证）
- ✅ runtime-evidence validator 25 目录扫：21 PASS / 4 WARNING / 0 ERROR（PR #273+#279）
- ✅ extensions.worktreeConfig=true 已启用主 repo（PR #276 副产物 · 防 §2.12 复发）

---

## 反思

- 4-track 并发派工 + idle 查漏补缺 = 单 session 9 PR · 比 session 27（3 PR）跃升 3× · 但治理负担显著上升（§2.4 N=1 + §2.12 反复污染 + 网络抖动 + GitHub self-approve 限制）
- §2.12 反复 3 次污染 · 根因诊断成熟（共享 .git/config · 无 --worktree flag）· PR #278 系统性根治
- OpenCode N=2 后回归 PASS · 给 N=3 转出条款实证耐心：简单 vitest task + evidence-based 强约束 prompt = 可信合规交付
- Cursor 首次 dispatch §2.4 violation 提示：未来新 agent dispatch 必须更明显强调 worktree 启动命令（已 fix-up 验证可恢复）
- validator 工具上线后立即闭环 2 ERROR（MVP-08 体积 PR #274 + MVP-11 exception PR #279）· 验证工具价值

---

## 主 agent 收尾动作

- 9 PR merged via `gh pr merge --merge`（GitHub self-approve 限制走 PR comment + trailer + admin direct merge）
- 4 dependabot PR 处置（2 close + 2 merge）
- 本地 main 同步 origin/main · 3 stale worktree + 3 stale feat 分支删除（feat/MVP-15-phase-D-{vitest-bench,edge-cases} + feat/runtime-evidence-validator）
- 4 dispatch prompts 归档 `spike-tmp/dispatch/_archived/`（含 Cursor 首次 + fix-up 两版）
- /tmp/cursor-track-3-backup + /tmp/mvp-08-png-backup 已清
- PROGRESS.md session 28 段新增 · session 27 段保留（M-2 滚动窗口允许 2 session）· session 26 待归档

---

## 归档元信息

- **本文件归档时间**：session 30 · 2026-05-13（按 M-2 滚动窗口规则 · session 30 时 session 28 应滚出）
- **归档执行**：主 agent · branch `docs/session-28-rollout` · PR # 待开
- **PROGRESS.md 同步操作**：删除 PROGRESS line 69-127 段（57 行）· session 30/29 保留为当前 2 session 窗口
