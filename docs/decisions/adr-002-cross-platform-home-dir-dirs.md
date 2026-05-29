# ADR `002`: 跨平台家目录用 `dirs` crate

**Status**: Accepted
**Date**: 2026-05-29
**Category**: 依赖
**Related**: PRD §Decisions Log D2

## Context

`crates/app/src/lib.rs` 在两处硬编码 `std::env::var("HOME")`（约行 336–340 与 637–639 · `home_dir_or_root()`），缺失时回落 `PathBuf::from("/")`。`HOME` 在 Windows 上不是标准环境变量（用户主目录由 `USERPROFILE`，或 `HOMEDRIVE`+`HOMEPATH` 组合定义），因此 Windows 上：

- `pty_pool::refill_async(home_path)` 与 `config_import_scan(home)` 拿到 `/` 根目录而非 `C:\Users\<user>`。
- workspace 初始化、config import 扫描全部基于错误根目录，会去扫 `/Library/Preferences/...` 等虚假 Unix 路径（survey config_import 子系统确认）。

应用数据目录（DB 等）已由 Tauri `app_local_data_dir()` 正确解析到 `%APPDATA%`（survey + adapter Constraints 确认 DB schema 不变），**本 ADR 只解决"用户家目录"的跨平台解析**，不动 app data dir。

## Decision

新增 `dirs` crate，提供统一的 `home_dir()` 助手供 `crates/app/src/lib.rs` 的 workspace 初始化、`home_dir_or_root()`、PTY pool refill、config import scan 共用：

- 家目录解析统一走 `dirs::home_dir()`（内部已正确处理 Windows `USERPROFILE` / `HOMEDRIVE`+`HOMEPATH` 与 Unix `HOME`）。
- 应用数据目录继续用 Tauri `app_local_data_dir()`（已正确解析 `%APPDATA%`，不动）。
- 解析结果建议缓存，避免重复读环境变量。

`dirs` 是纯 Rust、无 C 依赖、事实标准的跨平台目录库，符合 PRD "不引入与现有锁定栈冲突的重依赖" 的硬约束。

## Rationale

- **避免手写边界 bug**：手写 cfg 读 `USERPROFILE`/`HOME` 容易漏 `HOMEDRIVE`+`HOMEPATH` 组合等边界（候选 (a) 的主要风险）。
- **最小且无重依赖**：`dirs` 纯 Rust、无 C 依赖，比 `directories`(ProjectDirs) 轻；且 app data dir 已由 Tauri path API 覆盖，不需要 ProjectDirs 的应用目录抽象。
- **事实标准**：`dirs` 是 Rust 生态家目录解析的事实标准，行为可预期、维护活跃。

## Alternatives

- **(a) 手写 `cfg` 读 `USERPROFILE`/`HOME`**：拒绝 —— 易漏 `HOMEDRIVE`+`HOMEPATH` 组合等 Windows 边界，需自己维护跨平台解析逻辑。
- **(c) `directories` crate（ProjectDirs）**：拒绝 —— 偏重，其核心价值（应用专属配置/数据目录）已由 Tauri `app_local_data_dir()` 覆盖，引入会与现有 path API 职责重叠。
- **(b) `dirs` crate**（**选定**）：纯 Rust、无 C 依赖、事实标准、最小。

## Consequences

**正面**：

- Windows 上 `home_dir()` 解析到 `C:\Users\<user>`，workspace 初始化 / config import scan / PTY pool refill 全部拿到正确根目录（PRD User Flow 异常流 "HOME 未设" 缓解）。
- mac/Linux 行为不变（`dirs::home_dir()` 在 Unix 上等价读 `HOME`），符合兼容性硬约束。
- 单一 helper 收敛家目录解析，消除两处重复的 `env::var("HOME")` + `/` 兜底反模式。

**负面 / 风险**：

- 新增一个外部依赖（`dirs`）—— 但纯 Rust、无 C 依赖、广泛使用，CI/lockfile 影响可控；按 standard §17.7 依赖变更同步本 ADR + adapter 命令约束。
- 路径分隔符 / UNC / 编码差异（PRD R4，概率中 / 影响中）：家目录解析正确后，下游路径仍需 `dunce::canonicalize` 规范化 —— 该部分由 task-3.2（config-import-paths）/ R4 缓解项处理，不在本 ADR 的依赖决策范围。

## Rollback Or Migration Plan

- **回滚**：从 `Cargo.toml` 移除 `dirs`，把 `home_dir()` helper 退回 `#[cfg(windows)] USERPROFILE` / `#[cfg(unix)] HOME` 的手写分支即可；不改 DB schema、不改 IPC 契约、无持久化数据格式变化。
- **迁移**：无数据迁移 —— DB 仍在 `app_local_data_dir()`（跨平台不变）。Windows 用户此前因 `/` 兜底而 config import 扫不到的行为是 bug，修复后即正确，无历史状态需要迁移。

## Follow-ups

- task-1.2（home-dir-helper）落地 `dirs` 依赖 + `home_dir()` 助手 + 替换两处 `env::var("HOME")`。
- task-3.2（config-import-paths）消费 `home_dir()`，落地 `%APPDATA%` 配置路径分支 + `dunce::canonicalize` 规范化（PRD R4）。
- adapter §Constraints / §Commands 同步（依赖变更，standard §17.7）。
- 关联 ADR-003（shell 探测同样依赖正确的 PATH/家目录解析）。
