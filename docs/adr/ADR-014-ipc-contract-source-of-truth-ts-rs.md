# ADR-014: IPC contract source of truth = Rust struct + ts-rs codegen

**状态**：accepted
**日期**：2026-04-21
**决策者**：Claude Code（作者 · ADR 正式化）· Codex CLI（SPIKE-08 §A 实施者）· tajiaoyezi（Arbiter · 即用户）
**对应 `CLAUDE.md` 决策表**：—（未显式列入 · 属规范类跨模块决策 · 本 ADR 正式化）
**对应 Spike**：[SPIKE-08](../tasks/SPIKE-08-e2e-and-contract-harness.md) · §A Contract Layer PASS（PR #60 · 2026-04-20）

---

## 背景与问题（Context and Problem Statement）

### H2 事件 · 2026-04-19 · MVP-02 · PR #47

Rust `#[serde(rename_all = "camelCase")]` 输出字段 `workspaceId` · TS interface 手工声明 `workspace_id` · runtime `delete` 全 broken。CI 7/7 全绿（cargo test · tsc --noEmit · pnpm build · Tauri build smoke · secret scan · markdown lint · frontmatter validator）· **产品 broken 未被 CI 发现**。

### 根本问题

前端 TypeScript interface 和 Rust struct 是**两套独立定义** · tsc 无法跨边界比对：

- Rust 改字段 → 前端不改 → compile 通过 → runtime broken
- Rust 改 serde rename → 前端不改 → compile 通过 → runtime broken
- Rust 增字段 → 前端缺字段 → compile 通过 → runtime 部分功能 broken
- 人肉同步 · 高频出错

### SPIKE-08 §A 结论（2026-04-20 · PR #60 accepted）

选型对比 2 候选 · 选 **`ts-rs`**（非 `tauri-specta`）：

| 维度 | `ts-rs` 12.x | `tauri-specta` 2.x |
|---|---|---|
| 依赖体积 | 轻（仅 ts-rs）| 重（+ specta + specta-typescript 等） |
| Tauri 绑定 | 不绑（通用 Rust→TS）| 深绑 Tauri 2 |
| build.rs 集成 | 原生 · 3 行代码 | 较复杂 · 需注入 invoke wrapper |
| H2 回归验证 | compile-time tsc FAIL（已实证 · SPIKE-08 §A）| 同效果（但未实测） |
| 维护活跃度 | 活跃（2024-2025 持续更新）| 活跃 |
| 禁 invoke wrapper（项目偏好）| ✅ 不生成 invoke wrapper | ❌ 自动生成（但可关）|
| 学习成本 | 低（`#[derive(TS)] #[ts(export)]`）| 中（需学 specta 宏）|

SPIKE-08 §A POC（PR #60）已验证 ts-rs 方案跑通 · PR #63 ts-rs rollout 把 5 个 MVP-02 IPC struct 从手写 interface 切到 codegen · H2 regression proof（临时 rename 字段 · tsc 抓 10 处 drift）PASS。

### 不决策的后果

- 每个 MVP 的 IPC struct 都可能踩 H2（MVP-04 5 个 · MVP-05 10 个 · MVP-06 7 个 · MVP-07 7 个 · MVP-08 8 个 · MVP-09 9 个 · MVP-10 未知 · 合计 50+ · 不做 codegen 会反复人肉翻译 + 反复踩）
- 未来想 systematically 改 IPC（如加 version 字段）· 无规范可依
- ADR-004 前端栈只写 SolidJS/TS/Vite/xterm.js · 未含 ts-rs · 没 ADR 锁死此决策 · 未来有人质疑可动摇

## 决策驱动因素（Decision Drivers）

