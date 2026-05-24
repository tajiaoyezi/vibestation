# Vibestation 项目整体进度 · Session 16 结束时快照

> **时间**：2026-04-22（session 16 收尾）
> **HEAD**：`d975e10`
> **项目阶段**：Pre-code 100% done + Spike 核心 done + MVP v0.1 代码实施 ~40%
> **性质**：Session 16 结束时间点的项目状态 snapshot（`docs/PROGRESS.md` 仍是 rolling source of truth · 本文档为归档快照）

---

## 🎯 一句话定位

**给 Claude CLI / Codex CLI 用户的多 Tab 终端 + JetBrains 级 Git 工作台**（Tauri 2 桌面应用 · Apache 2.0 · v0.1.0-alpha 目标 macOS-first · 私有仓库）

---

## 📊 完成度仪表盘

```
Pre-code Phase 1-4  ████████████████████  100% (文档 · ADR · CI · task spec 全就绪)
Spike (W0 decision) ████████████████░░░░   80% (6 done · 3 blocked · 1 draft)
v0.1 MVP (10 个)    ████████████░░░░░░░░   60% (3 done · 7 ready 含部分 phase done)
v0.1.0-alpha GA     ██░░░░░░░░░░░░░░░░░░   10% (仍需 Phase F 证据 + 3-4 个 MVP 实施 + 打包)
v0.2+ (MVP-11..20)  ░░░░░░░░░░░░░░░░░░░░    0% (spec draft · 未启动)
```

---

## 📈 代码 / 测试 / 协作量化

| 维度              |   数值 | 备注                                               |
| ----------------- | -----: | -------------------------------------------------- |
| **Merged PR**     |     92 | session 1-16 累计                                  |
| **Rust LOC**      |  5,743 | `crates/app/` + `crates/core/`                     |
| **Frontend LOC**  |  4,267 | SolidJS + TypeScript（不含 ts-rs bindings）        |
| **Test 总数**     |    128 | 单元 + 集成 + parser + PTY scrollback              |
| **ADR accepted**  |     14 | 全部 proposed → accepted 收敛                      |
| **Spike done**    | 6 / 10 | SPIKE-03 / 04 / 04.5 / 05 / 05.5 / 08              |
| **v0.1 MVP done** | 3 / 10 | MVP-02 / 03 / 07（MVP-04 A/B/C/E 部分 phase done） |

---

## 🗺️ v0.1 MVP 状态矩阵

| ID         | 标题                        | 状态        | 所有者      | 进度 / 备注                                                                   |
| ---------- | --------------------------- | ----------- | ----------- | ----------------------------------------------------------------------------- |
| **MVP-01** | Tauri app shell             | 🟡 ready    | Claude Code | Phase A/B done（macOS）· Phase C Ubuntu 待环境 · **最低优先**                 |
| **MVP-02** | Workspace 管理              | ✅ **done** | OpenCode    | PR #40/#44/#45/#47                                                            |
| **MVP-03** | Tool Windows 5-zone         | ✅ **done** | OpenCode    | PR #61 · 布局持久化 + 主题                                                    |
| **MVP-04** | 多 Tab 终端（PTY + xterm）  | 🟢 ready+   | Codex CLI   | Phase A/B/C/E done · **Phase D/F 待**（shell 兼容 Ubuntu + runtime 证据量化） |
| **MVP-05** | Pane 分屏                   | 🟡 ready+   | —           | spec 已对齐（PR #89 Kimi）· 等 MVP-04 全 done 后启动                          |
| **MVP-06** | 配置导入                    | 🟡 ready+   | —           | Phase A parser done（PR #80/#81）· Phase B IPC/UI/apply 待 MVP-04 收尾        |
| **MVP-07** | Git Log 只读                | ✅ **done** | OpenCode    | PR #83 · gix 0.70 · 937 行 · 7 ts-rs bindings                                 |
| **MVP-08** | Diff + Git Status           | 🟡 ready+   | —           | spec 已对齐（PR #93 Kimi · 5 件加强）· **实施 Phase A 可启动**                |
| **MVP-09** | Stage / Unstage / Commit    | 🟡 ready    | —           | 依赖 MVP-08 · 未启动                                                          |
| **MVP-10** | Settings + Telemetry + 打包 | 🟡 ready    | —           | spec ready（PR #88 Kimi）· 依赖 Apple Dev Program（§B pending）               |

