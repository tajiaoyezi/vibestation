# Task `3.2`: `config-import-paths`

**Status**: Ready

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`（详见 `docs/s2v/standard.md` §10.5.1 状态机）。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 3 `terminal-integration`
**Dependencies**: 依赖 1.2（`crates/app/src/lib.rs` 跨平台 `home_dir()` helper · `dirs` crate）

## 1. Background

`crates/core/src/config_import/` 的终端配置导入全是 Unix/macOS 路径假设：

- `ghostty.rs::scan`：primary `~/.config/ghostty/config`（Linux）+ fallback `~/Library/Application Support/...`（macOS），无 `%APPDATA%` 分支 → Windows 原生 Ghostty 配置永远找不到。
- `alacritty.rs::scan`：写死 `~/.config/alacritty/alacritty.{toml,yml}`，无 `%APPDATA%` 分支 → Windows 原生 Alacritty 配置找不到。
- `iterm2.rs::scan`：构造 macOS plist 路径；iTerm2 是 macOS 独占产品，Windows 上拼接虚假路径（虽 `path.exists()` 必 false，但应显式短路）。
- `ipc.rs::prettify_home_path`：只查 `HOME` 环境变量；Windows 上 `HOME` 通常不存在 → 返回完整绝对路径而非 `~/` 形式。
- `crates/app/src/lib.rs::home_dir_or_root`：`HOME` 缺失时 fallback `PathBuf::from("/")`（Unix root），Windows 上导致 `scan(home)` 拿到 `/Library/...` 等虚假路径。

这让 PRD §Core Capabilities #4 的「config import 支持 `%APPDATA%` 路径」与 §Users 场景 3「配置迁移」在 Windows 失效。

## 2. Goal

Windows 上 config import 扫描走正确路径：Ghostty / Alacritty 优先探测 `%APPDATA%`（保留 `.config` fallback 以兼容 WSL），iTerm2 在 Windows 直接返回 `path_exists=false` 不拼虚假路径，`prettify_home_path` 在 Windows 用 `USERPROFILE`/`dirs` 把家目录折成 `~/`，`home_dir_or_root` 复用 1.2 的 `home_dir()` helper。mac/Linux 行为零回归（Windows 路径为**新增分支**而非替换）。

## 3. Scope

### In Scope

- `crates/core/src/config_import/ghostty.rs`：`scan()` 加 `#[cfg(target_os = "windows")]` 路径分支，优先 `%APPDATA%/ghostty/config`（用 `std::env::var("APPDATA")`），保留 `~/.config/ghostty/config` fallback（WSL 兼容）。
- `crates/core/src/config_import/alacritty.rs`：`scan()` 加 Windows 分支，优先 `%APPDATA%/alacritty/alacritty.toml`（再 `.yml`），保留 `~/.config/alacritty/...` fallback。
- `crates/core/src/config_import/iterm2.rs`：`scan()` 首行加 `#[cfg(not(target_os = "macos"))]` 短路，Windows/Linux 直接返回 `path_exists=false` + 空 `detected_fields`（不构造 macOS plist 路径）。
- `crates/core/src/config_import/ipc.rs`：`prettify_home_path()` 加 Windows 分支，用 `USERPROFILE`（或 1.2 `home_dir()` 复用）做 `~` 折叠。
- `crates/app/src/lib.rs`：`home_dir_or_root()` 改为复用 Task 1.2 的跨平台 `home_dir()` helper（不再硬 fallback `"/"`）。
- Windows 专属单元测试（`%APPDATA%` 路径命中 / WSL fallback / iTerm2 Windows 短路 / prettify_home_path Windows）。

### Out Of Scope

- keybinding `Cmd/Ctrl` 平台映射（Task 3.3 负责；本 task 只管路径与家目录解析）。
- config import parser 本身的字段解析逻辑（TOML/YAML 解析跨平台，survey 标 already-ok，不动）。
- `dirs` crate 的引入（Task 1.2 已引入并提供 `home_dir()`；本 task 复用）。
- 新的 import source（只补现有 Ghostty/Alacritty/iTerm2 的 Windows 路径）。

## 4. Users / Actors

- **Windows 11 AI-agent 开发者**：从已装的 Windows 原生 Alacritty / Ghostty（配置在 `%APPDATA%`）导入字体/快捷键设置。
- **`crates/app` config import IPC 层**：调用 `scan_all_sources(home)` 与 `prettify_home_path`，期望 Windows 上扫到 `%APPDATA%` 配置、显示 `~/` 折叠路径。
- **WSL 用户**：配置可能在 `~/.config`（Linux 约定），期望 fallback 仍命中。

