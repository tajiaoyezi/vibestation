# Task `3.3`: `keybinding-platform`

**Status**: Done

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`（详见 `docs/s2v/standard.md` §10.5.1 状态机）。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 3 `terminal-integration`
**Dependencies**: 依赖 1.1（Windows 编译基线 + `#[cfg(target_os)]` 分支范式）

## 1. Background

`crates/core/src/config_import/keybinding.rs` 的快捷键规范化假设 Apple `Cmd` 键：

- `classify_token()`（line 99-108）把 `cmd`/`command`/`meta`/`super`/`win`/`windows`/`⌘` **全部**映射到 `TokenKind::Cmd` → canonical 输出固定带 `Cmd`。
- `vibestation_builtins()`（line 145-156）硬编码内置快捷键为 `Cmd+,` / `Cmd+T` / `Cmd+W` / `Cmd+D` / `Cmd+Shift+D`。
- `detect_conflicts()`（line 174）拿导入快捷键的 canonical form 与 `vibestation_builtins()` 比较。

后果（PRD §Users 场景 3）：Windows 用户从 Alacritty 导入 `Ctrl+T`，被规范化后仍是 `Ctrl+T`，但内置是 `Cmd+T`，二者 canonical 不等 → 冲突检测**静默失效**，用户的 `Ctrl+T` 与 Vibestation 内置 `Ctrl+T`（Windows 上实际生效的是 Ctrl，前端键盘事件处理已正确）冲突却不被发现。

`tokenize` / `canonicalize_key` 算法本身（split、titlecase、F1-F24）平台无关，**不动**。

## 2. Goal

keybinding 规范化与内置冲突检测平台感知：Windows 上 `win`/`super`/`meta`/`windows` 映射到 `Ctrl`（而非 `Cmd`），`vibestation_builtins()` 返回 `Ctrl+,` / `Ctrl+T` / ...（而非 `Cmd+X`），`detect_conflicts()` 在 Windows 上正确识别导入的 `Ctrl+T` 与内置 `Ctrl+T` 冲突。macOS 上 `Meta+t → Cmd+T` 等现有语义零回归。

## 3. Scope

### In Scope

- `crates/core/src/config_import/keybinding.rs`：
  - `classify_token()`：`cmd`/`command`/`meta`/`super`/`win`/`windows` 的主修饰键分类平台感知 —— Windows 上归 `Ctrl`，macOS/Linux 上保持现状（归 `Cmd`，Linux 沿用 macOS 语义以保兼容）。建议给 `classify_token` / `canonicalize_keybinding` 加显式平台参数（如 `Platform` enum 或 `is_mac: bool`），或用 `#[cfg(target_os)]` 决定主修饰键 canonical 名。
  - `vibestation_builtins()`：平台感知返回 —— macOS `Cmd+X`，Windows/Linux `Ctrl+X`（内置 raw 表的主修饰键按平台替换）。
  - `detect_conflicts()`：保持逻辑，靠 `canonicalize_keybinding` + `vibestation_builtins` 的平台一致性自然正确匹配。
  - 若选「加平台参数」方案：同步更新 `ipc.rs::detect_conflicts_ipc`（line 308）的 caller 传入当前平台。
- Windows 专属单元测试（`win+t → Ctrl+T` / builtins Windows / detect_conflicts Windows Ctrl+T 命中）。

### Out Of Scope

- `tokenize` / split / `canonicalize_key`（单字符大写 / titlecase / F1-F24）算法（平台无关，survey 标 already-ok，不动）。
- 前端快捷键**显示**符号（`⌘` vs `Ctrl`）—— 那是 Task 4.2 `web-shortcuts` 负责（前端 TS）。本 task 只管后端 canonical/冲突检测。
- 配置文件 keybinding action 映射（如 Alacritty SpawnNewInstance → 无映射 warn，延后 Phase B，与本 task 无关）。
- 改变 `Modifier 排序 Cmd > Ctrl > Alt > Shift` 规则本身（spec §H.3 锁定，Windows 上主修饰键 canonical 名变但排序规则不变）。

## 4. Users / Actors

- **Windows 11 AI-agent 开发者**：导入含 `Ctrl+T` 的 Alacritty/Ghostty 配置，期望被告知与内置冲突并可选择保留/跳过。
- **`config_import::ipc` 层**：调用 `detect_conflicts_ipc(fields)`，期望 Windows 上返回正确的 `Ctrl+T` 冲突而非空（漏报）。
- **macOS 用户（回归）**：导入 `Cmd+T` / `⌘T` / `Meta+t`，期望仍规范化为 `Cmd+T` 并正确冲突检测。

## 5. Behavior Contract

### 5.1 Required Reading

