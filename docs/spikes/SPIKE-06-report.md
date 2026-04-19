# SPIKE-06 · Phase A Report（PR 1 起手 · 2026-04-19）

> **Task spec**：[`docs/tasks/SPIKE-06-cli-protocol-and-codesign.md`](../tasks/SPIKE-06-cli-protocol-and-codesign.md) · status: ready（PR #36 merged）
> **结论**：**起手 · 无决策级结论**（harness 建立 + pipeline smoke 通过 · 真 36 样本留 PR 2）
> **实施者**：Claude Code (Sonnet 4.6 · 主 agent · 本机 macOS)
> **Review**：Arbiter PR approve
> **phase-4-infra-landing 依赖已满足**：PR #11 merged @ 2026-04-18（`.github/workflows/secret-scan.yml` + `docs/BRANCH-PROTECTION.md`）
> **前置 Spike**：SPIKE-05 done（PR #30 · shared-reader HOL PASS · visible throughput FAIL → SPIKE-05.5 进行中）

---

## 测试环境

| 项 | 值 |
|---|---|
| OS | macOS Darwin 25.3.0（Apple Silicon 假设 · 用户机器） |
| Claude CLI | `/Users/USER/.local/bin/claude` · **v2.1.114 (Claude Code)** |
| Codex CLI | `/Users/USER/.nvm/versions/node/v24.14.1/bin/codex` · **codex-cli 0.121.0** |
| script(1) | BSD 自带 · `/usr/bin/script` |
| asciinema | 未装（可选 · PR 2 可升级） |
| gitleaks | 未装（**PR 2 前必装** · brew install gitleaks） |

---

## §A.1 · Pre-flight Check

按 spec §A.5.3 的 conditional 要求（Phase 4 是否 landed）· 本次执行路径走 **情况 A**（gitleaks CI 硬阻塞已上线）：

| 依赖 | 状态 | 备注 |
|---|---|---|
| `.github/workflows/secret-scan.yml` | ✅ 存在 | PR #11 落地 |
| `docs/BRANCH-PROTECTION.md` | ✅ 存在 | PR #11 落地 |
| gitleaks required-check on main | ⚠️ advisory mode | private repo · GitHub Pro 才开放 branch protection API · 用户已显式 accepted tech debt |
| Claude CLI 可用 | ✅ v2.1.114 | |
| Codex CLI 可用 | ✅ 0.121.0 | |
| Raw 路径 `~/.vibestation-spike-raw/SPIKE-06/` | ✅ 建 | home · repo worktree 外 · 天然 gitignore 覆盖不到 |
| `.gitignore` `*.raw` / `spike-raw/` | ✅ 已配 | PR #26 落地 · spec §A.5.1 要求 |

**结论**：pre-flight 关键项全绿 · 可进入 §A.2 样本录制（PR 2）。

---

## §A.2 · 样本录制（PR 1 部分 · PR 2 完整）

### PR 1 范围（本 PR）

只录 2 条 **零敏感 smoke**· 验证 pipeline 通：

| 样本 | 命令 | 敏感度 | 状态 |
|---|---|---|---|
| `claude-version-{01,02,03}` | `claude --version` | 0（只含版本号） | ✅ 录制 + 脱敏 + 归档 |
| `codex-version-{01,02,03}` | `codex --version` | 0（只含版本号） | ✅ 录制 + 脱敏 + 归档 |

归档位置：
- Raw: `~/.vibestation-spike-raw/SPIKE-06/<name>-<N>.raw`（本机 · 不进 git）
- 脱敏: `docs/spikes/raw/SPIKE-06/<name>-<N>.txt`（进 git · 零敏感所以 redaction 命中 = 0）

### PR 2 范围（下 session）

按 spec §A.2 完整矩阵 · 共 36 条样本（2 CLI × 6 场景 × 3 次）：

| CLI | 场景 | 覆盖风险 |
|---|---|---|
| Claude / Codex | happy path（启动 + 简单对话 + 正常 exit） | 基线 |
| Claude / Codex | 中断后残帧（对话进行中 Ctrl+C） | PTY 半帧处理 |
| Claude / Codex | 认证失败（错误 token） | auth fail 输出协议 |
| Claude / Codex | 网络错误（断网 / pfctl 拦截） | 重试 / 超时输出 |
| Claude / Codex | 长流式输出（10k+ token） | 流式完整片段 |
| Claude / Codex | 混合 ANSI + JSON 结构化 | 真实解析场景 |

