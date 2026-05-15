# SPIKE-07 · CLI 输出协议 parser 验证 Report（R1 降级前置 · 2026-05-15 session 32）

> **Task spec**：[`docs/tasks/SPIKE-07-cli-protocol-parser.md`](../tasks/SPIKE-07-cli-protocol-parser.md) · status: in-progress
> **阶段结论**：**§H 路径 3 · deferred** —— R1 **保留 HIGH/HIGH** · AI-Aware v1.0 vision 推迟 · MVP-18/19/20 保持 draft
> **判据**：§H（spec single source of truth · session 32 Arbiter 钦定）· §E.3/§E.5 为 informative 诊断 · 与 §H 冲突以 §H 为准
> **Review**：待独立评审 + Arbiter 拍板（§2.1 · 主 agent 不自 accept ADR-017 · 不自 flip CLAUDE.md 决策表 #3 / spec status）
> **数据真实性声明**：本报告 0 条 fabricated 数据 · 每个数字溯源 `docs/spikes/raw/SPIKE-07/phase-c-matrix.{md,json}` · 样本量 n=36（置信度限制见 §置信度与 caveat）

---

## 测试环境（§E.9）

| 项          | 值                                                                            |
| ----------- | ----------------------------------------------------------------------------- |
| OS          | macOS 26.3.1 (Build 25D771280a)                                               |
| cargo       | `1.95.0 (f2d3ce0bd 2026-03-21)`                                               |
| rustc       | `1.95.0 (59807616e 2026-04-14)`                                               |
| corpus      | SPIKE-06 PR #71 · `docs/spikes/raw/SPIKE-06/` 36 条 `*.redacted.cast`         |
| corpus 结构 | 2 CLI（claude/codex）× 6 场景 × 3 take = 36                                   |
| 原型代码    | `docs/spikes/code/SPIKE-07/`（进 git · Cargo.lock 冻结 · `cargo run` 可复现） |
| 工作分支    | `spike/SPIKE-07-cdef`                                                         |

可复现性（§E.9）：parser 无随机性 · 同一样本跑 3 次结果一致（`cast` 解码 + adapter 解析均确定性）· `cargo test` 含真实 36-corpus 完整矩阵集成 test（`loads_real_corpus_36_complete_matrix`）。

---

## §C · Phase 进度回溯

| Phase | 范围                                                           | 状态 / PR                                              |
| ----- | -------------------------------------------------------------- | ------------------------------------------------------ |
| A     | cast v3 解码 + fixture loader + `CliEvent` IR + trait + survey | ✅ merged PR #333（主 agent 单做）                     |
| B-1   | `parser::claude` 薄协议 adapter                                | ✅ merged PR #334（Droid dispatch）                    |
| B-2   | `parser::codex` 厚结构 adapter                                 | ✅ merged PR #335（Codex CLI dispatch）                |
| C     | §F 测试矩阵 36 样本逐条断言                                    | ✅ 本报告（`src/assertions.rs` + `src/bin/matrix.rs`） |
| D     | 准确率统计 + 统一抽象分析                                      | ✅ 本报告                                              |
| E     | ADR-017 起草（§H 路径 3 deferred 变体）                        | ✅ `docs/adr/ADR-017-ai-aware-deferred.md`（proposed） |
| F     | report + raw 归档 + 3 样齐全                                   | ✅ 本 PR                                               |

---

## §F · 测试矩阵实测结果（informative · 数字溯源 phase-c-matrix.json）

整体：**36 样本 · 0 panic · PASS 24/36 = 66.7%**（panic 计数证 §G fail signal #3 不触发）。

### 场景级正确率

| 场景               | PASS/total | 正确率 | 平均 Unrecognized | 判定       |
| ------------------ | ---------- | ------ | ----------------- | ---------- |
| happy_path         | 6/6        | 100%   | 0%                | ✅         |
| auth_fail          | 6/6        | 100%   | 0%                | ✅         |
| network_error      | 6/6        | 100%   | 11%               | ✅         |
| interrupt_residual | 6/6        | 100%   | 100%              | ✅（宽松） |
| long_stream        | 0/6        | 0%     | 50%               | ❌         |
| mixed_ansi_json    | 0/6        | 0%     | 100%              | ❌         |

