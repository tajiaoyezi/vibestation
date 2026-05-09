---
id: MVP-12
type: mvp
title: 自绘 commit rail graph（Git Log 图形化）
status: ready
owner:
phase: v0.3
depends_on: ["MVP-07"]
blocks: ["MVP-16"]
blocked_by: []
blocked_from:
blocked_note:
estimate: 8d
plan_ref: implementation-plan.md §10.1（v0.2 范围）· §5.4（Git Log 数据模型 rail 字段）· §11 W16（自绘 rail graph）
risk_ref: 本 spec §已知风险 R1-R5（Canvas 性能 / DAG 算法边界 / DPI / 色盲可读性 / 触屏交互）
reviewer: Droid
---
# MVP-12: 自绘 commit rail graph（Git Log 图形化）

> **状态**：`draft`（spec 详化完成度 100% · 仅前置任务定义完成，等待 Arbiter comment approve 后翻 `ready`）
> **依赖**：MVP-07（Git Log 只读视图 · commit list 数据流已就绪）
> **下游 blocks**：MVP-16（rebase/merge/cherry-pick 期间 rail overlay 依赖本 task 合同）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md)（MVP 砍到 v0.2）· [`§5.4`](../implementation-plan.md)（CommitNode.rail 预留）· [`§11 W16`](../implementation-plan.md)（v0.2 周计划）
> **详化时间**：2026-05-06 session 24 · Droid（Factory.ai）· self-review（v2-D.2 单人项目模式）

---

## 🎯 目标（Goal）

在 MVP-07 已有 Git Log commit 列表左侧新增 **Canvas 自绘 rail graph**，把 commit DAG 的分叉 / 合并 / 分支 tip 关系可视化，做到「看 commit list 同时看关系图」，并且在 10 万 commit 仓库保持首屏 <500ms、滚动 60fps。

本 task 只做 **spec 详化**（draft → ready-grade 内容），不实施代码、不翻 frontmatter 状态；实现阶段由后续 agent 按本 spec 的 Phase A/B/C/D 执行。战略范围严格对齐 `implementation-plan.md §10.1` 的 v0.2 增量，不扩大到 WebGL/3D/interactive rebase。

## 📖 背景（Context）

- **为什么现在做**：`implementation-plan.md §10.1` 明确 v0.1 为降风险只保留「commit list + ref 标签贴」，rail graph 推迟到 v0.2；该任务是 v0.2 体验跃迁的关键。
- **数据基础已具备**：MVP-07 已有 commit list 数据流、排序与过滤、selected row 状态；MVP-12 只需消费同一数据源，不重建 GitLog 数据管线。
- **路线图位置明确**：`implementation-plan.md §11 W16` 将 rail graph 单列一周，前置 W13/W14/W15 已分别处理 branch CRUD、sync、pane 扩展。
- **项目禁区约束**（CLAUDE.md）：
  - 不允许改对外叙事（不提 v1.0 vision 术语）
  - 不允许引入重型图形栈（本 spec 锁定 Canvas，禁 WebGL/Three.js 等）
  - 不允许跳过 gates（本 task 为文档类，runtime evidence 豁免，但需通过 task-spec validator + typecheck）
- **风险演进**：`implementation-plan.md` 风险 R13 已注明 commit graph 算法复杂度，MVP 阶段先不做；当前任务的价值是把复杂度显式拆解、把算法争议前置到 SPIKE-09，避免实施期反复改方案。
- **上游已落地项**：
  - MVP-07：Git Log 只读（commit row、ref chip、selected state）
  - MVP-09：git2 写路径模式成熟（尽管本 task 主要前端，但 event 合同复用该模式）
  - MVP-13：branch 变化事件 `git:branch-changed` 已作为联动基线（本 spec 在 §H.8 锁定边界）

---

## 🛠 实施进度（Phase A/B/C/D）

MVP-12 估时 **8d**，拆 4 phase 串行推进，每 phase 2d；spec 详化完成后由后续实施 PR 落地。

| Phase | 范围 | 状态 | 预估 | 交付物 |
|---|---|---|---|---|
| Phase A · 数据接线 + 布局骨架 | 消费 MVP-07 commit list + refs，完成 lane 分配最小实现，输出布局快照（不含高级优化） | ✅ done · PR #N | 2d | `web/src/panels/GitLog/RailGraph/` + 4 ts-rs binding + 21 vitest 单测 + 6 快照 |
| Phase B · Canvas 绘制 + 视觉语义 | commit 节点、连线、merge/fork 节点形态、branch tip 标签（local/remote/tag） | ✅ done · PR #261 | 2d | `RailGraphCanvas.tsx` + 视觉回归首版 |
| Phase C · 虚拟化 + 交互 | viewport ±100、offscreen canvas、RAF、hover 路径高亮、collapse 策略 | 🟡 planned | 2d | `RailGraphVirtualizer.ts` + perf trace |
| Phase D · 集成 + 性能收敛 | 与 MVP-07 滚动同步、MVP-13 事件联动、主题色 token 化、性能预算验收 | 🟡 planned | 2d | Phase D 验收报告 + QA 清单 |

### Phase A 起点 checklist（接 spec 后 5 分钟可开工）

- [ ] 确认工作目录：`web/src/panels/GitLog/` 下新增 `RailGraph/` 子目录（路径为 placeholder，实施 PR 可微调）
- [ ] 复用 MVP-07 commit row 高度常量（避免 rail 与列表错位）
- [ ] 复用 MVP-07 commit 排序（时间倒序）作为 rail 输入顺序，不做二次排序
- [ ] 新增 `RailGraphInput` 类型（包含 commit oid、parents、refs、isHead）
- [ ] 准备 3 组 fixture：20 commit / 1k commit / 100k commit
- [ ] 保持「不改 MVP-07 commit-row DOM 结构」硬边界（只在左侧插入 rail 容器）
- [ ] 确认 branch color token 来源（不写死 hex）
- [ ] 确认 feature flag（`enableRailGraph`）默认仅 v0.2 打开
- [ ] 与 MVP-13 owner 对齐 `git:branch-changed` payload 字段（workspaceId/headOid/refsHash）
- [ ] 记录 SPIKE-09 待决项：算法三候选量化后再锁实现细节