PR 2 流程：
1. `brew install gitleaks asciinema`
2. 每场景写 scenario 脚本（`harness/scenarios/{claude,codex}-<scenario>.sh`）
3. `./harness/record.sh <scenario>` × 12
4. `./harness/redact.py` 批量 × 36
5. `./harness/verify.sh docs/spikes/raw/SPIKE-06/` · **gitleaks zero-hit 硬要求**
6. 归档 + 补 report §A.2/A.3/A.5

---

## §A.3 · 结构观察报告（pending PR 2）

本 PR 不产出（只 smoke · 无真实 CLI 交互数据）。

PR 2 输出：
- Claude CLI 输出结构描述（stream / block / JSON-line / ANSI）
- Codex CLI 输出结构描述
- 关键差异点粗略描述（token 分隔、role 标识、error format）
- ⚠️ **禁止写"协议已消除 R1"**（spec §A.3 硬要求）

---

## §A.4 · 归档（本 PR 部分完成）

按 spec §A.4 + `.claude/rules/spike-delivery-checklist.md`（4 样齐全）：

| 物料 | 位置 | 本 PR 状态 | PR 2 状态 |
|---|---|---|---|
| 决策文档 report | `docs/spikes/SPIKE-06-report.md` | ✅ 起手版（本文件） | 🟡 补 §A.2/A.3/A.5 |
| 实测源码 code | `docs/spikes/code/SPIKE-06/` | ✅ harness 全套 | 🟡 补 scenario 脚本 × 10 |
| Raw 数据 | `docs/spikes/raw/SPIKE-06/` | ✅ 6 txt smoke（2 CLI × 3 次） | 🟡 36 txt + 36 redaction.json |
| 冷备 | `spike-tmp/archive/SPIKE-06/` | ✅ placeholder（gitignored） | 🟡 build 产物 / 失败捕获 |

**关键**：本 PR 的 harness 代码 + smoke 样本 + pipeline 已可用 · 任何 agent clone 本 repo + `brew install gitleaks` 后能复现 PR 2 的全流程。

---

## §A.5 · 脱敏 + gitleaks（本 PR smoke · PR 2 full）

### A.5.1 · Raw 隔离 ✅

- `~/.vibestation-spike-raw/SPIKE-06/` 在 home · 天然 repo 外
- `.gitignore` 的 `*.raw` / `spike-raw/` 在 PR #26 落地 · 双保险

### A.5.2 · 脱敏 Python 脚本 ✅

`docs/spikes/code/SPIKE-06/harness/redact.py` 实现 7 类 pattern（按风险排序）：

1. Anthropic API key（`sk-ant-...`）
2. OpenAI API key（`sk-...`）
3. JWT（3-part base64）
4. Bearer / Authorization header
5. GitHub token（`ghp_` / `gho_` / `ghs_` / `github_pat_`）
6. 本地路径（`/Users/<name>` / `/home/<name>`）
7. Git remote（https / SSH）
8. PII（email / 手机号 / 身份证号 18 位）

原则：**结构保留 · 值丢失**（按 spec §A.5.4）· 例：
- `sk-ant-abc123...` → `sk-ant-REDACTED_ANTHROPIC_KEY`（前缀保留 · 结构信息可识别）
- `eyJ<header>.<payload>.<sig>` → `eyJ...REDACTED_JWT_HEADER....REDACTED_JWT_PAYLOAD....REDACTED_JWT_SIG`（3-part 保留）

### A.5.3 · gitleaks 扫描

**本 PR smoke 情况**：脱敏后样本是 `--version` 输出（只含版本号）· regex 命中 = 0 · trivial zero-hit。

**PR 2 情况**：
- 真 36 样本含真实 CLI 输出 · 可能含 auth fail 回显 token 残片
- **硬要求**：`gitleaks detect --source docs/spikes/raw/SPIKE-06/` 必须 zero-hit 才 merge
- 若命中 → 调整 redact.py pattern + 重跑 · 不得关 workflow / disable rule type