**v0.1.0-alpha 主线**：MVP-02 ✅ / MVP-03 ✅ / MVP-04 🟢 85% / MVP-07 ✅ / MVP-08 起步 → 剩 MVP-05/06/08/09/10 大块实施

---

## 🔬 SPIKE 状态（决策支撑）

| ID              | 主题                  | 状态                                       | ADR 输出                                                  |
| --------------- | --------------------- | ------------------------------------------ | --------------------------------------------------------- |
| SPIKE-01        | Tauri 三平台启动      | 🔴 blocked · Ubuntu 待                     | ADR-006 accepted（macOS PASS）                            |
| SPIKE-02        | Tauri 硬通过矩阵      | 🔴 blocked · Ubuntu 待                     | macOS 10/10 · bundle 10MB                                 |
| SPIKE-03        | git2 vs gix benchmark | ✅ **done**                                | ADR-007 · gix 1973× 快                                    |
| SPIKE-04 / 04.5 | rusqlite 数据安全     | ✅ **done**                                | ADR-005 · redb silent corruption FAIL · rusqlite accepted |
| SPIKE-05 / 05.5 | PTY 多 Tab 压测       | ✅ **done**                                | ADR-003 · portable-pty + shared-reader + drop-oldest      |
| **SPIKE-06**    | CLI 协议 + codesign   | 🔴 blocked · §A done · §B Apple Dev 申请中 | §A 36 脱敏样本 · §B 影响 MVP-10 GA                        |
| SPIKE-07        | v1.0-pre AI parser    | 🟡 draft                                   | v1.0 范围 · 未启动                                        |
| SPIKE-08        | E2E + IPC contract    | ✅ **done**                                | ADR-014 · ts-rs codegen                                   |

---

## 🔒 核心决策锁定（CLAUDE.md A 栏 · 14 ADR）

| #            | 决策                                         | 锁定依据                |
| ------------ | -------------------------------------------- | ----------------------- |
| License      | Apache 2.0（不签 CLA）                       | ADR-001                 |
| MVP 范围     | B 折中方案（砍 push/pull/rail graph）        | ADR-002                 |
| PTY          | portable-pty + shared-reader + drop-oldest   | ADR-003 · SPIKE-05/05.5 |
| 前端栈       | SolidJS + TypeScript + xterm.js              | ADR-004                 |
| 本地存储     | **rusqlite**（redb superseded）              | ADR-005 · SPIKE-04.5    |
| 桌面框架     | **Tauri 2**（macOS 强 PASS · Ubuntu caveat） | ADR-006                 |
| Git 栈       | **git2 写 + gix 读**（1973× 加速）           | ADR-007 · SPIKE-03      |
| Diff 渲染    | 自建（similar crate · **禁 Monaco**）        | ADR-008                 |
| AI-Aware     | v1.0 vision · 对外不提                       | ADR-009                 |
| IPC contract | Rust struct + ts-rs codegen                  | ADR-014                 |
| Runtime 证据 | `docs/runtime-evidence/<task-id>/`           | ADR-011                 |
| v2-D.1 治理  | 单人项目 self-review + Arbiter approval      | ADR-012                 |
| Spike 冷备   | v1 强制 → v2 推荐                            | ADR-013                 |

---

## 🤝 多 Agent 协作成就（独特优势）

