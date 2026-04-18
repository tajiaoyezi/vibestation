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

### A. CLI 实机测试（主线）

- [ ] **Claude CLI 在 PTY 里正常运行**：
  - [ ] 启动 + 登录流程可完成
  - [ ] 简单对话（1 轮）输出完整显示，无乱码
  - [ ] 流式输出不卡顿（对应 SPIKE-05 验证的 xterm 渲染能力）
  - [ ] `Ctrl+C` 能正确中断
- [ ] **Codex CLI 在 PTY 里正常运行**（相同判据）
- [ ] **输出样本录制**（JSON-RPC / plain text / ANSI 控制字符）：
  - [ ] Claude CLI 样本 ≥ 3 个场景（启动 / 对话 / 错误）
  - [ ] Codex CLI 样本 ≥ 3 个场景（同上）
- [ ] **协议初探报告**：
  - [ ] Claude CLI 输出结构（stream / block / JSON-line / ANSI）
  - [ ] Codex CLI 输出结构
  - [ ] 关键差异点（如 token 分隔、role 标识、error format）
  - [ ] 结论：MVP 能否做到"两 CLI 统一抽象"？如否，v1.0 AI-Aware 要分开实现
- [ ] **macOS PATH 空问题验证**：Tauri 启动的子进程能读到用户 `$PATH`（`fix-path-env` crate 或等价方案）
- [ ] 结果写入 `docs/SPIKE-REPORT.md`（Phase 3 后建立）

### B. Apple Developer Program（副线）

- [ ] 已登录 Apple Developer 账号
- [ ] 已提交 Apple Developer Program 申请（$99/年）
- [ ] 保留申请提交日期与预计审核完成日期
- [ ] **不阻塞其他 Spike** —— 审核期间可继续 MVP 开发，签证前不发布 macOS 公测

## ❌ 失败信号（Fail Signals）

CLI 主线：

- Claude CLI / Codex CLI 在 PTY 里**无法启动**（缺 tty / 环境变量 / auth 路径问题）→ 调查阻塞点
- 输出乱码（ANSI 解析失败 / 编码问题）→ 调查 xterm 配置
- **两 CLI 输出差异巨大，无法抽象统一** → v1.0 AI-Aware 设计需要分开实现（本 Spike 不阻塞 MVP，只是记录到 v1.0 风险）

macOS Dev Program：

- Apple 拒绝申请 → 调查原因（通常是账号信息问题），升级为 Arbiter 仲裁

## 🔀 Fallback 方案

**CLI 通过** → MVP 可以"作为 PTY 里的普通程序"跑 Claude/Codex，协议解析推到 v1.0
**CLI 部分失败（仅一个能跑）** → MVP 优先支持能跑的那个，另一个推到 v0.2
**CLI 双失败** → 仅提供多 Tab 终端，不标榜 AI CLI 集成（README 措辞需修改）

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-06-cli/`：CLI 启动脚本 + 样本录制脚本
- [ ] CLI 输出样本 × 6+（Claude 3 + Codex 3，脱敏后可 attach 到 spike-artifacts）
- [ ] **协议差异分析报告**（`docs/SPIKE-REPORT.md` 对应 section）
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