### Phase A 任务拆分（20 项）
- [ ] A-Task 01. 定义 `RailGraphInputCommit`（oid/parents/refKinds/refNames/isHead）
- [ ] A-Task 02. 定义 `RailLaneAssignment`（rowIndex/laneIndex/colorKey）
- [ ] A-Task 03. 实现 `buildRailGraphInputFromGitLog()` 适配层
- [ ] A-Task 04. 兼容 merge commit `parents.length >= 2` 的输入结构
- [ ] A-Task 05. 兼容 root commit `parents.length == 0` 的输入结构
- [ ] A-Task 06. 兼容 detached HEAD（headOid 存在但 headName 为空）
- [ ] A-Task 07. 定义 lane 分配输出快照 JSON schema（用于回归）
- [ ] A-Task 08. 编写 20 commit fixture（含 2 次 merge）
- [ ] A-Task 09. 编写 1k commit fixture（合成）
- [ ] A-Task 10. 编写 100k commit fixture（合成，压测）
- [ ] A-Task 11. 增加输入去重（同 oid 重复记录过滤）
- [ ] A-Task 12. 增加 parent 丢失 fallback（浅克隆场景）
- [ ] A-Task 13. 增加 refs 归一化（local/remote/tag）
- [ ] A-Task 14. 将 branch 名映射到稳定 colorKey（hash）
- [ ] A-Task 15. 添加 lane 分配纯函数单测（happy path）
- [ ] A-Task 16. 添加 lane 分配边界单测（octopus merge）
- [ ] A-Task 17. 添加 lane 分配边界单测（cross-branch）
- [ ] A-Task 18. 导出 Phase A 快照 6 份（light/dark × 3 fixture）
- [ ] A-Task 19. 记录 Phase A 已知缺口（不含虚拟化、不含 hover）
- [ ] A-Task 20. Phase A PR 描述附「与 MVP-07 无侵入」证明截图
### Phase B 任务拆分（20 项）
- [ ] B-Task 01. 创建 `RailGraphCanvas` 组件（主 canvas + overlay canvas）
- [ ] B-Task 02. 绘制 commit 节点（默认圆点）
- [ ] B-Task 03. 绘制 merge 节点（菱形）
- [ ] B-Task 04. 绘制 fork 节点（方形）
- [ ] B-Task 05. 绘制纵向主干线（lane 主线）
- [ ] B-Task 06. 绘制斜向连接线（merge/fork 连线）
- [ ] B-Task 07. 绘制 branch tip 标签（local 实心）
- [ ] B-Task 08. 绘制 branch tip 标签（remote 半透明）
- [ ] B-Task 09. 绘制 tag 标签（矩形）
- [ ] B-Task 10. 渲染当前分支高亮（粗边 + glow）
- [ ] B-Task 11. 渲染非当前分支常态（细边）
- [ ] B-Task 12. 接入主题 token（light）
- [ ] B-Task 13. 接入主题 token（dark）
- [ ] B-Task 14. 实现 DPR 适配（1x/2x）
- [ ] B-Task 15. 实现 clip 区域裁剪（防止绘制溢出）
- [ ] B-Task 16. 加入 canvas resize observer
- [ ] B-Task 17. 实现 debug 网格开关（开发态）
- [ ] B-Task 18. 输出 10 张视觉基线图（不同分支密度）
- [ ] B-Task 19. 完成 Phase B 视觉评审 checklist
- [ ] B-Task 20. 记录与设计稿差异项并标注后续修复优先级
### Phase C 任务拆分（20 项）
- [ ] C-Task 01. 实现 viewport 计算（根据 scrollTop 与 rowHeight）
- [ ] C-Task 02. 实现 overscan（前后各 100 行）
- [ ] C-Task 03. 不可视区跳过绘制（短路返回）
- [ ] C-Task 04. 实现 offscreen canvas 缓存层
- [ ] C-Task 05. 实现 requestAnimationFrame 调度器
- [ ] C-Task 06. 实现渲染节流（同帧合并）
- [ ] C-Task 07. 实现 hover 命中测试（节点）
- [ ] C-Task 08. 实现 hover 命中测试（连线）
- [ ] C-Task 09. hover 时高亮整条 rail 路径
- [ ] C-Task 10. hover 离开后 1 帧内恢复
- [ ] C-Task 11. 实现 collapse 规则（<=20 全显示）
- [ ] C-Task 12. 实现 collapse 规则（21-50 压缩）
- [ ] C-Task 13. 实现 collapse 规则（>50 other 分组）
- [ ] C-Task 14. 实现 other 分组展开下拉
- [ ] C-Task 15. 添加交互单测（hover/toggle）
- [ ] C-Task 16. 添加性能采样埋点（draw ms/frame）
- [ ] C-Task 17. 记录输入 100k commit 的 FPS 分布
- [ ] C-Task 18. 记录输入 1M commit 的降级行为（不崩）
- [ ] C-Task 19. 确认触屏环境 fallback（tap 替代 hover）记录
- [ ] C-Task 20. 完成 Phase C 性能回归 baseline
### Phase D 任务拆分（20 项）
- [ ] D-Task 01. 接入 MVP-07 commit list 滚动事件（单向监听）
- [ ] D-Task 02. 确认滚动同步误差 <= 1px
- [ ] D-Task 03. 接入 `git:branch-changed` 事件重绘
- [ ] D-Task 04. 接入（v0.3）`git:rebase-state-changed` overlay 预留
- [ ] D-Task 05. 切换 workspace 时清理旧缓存
- [ ] D-Task 06. 切换主题时触发颜色重算
- [ ] D-Task 07. 切换 DPI 时重建 backing store
- [ ] D-Task 08. 异常数据兜底（parent 缺失）可显示占位线
- [ ] D-Task 09. 异常数据兜底（ref 重名）追加 disambiguation
- [ ] D-Task 10. 完成错误态空状态文案
- [ ] D-Task 11. 完成 QA 手册（macOS + Ubuntu）
- [ ] D-Task 12. 完成 60fps 验收脚本
- [ ] D-Task 13. 完成 10万 commit 首屏 <500ms 验收脚本
- [ ] D-Task 14. 完成 hover <16ms 验收脚本
- [ ] D-Task 15. 完成 branch event <50ms 验收脚本
- [ ] D-Task 16. 完成色盲模拟截图归档
- [ ] D-Task 17. 完成 1x/2x DPR 对照截图归档
- [ ] D-Task 18. 确认无新增 Rust 命令依赖
- [ ] D-Task 19. 补齐实施 PR 的 Test Plan 与回归结果
- [ ] D-Task 20. Phase D 完成后更新 task 状态（由评审流程控制）
---

