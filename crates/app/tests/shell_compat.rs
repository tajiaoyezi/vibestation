//! MVP-04 §I shell 兼容测试矩阵 · 22 用例。
//!
//! 对应 spec：[`docs/tasks/MVP-04-multi-tab-terminal.md`](../../../docs/tasks/MVP-04-multi-tab-terminal.md)
//! §I.1 默认 shell 矩阵（zsh × 4 + bash × 4 + fish × 4 = 12 用例）
//! §I.2 CLI 实机矩阵（Claude CLI × 5 + Codex CLI × 5 = 10 用例）
//!
//! 测试分两类：
//!
//! - **PASS（cargo test 可验）**：直接走 `PtyManager` runtime 验证 PTY 后端语义
//!   （shell 启动 / $TERM 一致性 / UTF-8 中文 IO / 长输出 scrollback 持久化）。
//! - **IGNORE-runtime（需 Tauri runtime + xterm 渲染 + 用户交互）**：标
//!   `#[ignore]` + 注明根因，对应 spec §I.4 "Vibestation app 本地编译 +
//!   `pnpm tauri:dev` 启动" 一行。这些 case 留作未来手动 + 截图证据
//!   （`docs/runtime-evidence/mvp-04/phase-d/`）的程序化占位 / 索引。
//!
//! Linux ignore 沿袭 PR #82 / `pty.rs` mod tests / `pty_scrollback_integration.rs`
//! 的跨平台 timing pattern · macOS（kqueue）稳定 · GitHub Actions Ubuntu runner
//! 的 epoll PTY close event 链路 timing 不稳定 · MVP-04 Phase D Ubuntu runtime
//! 验证时统一深挖。

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use tempfile::TempDir;
use vibestation_core::db;
use vibestation_core::{
    PtyEvent, PtyManager, PtySpawnRequest, TabCreateRequest, TabsDao, WorkspaceStore,
};

// -----------------------------------------------------------------------------
// 测试支撑：shell 探测、PTY 收发 helper、长输出生成器
// -----------------------------------------------------------------------------

/// 以登录 shell 风格收集输出 · 不依赖具体 prompt 串。
///
/// 不同 shell（zsh / bash / fish）的 prompt 字符不同 · 用 `printf` 输出固定
/// sentinel 来判定 "shell 启动 + 命令执行链路 OK" · 比 grep `%` / `$` 稳健。
const SHELL_READY_SENTINEL: &str = "VBSTN-READY";

/// 中文 IME 测试串 · 字面量 UTF-8 序列 · 验证 PTY → event → buffer 链路无截断。
const CHINESE_SAMPLE: &str = "你好";

// 长输出 scrollback 参考量：spec §I.2 case 17 / 22 用 5000+ token · 实测 6000 行
// 即可覆盖 trim threshold 之外的 sliding window · 但本 case 全部标 ignored
// （需 Tauri runtime + xterm 渲染）· 故不在 cargo test 内引用 · 留作未来
// runtime / fixture 时的参考。

/// 解析候选 shell 路径。
///
/// 优先看 `which`（PATH 解析）· 兜底常见 macOS 安装位置（Homebrew / system）。
fn locate_shell(name: &str) -> Option<PathBuf> {
    if let Ok(output) = std::process::Command::new("which").arg(name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && std::fs::metadata(&path).is_ok() {
                return Some(PathBuf::from(path));
            }
        }
    }

    let candidates = match name {
        "zsh" => vec!["/bin/zsh", "/usr/bin/zsh", "/opt/homebrew/bin/zsh"],
        "bash" => vec!["/bin/bash", "/usr/bin/bash", "/opt/homebrew/bin/bash"],
        "fish" => vec![
            "/opt/homebrew/bin/fish",
            "/usr/local/bin/fish",
            "/usr/bin/fish",
        ],
        _ => vec![],
    };

    candidates
        .into_iter()
        .map(PathBuf::from)
        .find(|path| std::fs::metadata(path).is_ok())
}

