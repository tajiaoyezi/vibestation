# ADR-022: dispatch-prompt-template.md §4 参考实现引用路径 stale（指 top-level 实在 \_archived/）

**状态**：accepted
**日期**：2026-05-16（proposed）· 2026-05-17（accepted · Arbiter tajiaoyezi 拍板方案 (d) · 见下「事实修正」）
**决策者**：Grok（dispatch 起草 · self-review v2-D.2 单人项目）· Claude Code 主 agent 独立 review（**证伪原 Context · 见「事实修正」段**）· Arbiter tajiaoyezi 拍板
**对应 `CLAUDE.md` 决策表**：—（治理规则 · 本 ADR 记录 dispatch 模板路径引用漂移）
**前置事件**：PR #329（dispatch-prompt-template.md 压缩重构 · 883→597 行 · 审计附录拆分到 docs/dispatch-incidents.md）· spike-tmp/dispatch/ 目录 cleanup（top-level 清空 · 示例 prompt 移入 \_archived/ 子目录保留）

---

## 背景与问题（Context）

`.claude/rules/dispatch-prompt-template.md` 第 419 行声明：

> dispatch prompt 文件统一放 `spike-tmp/dispatch/` · 命名格式：`<TASK-ID>[-<phase-or-pr-suffix>]-<agent>-prompt.md`

§4「参考实现 · 选择指南」（当前 origin/main 实际 §4 标题）内表格推荐 4 类范本：

- `MVP-04-storage-prep-opencode-prompt.md`
- `MVP-07-kimi-prompt.md`
- `SPIKE-06-pr2-codex-prompt.md`
- `MVP-02-opencode-prompt.md`

并在 "推荐参考全列表" 链接到 `docs/dispatch-incidents.md §4`。

**实测现实**：`spike-tmp/dispatch/` 顶层目录已被 cleanup 清空 · 这些示例 prompt 实际存放在 `spike-tmp/dispatch/_archived/` 子目录下（top-level 空 · 仅 \_archived/ 保留历史）。§4 及第 419 行的路径声明已**断链** —— 未来 agent 按文档所述路径无法找到范本文件。

注：#329 压缩后 §4 标题已从原 "4 · 参考实现 · 选择指南" 演进为当前形式 · 路径引用问题遗留未修。

## 🔴 事实修正（Claude Code 主 agent 2026-05-17 独立 review · git 核验证伪原 Context）

**原「背景与问题」段的事实描述有重大偏差** · 据此提出的 (a)/(b)/(c) 三选项**没有一个能真正修复断链**。git 核验坐实如下：