### CLI 级正确率

| CLI    | PASS/total | 正确率 |
| ------ | ---------- | ------ |
| claude | 12/18      | 67%    |
| codex  | 12/18      | 67%    |

两 CLI 对称：均在 happy/auth/network/interrupt 全过 · 均在 long_stream/mixed_ansi_json 全败（同一根因 · 见 Phase D）。

---

## Phase D · Decision-grade 分析

### D.1 · 12 条 FAIL 三类根因（关键发现 · 非单一"parser 差"）

| 根因类别                               | 样本                                  | 实测证据（raw 溯源）                                                                               | 性质                                       |
| -------------------------------------- | ------------------------------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| **(1) corpus 质量**：TUI 屏幕重绘 blob | claude long_stream 1/2/3              | `content 0/142588`、`0/300091`、`0/209721` = 0%（142–300KB · claude.rs >5KB 正确包 Unrecognized）  | 录制方式缺陷 · 非 parser bug               |
| **(2) §F 断言对厚协议校准失配**        | codex long_stream 1/2/3               | `content 8529/9183=93%`、`7703/8357=92%`、`7912/8566=92%`（events=28 · 0% unrec · 厚结构成功解析） | 断言阈值问题 · parser 实际优秀（差 2-3pp） |
| **(3) 协议现实**：CLI 不发机器 JSON    | claude+codex mixed_ansi_json 1/2/3 ×2 | `无可解析 JSON`（两 CLI stdout = 人类终端格式文本 · 无结构化 JSON events · Phase A survey 已实测） | 产品前提缺陷 · 与 parser 能力无关          |

**解读**：12 FAIL 中**无一条是 parser 实现 bug**。

- 类 (1)：claude long_stream 样本是 PTY 原始屏幕重绘字节（光标移动 + 区域重画），不是行式 assistant 流。任何 parser 都无法从屏幕重绘提取"95% content"——内容被数千 ANSI cursor/repaint 序列交织覆盖。这是 SPIKE-06 录制方式（捕获 raw PTY screen）导致 · 触发 §G fail signal #4（样本不够真实）。
- 类 (2)：codex long_stream **解析成功**（events=28 完整厚结构 · 0% Unrecognized），仅因 §F"content ≥ raw 95%"分母含厚协议脚手架（`key: value` header → SessionMeta · `hook:` → Hook · `tokens used` → Usage · 约占 raw 7-8%，正确地不计入 message content）而差 2-3pp。这是 §F 断言为薄协议设计、未对厚协议校准 · **不是 parser 缺陷**。
- 类 (3)：mixed_ansi_json 两 CLI 全败 · 根因是 **Claude CLI 与 Codex CLI 都不在 stdout 发机器可解析 JSON**（Phase A 全 36 样本 survey 已实测：claude = 纯 assistant 文本 + ANSI；codex = 厚文本结构 role marker/hook，无 JSON）。"mixed_ansi_json"场景名预设了一个这些 CLI 实际不具备的 JSON 协议。

### D.2 · §E.4 统一抽象可行性（正面发现 · 非 deferral 原因）

| 维度              | 实测                                                                                                                      |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------- |
| IR 共享           | **共享同一 `CliEvent` enum + `CliParser` trait**（`src/ir.rs` 锁定 · Phase B 两路 adapter 未改 IR · "IR gap 无"双方声明） |
| Claude（薄）变体  | MessageStart/MessageDelta/MessageEnd/Error/Unrecognized = 5/10 变体                                                       |
| Codex（厚）变体   | 上述 5 + SessionMeta/Hook/Usage = 8/10 变体（ToolUse 两 CLI 均未见 · 对齐 §J 决策表 #4）                                  |
| 核心变体重合      | 两 CLI 共用核心 5 变体 = **100% 重合**；Codex 专有 3 变体是 **additive 非冲突**                                           |
| §G fail signal #2 | **不触发**（IR 字段重合度 ≫ 70% · adapter 层成本可接受：claude.rs 12KB / codex.rs 16KB vs IR 核心 6KB · 比值合理）        |