/// 准备 PTY 测试环境 · 返回（temp dir · pool · workspace · tab · manager · events）。
///
/// 每个测试用例独立 temp dir + db · 防交叉污染。
fn pty_fixture(shell_path: &str) -> (TempDir, PtyManager, Receiver<PtyEvent>, String, String) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("vibestation.db");
    let pool = db::open_pool(&db_path).unwrap();

    let workspace_dir = dir.path().join("workspace");
    std::fs::create_dir_all(&workspace_dir).unwrap();
    let workspace = WorkspaceStore::create(&pool, workspace_dir.to_str().unwrap(), None).unwrap();
    let tab = TabsDao::create(
        &pool,
        &TabCreateRequest {
            workspace_id: workspace.workspace_id.clone(),
            name: Some("compat".to_string()),
            shell: Some(shell_path.to_string()),
            cwd: Some(workspace.path.clone()),
        },
    )
    .unwrap();

    let manager = PtyManager::new();
    manager.set_pool(pool);
    let events = manager.take_event_receiver().unwrap();

    let tab_id = tab.tab_id.clone();
    let tab_shell = tab.shell.clone();
    let tab_cwd = tab.cwd.clone();
    manager
        .spawn(PtySpawnRequest {
            tab_id: tab_id.clone(),
            shell: tab_shell,
            cwd: tab_cwd,
            cols: 80,
            rows: 24,
        })
        .unwrap();

    (dir, manager, events, tab_id, workspace.workspace_id)
}

/// 收集 stdout 直到某个 sentinel 出现 · 或超时。
///
/// 返回累计输出 + 是否命中 sentinel。不要求 shell exit · 适合启动 / sentinel 类
/// 用例（exit 由调用方按需触发）。
fn recv_until_sentinel(
    events: &Receiver<PtyEvent>,
    tab_id: &str,
    sentinel: &str,
    timeout: Duration,
) -> (String, bool) {
    let started = Instant::now();
    let mut output = String::new();

    loop {
        let remaining = timeout
            .checked_sub(started.elapsed())
            .unwrap_or_else(|| Duration::from_millis(1));
        match events.recv_timeout(remaining) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
                output.push_str(&event.data);
                if output.contains(sentinel) {
                    return (output, true);
                }
            }
            Ok(PtyEvent::Exited(event)) if event.tab_id == tab_id => {
                // 进程提前退出 · 返回当前已收集的输出 · 让调用方判定
                let hit = output.contains(sentinel);
                return (output, hit);
            }
            Ok(_) => continue,
            Err(RecvTimeoutError::Timeout) => return (output, false),
            Err(RecvTimeoutError::Disconnected) => return (output, false),
        }
    }
}

/// 收集 stdout 直到 exit 事件 · 返回累计输出 + exit code。
fn recv_until_exit(
    events: &Receiver<PtyEvent>,
    tab_id: &str,
    timeout: Duration,
) -> (String, Option<i32>) {
    let started = Instant::now();
    let mut output = String::new();

    loop {
        let remaining = timeout
            .checked_sub(started.elapsed())
            .unwrap_or_else(|| Duration::from_millis(1));
        match events.recv_timeout(remaining) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
                output.push_str(&event.data);
            }
            Ok(PtyEvent::Exited(event)) if event.tab_id == tab_id => {
                return (output, event.exit_code);
            }
            Ok(_) => continue,
            Err(error) => panic!("pty event recv failed for {tab_id}: {error}"),
        }
    }
}

// -----------------------------------------------------------------------------
// §I.1 默认 shell 矩阵 · zsh × 4
// -----------------------------------------------------------------------------