| 原 Context 声称                                          | git 核验实测                                                                                                                                                                                                                                                                                                                                                  |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spike-tmp/dispatch/` 顶层已清空 · 范本都在 `_archived/` | 顶层**不空**（MVP-18/19 等近期 prompt + `_archived/` 子目录并存）· `_archived/` 里**找不到** §4 写的 4 个范本（精确文件名核对全 ABSENT）                                                                                                                                                                                                                      |
| 根因 = 路径前缀错（top-level → `_archived/`）            | 真实根因**双层**：(i) `.gitignore:116` = `spike-tmp/` → **整个 `spike-tmp/` gitignored** · `git ls-files` 确认 git 中**零 dispatch 范本文件**；(ii) §4 写的 4 个文件名（`MVP-04-storage-prep-opencode-prompt.md` / `MVP-07-kimi-prompt.md` / `SPIKE-06-pr2-codex-prompt.md` / `MVP-02-opencode-prompt.md`）本身已 stale，本地 `_archived/` 也不存在这些精确名 |
| (a) 改 `_archived/` 前缀即修复                           | 改后路径仍：文件不存在 + gitignored，clone repo 的 agent 永远看不到 · 是"看起来修了实际没修"的**假对齐**（比 proposed 标记更危险）                                                                                                                                                                                                                            |

**结论**：clone repo 的 agent 看不到**任何** dispatch 范本文件（无论 top-level / `_archived/`），因为整个 `spike-tmp/` 不进 git。修复方向不是改路径前缀，而是**文档停止承诺一个 git 中不存在的可点击路径**。`docs/dispatch-incidents.md §4`（行 365-389）有同样的断链引用，须同步修。

## 决策（Decision · accepted · Arbiter 2026-05-17 拍板方案 (d)）

原 proposed 的 (a)/(b)/(c) 三选项均建立在被证伪的前提上（详见上「事实修正」段）· Arbiter 基于修正事实拍板**新增方案 (d)**：

- ~~**(a)** 更新 §4 路径为 `_archived/` 前缀~~ —— **作废**（文件不存在 + gitignored · 假对齐）
- ~~**(b)** 范本移回 top-level~~ —— **作废**（gitignored · clone 仍看不到）
- ~~**(c)** §4 泛指 `_archived/` 目录~~ —— **作废**（目录 gitignored · clone 仍看不到）
- **(d)【Arbiter 2026-05-17 选定】文档不再承诺 git 路径**：`.claude/rules/dispatch-prompt-template.md` §3.0 第 441 行 + §4 + `docs/dispatch-incidents.md` §4 删除指向具体范本文件名的可点击路径引用 · 改为明确「dispatch 范本是本地 `spike-tmp/dispatch/`（**gitignored · 不进 git · clone 后不可见**）工作产物 · 写 dispatch 时参照本规则 §3 标准模板 + `docs/dispatch-incidents.md §4` 的范本特征描述（按 agent 类型/任务类型选结构），不依赖具体范本文件可点击」

## 约束（Constraints）

- 本 ADR **仅记录+提议** · 不改 `.claude/rules/dispatch-prompt-template.md` / `docs/dispatch-incidents.md` / 任何决策文件（Arbiter accept 后另 PR 改）
- status **proposed** · 需 Arbiter 拍板 → accepted 后方生效（v2-D.2 单人项目 self-review + Arbiter approval 流程）
- 路径现状严格以 `git show origin/main:.claude/rules/dispatch-prompt-template.md | grep -n '参考实现\|spike-tmp/dispatch'` 实测为准 · 未臆断 §4 位置
- 不得声称 "Arbiter 已同意 X"

## 后果（Consequences）

**正面**：

- 修复 dispatch 模板引用断链 · 未来 agent（尤其是新 agent 首任务）能按 §4 推荐快速定位历史范本
- 明确 `spike-tmp/dispatch/_archived/` 是 cleanup 后的规范归档位置 · 与 spike-delivery-checklist 等规则保持一致
- 最小改动 (a) 成本最低 · 保持现有 basename 推荐表不变，仅修正目录前缀

**负面 / 风险**：

- 若选择 (b) 回迁：违反 "archive 保留历史 · top-level 保持干净" 的 cleanup 意图 · 增加维护负担
- 任何选项均需后续 PR 实际改 dispatch-prompt-template.md（本 PR 不执行）
- incidents.md §4 历史记录可能需同步更新（非本 PR 范围）

---

## Arbiter 拍板栏（tajiaoyezi · v2-D.2 单人项目 self-review + Arbiter approval · 2026-05-17 已拍板）

- [x] 事实准确性：**原 Context 已被主 agent git 核验证伪** · 修正事实见「事实修正」段（`.gitignore:116` spike-tmp/ 整个 gitignored · 4 范本文件名全 ABSENT · `git ls-files` 零范本）· Arbiter 基于修正事实拍板
- [x] 选项完整：原 (a)(b)(c) 作废理由已陈述 · 新增 (d) 由 Arbiter 基于修正事实选定
- [x] 约束遵守：proposed 阶段未碰 dispatch-prompt-template.md（本 accept PR 才执行 · v2-D.2 流程合规）· 证伪过程仅 git 只读核验未改源文件
- [x] **选定方案：(d)** —— Arbiter tajiaoyezi 2026-05-17 拍板「文档不再承诺 git 路径 · 范本是本地 gitignored 工作产物 · 参照 §3 模板 + incidents.md §4 特征描述」

**accepted 决议**（Arbiter 2026-05-17 flip · 本 PR 同步执行文档改写）：

1. 记录事实修正：原 ADR-022「背景与问题」事实偏差（误判根因为路径前缀 · 实为 spike-tmp/ 整个 gitignored + 文件名 stale 双层）· 由主 agent git 核验证伪并记入「事实修正」段（保留原 Context 不删 · 供未来 agent 看到"proposed 事实可能不准 · accept 前主 agent 必独立核验"的教训）
2. 选定 (d)：dispatch-prompt-template.md §3.0 第 441 行 + §4 + dispatch-incidents.md §4 删除断链的可点击范本路径引用 · 改为「范本 = 本地 gitignored 工作产物 · 参照 §3 模板 + incidents.md §4 特征描述」· 本 PR 执行
3. 衍生教训（已沉淀入 PR body）：dispatch 起草的 ADR proposed 事实段不可全信 · accept 翻转前主 agent 必须独立 git 核验事实，证伪即回 Arbiter 重新决策（本案 §2.1 + 08-systematic-debugging 正例）

---

**实测坐实**（Grok dispatch · 2026-05-16）：

- dispatch-prompt-template.md 路径声明（第 419 行附近）：`spike-tmp/dispatch/`（git show origin/main 确认）
- §4 标题：`## 4 · 参考实现 · 选择指南`（git show origin/main 确认）
- 实际 FS：spike-tmp/dispatch/ 顶层不存在或为空 · \_archived/ 子目录保留历史 prompt（find / ls 验证）
- 推荐模板表：4 个 basename 引用（MVP-04-... 等）存在于 §4