**结论**：统一抽象 **可行**——薄/厚 CLI 共用单一 IR，差异吸收在各自 adapter（薄 CLI 只是不发厚变体，非冲突）。**deferral 与统一抽象无关** · 若 corpus 能验证 structured-streaming 前提，本 IR + adapter 架构是 sound 的。对齐 §J 决策表 #3「假设可统一」实测成立。

### D.3 · §E.11 基线对比

| 方法                 | error-detection 准确率 | 说明                                    |
| -------------------- | ---------------------- | --------------------------------------- |
| Parser（Error 事件） | **97%**（35/36）       | 结构化解析                              |
| 基线 A 关键字扫描    | 69%（25/36）           | exit≠0 / error/unauthorized/failed 子串 |
| 基线 B 行首启发式    | 83%（30/36）           | `^Error:` / `^WARNING:`                 |

Parser − 最优基线 = **+14pp**。§E.11 要求 +20pp 才"值复杂度"——**未达 §E.11 门槛**。
诚实解读：+14pp 是**窄口径 error-detection** 比较；parser 真实价值在完整 IR（role/hook/usage/session 结构化），非仅判错。但就 §E.11 设定的这条窄指标，parser 未显著超基线 · 报告如实记录不粉饰。

### D.4 · §E.10 Unrecognized 人工审计

| 桶                          | 样本                                           | 归类依据                                                                           |
| --------------------------- | ---------------------------------------------- | ---------------------------------------------------------------------------------- |
| (c) 录制方式导致结构损坏    | claude long_stream/interrupt/mixed（高 unrec） | SPIKE-06 捕获 raw PTY 屏幕重绘 · 非行式流 · 需 **重录**（headless/结构化输出模式） |
| (b) 真正的新格式 / 协议现实 | codex mixed_ansi_json（5×Unrecognized）        | codex 短样本无 role marker · 非 JSON · 属协议现实非 parser 缺                      |
| (a) 已知模式 parser 未覆盖  | 无                                             | happy/auth/network 0% Unrecognized · 已覆盖                                        |
| (d) 无法分类                | 无                                             | —                                                                                  |

主桶 = (c)：corpus 录制方式问题（屏幕重绘 ≠ 流），不是 parser 设计缺陷。

---

## §H · Fallback 判定（single source of truth · session 32 Arbiter 钦定）

逐路径核对：

| §H 路径             | 条件                                                                  | 实测                                                         | 命中 |
| ------------------- | --------------------------------------------------------------------- | ------------------------------------------------------------ | ---- |
| 路径 1 greenlight   | 整体加权 ≥ 96% **且** 各场景 ≥ 90% **且** 两 CLI 可统一               | 整体 66.7% < 96% · long/mixed 0% ≪ 90%                       | ❌   |
| 路径 2 single-cli   | 一 CLI ≥ 96% · 另一 < 90%                                             | claude 67% · codex 67%（对称 · 无一 ≥ 96%）                  | ❌   |
| **路径 3 deferred** | 两 CLI 均 < 90% **或** 任一场景 < 85% **或** parser 多数 Unrecognized | 两 CLI 67% < 90% · long_stream 0% & mixed_ansi_json 0% ≪ 85% | ✅   |

**§H verdict = 路径 3 · deferred**。对齐 §M 路径 C/D（deferred · 含"样本不真实"意外路径）· §G fail signal #1（任一场景 < 90%）+ #4（样本不够真实）触发。

**R1 不降级** —— R1 保留 HIGH/HIGH · AI-Aware v1.0 vision 推迟 v2+ · MVP-18/19/20 保持 draft（不进 in-progress）· CLAUDE.md 决策表 #3 ⚠️ 保留。

### decision-grade nuance（Arbiter 拍板必读）