| Agent           | 类型     |  协作次数 |     merge 率 | 主要贡献                                                             |
| --------------- | -------- | --------: | -----------: | -------------------------------------------------------------------- |
| **Claude Code** | 主 agent |      全程 |            — | 编排 · cross-review · PROGRESS/CHANGELOG 维护 · CI fix（PR #86/#90） |
| **Kimi**        | 远程 API | **11 次** |         100% | spec review × 9 + 代码 × 2（MVP-06 parser）· 平均 23 min             |
| **Codex CLI**   | 本地 CLI |      多次 | 100%（近期） | MVP-04 Phase B/C/E 硬核实施 · PR #82/#91/#95                         |
| **OpenCode**    | 本地 CLI |      多次 |         100% | MVP-02/03/07 大块实施 · PR #94 rules README 零代修                   |

### 规则内化里程碑（session 16）

author 错归 3 连事件（PR #71/#82/#83）后 · §2.5.3 三条铁律（git config + trailer + log verify）全落地 · session 16 **零代修**交付（Codex PR #95 一次正确 · Kimi PR #93 fix-up 自主 amend）

---

## 🚧 关键阻塞 / 风险

| 阻塞                                              | 影响                                             | 决策路径                                            |
| ------------------------------------------------- | ------------------------------------------------ | --------------------------------------------------- |
| **Ubuntu 24 LTS 环境**                            | SPIKE-01/02 §B + MVP-01 Phase C + MVP-04 Phase D | **降为 v0.1 GA 最低优先**（S-3 · macOS-first 策略） |
| **Apple Developer Program**（$99/y · 2d-2w 审核） | MVP-10 codesign / notarization GA gate           | **用户决策中** · 不阻塞 MVP-04-09 实施              |
| **Linux PTY timing 技术债**                       | 2 个 pty test Linux-only ignore                  | MVP-04 Phase D 启动时解除 · 不阻塞 macOS            |

---

## 🏁 v0.1.0-alpha 距离（按优先级）

### 剩余 must-do（估 15-20 工作日）

1. **MVP-04 Phase F** · runtime 证据 + A.5/E.2/E.4 性能量化（0.5-1d · 主 agent 本地接）
2. **MVP-08 实施 Phase A-E** · Diff + Status 面板（2-3d · 派 Codex/OpenCode）
3. **MVP-05 实施 Phase A-D** · Pane 分屏（4d · 派 Codex）
4. **MVP-06 Phase B-C** · IPC + UI + apply（2-3d）
5. **MVP-09 实施** · Stage/Unstage/Commit（3-4d）
6. **MVP-10 实施**（条件：Apple Dev 到位）· Settings + Telemetry + 打包（3-4d）

### 可选降级路径

- Apple Dev 未到 → v0.1.0-alpha 走 ad-hoc sign（用户自己关 Gatekeeper 装）· GA 前再补正式 codesign

---

## ➡️ 下一 session 起点（推荐顺序）

| #   | 动作                                                                             | 难度  | 派谁                                       |
| --- | -------------------------------------------------------------------------------- | ----- | ------------------------------------------ |
| 1   | **MVP-04 Phase F**（跑 tauri:dev + Playwright 采样 A.5/E.2/E.4 · 4-5 截图）      | 🟢 低 | 主 agent（只有本地 macOS 可做）            |
| 2   | **MVP-08 实施 Phase A**（diff 算法 + IPC 后端 + 8 ts-rs bindings · spec 已对齐） | 🟡 中 | 派 Codex（对 Rust + IPC + ts-rs 模式最熟） |
| 3   | **MVP-05 / 06 并行**（Pane 分屏 + 配置导入 Phase B-C）                           | 🟡 中 | 派 OpenCode + Kimi                         |

---

## 关联文档

- `docs/PROGRESS.md` · rolling 状态面板（本快照的活水源头）
- `CHANGELOG.md` · Keep a Changelog 详细条目
- `CLAUDE.md` · 决策表 A/B/C + 禁区 + 5 步 checklist
- `docs/adr/` · 14 个 ADR accepted
- `docs/tasks/` · 7 SPIKE + 20 MVP spec
- `docs/session-history/` · session 反思归档

---

**归档时间戳**：2026-04-22 · session 16 收尾 · main@d975e10
**归档人**：Claude Code（主 agent）
**规则触发**：`~/.claude/rules/10-auto-analysis-doc.md`（"梳理"动词触发 · 双写项目内 + Obsidian）
