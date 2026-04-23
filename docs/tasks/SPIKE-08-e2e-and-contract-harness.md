---
id: SPIKE-08
type: spike
title: E2E + IPC contract 双层防御 harness 选型 + POC
status: done
owner: Codex CLI
phase: W0+-extension
depends_on: ["MVP-02"]
blocks: []
estimate: 2d
plan_ref: implementation-plan.md §5.2 · §10.6
risk_ref:
reviewer: Claude Code (cross-agent review · Codex CLI implementer · 单人项目 v2-D · Arbiter dialogue approved 2026-04-20)
---

# SPIKE-08: E2E + IPC contract 双层防御 harness 选型 + POC

> **状态**：`done`（2026-04-20 · ts-rs 选型通过 · Playwright 作为 v0.1 runtime 补层）
> **依赖**：MVP-02（真实 IPC 面 + rusqlite workspace 存储，作为 POC 的实验靶） / **阻塞**：无（session 11 并行 · 不卡 MVP-03 开工）
> **战略依据**：[`implementation-plan.md §5.2 质量门`](../implementation-plan.md) · [`~/.claude/rules/15-runtime-verification-gate.md`](../../../../.claude/rules/15-runtime-verification-gate.md)

---

## 🎯 目标（Goal）

验证 **IPC contract 生成（compile-time）** + **E2E runtime 驱动（runtime）** 两层防御在 Tauri 2 + SolidJS 栈上的可行性，给出 v0.1 GA 前强制覆盖的落地方案。

## 📖 背景（Context）

- **H2 事件（2026-04-19 · MVP-02 · PR #47）**：Rust `#[serde(rename_all = "camelCase")]` 输出 `workspaceId`，TS interface 误声明 `workspace_id`，runtime `delete` 全 broken。CI 7/7 全绿（cargo test + tsc --noEmit + pnpm build + Tauri build smoke）但 product broken。
- **CI 盲区双层性**：
  - **Compile-time 层**：前端 interface 和 Rust struct 是两套独立定义，tsc 无法跨边界比对，字段 rename / 新增 / 删除都需人肉同步
  - **Runtime 层**：Rust cargo test 只测 Rust 内部，Vite build 只静态 bundle，Tauri build smoke 只验打包，**都不触发真实 IPC 链路**
- **规则 15 活教材**："CI build smoke ≠ runtime smoke"。MVP-02 是第一个大型 IPC 面（7 commands · 5 struct），后续 MVP-03..10 的 IPC 面只会更大。**继续靠人肉 runtime 验证会反复踩同类坑**。
- **社区现状**（2026-04）：
  - `specta` + `tauri-specta`：Rust struct → TS type 自动生成，Tauri 官方推荐，被 tauri v2 社区大量使用
  - `ts-rs`：通用 Rust→TS codegen，不绑定 Tauri，更轻量但需手工接线
  - `tauri-driver`：Tauri 官方 WebDriver，**只支持 Windows/Linux（macOS WKWebView 无 BiDi 协议）**
  - Playwright：主要测 Chromium/Firefox/WebKit browser，不直接测 Tauri native window；可连 Vite dev server（`localhost:1420`）覆盖前端层，但 **dev server 模式下不经过真实 IPC**
  - WebdriverIO：基于 tauri-driver，同样 Linux/Win only

## ✅ 通过标准（Pass Criteria）

### §A Contract Layer（compile-time · 防字段 mismatch）

- [ ] specta 或 ts-rs 选型有结论（对比至少 2 个候选 · 记录依赖体积 / 维护活跃度 / Tauri 2 集成成本）
- [ ] 选定方案能对 MVP-02 全部 5 个 IPC struct（`Workspace` / `WorkspaceRecord` / `WorkspaceInitResponse` / `WorkspaceListResponse` / `WorkspaceCreateRequest` 或等价集）生成 TS types，输出到 `web/src/bindings/<module>.ts`
- [ ] `web/src/App.tsx` 能 `import` 生成的 types 替换所有手写 interface（不改功能，diff 只换声明来源）
- [ ] **H2 回归用例**：改 Rust struct 任一字段名（例 `workspace_id` → `workspaceIdentifier`）→ `cargo build` 重新生成 TS → `pnpm typecheck` **必须 FAIL**（证明 mismatch 被 tsc 抓住）
- [ ] codegen 集成到 build 流程（`pnpm tauri build` / `pnpm tauri:dev` 能触发 · 不需要额外手工 step）
- [ ] CI 加一个 check：生成的 TS 和仓库里 committed 的 bindings 一致（防生成被跳过）