/// **§I.1 case 01**：zsh 启动 → prompt 可见。
///
/// 通过判定（spec）：< 3s 内 prompt 可见 · 无 panic / 白屏。
/// cargo test 转译：spawn zsh + stdin 一个 sentinel echo · 5s 内收到 sentinel
/// = "shell 进程跑起来 + PTY stdout 链路通"。真正的 prompt 字符（`%`）肉眼
/// 可见性留 runtime 截图。
#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "Linux PTY epoll close event timing 在 GitHub Actions Ubuntu runner 不稳定 · 沿袭 PR #82 / pty.rs mod tests Linux ignore pattern · macOS 本地稳定"
)]
fn zsh_01_startup_shows_prompt() {
    let Some(shell) = locate_shell("zsh") else {
        // macOS / Linux blocker · zsh 应总在系统里 · 缺失即 fail
        panic!("§I.1 case 01 BLOCKER · zsh not found on this platform");
    };

    let (_dir, manager, events, tab_id, _ws) = pty_fixture(shell.to_str().unwrap());
    manager
        .stdin(
            &tab_id,
            &format!("printf '{SHELL_READY_SENTINEL}\\n'\nexit\n"),
        )
        .unwrap();

    let (output, hit) = recv_until_sentinel(
        &events,
        &tab_id,
        SHELL_READY_SENTINEL,
        Duration::from_secs(5),
    );
    assert!(hit, "zsh 启动后未在 5s 内输出 sentinel · 收到：{output:?}");
}

/// **§I.1 case 02**：zsh `echo $TERM` → `xterm-256color`。
///
/// PtyManager 在 spawn 时硬编码 `command.env("TERM", "xterm-256color")`
/// (`crates/core/src/pty.rs` line 480) · 本测验证从 shell 角度看到的 `$TERM`
/// 一致 · 防 child shell 启动脚本（.zshrc）覆盖。
#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "Linux PTY epoll timing · 沿袭 zsh_01_startup_shows_prompt"
)]
fn zsh_02_term_env_is_xterm_256color() {
    let Some(shell) = locate_shell("zsh") else {
        panic!("§I.1 case 02 BLOCKER · zsh not found");
    };

    let (_dir, manager, events, tab_id, _ws) = pty_fixture(shell.to_str().unwrap());
    // 用 sentinel 包裹 · 区分 prompt 噪声和真实 echo 输出
    manager
        .stdin(
            &tab_id,
            "printf 'TERM=%s\\n' \"$TERM\"\nprintf 'VBSTN-DONE\\n'\nexit\n",
        )
        .unwrap();

    let (output, _exit) = recv_until_exit(&events, &tab_id, Duration::from_secs(5));
    assert!(
        output.contains("TERM=xterm-256color"),
        "zsh $TERM 不一致 · 期望 xterm-256color · 收到：{output:?}"
    );
}

/// **§I.1 case 03**：zsh Tab 补全 `ls /u` + Tab → `/usr/`。
///
/// **IGNORE-runtime**：Tab 补全依赖 zsh 的 zle / readline 交互模式 · 需 PTY
/// 进入 raw mode + 用户交互或 expect 风格脚本 · 当前 cargo test 不实施。
/// 留作 `pnpm tauri:dev` 手动验证 · 截图归档 `docs/runtime-evidence/mvp-04/phase-d/shell-zsh-03-tab-completion.jpg`。
#[test]
#[ignore = "需 Tauri runtime · zsh Tab 补全是 zle 交互式行为 · cargo test 无法稳定模拟"]
fn zsh_03_tab_completion_runtime_only() {
    eprintln!(
        "§I.1 case 03 · zsh Tab 补全 · runtime-only placeholder · \
         需 pnpm tauri:dev 手动验证 + docs/runtime-evidence/mvp-04/phase-d/shell-zsh-03-tab-completion.jpg"
    );
}

/// **§I.1 case 04**：zsh 中文 IME 输入 + 输出 · UTF-8 round-trip。
///
/// PTY env `LANG=en_US.UTF-8` + `LC_ALL=en_US.UTF-8`（pty.rs line 482-483）·
/// 验证 stdin → shell echo → stdout event 全链路 UTF-8 字节不被截断 / 替换为
/// `?` / mojibake。
#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "Linux PTY epoll timing · 沿袭 zsh_01_startup_shows_prompt"
)]
fn zsh_04_chinese_ime_utf8_roundtrip() {
    let Some(shell) = locate_shell("zsh") else {
        panic!("§I.1 case 04 BLOCKER · zsh not found");
    };

    let (_dir, manager, events, tab_id, _ws) = pty_fixture(shell.to_str().unwrap());
    manager
        .stdin(
            &tab_id,
            &format!("printf '{CHINESE_SAMPLE}\\n'\nprintf '{SHELL_READY_SENTINEL}\\n'\nexit\n"),
        )
        .unwrap();

    let (output, _exit) = recv_until_exit(&events, &tab_id, Duration::from_secs(5));
    assert!(
        output.contains(CHINESE_SAMPLE),
        "zsh UTF-8 中文 round-trip 失败 · 期望含 '{CHINESE_SAMPLE}' · 收到：{output:?}"
    );
}