## 5. Behavior Contract

### 5.1 Required Reading

- 上游 task spec：`docs/specs/tasks/task-1.2-home-dir-helper.md`（跨平台 `home_dir()` helper · `dirs` crate · Windows `USERPROFILE`）。
- 同 phase 参考：`docs/specs/phases/phase-3-terminal-integration.md` §3 涉及模块。
- BDD：`test/features/config-import.feature`（Task 3.2 + 3.3 场景）。
- 相关 ADR：`docs/decisions/adr-002-cross-platform-home-dir-dirs.md`（`dirs` crate 决策，本 task 消费其产物）。
- 现状源码：`crates/core/src/config_import/ghostty.rs` (`scan`) · `alacritty.rs` (`scan`) · `iterm2.rs` (`scan`) · `ipc.rs` (`prettify_home_path` ~line 142) · `crates/app/src/lib.rs` (`home_dir_or_root` ~line 636)。

### 5.2 Imports

- `std::path::Path` / `std::path::PathBuf`（已有）。
- `std::env`（`std::env::var("APPDATA")` / `var("USERPROFILE")` — Windows 分支）。
- Task 1.2 提供的 `home_dir()` helper（`crates/app/src/lib.rs` 内，供 `home_dir_or_root` 复用）。
- `dirs`（经 1.2 引入；`prettify_home_path` 可选用 `dirs::home_dir()` 兜底）。
- `serde`（已有，config struct 反序列化）。

### 5.3 函数签名

Windows 适配后的真实签名骨架（公开签名不变，内部加 cfg 分支）：

