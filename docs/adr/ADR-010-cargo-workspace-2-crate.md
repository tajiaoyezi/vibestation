# ADR-010: Cargo workspace = 2 crate（`app` + `core`）· v0.2 再按需拆

**状态**：accepted
**日期**：2026-04-18（Phase 1 锁定 · Phase 3 ADR 建立）
**决策者**：项目发起人 · 多 agent 评审
**对应 `CLAUDE.md` 决策表**：#5

---

## 背景与问题

Rust 项目的 Cargo workspace 结构是早期关键决策：
- 拆太细（5+ crate）：编译慢 · CI 矩阵复杂 · 对单人项目过度设计
- 拆太粗（1 crate）：业务层 / 平台层 / IPC 耦合 · 单元测试难做

v1 原计划 **4 crate**（app / core / platform / ipc）· v2 收紧到 **2 crate**。

## 决策驱动因素

- **D1 · 编译速度**：增量编译 ≤ 30s（开发热路径）
- **D2 · 测试隔离**：业务逻辑（core）单元测试不依赖 Tauri runtime
- **D3 · YAGNI**：v0.1 团队规模 1 人 · 过度拆分无收益
- **D4 · 未来扩展**：v0.2+ 若 CLI tool / 独立 daemon 需要 · 再拆

## 考虑的选项

- **A · 1 crate monolith**：简单 · 但 business logic 单测需 Tauri runtime · 不好
- **B · 2 crate（app + core）**：business in `core` · platform/IPC/UI in `app` · 核心 / 外壳清晰
- **C · 4 crate（app + core + platform + ipc）**：清晰但 3 crate 就够了 · app/core 边界足以
- **D · 5+ crate（app + core + git + terminal + storage + ipc）**：过度 · 编译时间爆 · 拒绝

## 决策

**选择**：选项 B · **2 crate · `app` + `core`**

**crate 职责**：
- **`core`**（纯 Rust 业务逻辑 · 无 Tauri 依赖）：
  - Git 操作封装（git2 / gix wrapper）
  - 存储层（redb / rusqlite schema + migration）
  - PTY 管理（portable-pty wrapper）
  - 数据模型（workspace / profile / session / pane）
  - 配置导入（Ghostty / iTerm2 / Alacritty parser）
  - `cargo test` 可跑 · 无 UI 依赖
- **`app`**（Tauri 应用层 · 依赖 `core`）：
  - Tauri `main.rs` + commands（IPC 入口）
  - 前端 SolidJS build（`dist/` 嵌入）
  - 平台特定代码（mac code sign / linux AppImage）
  - 启动流程 + 崩溃恢复
- 单一 `Cargo.toml` workspace root + `Cargo.lock`（工作区共享）

**v0.2+ 扩展触发条件**（才拆更多 crate）：
- 有 CLI tool 需要独立分发 → 拆 `cli` crate
- 有第三方 plugin 接口 → 拆 `plugin-api` crate
- 某模块编译 > 3 分钟 → 拆以独立缓存

**理由**：
1. **YAGNI**：v0.1 团队 1 人 · 2 crate 足以隔离核心业务
2. **测试隔离**：`core` 无 Tauri 依赖 · `cargo test -p core` 秒级
3. **未来扩展明确**：拆更多 crate 的 trigger 条件清晰（不是"凭感觉"）

## 后果

### 正面

- **编译快**：2 crate workspace · 增量编译 10-30s · 开发热路径健康
- **测试健康**：`core` 无 Tauri · 纯单元测试跑得快 · CI 稳定
- **扩展清晰**：v0.2 CLI tool / plugin API 何时拆有明确 trigger
- **新人上手快**：`app` / `core` 职责清晰 · 找代码不绕

### 负面

- **无 `platform` crate**：mac / linux 特定代码直接放 `app` 里 · 用 `#[cfg(target_os = "...")]` · 行数少于 100 行前不痛
- **`core` 边界模糊处**：如 IPC types 既在 `core` 定义又在 `app` 序列化 · 需要 `serde` feature gate · 已验证模式

### 风险

- **`core` 膨胀**：若业务代码都堆 `core` · 某天 `core` 10MB 代码 · 编译变慢 · **对策**：半年 review 一次 · 按上述 trigger 拆分
- **platform code 散落 `app`**：mac-only 代码多了后需要独立 crate · 但 100 行以下 `#[cfg]` 够用

## 与 `implementation-plan.md` 的映射

- 对应章节：§3.2（workspace 结构）· §附录 A（Spike 不涉及 workspace）
- 对应风险：无（拆分过度是 v2 已规避的风险）

## 相关

- `CLAUDE.md` 决策表：#5
- 详细 spec：[MVP-01 Tauri 应用骨架](../tasks/MVP-01-tauri-app-shell.md)（首次建立 workspace）
- 相关 ADR：ADR-004（前端在 `app/web/`）· ADR-006（Tauri 2 · 决定 `app` crate 上级）

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code（Phase 3 · 把 Phase 1 锁定决策正式化为 ADR）