- **D1 · H2 根因消除**：compile-time 防御字段名 drift · tsc 报错而非 runtime broken
- **D2 · 边际成本低**：每个新 IPC struct 仅需 `#[derive(Debug, Clone, Serialize, Deserialize, TS)] #[ts(export)] #[serde(rename_all = "camelCase")]`（4 行 derive + 1 行 serde attr）· 前端直接 `import type { X } from "./bindings/X"`
- **D3 · 无 invoke wrapper 约束**：项目偏好保持 Tauri `invoke("command_name", args)` 原生调用 · 不包装 · ts-rs 刚好不生成 wrapper
- **D4 · 依赖轻量**：单一 crate `ts-rs` · 无 specta 生态链
- **D5 · 可 opt-out 迁移**：ADR accepted 不强制立即全改 · 每个新 MVP 用 ts-rs · 旧 MVP-02 的 5 struct 已通过 PR #63 rollout 切完（先例）· 其他 MVP 按 §G 规范逐个落

## 考虑的选项（Considered Options）

### 选项 A · ts-rs（SPIKE-08 §A PASS · 已生产验证）

- `crates/core/*.rs` struct 加 `#[derive(TS)]` + `#[ts(export)]`
- `crates/app/build.rs` 在 `cargo build` 时调 `export_all_to("web/src/bindings")` 生成 TS
- 前端从 `./bindings/*` import · 禁手写对偶 interface
- `.prettierignore` 排除 `web/src/bindings/`（generated · 不参与 format）

### 选项 B · tauri-specta

- 深绑 Tauri 2 · 会自动生成 invoke wrapper（违反 D3）
- 依赖链更重
- SPIKE-08 §A 未深测（选型对比即排除 · 不符 D3/D4）

### 选项 C · 继续手写对偶 interface（维持 H2 前状态）

- 零新依赖 · 但继续踩 H2
- SPIKE-08 §A 实证 MVP-02 临时 rename 字段 → tsc FAIL 抓 10 处 drift · 没 codegen 则需 runtime 才爆
- 被 H2 事件否决

### 选项 D · 第三方 codegen（typegen · tsify 等）

- 生态成熟度低 · 活跃度不及 ts-rs
- SPIKE-08 §A 未列入对比 · YAGNI

## 决策（Decision Outcome）

**选择**：选项 A · **ts-rs** 作为 IPC contract source of truth

**理由**：

- SPIKE-08 §A 已 PASS + PR #63 rollout 已生产验证（5 struct · H2 regression proof 抓 10 处 drift）
- 满足 D1/D2/D3/D4 全部
- 选项 B/C/D 各有阻塞点（wrapper / 继续踩坑 / 生态）

### 规范（所有新 IPC struct 必守 · 已写入各 MVP §G）

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MyIpcStruct {
    pub field_a: String,
    #[ts(type = "number")]        // i64/f64 时间戳 / 大数 · 强制 number 而非 bigint
    pub timestamp_seconds: i64,
}
```

必守规则（5 条 · 已写入 MVP-04/05/06/07/08/09 §G）：

1. 所有 IPC struct `#[derive(Debug, Clone, Serialize, Deserialize, TS)]`
2. 导出标记 `#[ts(export)]`
3. 字段 camelCase：`#[serde(rename_all = "camelCase")]`
4. `i64` / `f64` 时间戳加 `#[ts(type = "number")]`（防 TS 默认生成 `bigint`）
5. bindings 由 `crates/app/build.rs` 生成到 `web/src/bindings/` · 前端禁手写 interface

### H2 regression proof（每个 MVP PR merge 前必做一次）

1. 临时对任一 IPC struct 字段加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'StructName'`（FAIL 证明防御生效）
5. **回滚**：撤销 `#[ts(rename = ...)]`· 确认 `pnpm typecheck` 恢复 PASS
6. 结果记录到 PR body 或 `docs/runtime-evidence/<task-id>/h2-regression-proof.md`

## 后果（Consequences）

### 正面

- H2 根因消除 · compile-time 防御代替 runtime 试错
- 每个新 IPC struct 的成本固定（5 行 attr · 前端 1 行 import）· 边际成本恒定
- `cargo build` 自动重生成 bindings · 没有"忘记 codegen"的漏洞
- PR diff 清晰：Rust struct 改 → 前端 import 自动对齐 · 代码 review 只看一边

