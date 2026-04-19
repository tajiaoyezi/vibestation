# SPIKE-06 §A · Phase A Harness · Code Archive

## 来源

- **交付 agent**：Claude Code (Sonnet 4.6 · 主 agent)
- **产出时间**：2026-04-19（session 9 末 · PR 1 起手）
- **Review**：主 agent self + Arbiter PR approve
- **Parent task**：[SPIKE-06](../../../tasks/SPIKE-06-cli-protocol-and-codesign.md) · status: ready（PR #36 merged）

## PR 1 vs PR 2 范围

本 PR 1（**起手 · 本 session**）：
- ✅ `harness/` · CLI 录制 + 脱敏 + gitleaks 验证 pipeline
- ✅ 2 条 smoke 样本（`claude --version` / `codex --version` · 零敏感）· 证明 pipeline 通
- ❌ 36 条完整样本 · 留 **PR 2**（下 session · 清爽上下文 + 完整脱敏审查）

PR 2（**下 session**）预期：
- 按 spec §A.2 录 36 条样本（2 CLI × 6 场景 × 3 次）
- happy / interrupt-residual / auth-fail / network-error / long-stream / mixed-ansi 全覆盖
- 全量 gitleaks 扫描 · zero-hit 通过才 merge
- 补 report §A.2/A.3/A.5 · `fix-path-env` macOS PATH 验证片段

## 原始归档位置

| 物料 | 位置 | 进 git |
|---|---|:---:|
| Harness 脚本 + README | `docs/spikes/code/SPIKE-06/` | ✅ |
| 脱敏后样本 | `docs/spikes/raw/SPIKE-06/*.txt` | ✅ |
| Report | `docs/spikes/SPIKE-06-report.md` | ✅ |
| Raw 原始捕获（机密） | `~/.vibestation-spike-raw/SPIKE-06/*.raw` | ❌（home · repo 外） |
| 冷备（build 产物 / 失败捕获） | `spike-tmp/archive/SPIKE-06/` | ❌（gitignored） |

## 复现命令

### 依赖（首次）

```bash
brew install gitleaks          # 必需 · PR 2 前
brew install asciinema         # 可选 · BSD script(1) 也可
mkdir -p ~/.vibestation-spike-raw/SPIKE-06
```

### 跑 smoke（本 PR 1）

```bash
cd docs/spikes/code/SPIKE-06

# 1. 录制（生成 ~/.vibestation-spike-raw/SPIKE-06/<scenario>-{01,02,03}.raw）
./harness/record.sh claude-version
./harness/record.sh codex-version

# 2. 脱敏（写入 docs/spikes/raw/SPIKE-06/<scenario>-<N>.txt）
for scenario in claude-version codex-version; do
  for n in 01 02 03; do
    ./harness/redact.py \
      --input ~/.vibestation-spike-raw/SPIKE-06/${scenario}-${n}.raw \
      --output ../../raw/SPIKE-06/${scenario}-${n}.txt
  done
done

# 3. 验证 (gitleaks 必须装)
./harness/verify.sh ../../raw/SPIKE-06/
```

### 跑 36 样本（PR 2 · 下 session）

```bash
# 每个 scenario 跑 3 次 · 2 CLI × 6 场景 × 3 = 36 runs
for cli in claude codex; do
  for scenario in happy interrupt auth-fail network-err long-stream mixed-ansi; do
    ./harness/record.sh ${cli}-${scenario}
  done
done

# 脱敏 + gitleaks + 归档 + report 补 §A.2/3/5
```

## 关键结论溯源

本 PR 不产生决策级结论（只是 harness build-out + pipeline smoke）。

- Report：[`../../SPIKE-06-report.md`](../../SPIKE-06-report.md) · §A.1 pre-flight done · §A.2-A.5 pending PR 2
- R1 降级**未触发**（本 PR 和 PR 2 都不降 R1 · 降级由 SPIKE-07 ADR 完成）

## 硬约束符合情况

按 [`.claude/rules/dispatch-prompt-template.md`](../../../../.claude/rules/dispatch-prompt-template.md) §2 · 主 agent 自己执行的任务同样适用：

- [x] 2.1 · 不自行 accept decision-grade（本 PR 无决策 · R1 保留）
- [🟡] 2.2 · Acceptance 部分覆盖（§A.1 done · §A.2-A.5 留 PR 2 · PR body 明说 skip reason）
- [x] 2.3 · Runtime 证据（2 条 smoke 实际跑过 · 归档到 raw/SPIKE-06/）
- [N/A] 2.4 · 独立 worktree（主 agent 自己 · 豁免）
- [x] 2.5 · Commit trailer Co-authored-by: Claude Code
- [x] 2.6 · 分支命名 `spike/SPIKE-06-phase-a-harness`
- [x] 2.7 · 不碰 decision files（本 PR 只改 docs/spikes/* + 新增 code/SPIKE-06/*）
