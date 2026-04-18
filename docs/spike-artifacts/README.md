# Spike Artifacts · per-task 录屏 / 截图 / 数据文件

> 本目录存放**每个 Spike task 的二进制 artifact**：录屏 / 截图 / 火焰图 / benchmark 数据 / 脱敏后的 CLI 样本。
> Phase 3 建立（2026-04-18）· Spike W0 Day 1 启动后填充。

---

## 📂 目录结构

每个 Spike 一个子目录：

```
docs/spike-artifacts/
├── README.md                   (本文件)
├── SPIKE-01/
│   ├── mac-boot.mp4            录屏
│   ├── ubuntu-wayland-boot.png 截图
│   └── cold-start-times.csv    数据
├── SPIKE-02/
│   └── hard-pass-matrix/
│       ├── ime-chinese.mp4
│       └── plugin-smoke-test.log
├── SPIKE-03/
│   └── git2-vs-gix-benchmark.csv
├── SPIKE-04/
│   ├── redb-100m-write.csv
│   ├── rusqlite-100m-write.csv
│   └── crash-recovery-logs/
├── SPIKE-05/
│   ├── 4tab-yes-flame.svg      主线程阻塞火焰图
│   ├── channel-depth-over-time.csv
│   └── b4-onetab-slow-hol.mp4
├── SPIKE-06/
│   ├── claude-cli-*.txt        脱敏后样本
│   ├── codex-cli-*.txt
│   └── gitleaks-scan-output.png (零 hit 截图 · reviewer 硬要求)
└── SPIKE-07/                   (未来 · v1.0 前)
```

**per-task 原则**：`docs/tasks/README.md §原则 5 + 6` — 每个 task 一个子目录 · 不跨 task 共享 artifact。

---

## 📦 允许的文件类型

| 类型 | 用途 | 大小建议 |
|------|------|---------|
| `.mp4` / `.webm` / `.gif` | 录屏 | ≤ 10 MB（超过走 Git LFS · Phase 5 启用） |
| `.png` / `.jpg` | 截图 | ≤ 2 MB |
| `.svg` | 火焰图 / 架构图 | ≤ 500 KB |
| `.csv` / `.json` | benchmark 数据 | ≤ 5 MB |
| `.log` / `.txt` | 纯文本日志 | ≤ 1 MB |
| **脱敏后的** CLI 输出（SPIKE-06）| `.txt` 或 `.json` | ≤ 500 KB |

---

## 🚫 禁区

- ❌ **raw 未脱敏 CLI 输出**（含 auth token / PII）· 必须放 `~/.vibestation-spike-raw/<SPIKE-NN>/`（home 路径 · repo 外）
- ❌ **生产服务器日志 / 用户真实数据**
- ❌ **含 API key / private key / session cookie** 的任何文件
- ❌ **非本项目数据**（如别的 repo 的 dump）

### Secret leakage 防护状态（**Codex PR #12 F3 复核 · 精确描述实际落地**）

> ⚠️ **不要把计划当成已落地**。以下防护分两层 · 每层的真实状态明确标注：

**第 1 层 · `.gitignore` `.raw` 拦截**（PR #9 落地 · 合入 main 后生效）
- 来源：PR #9 commit `2b86def` 加入 `*.raw` / `spike-raw/` / `.spike-raw/` 到 `.gitignore`
- 状态：**depends on PR #9 merge** · 在 PR #9 merge 到 main 之前 · 本 repo `.gitignore` 不含这些规则 · 放错位置的 `.raw` 文件**可能**被 commit
- 操作：SPIKE-06 实施 agent 必须在 PR #9 merge 之后才能开始 §A.5 样本录制 · 或自行 verify 当前 main 的 `.gitignore` 已含相关规则

**第 2 层 · `gitleaks` CI 扫描**（PR #11 Phase 4 基础设施落地）
- 来源：PR #11 `.github/workflows/secret-scan.yml` · 对所有 PR 跑 gitleaks
- 状态：**depends on PR #11 merge** · 未 merge 前 CI 不跑 gitleaks · commit 后 secret 不会被自动检测
- 操作：在 PR #11 merge 前 · SPIKE-06 实施 agent **必须本地跑** `gitleaks detect` 作为唯一扫描（SPIKE-06 A.5.3 要求）

**当前合入状态参考**（以 main 实际内容为准）：
```bash
# 检查 .gitignore 是否已含 raw 拦截
grep -E '^\*\.raw$|^spike-raw/$|^\.spike-raw/$' .gitignore || echo "❌ PR #9 未 merge · raw 拦截未生效"
# 检查 gitleaks CI 是否已落地
test -f .github/workflows/secret-scan.yml || echo "❌ PR #11 未 merge · CI gitleaks 未生效"
```

任一检查失败 → 按上文操作（延后 SPIKE-06 §A.5 实施 · 或使用本地 gitleaks 替代）。

---

## 📝 Artifact 命名约定

- **可读性优先**：`4tab-yes-flame.svg` 好于 `output.svg`
- **场景前缀**：`mac-`/`ubuntu-wayland-`/`ubuntu-x11-` 标平台
- **日期后缀**（可选）：如果 Spike 多次跑 · `redb-100m-write-2026-04-25.csv`
- **脱敏标记**（SPIKE-06 强制）：文件名含 `-redacted` 或放在 `redacted/` 子目录

---

## 🔗 相关

- `docs/spikes/` · per-task markdown 报告
- `docs/tasks/SPIKE-NN-*.md` · Spike 任务 spec
- `.github/workflows/secret-scan.yml` · gitleaks 扫 `docs/spike-artifacts/`
- `.gitignore` · `*.raw` / `spike-raw/` / `.spike-raw/` 拦截（防未脱敏文件误 commit）

---

**本目录 Phase 3 建立（2026-04-18）· 具体 artifact 在 Spike W0 启动后填充。**