// -----------------------------------------------------------------------------
// §I.1 默认 shell 矩阵 · bash × 4
// -----------------------------------------------------------------------------

/// **§I.1 case 05**：bash 启动 → prompt 可见。
#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "Linux PTY epoll timing · 沿袭 zsh_01_startup_shows_prompt"
)]
fn bash_05_startup_shows_prompt() {
    let Some(shell) = locate_shell("bash") else {
        panic!("§I.1 case 05 BLOCKER · bash not found");
    };

    let (_dir, manager, events, tab_id, _ws) = pty_fixture(shell.to_str().unwrap());
    manager
        .stdin(
            &tab_id,
            &format!("printf '{SHELL_READY_SENTINEL}\\n'\nexit\n"),
        )
        .unwrap();

    let (output, hit) = recv_until_sentinel(
        &events,
        &tab_id,
        SHELL_READY_SENTINEL,
        Duration::from_secs(5),
    );
    assert!(hit, "bash 启动后未在 5s 内输出 sentinel · 收到：{output:?}");
}

/// **§I.1 case 06**：bash `echo $TERM` → `xterm-256color`。
#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "Linux PTY epoll timing · 沿袭 zsh_01_startup_shows_prompt"
)]
fn bash_06_term_env_is_xterm_256color() {
    let Some(shell) = locate_shell("bash") else {
        panic!("§I.1 case 06 BLOCKER · bash not found");
    };

    let (_dir, manager, events, tab_id, _ws) = pty_fixture(shell.to_str().unwrap());
    manager
        .stdin(
            &tab_id,
            "printf 'TERM=%s\\n' \"$TERM\"\nprintf 'VBSTN-DONE\\n'\nexit\n",
        )
        .unwrap();

    let (output, _exit) = recv_until_exit(&events, &tab_id, Duration::from_secs(5));
    assert!(
        output.contains("TERM=xterm-256color"),
        "bash $TERM 不一致 · 期望 xterm-256color · 收到：{output:?}"
    );
}

/// **§I.1 case 07**：bash Tab 补全 → 候选列表。
///
/// **IGNORE-runtime**：同 zsh case 03 · readline 交互式行为。
#[test]
#[ignore = "需 Tauri runtime · bash Tab 补全是 readline 交互式行为 · cargo test 无法稳定模拟"]
fn bash_07_tab_completion_runtime_only() {
    eprintln!(
        "§I.1 case 07 · bash Tab 补全 · runtime-only placeholder · \
         需 pnpm tauri:dev 手动验证 + docs/runtime-evidence/mvp-04/phase-d/shell-bash-07-tab-completion.jpg"
    );
}

/// **§I.1 case 08**：bash history 上下箭头 → 显示历史命令。
///
/// **IGNORE-runtime**：history 上下箭头是 readline 的 ESC `[A` / `[B` 序列
/// 交互响应 · cargo test 即使发 raw bytes 也无法稳定验证 history 状态。
#[test]
#[ignore = "需 Tauri runtime · bash history 上下箭头是 readline 交互式 · cargo test 无法稳定模拟"]
fn bash_08_history_arrows_runtime_only() {
    eprintln!(
        "§I.1 case 08 · bash history 上下箭头 · runtime-only placeholder · \
         需 pnpm tauri:dev 手动验证 + docs/runtime-evidence/mvp-04/phase-d/shell-bash-08-history.jpg"
    );
}