### §B Runtime Layer（E2E · 防交互 broken）

- [ ] 3 选型各跑过一遍 smoke 且有结论表：
  - **B.1 tauri-driver** on Linux（Ubuntu runner 或本机 Docker）· 能启动 app / click button / read DOM
  - **B.2 Playwright + Vite dev server**（只前端层 · 不测 IPC · 补 contract 的视觉回归缺口）· 能截图 / assert DOM
  - **B.3 Playwright + Tauri dev**（如社区有 recipe · 例通过 `playwright` 连 Tauri 自带 debug port）· 可行性 yes/no
- [ ] 至少 1 种选型能覆盖 MVP-02 的 create/list/delete workspace 完整 golden path（从 welcome page → 选目录 → 列表出现 → delete 触发 confirm → 确认后消失）
- [ ] **H2 回归用例**：临时把 `App.tsx` 的 `workspace.id` 改回 `workspace.workspace_id`（模拟 H2 mismatch）→ E2E **必须 FAIL**（golden path 走不下去）
- [ ] 单次 E2E 总耗时 < 5min（不阻塞 PR 节奏）· 10 连跑 flaky rate < 10%

### §C CI 集成 · 跨平台策略

- [ ] 给出 Contract + E2E 在 GitHub Actions 的跑法推荐（OS matrix / 是否分 job / required vs informational）
- [ ] 明确 **macOS E2E 不支持时的降级方案**（例：macOS 仅跑 Contract · Linux 跑完整 Contract+E2E · 都作为 required check）
- [ ] 明确 E2E flaky 时的处理流程（重试策略 / quarantine 规则 · 不允许 silent skip）

## ❌ 失败信号（Fail Signals）

任一条触发即 §A / §B 宣告 FAIL · 走对应 Fallback：

- §A：specta / ts-rs 都无法在 MVP-02 代码上跑通（Serde 特性冲突 / Tauri 2 breaking change）
- §A：生成 TS 但 `tsc` 未抓住 H2 类 mismatch（证明防御无效）
- §B：3 选型都无法覆盖 create/list/delete 完整 golden path
- §B：CI E2E 单次 > 10min 或 flaky > 20%（实际使用不可行）
- §C：macOS 降级方案无法保证 v0.1 GA 产品质量（例：Linux-only E2E 遗漏 macOS-only bug）

## 🔀 Fallback 方案

**§A + §B 都 PASS** → v0.1 GA 前强制：所有 MVP 新增 IPC command 必带 contract 生成 · 核心 golden path 必带 E2E · 写 ADR-012 定架构。

**仅 §A PASS（contract 够 · E2E 跑不起）** → Contract 强制 + `runtime-evidence/` 手工 checklist 补 E2E 缺口 · rule 15 维持。

**仅 §B PASS（E2E 够 · contract 失败）** → 罕见 · 但技术上可行：E2E 每 PR 强跑 · 接受手工对齐 interface（MVP-02 踩坑模式继续）。不推荐。

**§A + §B 都 FAIL** → 回到纯 manual runtime verification · 写 `docs/runtime-evidence/<task-id>/` 硬门槛（每 PR reviewer 必查 N 张截图 / 录屏）· rule 15 升级为强制 checklist 而非 advisory。

对应 `CLAUDE.md` 决策表：暂无（若 PASS 会新增 A 栏 row "E2E + contract harness 架构"）。

## 📦 产出（Deliverables）

