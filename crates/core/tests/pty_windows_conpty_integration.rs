//! task-2.2 · ConPTY spawn / echo / exit / signal-terminate 集成测试（Windows 专属）。
//!
//! 全文件 `#![cfg(windows)]` 门控（ADR-005 · Windows 专测）· 纯进程级（无 GUI 依赖）·
//! 自包含（不复用 Unix 集成测试 helper）· 经 portable-pty ConPTY 真 spawn `cmd.exe` 验证：
//! - SCEN-2.2.1 / AC1：spawn 后从 PtyEvent::Stdout 读到非空 prompt 输出（不 hang）。
//! - SCEN-2.2.2 / AC2：写 echo 命令后读到回显内容。
//! - SCEN-2.2.3 / AC3：写 `exit` 后 reader 经 child.try_wait() 检测到退出 emit Exited（不 hang）。
//! - SCEN-2.2.4 / AC4：signal("SIGTERM") / kill 经 child.kill()（TerminateProcess）终止子进程 emit Exited。
//!
//! timeout 取本地最大运行时长 ×2 余量（R-2.2-a · dispatch §2.11）· ConPTY 冷启动 + 命令执行
//! 本地观察通常 < 3s · 这里给 20s 上界防 CI/慢机 flaky；只用作"不 hang"断言上界 · 非性能门。
//!
//! ⚠️ ConPTY DSR 握手（实测根因 · 见 README/§10）：`cmd.exe` 启动后先 emit `ESC[6n`
//! （DSR · 查询光标位置）· 在收到终端回复 `ESC[<row>;<col>R` 前**不画 prompt / 不处理 stdin**·
//! 生产环境由前端 xterm.js 自动回复；headless 集成测试无前端 · 故测试 helper 必须代为回复
//! （`reply_dsr` · 模拟 VT-capable 消费端）· 否则 cmd.exe 永久 stall（非 reader bug · 是 ConPTY 语义）。
#![cfg(windows)]

use std::{
    ffi::OsString,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use vibestation_core::pty::{PtyEvent, PtyManager, PtySpawnRequest};

/// ConPTY 命令执行不 hang 的宽松上界（本地最大 ×2 余量 · R-2.2-a）。
const CONPTY_TIMEOUT: Duration = Duration::from_secs(20);

/// ConPTY DSR 查询序列：cmd.exe 启动时 emit · 等终端回复光标位置后才继续。
const DSR_QUERY: &str = "\u{1b}[6n";
/// 模拟前端回复光标在 (1,1)（ESC[1;1R）· 让 cmd.exe 越过 DSR 握手开始画 prompt。
const DSR_REPLY: &str = "\u{1b}[1;1R";

static APPDATA_ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
    _lock: MutexGuard<'static, ()>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let lock = APPDATA_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let original = std::env::var_os(key);
        std::env::set_var(key, value);
        Self {
            key,
            original,
            _lock: lock,
        }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.original {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

/// 若 stdout 段含 DSR 查询 · 代前端回复（模拟 xterm.js）· 让 cmd.exe 越过握手。
/// 回复失败容错（tab 可能已退出）· 不 panic。
fn reply_dsr_if_needed(manager: &PtyManager, tab_id: &str, data: &str) {
    if data.contains(DSR_QUERY) {
        let _ = manager.stdin(tab_id, DSR_REPLY);
    }
}

fn manager_with_events() -> (PtyManager, Receiver<PtyEvent>) {
    let manager = PtyManager::new();
    let events = manager
        .take_event_receiver()
        .expect("event receiver should be available once");
    (manager, events)
}

fn spawn_cmd(manager: &PtyManager, tab_id: &str) {
    // 用裸名 cmd.exe（task-2.1 resolve_shell 经 where.exe 解析为全路径）· cwd 用系统临时目录。
    spawn_shell(manager, tab_id, "cmd.exe");
}

fn spawn_shell(manager: &PtyManager, tab_id: &str, shell: &str) {
    let cwd = std::env::temp_dir().to_string_lossy().to_string();
    manager
        .spawn(PtySpawnRequest {
            tab_id: tab_id.to_string(),
            shell: shell.to_string(),
            cwd,
            cols: 80,
            rows: 24,
        })
        .unwrap_or_else(|e| panic!("ConPTY spawn {shell} should succeed on Windows: {e}"));
}

/// 收 Stdout 直到累计输出含 `needle`（大小写不敏感）或超时 · 返回累计输出。
/// 期间见到 DSR 查询即代前端回复（让 cmd.exe 越过握手）。超时 panic（= hang / 漏输出）。
fn recv_until_contains(
    manager: &PtyManager,
    events: &Receiver<PtyEvent>,
    tab_id: &str,
    needle: &str,
    timeout: Duration,
) -> String {
    let started = Instant::now();
    let mut output = String::new();
    let needle_lower = needle.to_ascii_lowercase();

    loop {
        if output.to_ascii_lowercase().contains(&needle_lower) {
            return output;
        }
        let remaining = timeout
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .unwrap_or_else(|| Duration::from_millis(1));
        match events.recv_timeout(remaining) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
                reply_dsr_if_needed(manager, tab_id, &event.data);
                output.push_str(&event.data);
            }
            Ok(PtyEvent::Exited(event)) if event.tab_id == tab_id => {
                // 进程提前退出 · 再检查一次累计输出
                if output.to_ascii_lowercase().contains(&needle_lower) {
                    return output;
                }
                panic!("tab {tab_id} exited before producing {needle:?} · got output: {output:?}");
            }
            Ok(_) => continue,
            Err(RecvTimeoutError::Timeout) => {
                panic!(
                    "timed out ({timeout:?}) waiting for {needle:?} from {tab_id} · got output: {output:?}"
                );
            }
            Err(RecvTimeoutError::Disconnected) => {
                panic!("event channel disconnected while waiting for {tab_id}");
            }
        }
    }
}

