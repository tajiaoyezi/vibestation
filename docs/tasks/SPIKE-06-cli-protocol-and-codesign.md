---
id: SPIKE-06
type: spike
title: Claude CLI / Codex CLI 实机 + macOS Developer Program 申请
status: draft
owner:
phase: W0-D6
depends_on: ["SPIKE-05"]
blocks: []
estimate: 1d
plan_ref: implementation-plan.md §附录 A D6 · §9 R1
risk_ref: R1
reviewer:
---

# SPIKE-06: Claude CLI / Codex CLI 实机 + macOS Dev Program

> **状态**：`draft`
> **依赖**：SPIKE-05（PTY 架构已验证）
> **战略依据**：[`implementation-plan.md §附录 A D6`](../implementation-plan.md) · [`§9 R1`](../implementation-plan.md)

---

## 🎯 目标（Goal）

（1）在 SPIKE-05 验证的 PTY 里**实机运行 Claude CLI + Codex CLI**，录制输出样本，初探协议差异——消除 R1 风险。
（2）**提交 Apple Developer Program 申请**，为 v0.1 发布 macOS 公证准备（审核周期 2 天-2 周）。

## 📖 背景（Context）

- `CLAUDE.md` "⚠️ Claude CLI / Codex CLI 输出协议 Spike Day 5 前未经实机验证" —— 本 Spike 消除这条警告
- **R1**（HIGH / HIGH）：CLI 输出协议与猜测不同，解析失败 → 核心功能块
- **Apple Developer Program 是 macOS 分发硬门槛**（未公证的 dmg 用户需绕过 Gatekeeper），审核不可控，必须 W0 提交
- AI-Aware Pane 联动是 v1.0 vision（对外不提），但 CLI 输出协议验证 MVP 就需要——MVP 只做"多 Tab 里跑 CLI"，不做联动

---

## ✅ 通过标准（Pass Criteria）

> ⚠️ **Codex PR #3 Round 1 Finding 3 教训**：本 Spike 只能得出"CLI 能在 PTY 里运行"的结论，**不能**得出"协议足够清楚可指导实现"的结论——后者需要独立的 parser-oriented spike（SPIKE-07，v1.0 前做）。**R1 警告不得下调**。

本 Spike 结论严格拆分为两个独立判据：

### 结论 A · "CLI 可在 PTY 运行"（本 Spike 验证 · 可消除"进程级"阻塞风险）

### 结论 B · "协议足够清楚可指导实现"（**本 Spike 不验证** · R1 保留）

---

### A.1 · CLI 实机测试（主线）

- [ ] **Claude CLI 在 PTY 里正常运行**：
  - [ ] 启动 + 登录流程可完成
  - [ ] 简单对话（1 轮）输出完整显示，无乱码
  - [ ] 流式输出不卡顿（对应 SPIKE-05 验证的 xterm 渲染能力）
  - [ ] `Ctrl+C` 能正确中断
- [ ] **Codex CLI 在 PTY 里正常运行**（相同判据）
- [ ] **macOS PATH 空问题验证**：Tauri 启动的子进程能读到用户 `$PATH`（`fix-path-env` crate 或等价方案）

### A.2 · 输出样本录制（失败路径覆盖 · Codex 加入）

> Round 1 教训：仅录启动/对话/错误 3 个 happy path 样本不够。必须覆盖会真正打爆实现的失败路径。

**每个 CLI 必须录制以下样本**（Claude 和 Codex 各一套）：

- [ ] Happy path：启动 / 简单对话 / 普通错误
- [ ] **中断后的残帧**：对话进行中 `Ctrl+C`，录剩余半帧输出
- [ ] **认证失败**：故意用错误 token，录 auth fail 输出
- [ ] **网络错误**：断网场景（拔线 / 防火墙拦截），录 network error 输出
- [ ] **长流式输出**：让 CLI 生成 10k+ token 响应，录流式完整片段
- [ ] **混合 ANSI / 结构化**：颜色 ANSI + JSON 嵌入同一输出（测解析器要应付的真实情况）

每种场景录制**至少 3 次**（覆盖平台差异：mac/linux），**合计样本 ≥ 36 条**（2 CLI × 6 场景 × 3 次）。

### A.3 · 结构观察报告（描述性 · 不作为协议结论）

- [ ] Claude CLI 输出结构描述（stream / block / JSON-line / ANSI）
- [ ] Codex CLI 输出结构描述
- [ ] 关键差异点粗略描述（token 分隔、role 标识、error format）
- [ ] 样本归档到 `docs/spikes/SPIKE-06-report.md` + `spike-artifacts/SPIKE-06/` 供 SPIKE-07 使用

> ⚠️ **禁止在本报告里写"协议已消除 R1"类表述**。最多只能写"已录制样本，协议特征化待 SPIKE-07"。

### A.4 · 结果归档

- [ ] 样本全部写入 **`docs/spikes/SPIKE-06-report.md`**（per-task；Phase 3 建立 `docs/spikes/` 目录）
- [ ] 录屏 / 样本文件归档到 `docs/spike-artifacts/SPIKE-06/`（Phase 3 建立）

### B. Apple Developer Program（副线）

- [ ] 已登录 Apple Developer 账号
- [ ] 已提交 Apple Developer Program 申请（$99/年）
- [ ] 保留申请提交日期与预计审核完成日期
- [ ] **不阻塞其他 Spike** —— 审核期间可继续 MVP 开发，签证前不发布 macOS 公测

