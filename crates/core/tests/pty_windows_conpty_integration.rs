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
#![cfg(windows)]

use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, RecvTimeoutError};
use vibestation_core::pty::{PtyEvent, PtyManager, PtySpawnRequest};

/// ConPTY 命令执行不 hang 的宽松上界（本地最大 ×2 余量 · R-2.2-a）。
const CONPTY_TIMEOUT: Duration = Duration::from_secs(20);

fn manager_with_events() -> (PtyManager, Receiver<PtyEvent>) {
    let manager = PtyManager::new();
    let events = manager
        .take_event_receiver()
        .expect("event receiver should be available once");
    (manager, events)
}

fn spawn_cmd(manager: &PtyManager, tab_id: &str) {
    // 用裸名 cmd.exe（task-2.1 resolve_shell 经 where.exe 解析为全路径）· cwd 用系统临时目录。
    let cwd = std::env::temp_dir().to_string_lossy().to_string();
    manager
        .spawn(PtySpawnRequest {
            tab_id: tab_id.to_string(),
            shell: "cmd.exe".to_string(),
            cwd,
            cols: 80,
            rows: 24,
        })
        .expect("ConPTY spawn cmd.exe should succeed on Windows");
}

/// 收 Stdout 直到累计输出含 `needle`（大小写不敏感）或超时 · 返回累计输出。
/// 超时 panic（= hang / 漏输出 · 测试失败）。
fn recv_until_contains(
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
                output.push_str(&event.data);
            }
            Ok(PtyEvent::Exited(event)) if event.tab_id == tab_id => {
                // 进程提前退出 · 再检查一次累计输出
                if output.to_ascii_lowercase().contains(&needle_lower) {
                    return output;
                }
                panic!(
                    "tab {tab_id} exited before producing {needle:?} · got output: {output:?}"
                );
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

/// 收事件直到 tab 的 Exited 出现或超时 · 返回 (累计 stdout, exit_code)。
/// 超时 panic（= hang / 漏 exit · 测试失败）。
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
            .filter(|d| !d.is_zero())
            .unwrap_or_else(|| Duration::from_millis(1));
        match events.recv_timeout(remaining) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
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

/// TEST-2.2.1（SCEN-2.2.1 · AC1）：ConPTY spawn cmd.exe → 读到非空 prompt 输出（不 hang）。
#[test]
fn test_2_2_1_conpty_spawn_cmd_reads_prompt() {
    let (manager, events) = manager_with_events();
    spawn_cmd(&manager, "conpty-prompt");

    // 收第一段非空 stdout（cmd.exe 横幅 / prompt） · 不 hang
    let started = Instant::now();
    let mut got_nonempty = false;
    while started.elapsed() < CONPTY_TIMEOUT {
        let remaining = CONPTY_TIMEOUT
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .unwrap_or_else(|| Duration::from_millis(1));
        match events.recv_timeout(remaining) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == "conpty-prompt" => {
                if !event.data.trim().is_empty() {
                    got_nonempty = true;
                    break;
                }
            }
            Ok(_) => continue,
            Err(RecvTimeoutError::Timeout) => break,
            Err(RecvTimeoutError::Disconnected) => {
                panic!("event channel disconnected");
            }
        }
    }

    assert!(
        got_nonempty,
        "ConPTY spawn cmd.exe 应在 {CONPTY_TIMEOUT:?} 内 emit 非空 prompt 输出（不 hang）"
    );

    let _ = manager.kill("conpty-prompt");
}

/// TEST-2.2.2（SCEN-2.2.2 · AC2）：写 echo 命令 → 读到回显内容。
#[test]
fn test_2_2_2_conpty_echo_roundtrip() {
    let (manager, events) = manager_with_events();
    spawn_cmd(&manager, "conpty-echo");

    // 给 cmd.exe 一点冷启动时间再写命令（避免命令在 prompt ready 前被吞）
    std::thread::sleep(Duration::from_millis(500));
    manager
        .stdin("conpty-echo", "echo vibestation-conpty-marker\r\n")
        .expect("stdin write should succeed");

    let output = recv_until_contains(
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

/// TEST-2.2.3（SCEN-2.2.3 · AC3）：写 `exit` → reader 经 try_wait 检测退出 emit Exited（不 hang）。
#[test]
fn test_2_2_3_conpty_detects_process_exit_no_hang() {
    let (manager, events) = manager_with_events();
    spawn_cmd(&manager, "conpty-exit");

    std::thread::sleep(Duration::from_millis(500));
    manager
        .stdin("conpty-exit", "exit\r\n")
        .expect("stdin write should succeed");

    let (_output, exit_code) = recv_until_exit(&events, "conpty-exit", CONPTY_TIMEOUT);
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

    // 让 cmd.exe 跑起来 · 不写 exit · 保持存活
    std::thread::sleep(Duration::from_millis(500));

    // signal SIGTERM → Windows 退化为 child.kill()（TerminateProcess · 无 libc）
    manager
        .signal("conpty-signal", "SIGTERM")
        .expect("signal SIGTERM should map to child.kill on Windows");

    let (_output, _exit_code) = recv_until_exit(&events, "conpty-signal", CONPTY_TIMEOUT);
    // 到达此处即证明 child 被 kill 后 reader 检测退出并 emit Exited（不 hang）。

    let _ = manager.kill("conpty-signal");
}

/// signal/kill 经 PtyManager 对未知 tab 返回错误（不 panic · 平台无关行为锁定 · 补强 AC4）。
#[test]
fn test_2_2_signal_unknown_tab_errors() {
    let (manager, _events) = manager_with_events();
    assert!(manager.signal("no-such-tab", "SIGTERM").is_err());
    assert!(manager.kill("no-such-tab").is_err());
}