### 负面

- `web/src/bindings/` 进 git（generated 但进 git · 保证 CI / clone 一致性）· 占 ~10 KB 每次 bindings 变更
- 新 crate `ts-rs` 加入 workspace · 编译时间 +1-2s（每次 build.rs 跑）
- 不支持的 Rust 类型（如 `std::time::SystemTime` 等复杂类型）需手工 `#[ts(type = "...")]` 标注 · 有轻微学习曲线

### 风险

- **R1 · ts-rs crate 被弃维护**：
  - 监控：定期（每季度）检查 `cargo outdated` 和 GitHub issue 响应时间
  - Fallback：迁移到 `tauri-specta`（选项 B · 已评估）· 或接受项目内 fork（ts-rs 是纯 Rust codegen · 移植难度低）
- **R2 · 生成的 TS 和 committed bindings 不一致**（开发者忘 commit 重生成的文件）：
  - 监控：CI 加 check · `cargo build` 后 `git diff --exit-code web/src/bindings/` · 不一致则 FAIL（SPIKE-08 §A 已识别 · PR #63 已加）
- **R3 · camelCase serde rename 遗漏**：
  - 防御：§G 强制规范 5 条 · review 时逐条对照
  - Regression proof：每 PR 一次 H2 proof 验证机制生效

## 与 `implementation-plan.md` 的映射

- 对应章节：§5.2 质量门 · §10.6 终端正确性矩阵（IPC 契约属质量门）
- 对应风险：R30（隐私合规）的技术底座 · H2 事件后补的 compile-time gate

## 相关（Links）

- SPIKE：[SPIKE-08 §A](../tasks/SPIKE-08-e2e-and-contract-harness.md)（done · PR #60 · 2026-04-20）
- 生产化 PR：PR #63 ts-rs rollout（MVP-02 5 struct 切到 codegen · H2 regression proof PASS · 2026-04-20）
- 所有 MVP §G（规范引用此 ADR）：MVP-04 · MVP-05 · MVP-06 · MVP-07 · MVP-08 · MVP-09
- 事件：H2（PR #47 · 2026-04-19 · IPC 字段命名 camelCase mismatch 修复）· 触发规则 15（CI build smoke ≠ runtime smoke）升级
- 规则：`~/.claude/rules/15-runtime-verification-gate.md` · 上位通用规则
- 相关 ADR：[ADR-004](./ADR-004-frontend-stack.md)（前端栈 · SolidJS/TS/Vite/xterm.js · 本 ADR 扩展 IPC 层）· [ADR-010](./ADR-010-cargo-workspace-2-crate.md)（core + app 2 crate · build.rs 位于 app）

---

**修订历史**：
- 2026-04-21 · 初版 · accepted · Claude Code + Codex CLI（SPIKE-08 §A 实施证据）+ tajiaoyezi
- 追溯时点：SPIKE-08 §A accept @ 2026-04-20（PR #60）· PR #63 rollout @ 2026-04-20 · 本 ADR 为 session 13 audit X-4 补正式文档

**自审四问**：

1. **递归完备性**：ADR 覆盖选型对比（A/B/C/D 四选项）+ 规范（5 条）+ H2 proof（6 步）+ 风险 fallback（R1/R2/R3）· 完整 ✅
2. **反向场景**：R1 ts-rs 弃维护 → fallback tauri-specta 或 fork · R2 bindings 不一致 → CI check 抓（已有）· R3 serde rename 遗漏 → §G 规范 + review ✅
3. **边界适用性**：适用所有 IPC struct（Rust struct 与 TS interface 跨边界场景）· 不适用纯 Rust 内部 struct（不导出）或纯 TS 类型（React 状态等） ✅
4. **YAGNI**：H2 实证事件驱动 · 不是投机 ADR · 50+ struct 未来必踩 · 非过度工程 ✅