fn recv_for(
    manager: &PtyManager,
    events: &Receiver<PtyEvent>,
    tab_id: &str,
    dwell: Duration,
) -> String {
    let started = Instant::now();
    let mut output = String::new();

    while started.elapsed() < dwell {
        let remaining = dwell
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .unwrap_or_else(|| Duration::from_millis(1));
        match events.recv_timeout(remaining.min(Duration::from_millis(150))) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
                reply_dsr_if_needed(manager, tab_id, &event.data);
                output.push_str(&event.data);
            }
            Ok(_) => {}
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return output,
        }
    }

    output
}

/// 收事件直到 tab 的 Exited 出现或超时 · 返回 (累计 stdout, exit_code)。
/// 期间见到 DSR 查询即代前端回复。超时 panic（= hang / 漏 exit · 测试失败）。
fn recv_until_exit(
    manager: &PtyManager,
    events: &Receiver<PtyEvent>,
    tab_id: &str,
    timeout: Duration,
) -> (String, Option<i32>) {
    let started = Instant::now();
    let mut output = String::new();

    loop {
        let remaining = timeout
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .unwrap_or_else(|| Duration::from_millis(1));
        match events.recv_timeout(remaining) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
                reply_dsr_if_needed(manager, tab_id, &event.data);
                output.push_str(&event.data);
            }
            Ok(PtyEvent::Exited(event)) if event.tab_id == tab_id => {
                return (output, event.exit_code);
            }
            Ok(_) => continue,
            Err(RecvTimeoutError::Timeout) => {
                panic!("timed out ({timeout:?}) waiting for exit of {tab_id} · output: {output:?}");
            }
            Err(RecvTimeoutError::Disconnected) => {
                panic!("event channel disconnected while waiting for {tab_id}");
            }
        }
    }
}