### A.5.4 · 脱敏真实性与结构平衡 ✅

PR 2 每个 `.txt` 附 `.redaction.json`（见 `docs/spikes/raw/SPIKE-06/README.md` 模板）· 本 PR 不需要（smoke 零敏感）。

---

## §B · Apple Developer Program（副线 · 未启动）

独立于 §A · 需用户出钱 $99/年 · Apple 审核 2d-2w。

**本 PR 不触发**。PR 2 或单独 PR 处理 · 需要 Arbiter 支付 + 跑申请流程。

---

## §C · SPIKE-07 Parser 依赖（下游 · v1.0-pre）

SPIKE-07 spec 已存在：[`docs/tasks/SPIKE-07-cli-protocol-parser.md`](../tasks/SPIKE-07-cli-protocol-parser.md) · status: draft。

解锁条件（spec §C）：
- [x] SPIKE-07 spec 已新建
- [ ] 等 SPIKE-06 §A 完整交付（PR 2 · 36 样本 zero-hit gitleaks）
- [ ] SPIKE-07 按 36 样本建 parser 原型 → v1.0 前执行

---

## ❌ R1 风险陈述（spec 硬要求 · 不得下调）

**本 PR 不下调 R1**（R1：CLI 输出协议未实机验证 · HIGH/HIGH）：

- 只建 harness + 跑 smoke · 未录真实交互样本
- **PR 2 也不降 R1**（即使 36 样本完整 · 按 spec §C 只能到"样本已录 · 协议特征化待 SPIKE-07"）
- `CLAUDE.md` ⚠️ "Claude CLI / Codex CLI 输出协议 Spike Day 5 前未经实机验证" **保留原文不变**
- R1 降级授权**只能由 SPIKE-07 的 ADR-011 完成**

---

## Review checklist

按 `.claude/rules/spike-delivery-checklist.md`（4 样齐全）：

- [x] 决策文档 `docs/spikes/SPIKE-06-report.md` 入库（本文件 · 起手版 · 明确标 PR 2 待补）
- [x] 源码 `docs/spikes/code/SPIKE-06/` 入库（harness 全套 + Cargo.lock N/A 纯 shell/python · README 含复现命令）
- [x] Raw 归档 `docs/spikes/raw/SPIKE-06/` 入库（6 条 smoke txt · README 索引）
- [x] 冷备 `spike-tmp/archive/SPIKE-06/` 本地保留（PR 1 只占位 · PR 2 填真实 build 产物）
- [x] Report 每个数字可 raw 溯源（本 PR 只有版本号 · 直接溯源 CLI 输出）
- [x] Clone 本 repo + `brew install gitleaks` 后可复现 PR 2 全流程

按 `.claude/rules/dispatch-prompt-template.md` §2（硬约束）：
- [x] 2.1 不自行 accept decision-grade（本 PR 明示 R1 保留 · 无 ADR 翻转）
- [🟡] 2.2 Acceptance 部分覆盖（§A.1/A.4 done · §A.2/A.3/A.5 留 PR 2 · skip reason explicit）
- [x] 2.3 Runtime 证据（6 条 smoke txt 实际跑过）
- [N/A] 2.4 独立 worktree（主 agent 自己 · 豁免）
- [x] 2.5 Commit trailer Co-authored-by: Claude Code
- [x] 2.6 分支命名 `spike/SPIKE-06-phase-a-harness`
- [x] 2.7 不碰 decision files（本 PR 只改 docs/spikes/* + 新增 code/SPIKE-06/*）

---

## 下一步（PR 2）

1. `brew install gitleaks asciinema`
2. 按 spec §A.2 编写 10 scenario 脚本（已有 2 smoke · 新增 10 正式场景 × 2 CLI = 12 scenarios）
3. 运行 `./harness/record.sh <scenario>` × 12 = 36 raw
4. 批量 redact → 36 txt + 36 redaction.json
5. `./harness/verify.sh docs/spikes/raw/SPIKE-06/` · **gitleaks zero-hit**
6. 补 report §A.2/A.3/A.5 完整段
7. macOS `fix-path-env` 验证片段（spec §A.1 Tauri 子进程 PATH 问题）

预期：1-2d · 独立 PR。