deferred **不是因为 parser 实现差**：

- 4/6 场景 100%（happy/auth/network/interrupt）· 0 panic（§G#3 clear）· codex 厚结构解析优秀（events=28 · 92-93% content · 完整 SessionMeta/Hook/Usage）· 统一 IR 抽象可行（§G#2 不触发）
- deferred 的真实原因 = **AI-Aware 产品前提（CLI 发可解析 structured stream）未被真实 corpus 验证**：long_stream corpus 是 TUI 屏幕重绘 blob（录制缺陷）· mixed_ansi_json 预设的 JSON 协议在两 CLI 中不存在（协议现实）

---

## 建议（供 Arbiter 决策 · 非主 agent 自定）

| 选项                                                                                                                                        | 代价   | 适用                                   |
| ------------------------------------------------------------------------------------------------------------------------------------------- | ------ | -------------------------------------- |
| A. 重录 SPIKE-06 corpus：抓 CLI **headless / 结构化输出模式**（如 `claude --output-format json` 若存在 · codex 非 TTY 模式）→ 重跑 SPIKE-07 | 0.5-1d | 若 CLI 确有结构化输出模式 · 推荐先验证 |
| B. 接受 AI-Aware 需启发式终端态推断（fragile）· 推 v2+ 评估                                                                                 | 0      | 若 CLI 无结构化输出模式                |
| C. v1.0 直接回归基础卖点（多 Tab 终端 + Git 工作台）· AI-Aware 不在 v1.0                                                                    | 0      | 当前 ADR-017 deferred 默认路径         |

主 agent 推荐：**先 A 调研**（成本低 · 若 CLI 有 headless JSON 模式则 parser+IR 架构已 sound · 可能翻盘）→ 否则 C。最终由 Arbiter 拍板，ADR-017 当前按 deferred（路径 C）起草 proposed。

---

## 置信度与 caveat（§E.6）

- **样本量 n=36**（2 CLI × 6 场景 × 3 take）· 统计置信度有限 · 每场景仅 3 take · 单次 CLI 版本快照（claude 2.1.114 / codex 0.121.0 · SPIKE-06 录制时）
- **corpus 质量 caveat 显式声明**：long_stream/interrupt SPIKE-06 样本为 raw PTY 屏幕重绘（非行式流）· mixed_ansi_json 预设 JSON 协议不存在于实测 CLI · 这是 deferred 的核心驱动 · 非 parser 能力上限
- **原型代码非生产级**：`docs/spikes/code/SPIKE-07/` 是归档级原型（spec §C Don't.5）· v1.0 实施时重写
- **0 fabricated**：所有数字来自 `cargo run --bin matrix` 实跑 · 溯源 `phase-c-matrix.json`（rows[36] + per_scenario + per_cli + baseline 字段）

---

## 交付物（3 样必交 · 对齐 `.claude/rules/spike-delivery-checklist.md`）

| #   | 物料     | 路径                             | 状态                                                     |
| --- | -------- | -------------------------------- | -------------------------------------------------------- |
| 1   | 决策文档 | `docs/spikes/SPIKE-07-report.md` | ✅ 本文件                                                |
| 2   | 实测源码 | `docs/spikes/code/SPIKE-07/`     | ✅ 进 git（Cargo.lock 冻结 · `cargo run` 可复现）        |
| 3   | Raw 数据 | `docs/spikes/raw/SPIKE-07/`      | ✅ phase-c-matrix.{md,json} + phase-a-\* + README        |
| 4   | 冷备     | `spike-tmp/archive/SPIKE-07/`    | 🟡 推荐可省（纯 Cargo + Cargo.lock 进 git · ADR-013 v2） |

每个 report 数字溯源：场景/CLI/整体正确率 → `phase-c-matrix.json` `per_scenario`/`per_cli`/`overall_*` · 12 FAIL 明细 → `rows[].assessment.checks` · 基线 → `baseline` 字段 · `cargo run --bin matrix`（在 `docs/spikes/code/SPIKE-07/`）可 byte-level 复现。