```rust
// ghostty.rs — scan 签名不变 · 内部路径解析加 Windows 分支
pub fn scan(home: &Path) -> RawScanResult {
    let candidates: Vec<PathBuf> = {
        #[cfg(target_os = "windows")]
        {
            let mut v = Vec::new();
            if let Ok(appdata) = std::env::var("APPDATA") {
                v.push(PathBuf::from(appdata).join("ghostty/config"));
            }
            v.push(home.join(".config/ghostty/config")); // WSL fallback
            v
        }
        #[cfg(not(target_os = "windows"))]
        {
            vec![
                home.join(".config/ghostty/config"),
                home.join("Library/Application Support/com.mitchellh.ghostty/config"),
            ]
        }
    };
    let path = candidates.into_iter().find(|p| p.exists());
    // ...既有 parse_file 逻辑不变...
}

// alacritty.rs — scan 同模式：Windows 优先 %APPDATA%/alacritty/alacritty.{toml,yml}
pub fn scan(home: &Path) -> RawScanResult;

// iterm2.rs — Windows/Linux 短路（iTerm2 macOS 独占）
pub fn scan(home: &Path) -> RawScanResult {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = home;
        return RawScanResult {
            source: ImportSource::ITerm2,
            path: None,
            path_exists: false,
            detected_fields: Vec::new(),
            errors: Vec::new(),
        };
    }
    #[cfg(target_os = "macos")]
    {
        // ...既有 macOS plist 逻辑不变...
    }
}

// ipc.rs — prettify_home_path 加 Windows 家目录解析
fn prettify_home_path(p: &Path) -> String {
    let s = p.to_string_lossy();
    #[cfg(target_os = "windows")]
    let home = std::env::var("USERPROFILE").ok();
    #[cfg(not(target_os = "windows"))]
    let home = std::env::var("HOME").ok();
    if let Some(home) = home.filter(|h| !h.is_empty()) {
        if let Some(stripped) = s.strip_prefix(&home) {
            return format!("~{stripped}");
        }
    }
    s.into_owned()
}

// crates/app/src/lib.rs — home_dir_or_root 复用 1.2 helper（不再硬 "/"）
fn home_dir_or_root() -> PathBuf {
    home_dir().unwrap_or_else(|| {
        #[cfg(target_os = "windows")] { PathBuf::from("C:\\") }
        #[cfg(not(target_os = "windows"))] { PathBuf::from("/") }
    })
}
```

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities #4 · §Users 场景 3): Windows 上构造 `%APPDATA%/alacritty/alacritty.toml`（含 `[font]` section），`alacritty::scan` 返回 `path_exists=true` 且 font 字段被 detect；构造 `%APPDATA%/ghostty/config` 时 `ghostty::scan` 同样 `path_exists=true`。
- [ ] **AC2** (PRD §User Flow 异常流「HOME 未设」): Windows 上 `~/.config/ghostty/config`（WSL 风格）存在而 `%APPDATA%` 不存在时，`ghostty::scan` fallback 仍命中（`path_exists=true`）。
- [ ] **AC3** (PRD §Core Capabilities #4 · iTerm2 macOS 独占): 非 macOS 平台 `iterm2::scan` 返回 `path_exists=false` + 空 `detected_fields` + 空 `errors`，不构造任何 `Library/Preferences/...` 路径。
- [ ] **AC4** (PRD §User Flow 异常流 · prettify): Windows 上设 `USERPROFILE=C:\Users\alice`，`prettify_home_path("C:\\Users\\alice\\AppData\\Roaming\\alacritty\\config")` 返回 `~`-前缀形式（`~\AppData\Roaming\alacritty\config`），而非完整绝对路径。
- [ ] **AC5** (PRD §Problem Statement · home fallback): Windows 上 `HOME`/`USERPROFILE` 经 1.2 `home_dir()` 解析成 `C:\Users\<user>` 而非 `/`；`home_dir_or_root()` 不再返回 Unix root。
- [ ] **AC6** (PRD §Constraints 兼容性 · §Success Metrics 反指标): mac/Linux 上 `ghostty::scan` / `alacritty::scan` / `iterm2::scan`（macOS）/ `prettify_home_path` 行为零回归，现有 Unix 单元测试全绿。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 %APPDATA% 命中 | SCEN-3.2.1 | TEST-3.2.1 `test_3_2_1_windows_appdata_scan_detects` | N/A（tempdir + 注入 home 单测） | cargo test -p vibestation_core config_import:: | Not Started |
| AC2 WSL .config fallback | SCEN-3.2.2 | TEST-3.2.2 `test_3_2_2_windows_dotconfig_fallback` | N/A | cargo test -p vibestation_core config_import:: | Not Started |
| AC3 iTerm2 非 macOS 短路 | SCEN-3.2.3 | TEST-3.2.3 `test_3_2_3_iterm2_non_macos_not_found` | N/A | cargo test -p vibestation_core config_import::iterm2 | Not Started |
| AC4 prettify Windows | SCEN-3.2.4 | TEST-3.2.4 `test_3_2_4_prettify_userprofile_tilde` | N/A | cargo test -p vibestation_core config_import::ipc | Not Started |
| AC5 home_dir fallback | SCEN-3.2.5 | TEST-3.2.5 `test_3_2_5_home_dir_or_root_windows` | N/A | cargo test -p vibestation_app | Not Started |
| AC6 mac/Linux 零回归 | SCEN-3.2.6 | TEST-3.2.6 `test_3_2_6_unix_config_scan_unchanged`（现有用例集） | N/A | cargo test --workspace（mac/Linux CI） | Not Started |

## 8. Risks

- **R1（PRD §Technical Risks R4 · 路径/UNC/编码）**：`%APPDATA%` 含反斜杠 / 中文用户名 / 空格，`Path::join` 拼接与 `strip_prefix` 折叠需正确；含特殊字符的 fixture 走文件而非 inline（adapter §Fixture 约定）。
- **R2（环境变量缺失）**：CI / 测试环境可能不设 `APPDATA`/`USERPROFILE`；单元测试注入 `home` 参数 + tempdir 模拟，不依赖真实环境变量（`prettify_home_path` 的 env 读取用临时 set/restore 或抽出可注入的 inner 函数）。
- **R3（PRD §Technical Risks R3 · mac/Linux 回归）**：`#[cfg(not(target_os = "macos"))]` 误伤 Linux 上 iTerm2 行为 —— 但 iTerm2 本就 macOS 独占，Linux 上原本 `path.exists()` 即 false，短路语义等价；TDD 先 RED 锁 macOS 路径不变。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制；scoped: cargo test -p vibestation_core config_import:: + cargo test -p vibestation_app home_dir -->
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings

> Integration / E2E / Coverage / Runtime-smoke：config import 解析为纯逻辑，集成随 `cargo test --workspace` 跑（含 `crates/core/tests/`）；无独立 e2e；MVP 不强制覆盖率；GUI 导入对话框的 Windows 路径显示在 Phase 3 §6 / §2.14 本机验，不列入本 task §9。

## 10. Completion Notes

<TBD-after-impl>
