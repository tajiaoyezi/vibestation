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
├── SPIKE-04.5-report.md        (SPIKE-04.5 rusqlite 数据安全验证)
├── SPIKE-05-report.md          (SPIKE-05 portable-pty 多 Tab 压测)
├── SPIKE-05.5-report.md        (SPIKE-05.5 PTY visible throughput)
├── SPIKE-06-report.md          (SPIKE-06 CLI 实机 + Apple Dev Program)
└── SPIKE-08-report.md          (SPIKE-08 E2E + IPC contract harness)
```

**per-task 原则**：`docs/tasks/README.md §原则 5` — 每个 task 写自己的报告文件 · **不用共享 `SPIKE-REPORT.md`**（物理隔离比声明式并发治理可靠 · PR #4 close 反思）。

---

## 📋 10 SPIKE 状态索引表

| SPIKE ID   | 标题                            | Status                                   | 报告链接                                    | 关联 ADR                                                        | 关联 MVP                          | 4 样齐全状态                                  |
| ---------- | ------------------------------- | ---------------------------------------- | ------------------------------------------- | --------------------------------------------------------------- | --------------------------------- | --------------------------------------------- |
| SPIKE-01   | Tauri 三平台空壳启动            | **done**                                 | [SPIKE-01-report](./SPIKE-01-report.md)     | [ADR-006](../adr/ADR-006-desktop-framework.md)                  | MVP-01                            | report ✅ · code ✅ · raw ✅ · cold backup ❌ |
| SPIKE-02   | Tauri 硬通过矩阵                | **done**                                 | [SPIKE-02-report](./SPIKE-02-report.md)     | [ADR-006](../adr/ADR-006-desktop-framework.md)                  | MVP-01                            | report ✅ · code ✅ · raw ✅ · cold backup ❌ |
| SPIKE-03   | git2 vs gix 读 benchmark        | **done**                                 | [SPIKE-03-report](./SPIKE-03-report.md)     | [ADR-007](../adr/ADR-007-git-stack.md)                          | MVP-07                            | report ✅ · code ✅ · raw ✅ · cold backup ❌ |
| SPIKE-04   | redb vs rusqlite + git2 写      | **done**                                 | [SPIKE-04-report](./SPIKE-04-report.md)     | [ADR-005](../adr/ADR-005-local-storage.md)                      | MVP-05 / MVP-09                   | report ✅ · code ✅ · raw ✅ · cold backup ❌ |
| SPIKE-04.5 | rusqlite 数据安全验证           | **done**                                 | [SPIKE-04.5-report](./SPIKE-04.5-report.md) | [ADR-005](../adr/ADR-005-local-storage.md)                      | MVP-02 / MVP-06 / MVP-10 / MVP-19 | report ✅ · code ✅ · raw ✅ · cold backup ❌ |
| SPIKE-05   | portable-pty 多 Tab 压测        | **done**                                 | [SPIKE-05-report](./SPIKE-05-report.md)     | [ADR-003](../adr/ADR-003-pty-architecture.md)                   | MVP-04                            | report ✅ · code ✅ · raw ✅ · cold backup ✅ |
| SPIKE-05.5 | PTY visible throughput fallback | **done**                                 | [SPIKE-05.5-report](./SPIKE-05.5-report.md) | [ADR-003](../adr/ADR-003-pty-architecture.md)                   | MVP-04                            | report ✅ · code ✅ · raw ✅ · cold backup ❌ |
| SPIKE-06   | CLI 实机 + Apple Dev Program    | **blocked**                              | [SPIKE-06-report](./SPIKE-06-report.md)     | —                                                               | MVP-04 / MVP-10                   | report ✅ · code ✅ · raw ✅ · cold backup ✅ |
| SPIKE-07   | CLI 输出协议 parser 验证        | **draft**（session 31 详化完成 PR #311） | —（未启动）                                 | [ADR-011](../adr/ADR-009-ai-aware-v1-vision.md)（未来）         | MVP-18 / MVP-19 / MVP-20          | 未启动                                        |
| SPIKE-08   | E2E + IPC contract harness      | **done**                                 | [SPIKE-08-report](./SPIKE-08-report.md)     | [ADR-014](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md) | —                                 | report ✅ · code ✅ · raw ✅ · cold backup ❌ |

> **Status 来源**：`docs/tasks/SPIKE-*.md` frontmatter · 由 task spec 源头管理
> **4 样齐全**：从 `docs/spikes/code/` + `docs/spikes/raw/` + 报告存在性实读 · 冷备从 `spike-tmp/` ls 判定

---

## 🔀 SPIKE → ADR 触发流程

每个 Spike 完成后，**禁止自行 accept 决策**（`CLAUDE.md §决策表 A/B/C 栏` 翻转必须经过 Arbiter 拍板）。

**标准流程**：

1. **SPIKE-NN 实施完成** → 报告 `docs/spikes/SPIKE-NN-report.md` 产出
2. **报告结论 = PASS / PARTIAL / FAIL** → 按 `docs/tasks/SPIKE-NN-*.md §Fallback` 对应路径
3. **发起 ADR-NNN 状态翻转 PR**：`proposed → accepted`
4. **独立评审**（≠ Spike 实施者）review ADR 翻转 PR
5. **Arbiter 拍板** → ADR accepted → `CLAUDE.md §决策表` 对应项 B 栏 → A 栏

**历史实例**：

- SPIKE-05.5 PASS（visible throughput 瓶颈不在 shared-reader）→ ADR-003 proposed → accepted（session 10 PR #50）→ `CLAUDE.md` #15 B → A
- SPIKE-04 结论 (B) redb → rusqlite → SPIKE-04.5 补 B.1-5 on rusqlite 实测通过 → ADR-005 accepted（PR #50）→ `CLAUDE.md` #14 B → A

---

## 📦 4 样齐全归档规则

参照 `.claude/rules/spike-delivery-checklist.md` v2 + [ADR-013](../adr/ADR-013-spike-cold-backup-degradation.md)：

### 3 样必须（v2 标准）

1. **决策文档**：`docs/spikes/SPIKE-NN-report.md` · 含结论 + 数据 + fallback 路径
2. **实测源码**：`docs/spikes/code/SPIKE-NN/` · 可复现的 benchmark / POC 代码
3. **Raw 数据**：`docs/spikes/raw/SPIKE-NN/` · benchmark 原始输出 / profiling 数据

### 1 样推荐（v2 降级 · ADR-013 accepted）

4. **Cold backup**：`spike-tmp/archive/SPIKE-NN/` · 压缩归档 · 按 **3 场景判断**
   - 场景 1：Spike 有 > 100MB 随机测试数据
   - 场景 2：Spike 涉及外部二进制工具
   - 场景 3：Spike 非 Cargo 构建
   - 以上任一命中 → **必须做冷备**
   - 均不命中 → **推荐但不做不 block**

### v1 → v2 降级原因

- v1 期（SPIKE-01..04.5/05.5/08）冷备合规率 **22%**（2/9）
- 补齐 7 个欠账成本 4-7 小时 · 但收益为零（code + Cargo.lock 进 git · `cargo build` 可 byte-level 复现）
- v2 标准：3 样必须已 100% 合规 · 冷备按场景判断

---

## 🗂️ 当前 SPIKE 归档目录树

```
docs/spikes/
├── code/                           # 实测源码 · 按 SPIKE 分目录
│   ├── SPIKE-01/                   # Tauri 三平台空壳启动代码
│   ├── SPIKE-01-02-phase-B/        # Ubuntu Phase B 补充测试
│   ├── SPIKE-02/                   # Tauri 硬通过矩阵
│   ├── SPIKE-03/                   # git2 vs gix benchmark
│   ├── SPIKE-04/                   # redb vs rusqlite storage benchmark
│   ├── SPIKE-04.5/                 # rusqlite 数据安全验证
│   ├── SPIKE-05/                   # portable-pty 多 Tab 压测
│   ├── SPIKE-05.5/                 # PTY visible throughput fallback
│   ├── SPIKE-06/                   # CLI 实机 + Apple Dev Program
│   ├── SPIKE-08/                   # E2E + IPC contract harness
│   └── SPIKE-MVP-10-telemetry/     # telemetry 实验代码
├── raw/                            # 原始 benchmark 数据 / profiling
│   ├── SPIKE-01/                   # 冷启动时间 raw 日志
│   ├── SPIKE-01-02-phase-B/        # Ubuntu X11/Wayland raw 数据
│   ├── SPIKE-02/                   # hard-pass 矩阵原始结果
│   ├── SPIKE-03/                   # git2 vs gix P50/P99 raw 数据
│   ├── SPIKE-04/                   # redb/rusqlite 性能 raw 数据
│   ├── SPIKE-04.5/                 # 数据安全测试 raw 输出
│   ├── SPIKE-05/                   # PTY 压测 raw 日志
│   ├── SPIKE-05.5/                 # visible throughput raw 数据
│   ├── SPIKE-06/                   # CLI 输出样本（脱敏后）
│   └── SPIKE-08/                   # E2E contract raw 数据
├── scripts/                        # 复现脚本
│   ├── SPIKE-01/                   # Tauri 启动复现脚本
│   └── SPIKE-02/                   # hard-pass 复现脚本
├── SPIKE-01-report.md
├── SPIKE-02-report.md
├── SPIKE-03-report.md
├── SPIKE-04-report.md
├── SPIKE-04.5-report.md
├── SPIKE-05-report.md
├── SPIKE-05.5-report.md
├── SPIKE-06-report.md
└── SPIKE-08-report.md
```

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
- `.claude/rules/spike-delivery-checklist.md` · 4 样齐全规则 v2
- `ADR-013-spike-cold-backup-degradation.md` · v1 → v2 降级决策

---

## ⚠️ 安全约束（SPIKE-06 特别注意）

SPIKE-06 报告含 CLI 输出样本 · **必须脱敏后写入**（`docs/tasks/SPIKE-06-cli-protocol-and-codesign.md §A.5`）：

- 删所有 auth token / API key / JWT / session cookie / PII
- 原始未脱敏样本保留本地 `~/.vibestation-spike-raw/SPIKE-06/`（**不进 repo** · 见 `.gitignore`）
- commit 前必过 `gitleaks detect`（Phase 4 CI 已硬阻塞 · `.github/workflows/secret-scan.yml`）

---

**本目录 Phase 3 建立（2026-04-18）· 报告在 Spike W0 启动后逐个填充 · README 升级于 session 31。**