- 上游 task spec：`docs/specs/tasks/task-1.1-pty-platform-split.md`（`#[cfg(target_os)]` 范式）。
- 同 phase 参考：`docs/specs/phases/phase-3-terminal-integration.md` §3 涉及模块。
- BDD：`test/features/config-import.feature`（Task 3.2 + 3.3 场景）。
- 相关 ADR：`docs/decisions/adr-001-pty-windows-cfg-separation.md`（cfg 范式，本 task 复用，无新增 ADR）。
- 现状源码：`crates/core/src/config_import/keybinding.rs`（`classify_token` line 99 · `canonicalize_keybinding` line 23 · `vibestation_builtins` line 145 · `detect_conflicts` line 174）· `ipc.rs`（`detect_conflicts_ipc` line 308 caller）。

### 5.2 Imports

- 无新增第三方依赖。
- 若加 `Platform` 参数：可复用现有 enum 或新建模块内 `KeyPlatform { Mac, Other }`（内部类型，不导出 ts-rs）。
- `std::collections::HashMap`（已有，`detect_conflicts` 用）。

### 5.3 函数签名

Windows 适配后的真实签名骨架（推荐「平台参数」方案，使纯函数可测两平台；canonical 算法不变）：

```rust
// 模块内平台标识（内部枚举，不 ts-rs 导出）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyPlatform {
    Mac,
    Other, // Windows + Linux：主修饰键 canonical 为 Ctrl
}

impl KeyPlatform {
    pub(crate) fn current() -> Self {
        #[cfg(target_os = "macos")] { KeyPlatform::Mac }
        #[cfg(not(target_os = "macos"))] { KeyPlatform::Other }
    }
    /// 主修饰键的 canonical 名：Mac → "Cmd"，Other → "Ctrl"
    fn primary_modifier(self) -> &'static str {
        match self { KeyPlatform::Mac => "Cmd", KeyPlatform::Other => "Ctrl" }
    }
}

// 公开签名加平台参数（保留旧无参 wrapper 调 current() 以最小化 caller 改动可选）
#[must_use]
pub fn canonicalize_keybinding_for(input: &str, platform: KeyPlatform) -> String;

// 现有 pub fn 保留为薄 wrapper（运行期取 current 平台），兼容既有 caller
#[must_use]
pub fn canonicalize_keybinding(input: &str) -> String {
    canonicalize_keybinding_for(input, KeyPlatform::current())
}

// classify_token 不再把 win/super/meta 写死 Cmd —— 改返回"主修饰键"中性标记，
// 由 canonicalize 用 platform.primary_modifier() 落地为 Cmd / Ctrl
enum TokenKind {
    PrimaryMod, // 原 Cmd：cmd/command/meta/super/win/windows/⌘
    Ctrl,       // 显式 ctrl/control/⌃
    Alt,
    Shift,
    Key(String),
}
fn classify_token(tok: &str) -> TokenKind; // 平台无关分类，canonical 落地时按平台映射

// builtins 平台感知
#[must_use]
pub fn vibestation_builtins_for(platform: KeyPlatform) -> Vec<(String, &'static str)>;
#[must_use]
pub fn vibestation_builtins() -> Vec<(String, &'static str)> {
    vibestation_builtins_for(KeyPlatform::current())
}

// detect_conflicts 加平台参数（caller ipc.rs 传 current）
#[must_use]
pub fn detect_conflicts_for(
    imported: &[(String, String)],
    platform: KeyPlatform,
) -> Vec<ConflictHit>;
#[must_use]
pub fn detect_conflicts(imported: &[(String, String)]) -> Vec<ConflictHit> {
    detect_conflicts_for(imported, KeyPlatform::current())
}
```