### C. 新开 SPIKE-07 · Parser-Oriented Spike（Codex 加入）

本 Spike **不做**协议解析验证。解锁条件：

- [ ] 新建 `SPIKE-07-cli-protocol-parser.md` task spec（status: draft），作为 SPIKE-06 的下游，指向 v1.0 AI-Aware 前执行
- [ ] SPIKE-07 Acceptance 必须包含：
  - [ ] 基于 SPIKE-06 录制的 36+ 样本做可回放 fixture
  - [ ] 实现原型 parser，对每个样本做解析断言
  - [ ] 覆盖中断/认证/网络/流式/混合 5 类失败路径的解析正确性
  - [ ] 输出结论："两 CLI 能否统一抽象"有可信答案

## ❌ 失败信号（Fail Signals）

CLI 主线（A）：

- Claude CLI / Codex CLI 在 PTY 里**无法启动**（缺 tty / 环境变量 / auth 路径问题）→ 调查阻塞点
- 输出乱码（ANSI 解析失败 / 编码问题）→ 调查 xterm 配置
- 样本覆盖不全（6 场景任一缺失或 < 3 次） → **不允许**声明 R1 已初探

macOS Dev Program（B）：

- Apple 拒绝申请 → 调查原因（通常是账号信息问题），升级为 Arbiter 仲裁

## 🔀 Fallback 方案

**CLI 通过（A.1 + A.2 + A.3）** → 消除"进程级阻塞"风险；**R1 保留**（待 SPIKE-07 解决）
**CLI 部分失败（仅一个能跑）** → MVP 优先支持能跑的那个，另一个推到 v0.2；R1 保留
**CLI 双失败** → 仅提供多 Tab 终端，不标榜 AI CLI 集成（README 措辞需修改）；R1 保留

**R1 不下调规则**（Codex 加入）：
- 本 Spike 不允许在任何产出文档里声明"R1 已消除"或"协议已清楚"
- `CLAUDE.md` 的"⚠️ Claude CLI / Codex CLI 输出协议 Spike Day 5 前未经实机验证"**保留为"未经深度协议验证"**（即使改措辞也不能摘除警告）
- R1 降级授权只能通过 SPIKE-07 的 ADR 完成

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-06-cli/`：CLI 启动脚本 + 样本录制脚本
- [ ] CLI 输出样本 × 6+（Claude 3 + Codex 3，脱敏后可 attach 到 spike-artifacts）
- [ ] **协议差异分析报告**（**`docs/spikes/SPIKE-06-report.md`**，per-task）
- [ ] macOS `fix-path-env` 验证代码片段
- [ ] Apple Dev Program 申请截图 + 预计完成日期
- [ ] 更新 `CLAUDE.md` 的 "⚠️ Claude CLI / Codex CLI 输出协议 Spike Day 5 前未经实机验证" 这条警告（移除或改为"Spike D6 已初探，v1.0 前需二次深度 spike"）

## 🛠 依赖资源（Resources Needed）

- SPIKE-05 产出的 PTY + xterm demo
- Claude CLI（已安装 + 已登录 Anthropic 账号）
- Codex CLI（已安装 + 已登录 OpenAI 账号）
- Apple Developer 账号（邮箱 + 付费能力 $99）
- macOS 开发机（验证 PATH 问题）

## ⚠️ 已知风险

- **R1**（`implementation-plan.md §9`，HIGH / HIGH）：CLI 输出协议未实机验证，本 Spike 消除初探；**v1.0 AI-Aware 前需二次深度 spike**（W23 前后，本 Spike 不覆盖）
- **macOS PATH 空问题**：已知 macOS GUI app 启动子进程不读 shell profile，`fix-path-env` 是常见方案
- **Apple Dev Program 审核不可控**：最长 2 周，W0 不提交会阻塞 v0.1 发布（W12）

---

## 📝 Notes / 讨论

- CLI 输出协议"初探" ≠ "完整解析" —— MVP 不做 AI-Aware 联动，只需要能"作为一个普通终端程序运行"即可
- 对外文档必须坚持 AI-Aware 是 v1.0 vision（`CLAUDE.md` "🚫 禁区" 条款）—— 本 Spike 的协议报告**不得出现在对外 README / landing**，只在内部文档
- 样本录制要脱敏（auth token / 用户输入可能含敏感信息）

## 🔗 相关

- ADR：**暂不建 ADR**（CLI 协议 MVP 不锁，v1.0 前再建 ADR-006）
- 对应 `CLAUDE.md` ⚠️ 条款：Claude CLI / Codex CLI 输出协议未经实机验证
- `implementation-plan.md` 章节：§附录 A D6 · §9 R1 · §1.1 AI-Aware vision
- 上游：SPIKE-05
- 下游：v1.0 AI-Aware 深度 spike（未来 task）

---

**填写完毕后自审**：

1. **递归完备性**：主线（CLI 测 + 报告）+ 副线（Dev Program 申请）两条都覆盖 ✅
2. **反向场景**：双失败 → README 措辞修改；单失败 → v0.2 延期 ✅
3. **边界适用性**：MVP 只要能跑，v1.0 才要深度解析；本 Spike 边界清楚 ✅
4. **YAGNI**：不做完整协议解析（留给 v1.0），只初探够用 ✅