> **强制**：4 样齐全 · accept 前主 agent 必须逐项归档完成。详细规则见 [`.claude/rules/spike-delivery-checklist.md`](../../.claude/rules/spike-delivery-checklist.md)。

- [ ] **(1) 决策文档**：`docs/spikes/SPIKE-08-report.md`（必进 git · 含 §A 选型对比表 + §B 3 选型 smoke 结果 + §C CI 推荐 + H2 回归用例真实失败 log）
- [ ] **(2) 实测源码**：`docs/spikes/code/SPIKE-08/`（必进 git · 含 specta / Playwright POC + README.md）
  - POC 可基于 MVP-02 fork 的精简版（`Workspace` struct + 2-3 command）· 不要把整个 MVP-02 copy 进来
  - Cargo.lock 进 git（白名单已配）
  - README.md 必含：复现命令 / 选型结论索引 / CI 推荐摘要
- [ ] **(3) Raw 数据**：`docs/spikes/raw/SPIKE-08/`（必进 git · tsc typecheck log + playwright trace + smoke run log + H2 回归 log）
- [ ] **(4) 冷备**：`spike-tmp/archive/SPIKE-08/`（gitignored · 含 POC build 产物 + node_modules 如过大可省）
- [ ] **(5) 录屏 / 截图**：可选（E2E Playwright trace 已含视频 · 不额外）
- [ ] **(6) ADR**（若 §A+§B 都 PASS）：`docs/adr/ADR-012-e2e-contract-harness.md`（proposed · 由实施 PR 或后续 PR accept）

**独立评审必查项**：
- [ ] 4 样齐全
- [ ] Report 每个结论在 raw / code 可溯源
- [ ] H2 回归用例有真实 FAIL log 证据（不是口头声称）
- [ ] CI 推荐方案经 Ubuntu runner 实跑验证

## 🛠 依赖资源（Resources Needed）

- macOS（主 dev · contract 开发 + macOS 降级策略验证）
- Ubuntu CI runner（GitHub Actions 或本地 Docker ubuntu:24.04）· §B tauri-driver 跑
- 无需额外账号 / 硬件

## ⚠️ 已知风险

- **tauri-specta 2.x 尚在 beta**（2026-04 · 社区状态需 Spike 内实时核实 · 若选此方案需评估 breaking 风险）
- **tauri-driver macOS 不支持**是硬事实 · §C 必须给明确降级方案 · 否则 macOS-only bug 会漏网
- **Playwright + Tauri dev 组合**社区 recipe 不成熟 · 可能需要自研 wrapper · 时间不可控 → spike estimate 是 2d · 若 §B 全部失败提前收尾写 report
- **Codegen 和 build 流程集成**：若方案需要额外的 `cargo build --features specta-export` step · 可能污染主 build · 需在 POC 阶段想清楚

## 📝 Notes / 讨论

- 本 Spike 是 **rule 15 "CI 绿 ≠ runtime 过"** 的制度化落地 · H2 后已沉淀 ADR-011（runtime 证据存储位置）· SPIKE-08 接着沉淀 "自动化证据" 维度
- SPIKE-08 estimate 2d 是上限 · 实际 §B 1d 能出结论就立即写 report（不追求 POC 完美度 · 追求选型清晰度）
- SPIKE-08 不直接实施到 MVP-02 codebase（POC 独立 · 实施走后续 PR）· 避免 POC code 污染主分支

## 🔗 相关

- 事件源头：MVP-02 H2 bug（PR #47 `4f14c8f`）· [FU-1 closure 记录](../PROGRESS.md#L304)
- 上位规则：[`~/.claude/rules/15-runtime-verification-gate.md`](../../../.claude/rules/15-runtime-verification-gate.md)（rule 15）
- 相邻决策：[ADR-011 · runtime evidence location](../adr/ADR-011-runtime-evidence-location.md)（手工证据层）
- 潜在后续 ADR：ADR-012（E2E + contract 架构 · 若 §A+§B PASS）
- 后续被 block 的 task：无直接阻塞 · 但 MVP-03..10 将被 "强烈推荐" 覆盖本 Spike 产出的 harness