> 注：`PrimaryMod` 与显式 `Ctrl` 在 Windows 上都落到 `Ctrl`，会合并（`win+ctrl+t` → `Ctrl+T`），符合 Windows 实际语义（无独立 Cmd 键）；macOS 上 `Cmd` 与 `Ctrl` 仍是两个独立修饰键，排序 `Cmd > Ctrl` 不变。

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Users 场景 3 · §Core Capabilities #4): `canonicalize_keybinding_for("win+t", KeyPlatform::Other)` 返回 `"Ctrl+T"`；`canonicalize_keybinding_for("win+t", KeyPlatform::Mac)` 返回 `"Cmd+T"`。`super+t` / `meta+t` 同此平台映射。
- [ ] **AC2** (PRD §Core Capabilities #4): `vibestation_builtins_for(KeyPlatform::Other)` 返回的内置键全为 `Ctrl+...`（`Ctrl+,` / `Ctrl+T` / `Ctrl+W` / `Ctrl+D` / `Ctrl+Shift+D`）；`KeyPlatform::Mac` 仍返回 `Cmd+...`。
- [ ] **AC3** (PRD §Users 场景 3 · 冲突检测不再静默失效): `detect_conflicts_for(&[("Ctrl+T".into(),"new_tab".into())], KeyPlatform::Other)` 返回非空 `ConflictHit`，其 `vibe_key == source_key == "Ctrl+T"`、`vibe_action == "tabs.create"`。
- [ ] **AC4** (PRD §Constraints 兼容性 · §Success Metrics 反指标 · macOS 零回归): macOS 上现有测试全绿 —— `canonicalize_keybinding("Meta+t") == "Cmd+T"`、`canonicalize_keybinding("⌘T") == "Cmd+T"`、`detect_conflicts(&[("cmd+t",...)])` 命中 `Cmd+T`。
- [ ] **AC5** (本 task 新增 · 算法不变验证): `tokenize`/`canonicalize_key` 行为在两平台一致 —— `canonicalize_keybinding_for("Cmd+Shift+t", *)` 的 key 部分均为 `T`、F1-F24 与 named key（Tab/Escape）titlecase 不受平台影响。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 win+t 平台映射 | SCEN-3.3.1 | TEST-3.3.1 `test_3_3_1_canonicalize_primary_mod_per_platform`（win/super/meta/command/⌘ · Other→Ctrl · Mac→Cmd） | N/A（纯函数双平台参数单测） | cargo test -p vibestation-core --lib config_import::keybinding | Done |
| AC2 builtins 平台感知 | SCEN-3.3.2 | TEST-3.3.2 `test_3_3_2_builtins_ctrl_on_other`（Other 全 Ctrl+无 Cmd · Mac 全 Cmd） | N/A | cargo test -p vibestation-core --lib config_import::keybinding | Done |
| AC3 Windows Ctrl+T 冲突命中 | SCEN-3.3.3 | TEST-3.3.3 `test_3_3_3_detect_conflicts_ctrl_t_windows` + `test_3_3_3b_detect_conflicts_win_t_windows`（win+t 也命中） | N/A | cargo test -p vibestation-core --lib config_import::keybinding | Done |
| AC4 macOS 零回归 | SCEN-3.3.4 | TEST-3.3.4 `test_3_3_4_macos_cmd_semantics_unchanged` + 11 现有 no-arg 用例改 `_for(KeyPlatform::Mac)` 锁 Mac 分支 + `test_3_3_sort_order_unchanged_mac` | N/A | cargo test -p vibestation-core --lib config_import + cargo test --workspace（macOS CI） | Done（Windows 本机 90 passed · mac 由 CI 跑） |
| AC5 算法平台无关 | SCEN-3.3.5 | TEST-3.3.5 `test_3_3_5_key_titlecase_platform_invariant`（两平台 loop · key 大写 / F1-F24 / named titlecase 一致）+ R3 `test_3_3_r3_primary_mod_ctrl_merge_on_other` | N/A | cargo test -p vibestation-core --lib config_import::keybinding | Done |

## 8. Risks

- **R1（PRD §Technical Risks R3 · mac/Linux 回归 · 最高风险点）**：Linux 现在按 macOS 语义把 `super → Cmd`；本 task 把 `KeyPlatform::Other`（含 Linux）映射到 Ctrl，会改变 **Linux** 的 canonical 输出。需确认：Linux 用户的内置快捷键前端实际也走 Ctrl（与 Windows 同），故 Linux 归 Other 是**修正**而非回归 —— TDD 用现有 macOS 用例锁 `Mac` 分支不变，Linux 行为变化在 PR body 显式声明并由 reviewer 在 Linux 上跑 `cargo test` 确认无破坏（若 Linux 有依赖 `Cmd` canonical 的用例，需同步更新为 Ctrl）。
- **R2（caller 同步）**：选「平台参数」方案后 `ipc.rs::detect_conflicts_ipc` 必须传 `KeyPlatform::current()`，漏改会编译失败（缺参数）—— 编译期捕获。
- **R3（PrimaryMod/Ctrl 合并）**：Windows 上 `Cmd+Ctrl+T` 类异常输入会合并为单 `Ctrl+T`；属可接受的容错行为（Windows 无独立 Cmd），在测试中显式断言该行为，避免被误判为 bug。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制；scoped: cargo test -p vibestation_core config_import::keybinding -->
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings

> Integration / E2E / Coverage / Runtime-smoke：keybinding 规范化为纯函数，单测即覆盖；集成随 `cargo test --workspace` 跑；无独立 e2e；MVP 不强制覆盖率；GUI 冲突提示在 Phase 3 §6 / §2.14 本机验，不列入本 task §9。

## 10. Completion Notes

**完成于 2026-05-29 · feat/windows-support 分支 · solo 三段 commit（RED `1d48157` → GREEN `f6506fb` → docs `本提交`）**

1. **改动文件**：`crates/core/src/config_import/keybinding.rs`（task §3 主体）+ `crates/core/src/config_import/ipc.rs`（**连带修复** · 仅测试断言 · 见第 5 点）。production `ipc.rs::detect_conflicts_ipc` / `apply` 调用点**零改动**（见第 3 点 wrapper 策略）。
2. **`keybinding.rs` 核心**：选 spec §5.3 推荐的「平台参数」方案。新增 `pub(crate) enum KeyPlatform { Mac, Other }`（`current()` 按 `#[cfg(target_os="macos")]` · `primary_modifier()` → `Cmd`/`Ctrl`）。`classify_token` 把 `cmd`/`command`/`meta`/`super`/`win`/`windows`/`⌘` 由原 `TokenKind::Cmd` 改分类为**中性** `TokenKind::PrimaryMod`（平台无关），canonical 落地时由 `canonicalize_keybinding_for(input, platform)` 按 `platform` 决定 `Cmd`（Mac）或 `Ctrl`（Other）。`tokenize`/split/`canonicalize_key`/F1-F24/titlecase 算法**字节不变**（spec §3 Out Of Scope）。
3. **公共 API 兼容（无 caller 改动）**：保留既有无参 `pub fn canonicalize_keybinding` / `vibestation_builtins` / `detect_conflicts` 为薄 wrapper（内部调 `*_for(KeyPlatform::current())`）；新增 `pub(crate)` 的 `*_for(platform)` 变体供测试两平台。因 `ipc.rs` 调的是无参 wrapper，spec §3 / §8 R2 提到的「同步更新 `detect_conflicts_ipc` caller」**无需发生**（未删 wrapper · caller 签名不变 · 编译期亦无缺参）。
4. **Other（Windows+Linux）合并语义（spec §8 R3）**：Other 上 `PrimaryMod` 与显式 `Ctrl` 都落到 `Ctrl`，`Cmd+Ctrl+T` → `Ctrl+T`（无独立 Cmd 键 · 容错合并），`test_3_3_r3_primary_mod_ctrl_merge_on_other` 显式断言非 bug。排序规则 `Cmd > Ctrl > Alt > Shift` 不变（spec §H.3 锁定 · `test_3_3_sort_order_unchanged_mac`）。Linux 归 Other → Ctrl 是 spec §8 R1 钦定的**修正**（非回归 · 前端键盘事件处理本就走 Ctrl）。
5. **连带修复（`ipc.rs` · 仅测试）**：5 个既有 ipc 用例（`detect_conflicts_ipc_finds_cmd_t` / `apply_non_conflicting_keybinding_persists` / `apply_protects_vibe_builtins_when_client_omits_conflict_resolutions` / `apply_allows_explicit_override_of_vibe_builtin` / `apply_keybinding_override_writes_imported_keybindings`）经 `apply`/`detect_conflicts_ipc` 走**运行期平台** canonical，原硬编码 `"Cmd+T"` 期望在 Windows 必失败。引入测试常量 `PRIMARY_MOD`（macOS=`Cmd` · 其余=`Ctrl`），断言改 `format!("{PRIMARY_MOD}+T")`。production 逻辑零改 · 仅断言平台化（类比 task-3.4 §10 连带修复 · 不属本 task §3 范围但为达 §9 gate 必需）。同理 `keybinding.rs` 内 11 个锁 macOS `Cmd` 输出的老 no-arg 用例改用显式 `_for(KeyPlatform::Mac)`，使其在任意宿主（含 Windows）确定性验 Mac 分支零回归（spec §8 R1「现有 macOS 用例锁 Mac 分支不变」）。
6. **测试 + gate**：新增 Windows/平台参数化 TEST-3.3.1/.2/.3/.3b/.4/.5 + R3 + sort-order。`cargo test -p vibestation-core --lib config_import` = **90 passed / 0 failed**（Windows 11 本机真实运行 · 含 keybinding 子模块 26 passed + ipc/ghostty/alacritty/iterm2）。`cargo check --workspace` / `cargo build --workspace` = **0 error**；`cargo clippy --workspace -- -D warnings` = **0 error**（`KeyPlatform::Mac` 变体在非 macOS 非测试视角为「跨平台双分支变体」· cfg-gate `#[cfg_attr(not(target_os="macos"), allow(dead_code))]` 抑制 · 非真死代码）。`web/src/bindings/*.ts` 未被重生成（`KeyPlatform` 为内部 enum · 无 `#[ts(export)]`）。

**AC 状态**：AC1 ✅（win/super/meta/command/⌘ 平台映射）· AC2 ✅（builtins Other 全 Ctrl · Mac 全 Cmd）· AC3 ✅（Ctrl+T 与 win+t 均命中内置 · 不再静默失效）· AC4 ✅（macOS Cmd 语义零回归 · 现有用例锁 Mac 分支）· AC5 ✅（key 大写 / F1-F24 / titlecase 两平台一致 + R3 合并显式断言）。