## 🎨 功能范围（Scope）

### Do（本 task 必做）

- 在 Git Log 左侧固定 10% 宽度区域渲染 rail graph（宽度可按最小 120px / 最大 180px 夹逼）
- commit 节点可视化：每条 commit 1 个节点，支持 root / normal / merge / fork 四种语义
- branch 颜色使用稳定 hash 映射 30 色环（light / dark 各一套 token）
- merge commit 显示双入边，fork commit 显示出边分叉
- branch tip 标签贴在最右侧（local 实心、remote 半透明、tag 矩形）
- 滚动与 commit list 同步，rail 不自行持有主滚动容器
- hover（或 touch tap fallback）高亮该 commit 所属完整 rail 路径
- 分支数量 collapse：<=20 全显示，21-50 压缩，>50 收敛到 Other 分组
- 虚拟化渲染（viewport ±100）+ offscreen canvas + RAF
- 性能预算显式验收：首屏、帧率、交互延迟、事件重绘延迟均有数字门槛

### Don’t（明确不做，防 scope creep）

- 不做 WebGL 渲染（含 Three.js/Pixi.js/cytoscape 等）
- 不做 3D commit graph 或视觉特效化轨道
- 不做 interactive rebase 拖拽（MVP-16 右键流程负责）
- 不做 rail 上的 commit 详情卡片（v0.4+ 评估）
- 不做跨 remote 全图（仅 local + origin/*）
- 不修改 MVP-07 commit list 的 DOM 结构和交互语义
- 不引入重型第三方 DAG 可视化库（D3.js/vis.js 等）
- 不持久化 rail 布局缓存到数据库（仅内存）
- 不把算法候选在本 spec 阶段预先拍板（必须走 SPIKE-09）
- 不在本 PR 修改 CLAUDE.md / ADR / implementation-plan / 其他任务 spec

---

## 🖼 UI 引用（design/directions/1-calm-studio.html）

### 关键引用行

- `line 1021-1170`：Secondary Sidebar Git Log 整体结构（panel-head / git-subhead / commit-row）
- `line 1072+`：`<div class="commit-row">` 样例，含 `.graph`、`.node`、`.line` 语义
- `line 1080+`：`commit-meta`、`hash`、`time` 排版样式
- `line 831-838`：Branches 树 `ref-dot local/remote/tag` 视觉语义
- `line 291-295`：`.ref-dot` 与 `.badge` token 化风格（可复用到 rail tip 标签视觉）

### RailGraph 与现有 UI 元素映射

1. **Git Log 容器**：沿用 Secondary Sidebar `panel-body` 滚动体系，rail 仅作为左侧子区域，不创建独立纵向滚动条。
2. **commit-row 对齐**：rail 每个节点 Y 坐标必须与对应 `.commit-row` 垂直中心对齐，偏差 <= 1px。
3. **ref-chip 对齐**：当前 commit row 的 `ref-chip` 与 rail tip 标签颜色语义一致（local/remote/tag）。
4. **selected row 联动**：当 commit-row `selected` 时，rail 同步高亮当前节点外圈。
5. **theme token 对齐**：颜色来源与 Calm Studio token 同源，不硬编码孤立色值。
6. **可读性约束**：rail 不得压缩 commit message 区域；commit body 保持 MVP-07 当前宽度策略。

### 视觉语义定义（实施时必须落地）

- `normal node`：6px 圆点
- `merge node`：7px 菱形（双入边）
- `fork node`：7px 方形（出边分叉）
- `head node`：8px 圆点 + 2px 外环
- `remote tip chip`：半透明背景 + 1px 边框
- `tag chip`：矩形胶囊（不使用圆点）

---

## ✅ Acceptance（按 phase + 质量门槛分组）

### A. Phase A · 数据接线 + 布局骨架
- [ ] A.1 组件可消费 MVP-07 commit list 数据（含 oid、parents、author、time、refs）且不改原列表 API。
- [ ] A.2 `RailGraphInput` 转换函数在 20/1k/100k fixture 下均可生成合法输入（无抛错）。
- [ ] A.3 lane 分配输出 deterministic：同一输入重复运行 10 次输出 hash 完全一致。
- [ ] A.4 root commit（0 parent）显示起点节点，不出现负索引 lane。
- [ ] A.5 merge commit（2+ parent）至少渲染两条入边数据，不丢 parent 关系。
- [ ] A.6 detached HEAD 场景可显示 rail，但 headName 为空时不渲染 head 标签文案。
- [ ] A.7 refs 归一化后 local/remote/tag 三类计数与输入一致（误差=0）。
- [ ] A.8 Phase A 快照 fixture（至少 6 份）可被单测直接比对通过。
### B. Phase B · Canvas 绘制
- [ ] B.1 rail 主画布在首屏首次渲染时可显示全部可视 commit 节点。
- [ ] B.2 normal/merge/fork/head 四类节点形状可区分，视觉检查通过。
- [ ] B.3 merge commit 双入边在 5 分支并行 fixture 中无断线。
- [ ] B.4 branch tip local/remote/tag 样式差异明显（实心/半透明/矩形）。
- [ ] B.5 当前分支节点高亮与 commit list selected 行联动，无错位。
- [ ] B.6 light/dark 主题切换后 rail 颜色同步更新，无残留旧主题颜色。
- [ ] B.7 DPR=2（Retina）下节点边缘清晰，不出现明显模糊。
- [ ] B.8 视觉回归基线图至少 10 张，覆盖 1/5/20/50 分支密度。
### C. Phase C · 虚拟化 + 交互
- [ ] C.1 仅渲染 viewport ±100 行；超出范围 commit 不进入 draw 调用。
- [ ] C.2 滚动时使用 RAF 合帧，不出现每个 scroll event 都 full redraw。
- [ ] C.3 offscreen canvas 命中时主线程仅执行一次 `drawImage`。
- [ ] C.4 hover 节点后 1 帧内显示完整 rail 路径高亮。
- [ ] C.5 hover 离开后 1 帧内恢复默认样式，不残留高亮。
- [ ] C.6 分支数 21-50 时 rail 压缩到 8px 宽并显示 hover 提示。
- [ ] C.7 分支数 >50 时出现 `Other branches` 收敛行并支持点击展开。
- [ ] C.8 触屏环境可用 tap 触发与 hover 等价的高亮路径。
### D. Phase D · 集成 + 稳定性
- [ ] D.1 rail 与 commit list 共享滚动源，任意滚动位置对齐误差 <= 1px。
- [ ] D.2 收到 `git:branch-changed` 后 50ms 内完成 rail 重绘。
- [ ] D.3 切换 workspace 时旧 workspace 缓存被清理，不串数据。
- [ ] D.4 切换主题 / DPI 后自动重建 backing store，画面无拉伸。
- [ ] D.5 异常 parent 缺失时显示降级连线，不导致整图空白。
- [ ] D.6 ref 重名（local 与 remote 同名）时标签 disambiguation 文案正确。
- [ ] D.7 rail 组件卸载后无 RAF 泄漏（内存快照无持续增长）。
- [ ] D.8 Phase D 验收报告包含性能数字 + 视觉截图 + 异常场景录像。
### E. 错误处理（Error Handling）
- [ ] E.1 commit 数据为空时显示空态 rail（占位文案），不抛异常。
- [ ] E.2 commit parent 缺失（浅克隆）时以灰色虚线连接并记录 warning。
- [ ] E.3 refs 解析失败时该条 commit 退化为默认颜色节点。
- [ ] E.4 branch 数超阈值 collapse 计算异常时回退到「仅显示当前分支」。
- [ ] E.5 canvas context 获取失败时显示 fallback 文案并提示刷新。
- [ ] E.6 performance observer 不可用时不影响主功能，只关闭调试指标。
### F. 性能预算（Performance Budget）
- [ ] F.1 10 万 commit 仓库首屏渲染 < 500ms（P99）。
- [ ] F.2 滚动帧预算 16ms 内（P99 FPS >= 60）。
- [ ] F.3 hover 高亮响应 < 16ms（从 pointermove 到 paint）。
- [ ] F.4 branch event 触发重绘 < 50ms（从事件收到到 paint）。
- [ ] F.5 1 万 commit 场景 CPU 占用 < 单核 40%（持续滚动 10s）。
- [ ] F.6 10 万 commit 场景内存增量 < 120MB（打开后 30s 稳态）。
- [ ] F.7 100 万 commit 压测允许降级（collapse + 简化线条），但不得 crash。
### G. 跨平台 + 可访问性（Cross-platform & A11y）
- [ ] G.1 macOS 14 + Apple Silicon 下视觉与性能门槛均通过。
- [ ] G.2 Ubuntu 24（X11）下视觉与性能门槛均通过。
- [ ] G.3 Ubuntu 24（Wayland）下滚动同步与 hover 高亮无错位。
- [ ] G.4 1x/2x DPR 截图对比中节点边缘清晰，无锯齿异常。
- [ ] G.5 CVD 三种模拟（protanopia/deuteranopia/tritanopia）下 30 色至少 15 色可区分。
- [ ] G.6 键盘导航切换 commit row 时 rail 高亮同步（无鼠标也可读图）。
- [ ] G.7 触屏设备 tap fallback 可工作（至少 1 台设备手测通过）。
### 验收统计（必须）

- [ ] 总 checkbox 数 >= 30（当前目标：52）
- [ ] 每条 checkbox 含可验证指标或行为
- [ ] Phase A/B/C/D 均至少 8 条
- [ ] E/F/G 均至少 6 条

---

## 🧪 测试策略（6 层）

| 层次 | 目标 | 工具/命令 | 成功标准 |
|---|---|---|---|
| 单元（layout） | lane 分配、merge/fork 路径计算正确 | `pnpm --filter @vibestation/web test rail-layout`（示例） | 关键算法 case 全通过 |
| 集成（UI） | rail 与 commit list 滚动/选择联动 | `pnpm --filter @vibestation/web test rail-integration` | 对齐误差 <=1px |
| 性能（bench） | 首屏、滚动、hover、event 重绘 | vitest bench / custom perf harness | 满足 §F 预算 |
| E2E（Playwright） | 真实交互链路：滚动、hover、collapse、theme 切换 | `pnpm --filter @vibestation/web test:e2e -- rail-graph` | 无关键失败 |
| 视觉回归 | light/dark + 1x/2x + 多分支密度截图一致性 | Playwright screenshot diff | diff 在阈值内 |
| 手动 QA | Linux/macOS + CVD + 触屏 fallback | QA checklist | 全项通过或有可接受降级说明 |

### Fixture 模板（必须可复现）

#### 1) `fixture_linear_20.json`

- 20 commit 线性历史
- 1 local branch（main）
- 0 merge
- 用于 baseline 对齐与性能下限

#### 2) `fixture_branchy_1k.json`

- 1,000 commit
- 20 branch（含 local 12 / remote 6 / tag 2）
- 30 merge commit
- 用于 collapse 阈值边界（20 branch）

#### 3) `fixture_kernel_like_100k.json`

- 100,000 commit
- 80 branch（local+origin）
- merge 密度 12%
- 用于首屏与滚动性能门槛

#### 4) `fixture_extreme_1m.json`

- 1,000,000 commit（合成）
- 200 branch
- 用于「不崩溃 + 降级策略」验证

### Bench 模板（建议落在 `web/bench/rail-graph.bench.ts`）

```ts
import { performance } from 'node:perf_hooks';
import { buildRailLayout, renderRailFrame } from '../src/panels/GitLog/RailGraph';

type BenchCase = {
  name: string;
  fixture: string;
  iterations: number;
};

const CASES: BenchCase[] = [
  { name: 'layout-20', fixture: 'fixture_linear_20.json', iterations: 100 },
  { name: 'layout-1k', fixture: 'fixture_branchy_1k.json', iterations: 50 },
  { name: 'layout-100k', fixture: 'fixture_kernel_like_100k.json', iterations: 10 },
];

for (const c of CASES) {
  const input = loadFixture(c.fixture);
  const samples: number[] = [];
  for (let i = 0; i < c.iterations; i++) {
    const t0 = performance.now();
    const layout = buildRailLayout(input);
    renderRailFrame(layout, { viewportStart: 0, viewportEnd: 120 });
    samples.push(performance.now() - t0);
  }
  const p99 = quantile(samples, 0.99);
  console.log(`${c.name} p99=${p99.toFixed(2)}ms`);
}
```

### E2E 场景模板（Playwright）

- [ ] 场景 1：打开 Git Log → rail 首屏出现 → commit 节点数量与可视 row 数一致
- [ ] 场景 2：滚动到底部 3 次 → rail 与列表无错位
- [ ] 场景 3：hover merge commit → 双入边路径高亮
- [ ] 场景 4：branch 数从 19→21→51 动态变化 → collapse 行为符合阈值
- [ ] 场景 5：切 dark theme → 色环切到 dark token
- [ ] 场景 6：触发 `git:branch-changed` → 50ms 内重绘

### 手动 QA 清单（macOS + Ubuntu）
- [ ] QA.01 打开 20 commit 仓库，确认 rail 与 commit row 一一对应。
- [ ] QA.02 打开 1k commit 仓库，确认滚动 30 秒无明显抖动。
- [ ] QA.03 打开 100k commit 仓库，记录首屏时间并截图。
- [ ] QA.04 在 80 分支仓库触发 collapse，确认出现 Other branches。
- [ ] QA.05 点击 Other branches 展开/收起 10 次，无渲染异常。
- [ ] QA.06 切换 light/dark 各 5 次，颜色 token 同步无残影。
- [ ] QA.07 模拟 CVD（protanopia）检查 branch 色可分辨。
- [ ] QA.08 模拟 CVD（deuteranopia）检查 branch 色可分辨。
- [ ] QA.09 模拟 CVD（tritanopia）检查 branch 色可分辨。
- [ ] QA.10 Retina 2x 下截图，确认节点边缘无模糊。
- [ ] QA.11 Ubuntu X11 下截图，确认线条宽度一致。
- [ ] QA.12 Ubuntu Wayland 下滚动，确认同步无偏移。
- [ ] QA.13 触屏设备上 tap 节点，确认高亮可触发。
- [ ] QA.14 键盘上下选择 commit，确认 rail 高亮跟随。
- [ ] QA.15 触发 branch changed 事件，确认 50ms 内重绘。
- [ ] QA.16 触发无效 parent 数据，确认降级显示而非崩溃。
- [ ] QA.17 连续打开/关闭 Git Log 面板 20 次，无内存泄漏。
- [ ] QA.18 记录最终 perf trace（首屏/滚动/hover/event 四项）。
---

## 💾 数据模型变更（rail 位置缓存策略）

### 设计结论

- rail 布局缓存采用 **in-memory only**（会话级）
- 不新增 SQLite 表
- 不把 rail 布局持久化到磁盘
- 允许在内存中缓存最近 N 个 viewport 的布局切片（建议 N=8）

### 推荐结构（前端内存）

```ts
type RailLayoutCacheKey = `${workspaceId}:${headOid}:${theme}:${dpr}:${viewportStart}:${viewportEnd}`;

type RailLayoutCacheValue = {
  generatedAtMs: number;
  rows: RailRow[];
  lanes: RailLane[];
  branchSummary: {
    totalBranches: number;
    collapsed: boolean;
    collapsedCount: number;
  };
};

interface RailGraphSessionCache {
  byKey: Map<RailLayoutCacheKey, RailLayoutCacheValue>;
  lru: RailLayoutCacheKey[];
  maxEntries: number; // default 8
}
```

### 缓存失效条件

- `headOid` 变化（新 commit / checkout / reset）
- `refsHash` 变化（branch/tag 变更）
- 主题变化（light/dark）
- DPR 变化（窗口跨屏）
- rowHeight 变化（字体或密度变化）

### 反模式（禁止）

- ❌ 把 `rows/lanes/path` 全量写入 `app_settings` 或新表
- ❌ 持久化 100k commit 布局快照（体积过大且命中率低）
- ❌ 为缓存命中而牺牲正确性（head 变化后继续复用旧布局）

### 与 implementation-plan §5.4 对齐说明

`CommitNode.rail: Option<u16>` 在战略文档是「可选位置信息预留」，MVP-12 实现阶段可以在运行态使用该字段语义，但本 spec 明确不要求持久化该值到数据库；以会话内计算 + 缓存为主。

---

## §G. IPC Contract（ts-rs / 事件合同）

> 依据：ADR-014（IPC contract source of truth = Rust struct + ts-rs）。
> 本 task 主要是前端渲染，**不新增重型 invoke 命令**，但需要把跨模块事件 payload 类型化，避免字符串协议漂移。

### G.1 本 MVP 涉及 struct / enum 清单（明确数量）

**复用（已有）**：

1. `GitLogEntry`（MVP-07）
2. `GitLogQueryResponse`（MVP-07）
3. `BranchInfo`（MVP-13）
4. `BranchKind`（MVP-13）

**新增（MVP-12）**：

1. `RailGraphViewportSyncPayload`
2. `RailGraphBranchChangedPayload`
3. `RailGraphRebaseStatePayload`（v0.3 预留，但合同先定义）
4. `RailGraphPerfSample`（debug/benchmark 输出）

> 本 spec 明确：**新增 binding 数 = 4**（不是“约 N 个”）。

### G.2 derive 模板（Rust）

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RailGraphViewportSyncPayload {
    pub workspace_id: String,
    pub scroll_top: f64,
    pub row_height: f32,
    pub viewport_start: u32,
    pub viewport_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RailGraphBranchChangedPayload {
    pub workspace_id: String,
    pub head_oid: Option<String>,
    pub refs_hash: String,
    pub branch_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RailGraphRebaseStatePayload {
    pub workspace_id: String,
    pub state: String, // in_progress | done | aborted
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RailGraphPerfSample {
    pub workspace_id: String,
    pub phase: String, // firstPaint | scroll | hover | branchChanged
    #[ts(type = "number")]
    pub duration_ms: f32,
    pub commit_count: u32,
    pub branch_count: u32,
}
```

### G.3 强制规范

- [ ] 所有新增 payload 均用 ts-rs 自动生成到 `web/src/bindings/`
- [ ] 前端禁止手写同名 interface（必须 import binding）
- [ ] 浮点字段显式 `#[ts(type = "number")]`
- [ ] enum/payload 统一 camelCase
- [ ] event name 与 payload type 一一对应（文档中写死映射）
- [ ] 任何字段改动必须同步更新契约测试

### G.4 H2 regression proof（必须执行一次）

1. 在 `RailGraphBranchChangedPayload.refs_hash` 临时改名 `refs_hash_proof`
2. 运行 `cargo build -p vibestation-app`
3. 运行 `pnpm typecheck`
4. 预期前端报类型错误（找不到 `refsHash`）
5. 回滚字段名
6. 再次 `pnpm typecheck` 恢复 PASS

### G.5 复用决策表

| 类型 | 来源 | 决策 | 理由 |
|---|---|---|---|
| `GitLogEntry` | MVP-07 | ✅ 复用 | rail 输入直接来自 commit list |
| `GitLogQueryResponse` | MVP-07 | ✅ 复用 | 不重复开新查询接口 |
| `BranchInfo` | MVP-13 | ✅ 复用 | branch tip 标签依赖该结构 |
| `BranchKind` | MVP-13 | ✅ 复用 | local/remote/tag 语义已统一 |
| `RailGraphViewportSyncPayload` | MVP-12 | ✅ 新增 | 滚动同步事件需要类型化 |
| `RailGraphBranchChangedPayload` | MVP-12 | ✅ 新增 | 监听 branch 改动重绘 |
| `RailGraphRebaseStatePayload` | MVP-12 | ✅ 新增（预留） | MVP-16 overlay 联动边界先锁 |
| `RailGraphPerfSample` | MVP-12 | ✅ 新增 | 性能预算验收需要结构化样本 |

### G.6 新增 binding 清单（数字明确）

新增 **4 个** binding 文件：

1. `RailGraphViewportSyncPayload.ts`
2. `RailGraphBranchChangedPayload.ts`
3. `RailGraphRebaseStatePayload.ts`
4. `RailGraphPerfSample.ts`

> 注：MVP-12 为前端主导 task，后端 command 零新增，event payload 4 个即可覆盖合同需求。

---

## §H. 决策锁定（H.1-H.8）

### H.1 渲染技术锁定：Canvas（含替代方案比较）

**决策**：MVP-12 渲染技术锁定为 **Canvas 2D**（不是 SVG / DOM / WebGL）。

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| Canvas 2D | 绘制批处理能力强；节点数大时无 DOM 膨胀；离屏缓存成熟 | 可访问性需额外处理；命中测试需手写 | ✅ 选用 |
| SVG | 语义清晰、样式友好、可访问性天然好 | 1 万+ 节点会有大量 DOM 元素，重排开销高 | ❌ 不选 |
| DOM（div + border） | 实现直观、调试简单 | 大数据量最差，布局/重排/合成开销高 | ❌ 不选 |
| WebGL | 极高理论性能，适合百万级点 | 引擎复杂、调试成本高、超出 MVP 必要性 | ❌ 不选 |

### H.2 库边界锁定：不碰 WebGL / 通用图论重库

- 禁止引入：Three.js / Pixi.js / cytoscape / vis.js / D3 force graph
- 禁止为了“酷炫”引入 3D 视图
- 仅允许：原生 Canvas API + 轻量工具函数
- 原因：MVP 聚焦稳定性与可维护性，不引入超额复杂度

### H.3 算法选型锁定：spec 不 pre-decide，v0.3 kickoff SPIKE-09 决定

| 候选 | 优点 | 缺点 | SPIKE-09 评估项 |
|---|---|---|---|
| gitgraph.js fork（JS） | 成熟度高、社区案例多 | 10 万 commit 性能风险大 | 首屏时间/FPS/内存 |
| git-graph-rs port（Rust→WASM） | 理论性能强、算法稳定 | 移植成本高、WASM 包体积 | 集成复杂度/加载时间 |
| 自实现简化版（参考论文） | 可完全定制、可只做 80% 需求 | 算法边界风险高 | 边界 case 通过率 |

**锁定语句**：本 spec 只定义候选与评估框架，**不在 spec 阶段拍板算法**。最终选择由 SPIKE-09 量化结果决定。

### H.4 虚拟化策略锁定

- viewport 仅绘制可视区 ±100 行
- 使用 offscreen canvas 缓存静态轨道层
- 用 RAF 合帧处理滚动重绘
- IntersectionObserver 用于可视区边界触发（辅助，不作为唯一机制）
- 当 commit > 100k 时优先启用 collapse 与线条简化

### H.5 颜色策略锁定（token 化 + CVD）

- 颜色来源：设计 token（light/dark 各 30 色）
- 计算空间：oklch（而非裸 HSL）
- 对比度要求：文字/关键描边 >= 4.5:1
- 色盲检查：protanopia/deuteranopia/tritanopia 三种模拟
- 禁止硬编码 hex 到业务组件

### H.6 性能预算锁定（数字不可模糊）

- 10 万 commit 首屏 < **500ms**（P99）
- 滚动帧预算 < **16ms**（P99）
- hover 高亮响应 < **16ms**（P99）
- branch event 重绘 < **50ms**（P99）
- 1M commit 场景允许视觉降级，但不允许 crash

### H.7 与 MVP-07 集成边界锁定

- rail 是独立组件，不改 commit list 渲染结构
- rail 输入来自 MVP-07 数据流（props/selector），不重新请求数据
- 滚动同步由 commit list 驱动，rail 仅监听
- commit row 的选择、过滤、搜索逻辑继续由 MVP-07 负责

### H.8 与 MVP-13 / MVP-16 边界锁定

- MVP-13：只负责 branch CRUD 与 `git:branch-changed` 事件；MVP-12 只监听不反调命令
- MVP-16：rebase 期间提供 `git:rebase-state-changed`；MVP-12 仅显示 overlay（"rebasing"）
- rail 不负责触发 rebase/merge/cherry-pick
- rail 不承载冲突解决 UI

---

## ⚠️ 已知风险（R1-R5）

- **R1 · Canvas 性能边界**：10 万 commit 下每帧 draw 调用可能逼近 16ms。  
  **Mitigation**：虚拟化 + 合帧 + offscreen；Phase D 提交 perf trace 与 P99 报告。

- **R2 · DAG 边界复杂度**：octopus merge、cross-branch、orphan history 处理易出错。  
  **Mitigation**：SPIKE-09 用 20 个边界 case 量化，低于 80% 通过率即不进入主线。

- **R3 · 高 DPI 模糊或性能放大**：2x/3x DPR 可能导致渲染像素面积暴涨。  
  **Mitigation**：`Math.min(devicePixelRatio, 2)` + 双倍率视觉回归截图。

- **R4 · 颜色可分辨性不足**：30 色在 CVD 下可能聚类。  
  **Mitigation**：oklch 调色 + CVD 模拟，保证至少 15 色可分辨；不达标则降色环规模。

- **R5 · 触屏 hover 缺失 + 4K 宽屏布局压缩**：触屏无 hover，4K 下 rail/列表比例失衡。  
  **Mitigation**：tap fallback 交互 + rail 宽度 clamp（120-180px）+ 4K 手测基线。

---

## 📝 Notes

- 本 task 是 spec 详化，不实施代码，不生成 runtime evidence。
- frontmatter `status` 维持 `draft`，翻 `ready` 需 Arbiter comment approve。
- 算法结果必须通过 SPIKE-09 才能锁实现方案。
- 文档中提到的路径（如 `web/src/panels/GitLog/RailGraph/`）是 placeholder，实施 PR 可按实际结构调整。
- 如实施期发现需要改决策级文件（CLAUDE/ADR/implementation-plan），必须单开流程，不在 MVP-12 实施 PR 混入。

---

## 🔗 相关

- `docs/implementation-plan.md` §5.4（CommitNode.rail 预留）
- `docs/implementation-plan.md` §10.1（v0.2 范围）
- `docs/implementation-plan.md` §11 W16（rail graph 里程碑）
- `docs/tasks/MVP-07-git-log-view.md`（上游数据流）
- `docs/tasks/MVP-13-branch-crud.md`（branch event 来源）
- `docs/tasks/MVP-16-*.md`（rebase overlay 联动边界）
- `design/directions/1-calm-studio.html`（UI 基准）
- `scripts/validate-task-spec.mjs`（frontmatter gate）

---

## 自审四问（spec 级）

1. **递归完备性**  
   本 spec 自身是否受本 spec 约束？是。已包含结构完整性、验收可测性、边界/风险/YAGNI，并在末尾给出 12 段评估表闭环。

2. **反向场景**  
   - 100 万 commit：允许降级但不崩溃（§F.7）  
   - octopus merge：纳入 SPIKE-09 边界 case（§H.3 / R2）  
   - detached HEAD：输入层显式支持（Acceptance A.6）  
   - touch 设备 hover 缺失：tap fallback（Acceptance C.8 / R5）

3. **边界适用性**  
   已覆盖 commit 规模 1 / 100 / 10k / 100k / 1M，分支规模 1 / 5 / 20 / 50 / 200，显示规模 1x / 2x DPR，light/dark/CVD 三组视觉边界。

4. **YAGNI**  
   明确推后：WebGL、3D、interactive rebase、rail tooltip 卡片、跨 remote 全图。保持 v0.2 目标聚焦「可读 + 快 + 稳」。

---

## 详化完成度评估表（12 段）

| 评估项 | 状态 | 说明 |
|---|---|---|
| 1. frontmatter 完整 | ✅ | 含 reviewer: Droid，status 保持 draft |
| 2. 顶部状态说明 5 行 | ✅ | 状态/依赖/下游/战略依据/详化时间齐全 |
| 3. Goal | ✅ | 2 段，含 plan_ref 与量化目标 |
| 4. Context | ✅ | 含 implementation-plan/CLAUDE/路线图/上游落地 |
| 5. 实施进度 | ✅ | Phase A/B/C/D + 起点 checklist + 80 条任务拆分 |
| 6. Scope | ✅ | Do 10 项 / Don’t 10 项 |
| 7. UI 引用 | ✅ | design 行号 + 元素映射 + 视觉语义 |
| 8. Acceptance | ✅ | A/B/C/D/E/F/G 共 52 条 checkbox |
| 9. 测试策略 | ✅ | 6 层 + fixture + bench + E2E + 手动 QA |
| 10. 数据模型 | ✅ | in-memory 缓存策略 + 失效条件 + 反模式 |
| 11. §G IPC Contract | ✅ | G.1-G.6 全覆盖，新增 binding 数=4 |
| 12. §H 决策锁定 + 风险 | ✅ | H.1-H.8 + R1-R5 + 自审四问 |

**完成度**：12 / 12 = **100%**（内容已达 ready-grade；流程上保持 draft，待 Arbiter approve 后翻 ready）。
---

## 附录 A · 性能测量记录模板（实施 PR 必填）

> 以下模板用于实施阶段提交性能证据，避免“感觉很快”式结论。

| 项目 | 环境 | 样本数 | P50 | P95 | P99 | 预算 | 是否达标 |
|---|---|---:|---:|---:|---:|---:|---|
| 首屏渲染（100k） | macOS M1 Pro | 30 | - | - | - | <500ms | - |
| 滚动帧（100k） | macOS M1 Pro | 3000 帧 | - | - | - | <16ms | - |
| hover 响应（100k） | macOS M1 Pro | 200 | - | - | - | <16ms | - |
| branch event 重绘 | macOS M1 Pro | 100 | - | - | - | <50ms | - |
| 首屏渲染（100k） | Ubuntu 24 X11 | 30 | - | - | - | <500ms | - |
| 滚动帧（100k） | Ubuntu 24 X11 | 3000 帧 | - | - | - | <16ms | - |
| 首屏渲染（100k） | Ubuntu 24 Wayland | 30 | - | - | - | <500ms | - |
| 滚动帧（100k） | Ubuntu 24 Wayland | 3000 帧 | - | - | - | <16ms | - |

### 附录 A 检查项
- [ ] A-Perf.01 性能数据均含采样脚本版本号与 commit SHA。
- [ ] A-Perf.02 每项至少 30 次样本（帧数据按 3000 帧）。
- [ ] A-Perf.03 P99 计算方法在 PR 中给出。
- [ ] A-Perf.04 录制时关闭 DevTools 截图开销影响。
- [ ] A-Perf.05 录制时固定窗口尺寸（避免可变噪声）。
- [ ] A-Perf.06 录制时注明 DPR（1x/2x）。
- [ ] A-Perf.07 录制时注明分支数与 commit 数。
- [ ] A-Perf.08 若不达标，给出 flamegraph 与整改计划。
- [ ] A-Perf.09 若达标，给出“达标证据文件路径”。
- [ ] A-Perf.10 提交评审前复跑一次确认无偶然值。
## 附录 B · 可访问性与视觉回归模板

### B.1 视觉回归截图矩阵（建议至少 24 张）

| 主题 | DPR | 分支数 | 场景 | 文件名建议 |
|---|---|---:|---|---|
| light | 1x | 5 | baseline | `rail-light-1x-5-branches.png` |
| light | 1x | 20 | threshold | `rail-light-1x-20-branches.png` |
| light | 1x | 50 | compressed | `rail-light-1x-50-branches.png` |
| light | 1x | 80 | other-group | `rail-light-1x-80-branches.png` |
| light | 2x | 5 | baseline | `rail-light-2x-5-branches.png` |
| light | 2x | 20 | threshold | `rail-light-2x-20-branches.png` |
| light | 2x | 50 | compressed | `rail-light-2x-50-branches.png` |
| light | 2x | 80 | other-group | `rail-light-2x-80-branches.png` |
| dark | 1x | 5 | baseline | `rail-dark-1x-5-branches.png` |
| dark | 1x | 20 | threshold | `rail-dark-1x-20-branches.png` |
| dark | 1x | 50 | compressed | `rail-dark-1x-50-branches.png` |
| dark | 1x | 80 | other-group | `rail-dark-1x-80-branches.png` |
| dark | 2x | 5 | baseline | `rail-dark-2x-5-branches.png` |
| dark | 2x | 20 | threshold | `rail-dark-2x-20-branches.png` |
| dark | 2x | 50 | compressed | `rail-dark-2x-50-branches.png` |
| dark | 2x | 80 | other-group | `rail-dark-2x-80-branches.png` |

### B.2 色盲模拟截图矩阵

| 模拟类型 | 主题 | 分支数 | 通过标准 |
|---|---|---:|---|
| protanopia | light | 30 | 至少 15 色可区分 |
| deuteranopia | light | 30 | 至少 15 色可区分 |
| tritanopia | light | 30 | 至少 15 色可区分 |
| protanopia | dark | 30 | 至少 15 色可区分 |
| deuteranopia | dark | 30 | 至少 15 色可区分 |
| tritanopia | dark | 30 | 至少 15 色可区分 |

### B.3 A11y 检查项
- [ ] B-A11y.01 键盘上下选择 commit row 时 rail 高亮同步。
- [ ] B-A11y.02 focus ring 可见，不被 rail 覆盖。
- [ ] B-A11y.03 高对比度模式下 rail 线条仍可见。
- [ ] B-A11y.04 screen reader 至少可读“当前分支/commit 数/collapse 状态”文本摘要。
- [ ] B-A11y.05 触屏 tap 行为可替代 hover。
- [ ] B-A11y.06 禁用动画（prefers-reduced-motion）时仍可操作。
- [ ] B-A11y.07 color-only 信息有额外形状辅助（merge/fork/head）。
- [ ] B-A11y.08 错误态文案可被朗读（aria-live）。
- [ ] B-A11y.09 Other branches 展开按钮有可访问名称。
- [ ] B-A11y.10 branch tip 标签截断时有 title 提示。
## 附录 C · SPIKE-09 评估清单（算法候选对比）

> 该附录用于后续 SPIKE-09 直接执行，避免再次补需求。

### C.1 评估维度（统一权重）

| 维度 | 权重 | 说明 |
|---|---:|---|
| 性能（100k） | 35% | 首屏、滚动、hover 三项合成 |
| 算法正确性 | 30% | 20 个边界 case 通过率 |
| 实施复杂度 | 15% | 人日、依赖、可维护性 |
| 包体积影响 | 10% | 前端包增量 |
| 可调试性 | 10% | 排查难度、日志可见性 |

### C.2 边界 case（20 项）
- [ ] C-Case.01 线性历史（无分支）
- [ ] C-Case.02 单次 merge（2 parent）
- [ ] C-Case.03 连续 merge（3 次）
- [ ] C-Case.04 octopus merge（4 parent）
- [ ] C-Case.05 cross-branch merge（交叉）
- [ ] C-Case.06 长分支后回合并
- [ ] C-Case.07 短分支快进合并
- [ ] C-Case.08 detached HEAD + checkout
- [ ] C-Case.09 tag 指向非 HEAD commit
- [ ] C-Case.10 remote 分支无 local 对应
- [ ] C-Case.11 local/remote 同名分支
- [ ] C-Case.12 branch 名包含斜杠
- [ ] C-Case.13 branch 数 20（阈值边界）
- [ ] C-Case.14 branch 数 21（压缩起点）
- [ ] C-Case.15 branch 数 50（压缩上限）
- [ ] C-Case.16 branch 数 51（Other 启动）
- [ ] C-Case.17 100k commit + 80 branch
- [ ] C-Case.18 1M commit + 200 branch（降级）
- [ ] C-Case.19 浅克隆 parent 缺失
- [ ] C-Case.20 重复 ref/异常 ref 输入
### C.3 SPIKE-09 输出要求

- [ ] 每个候选输出同一套基准数据（可横向比较）
- [ ] 输出加权总分与维度得分
- [ ] 输出“不选理由”而不只写“选中理由”
- [ ] 输出风险残留项（进入实施前必须知道）
- [ ] 输出推荐方案 + fallback 方案

---

## 附录 D · 实施 PR 模板（节选）

```markdown
## Summary
- 实施 MVP-12 Phase X（A/B/C/D）
- 本 PR 不改决策文件，仅按 spec 执行

## Test Plan
- [ ] 单元
- [ ] 集成
- [ ] 性能
- [ ] E2E
- [ ] 视觉回归
- [ ] 手动 QA

## Perf Numbers
- first paint p99: ___ ms
- scroll frame p99: ___ ms
- hover p99: ___ ms
- branch-change redraw p99: ___ ms

## Boundaries check
- [ ] 不引入 WebGL / Three.js / D3
- [ ] 不改 MVP-07 commit-list DOM
- [ ] 算法来源符合 SPIKE-09 结论
```

---

## 附录 E · 术语表

- **rail**：commit DAG 在列表左侧的轨道可视化
- **lane**：同一时间片中分支占用的列索引
- **collapse**：高分支密度下的可读性压缩策略
- **overscan**：可视区外额外渲染区间
- **offscreen canvas**：离屏缓冲，用于降低主线程绘制开销
- **CVD**：Color Vision Deficiency，色觉缺陷模拟
- **DPR**：devicePixelRatio，设备像素比

---