// -----------------------------------------------------------------------------
// §I.1 默认 shell 矩阵 · fish × 4
//
// fish 是 non-blocker · macOS 默认不装 · 用 locate_shell 运行时判断 ·
// 本机无 fish 则 skip(不 panic) · CI 装了 fish 则跑。
// -----------------------------------------------------------------------------

/// **§I.1 case 09**：fish 启动 → prompt 可见。
///
/// 失败处理（spec）：non-blocker · 推 v0.2。本测的 fish 缺失会 skip 而不 panic ·
/// 不阻塞 macOS-first v0.1。
#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "Linux PTY epoll timing · 沿袭 zsh_01_startup_shows_prompt"
)]
fn fish_09_startup_shows_prompt() {
    let Some(shell) = locate_shell("fish") else {
        // non-blocker · 系统未装 fish · 静默 skip
        eprintln!("§I.1 case 09 SKIP · fish not installed (non-blocker · spec § fail-rule)");
        return;
    };

    let (_dir, manager, events, tab_id, _ws) = pty_fixture(shell.to_str().unwrap());
    manager
        .stdin(
            &tab_id,
            &format!("printf '{SHELL_READY_SENTINEL}\\n'\nexit\n"),
        )
        .unwrap();

    let (output, hit) = recv_until_sentinel(
        &events,
        &tab_id,
        SHELL_READY_SENTINEL,
        Duration::from_secs(5),
    );
    assert!(hit, "fish 启动后未在 5s 内输出 sentinel · 收到：{output:?}");
}

/// **§I.1 case 10**：fish autosuggestion 灰字（输入 `ec` → 灰字 `echo`）。
///
/// **IGNORE-runtime**：autosuggestion 是 fish 实时插值显示 · 需 xterm 渲染
/// 验证灰字色（ANSI 8;5;240 风格）· cargo test 无法识别颜色语义。
#[test]
#[ignore = "需 Tauri runtime · fish autosuggestion 是 xterm 灰字渲染 · cargo test 无法识别"]
fn fish_10_autosuggestion_runtime_only() {
    eprintln!(
        "§I.1 case 10 · fish autosuggestion · runtime-only placeholder · \
         需 pnpm tauri:dev 手动验证 + docs/runtime-evidence/mvp-04/phase-d/shell-fish-10-autosuggestion.jpg"
    );
}

/// **§I.1 case 11**：fish Tab 补全（fish 风格 · 含描述）。
///
/// **IGNORE-runtime**：fish 补全弹出 paged list · 交互式。
#[test]
#[ignore = "需 Tauri runtime · fish Tab 补全是 paged-list 交互式 · cargo test 无法模拟"]
fn fish_11_tab_completion_runtime_only() {
    eprintln!(
        "§I.1 case 11 · fish Tab 补全 · runtime-only placeholder · \
         需 pnpm tauri:dev 手动验证 + docs/runtime-evidence/mvp-04/phase-d/shell-fish-11-tab-completion.jpg"
    );
}

/// **§I.1 case 12**：fish 中文 IME · UTF-8 round-trip。
#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "Linux PTY epoll timing · 沿袭 zsh_01_startup_shows_prompt"
)]
fn fish_12_chinese_ime_utf8_roundtrip() {
    let Some(shell) = locate_shell("fish") else {
        eprintln!("§I.1 case 12 SKIP · fish not installed (non-blocker)");
        return;
    };

    let (_dir, manager, events, tab_id, _ws) = pty_fixture(shell.to_str().unwrap());
    // fish 的 echo 内置不需要 -e · 用 printf 风格保持一致
    manager
        .stdin(
            &tab_id,
            &format!("printf '{CHINESE_SAMPLE}\\n'\nprintf '{SHELL_READY_SENTINEL}\\n'\nexit\n"),
        )
        .unwrap();

    let (output, _exit) = recv_until_exit(&events, &tab_id, Duration::from_secs(5));
    assert!(
        output.contains(CHINESE_SAMPLE),
        "fish UTF-8 中文 round-trip 失败 · 期望含 '{CHINESE_SAMPLE}' · 收到：{output:?}"
    );
}

