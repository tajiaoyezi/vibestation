# Spike Reports · per-task 报告目录

> 本目录存放**每个 Spike task 的 benchmark 报告 / 结论**。
> Phase 3 建立（2026-04-18）· 具体 spike 报告在 Spike W0 Day 1 启动后逐个填充。

---

## 📂 目录结构

每个 Spike 对应**一份** markdown 文件：

```
docs/spikes/
├── README.md                   (本文件)
├── SPIKE-01-report.md          (SPIKE-01 Tauri 三平台空壳启动)
├── SPIKE-02-report.md          (SPIKE-02 Tauri 硬通过矩阵)
├── SPIKE-03-report.md          (SPIKE-03 git2/gix 读 benchmark)
├── SPIKE-04-report.md          (SPIKE-04 redb/rusqlite benchmark + git2 写)
├── SPIKE-05-report.md          (SPIKE-05 portable-pty 多 Tab 压测)
├── SPIKE-06-report.md          (SPIKE-06 CLI 实机 + Apple Dev Program)
└── SPIKE-07-report.md          (未来：CLI parser spike · v1.0 前)
```

**per-task 原则**：`docs/tasks/README.md §原则 5` — 每个 task 写自己的报告文件 · **不用共享 `SPIKE-REPORT.md`**（物理隔离比声明式并发治理可靠 · PR #4 close 反思）。

---

## 📝 每份报告的推荐结构

每份 `SPIKE-NN-report.md` 必含以下章节：

```markdown
# SPIKE-NN Report · <中文标题>

**Spike task**：[SPIKE-NN](../tasks/SPIKE-NN-<slug>.md)
**执行日期**：YYYY-MM-DD
**执行者**：<agent-id>
**结论**：通过 / 失败 / 部分通过 · 触发 fallback ? · 对应 `CLAUDE.md` 决策表 # 切换

## 测试环境

- 硬件：mac M1 / Ubuntu 24 Wayland / Ubuntu 24 X11
- 软件版本：Tauri 2.x · portable-pty 0.8.x · etc.
- 数据集：linux kernel git clone / 1000 万行 benchmark 数据等

## 结果数据

- 按 Spike Pass Criteria 逐项列表格 · 贴具体数字
- 录屏 / 截图放 `docs/spike-artifacts/SPIKE-NN/`

## 结论

- 通过：锁定 `<默认方案>`，更新 ADR-NNN 状态 `proposed → accepted`
- 失败：触发 fallback，更新 `CLAUDE.md` 决策表 + ADR-NNN

## 后续动作

- `CLAUDE.md` 决策表 #N 状态翻转 PR 链接
- 对应 ADR 状态翻转 PR 链接
- 受影响的 task spec（下游）
```

---

## 🔗 相关

- `docs/tasks/` · Spike task spec 源头
- `docs/adr/` · Spike 结论翻转 ADR 状态
- `docs/spike-artifacts/` · 录屏 / 截图 / 火焰图（per-task 目录）
- `CLAUDE.md §决策状态表`：Spike 通过后 B 栏 → A 栏

---

## ⚠️ 安全约束（SPIKE-06 特别注意）

SPIKE-06 报告含 CLI 输出样本 · **必须脱敏后写入**（`docs/tasks/SPIKE-06-cli-protocol-and-codesign.md §A.5`）：
- 删所有 auth token / API key / JWT / session cookie / PII
- 原始未脱敏样本保留本地 `~/.vibestation-spike-raw/SPIKE-06/`（**不进 repo** · 见 `.gitignore`）
- commit 前必过 `gitleaks detect`（Phase 4 CI 已硬阻塞 · `.github/workflows/secret-scan.yml`）

---

**本目录 Phase 3 建立（2026-04-18）· 具体报告在 Spike W0 启动后填充。**