/// 排空事件并代回复 DSR · 持续 `dwell` 时长（用于让 cmd.exe 在写 stdin 前完成 DSR 握手 + 画 prompt）。
fn pump_dsr(manager: &PtyManager, events: &Receiver<PtyEvent>, tab_id: &str, dwell: Duration) {
    let started = Instant::now();
    while started.elapsed() < dwell {
        let remaining = dwell
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .unwrap_or_else(|| Duration::from_millis(1));
        match events.recv_timeout(remaining.min(Duration::from_millis(150))) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
                reply_dsr_if_needed(manager, tab_id, &event.data);
            }
            Ok(_) => {}
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

/// TEST-2.2.1（SCEN-2.2.1 · AC1）：ConPTY spawn cmd.exe → 读到非空 prompt 输出（不 hang）。
/// "prompt" 以 cmd.exe 横幅特征 "Microsoft" 为锚（DSR 握手后必出）· 比"任意非空"更强地证明 spawn 成活。
#[test]
fn test_2_2_1_conpty_spawn_cmd_reads_prompt() {
    let (manager, events) = manager_with_events();
    spawn_cmd(&manager, "conpty-prompt");

    // 收到含 "Microsoft"（cmd.exe 版权横幅）的输出 · 期间 helper 代回复 DSR · 不 hang
    let output = recv_until_contains(
        &manager,
        &events,
        "conpty-prompt",
        "Microsoft",
        CONPTY_TIMEOUT,
    );
    assert!(
        !output.trim().is_empty(),
        "ConPTY spawn cmd.exe 应 emit 非空 prompt 输出（不 hang）"
    );
    assert!(
        output.contains("Microsoft"),
        "cmd.exe 横幅应含 'Microsoft' · got {output:?}"
    );

    let _ = manager.kill("conpty-prompt");
}

/// TEST-2.2.2（SCEN-2.2.2 · AC2）：写 echo 命令 → 读到回显内容。
#[test]
fn test_2_2_2_conpty_echo_roundtrip() {
    let (manager, events) = manager_with_events();
    spawn_cmd(&manager, "conpty-echo");

    // 先完成 DSR 握手 + 画 prompt（cmd.exe 在 prompt ready 前不处理 stdin）· 再写命令
    pump_dsr(&manager, &events, "conpty-echo", Duration::from_millis(800));
    manager
        .stdin("conpty-echo", "echo vibestation-conpty-marker\r\n")
        .expect("stdin write should succeed");

    let output = recv_until_contains(
        &manager,
        &events,
        "conpty-echo",
        "vibestation-conpty-marker",
        CONPTY_TIMEOUT,
    );
    assert!(
        output
            .to_ascii_lowercase()
            .contains("vibestation-conpty-marker"),
        "echo 回显应含 marker · got {output:?}"
    );

    let _ = manager.kill("conpty-echo");
}

/// Regression: PowerShell spawned by Vibestation must enable PSReadLine inline
/// history prediction. The test seeds a unique command in the same session,
/// then types the prefix without Enter and expects the suffix to be rendered.
#[test]
fn test_2_2_5_pwsh_history_prediction_renders_after_prefix() {
    let temp_appdata = tempfile::tempdir().expect("temp APPDATA should be created");
    let history_dir = temp_appdata
        .path()
        .join("Microsoft")
        .join("Windows")
        .join("PowerShell")
        .join("PSReadLine");
    std::fs::create_dir_all(&history_dir).expect("PSReadLine history dir should be created");
    std::fs::write(
        history_dir.join("ConsoleHost_history.txt"),
        "Write-Output VibestationPredictSeed123\r\n",
    )
    .expect("PSReadLine history fixture should be written");
    let _appdata = EnvVarGuard::set("APPDATA", temp_appdata.path());

    let (manager, events) = manager_with_events();
    spawn_shell(&manager, "conpty-pwsh-prediction", "pwsh.exe");

    pump_dsr(
        &manager,
        &events,
        "conpty-pwsh-prediction",
        Duration::from_millis(1500),
    );
    manager
        .stdin("conpty-pwsh-prediction", "Write-Output Vibe")
        .expect("stdin write should succeed");

    let output = recv_for(
        &manager,
        &events,
        "conpty-pwsh-prediction",
        Duration::from_millis(2000),
    );
    let _ = manager.kill("conpty-pwsh-prediction");

    assert!(
        output
            .to_ascii_lowercase()
            .contains("stationpredictseed123"),
        "typing a known history prefix should render the PSReadLine prediction suffix · output: {output:?}"
    );
}

/// Regression: parent dev/test wrappers may set NO_COLOR for their own logs, but
/// user PTY sessions must not inherit it because CLIs such as Claude disable
/// their ANSI color output when NO_COLOR is present.
#[test]
fn test_2_2_6_spawn_env_does_not_inherit_no_color() {
    let _no_color = EnvVarGuard::set("NO_COLOR", "1");
    let (manager, events) = manager_with_events();
    spawn_shell(&manager, "conpty-env-color", "pwsh.exe");

    let env = manager
        .environment("conpty-env-color")
        .expect("spawned ConPTY session should expose its initial environment");

    assert_eq!(env.get("TERM"), Some(&"xterm-256color".to_string()));
    assert_eq!(env.get("COLORTERM"), Some(&"truecolor".to_string()));
    assert!(
        !env.contains_key("NO_COLOR"),
        "NO_COLOR from Vibestation's parent process must not disable colors inside user terminals"
    );

    pump_dsr(
        &manager,
        &events,
        "conpty-env-color",
        Duration::from_millis(1500),
    );
    manager
        .stdin(
            "conpty-env-color",
            "Write-Output ('VIBE_MARKER_' + $env:NO_COLOR + '_END')\r\n",
        )
        .expect("stdin write should succeed");
    let output = recv_until_contains(
        &manager,
        &events,
        "conpty-env-color",
        "VIBE_MARKER__END",
        CONPTY_TIMEOUT,
    );
    let _ = manager.kill("conpty-env-color");

    assert!(
        !output.contains("VIBE_MARKER_1_END"),
        "spawned cmd.exe should not observe NO_COLOR in its process environment · output: {output:?}"
    );
}

/// TEST-2.2.3（SCEN-2.2.3 · AC3）：写 `exit` → reader 经 try_wait 检测退出 emit Exited（不 hang）。
#[test]
fn test_2_2_3_conpty_detects_process_exit_no_hang() {
    let (manager, events) = manager_with_events();
    spawn_cmd(&manager, "conpty-exit");

    // 完成 DSR 握手 + prompt · 再写 exit（否则 cmd.exe 未越过握手 · stdin 被忽略）
    pump_dsr(&manager, &events, "conpty-exit", Duration::from_millis(800));
    manager
        .stdin("conpty-exit", "exit\r\n")
        .expect("stdin write should succeed");

    let (_output, exit_code) = recv_until_exit(&manager, &events, "conpty-exit", CONPTY_TIMEOUT);
    // cmd.exe 正常 exit → 退出码应为 Some（非 signal · Windows 无 signal 概念）
    assert!(
        exit_code.is_some(),
        "cmd.exe 正常退出应有 exit_code · got {exit_code:?}"
    );
    // 不需要再 kill（已退出）· kill 容错
    let _ = manager.kill("conpty-exit");
}

/// TEST-2.2.4（SCEN-2.2.4 · AC4）：signal("SIGTERM") 经 child.kill() 终止子进程 emit Exited。
#[test]
fn test_2_2_4_signal_terminate_kills_conpty_child() {
    let (manager, events) = manager_with_events();
    spawn_cmd(&manager, "conpty-signal");

    // 让 cmd.exe 越过 DSR 握手跑起来 · 不写 exit · 保持存活
    pump_dsr(
        &manager,
        &events,
        "conpty-signal",
        Duration::from_millis(800),
    );

    // signal SIGTERM → Windows 退化为 child.kill()（TerminateProcess · 无 libc）
    manager
        .signal("conpty-signal", "SIGTERM")
        .expect("signal SIGTERM should map to child.kill on Windows");

    // signal() 只杀 child · 不 drop master（reader 仍 block conhost 管道）· 故由 kill()/terminate()
    // 走 close_master 收口 emit Exited。先 kill 触发 terminate（drop master → reader EOF → Exited）·
    // 再收 Exited（验证 signal+terminate 链路终止子进程不 hang · AC4）。
    let exit_code = manager
        .kill("conpty-signal")
        .expect("kill after signal should succeed and terminate the ConPTY child");
    let (_output, emitted) = recv_until_exit(&manager, &events, "conpty-signal", CONPTY_TIMEOUT);
    // 到达此处即证明 child 被 kill()（TerminateProcess）终止后 reader 检测退出并 emit Exited（不 hang）。
    let _ = (exit_code, emitted);
}

/// signal/kill 经 PtyManager 对未知 tab 返回错误（不 panic · 平台无关行为锁定 · 补强 AC4）。
#[test]
fn test_2_2_signal_unknown_tab_errors() {
    let (manager, _events) = manager_with_events();
    assert!(manager.signal("no-such-tab", "SIGTERM").is_err());
    assert!(manager.kill("no-such-tab").is_err());
}