// -----------------------------------------------------------------------------
// §I.2 CLI 实机矩阵 · Claude CLI × 5
//
// 全 IGNORE-runtime · 原因：
//
// 1. 启动需 login / API key / 网络 · CI 不可重复
// 2. 流式输出验证需要 sandbox 化的 fake LLM endpoint · v0.1 不实施
// 3. 残帧污染（R1 风险）需 xterm 渲染验证 · cargo test 不可见
//
// 这些 case 留作 spec §I.4 的运行态截图证据占位 ·
// `docs/runtime-evidence/mvp-04/phase-d/cli-claude-{01..05}-*.{jpg,mp4}`。
// -----------------------------------------------------------------------------

/// **§I.2 case 13**：Claude CLI 启动 + login flow · < 5s 显示 ready prompt。
#[test]
#[ignore = "需 Tauri runtime + Claude CLI login · cargo test 不可重复 · spec §I.4 手动 + 截图证据"]
fn claude_cli_13_startup_login_runtime_only() {
    eprintln!(
        "§I.2 case 13 · Claude CLI startup + login · runtime-only placeholder · \
         需 pnpm tauri:dev + 真实 Claude CLI 安装 · 截图归档 docs/runtime-evidence/mvp-04/phase-d/cli-claude-13-startup.jpg"
    );
}

/// **§I.2 case 14**：Claude CLI 输入"你好" → 流式中文回复 · 无乱码完整结束。
#[test]
#[ignore = "需 Tauri runtime + 真实 LLM endpoint · cargo test 不可重复 · spec §I.4 手动 + 截图证据"]
fn claude_cli_14_streaming_chinese_runtime_only() {
    eprintln!(
        "§I.2 case 14 · Claude CLI 流式中文 · runtime-only placeholder · \
         需 pnpm tauri:dev + login + 30s 录屏 docs/runtime-evidence/mvp-04/phase-d/cli-claude-14-streaming.mp4"
    );
}

/// **§I.2 case 15**：Claude CLI Ctrl+C 中断流式输出中途 · 无残帧污染下条 prompt。
///
/// 这是 R1 风险 · spec § 已知风险 第三条："CLI 中断残帧（SPIKE-06 A.2）"。
/// SPIKE-06 §A 已用 36 脱敏样本预验 · cargo test 层不再重复。
#[test]
#[ignore = "需 Tauri runtime · Ctrl+C 残帧污染是 xterm 渲染问题 · spec §I.4 手动 + 截图证据"]
fn claude_cli_15_ctrl_c_no_residual_runtime_only() {
    eprintln!(
        "§I.2 case 15 · Claude CLI Ctrl+C 残帧 · runtime-only placeholder · \
         R1 风险 · 需 pnpm tauri:dev + 30s 录屏 docs/runtime-evidence/mvp-04/phase-d/cli-claude-15-ctrl-c.mp4 · \
         参考 SPIKE-06 §A 36 脱敏样本"
    );
}

/// **§I.2 case 16**：Claude CLI 退出（exit / Ctrl+D）· shell 回 prompt + PTY 清理。
///
/// PTY 进程清理在 `pty.rs::PtySession::terminate` 已有单元测试覆盖
/// (`kill_force_terminates_session`) · 这里的 case 是 CLI **业务层** 的清理 ·
/// 即 Claude CLI 用户退出后对 host shell 的影响 · 仍需 runtime。
#[test]
#[ignore = "需 Tauri runtime + Claude CLI 真实退出流 · cargo test 不可重复 · spec §I.4 手动 + 截图证据"]
fn claude_cli_16_exit_returns_to_shell_runtime_only() {
    eprintln!(
        "§I.2 case 16 · Claude CLI 退出 · runtime-only placeholder · \
         需 pnpm tauri:dev · 截图 docs/runtime-evidence/mvp-04/phase-d/cli-claude-16-exit.jpg · \
         PTY 清理已被 pty.rs kill_force_terminates_session 覆盖"
    );
}

/// **§I.2 case 17**：Claude CLI 长输出（5000+ token）滚动 · scrollback 全保留。
///
/// scrollback 持久化已被 `pty_scrollback_integration.rs::stdout_is_persisted_and_forced_flush_keeps_partial_tail`
/// 覆盖 PTY → DAO 链路。本 case 的"长输出滚动"额外验证 xterm 渲染流畅 ·
/// 仍需 runtime + 帧率采样。
#[test]
#[ignore = "需 Tauri runtime · 滚动流畅是 xterm 渲染指标 · cargo test 无法采帧率 · spec §I.4 手动 + 录屏证据"]
fn claude_cli_17_long_output_scroll_runtime_only() {
    eprintln!(
        "§I.2 case 17 · Claude CLI 长输出滚动 · runtime-only placeholder · \
         需 pnpm tauri:dev · 30s 录屏 docs/runtime-evidence/mvp-04/phase-d/cli-claude-17-long-output.mp4 · \
         scrollback 持久化已被 pty_scrollback_integration.rs 覆盖"
    );
}

// -----------------------------------------------------------------------------
// §I.2 CLI 实机矩阵 · Codex CLI × 5
//
// 全 IGNORE-runtime · 同 Claude CLI 五项 · 原因略。
// -----------------------------------------------------------------------------

/// **§I.2 case 18**：Codex CLI 启动 + login flow · < 5s ready。
#[test]
#[ignore = "需 Tauri runtime + Codex CLI login · cargo test 不可重复 · spec §I.4 手动 + 截图证据"]
fn codex_cli_18_startup_login_runtime_only() {
    eprintln!(
        "§I.2 case 18 · Codex CLI startup + login · runtime-only placeholder · \
         需 pnpm tauri:dev + 真实 Codex CLI 安装 · 截图 docs/runtime-evidence/mvp-04/phase-d/cli-codex-18-startup.jpg"
    );
}

/// **§I.2 case 19**：Codex CLI 单轮对话输入/输出 · 显示正确。
#[test]
#[ignore = "需 Tauri runtime + 真实 OpenAI endpoint · cargo test 不可重复 · spec §I.4 手动 + 截图证据"]
fn codex_cli_19_single_turn_runtime_only() {
    eprintln!(
        "§I.2 case 19 · Codex CLI 单轮对话 · runtime-only placeholder · \
         需 pnpm tauri:dev + login · 截图 docs/runtime-evidence/mvp-04/phase-d/cli-codex-19-single-turn.jpg"
    );
}

/// **§I.2 case 20**：Codex CLI Ctrl+C 中断 · 无残帧。
#[test]
#[ignore = "需 Tauri runtime · 同 Claude case 15 · spec §I.4 手动 + 截图证据"]
fn codex_cli_20_ctrl_c_no_residual_runtime_only() {
    eprintln!(
        "§I.2 case 20 · Codex CLI Ctrl+C 残帧 · runtime-only placeholder · \
         同 Claude case 15 · 录屏 docs/runtime-evidence/mvp-04/phase-d/cli-codex-20-ctrl-c.mp4"
    );
}

/// **§I.2 case 21**：Codex CLI 退出 · shell 回 prompt。
#[test]
#[ignore = "需 Tauri runtime · 同 Claude case 16 · spec §I.4 手动 + 截图证据"]
fn codex_cli_21_exit_returns_to_shell_runtime_only() {
    eprintln!(
        "§I.2 case 21 · Codex CLI 退出 · runtime-only placeholder · \
         同 Claude case 16 · 截图 docs/runtime-evidence/mvp-04/phase-d/cli-codex-21-exit.jpg"
    );
}

/// **§I.2 case 22**：Codex CLI 长输出滚动 · scrollback 保留。
#[test]
#[ignore = "需 Tauri runtime · 同 Claude case 17 · spec §I.4 手动 + 录屏证据"]
fn codex_cli_22_long_output_scroll_runtime_only() {
    eprintln!(
        "§I.2 case 22 · Codex CLI 长输出滚动 · runtime-only placeholder · \
         同 Claude case 17 · 录屏 docs/runtime-evidence/mvp-04/phase-d/cli-codex-22-long-output.mp4"
    );
}
