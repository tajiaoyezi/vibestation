//! MVP-20 Phase D · PTY 预热池 backend benchmark
//!
//! 自动测 cold spawn 与 warm hit 的 spawn→第一次 stdout 时延 · 不依赖 GUI / dev mode。
//! 用户 shell 来自 `$SHELL` env · 反映真实用户环境（如 macOS + zsh + omz）。
//!
//! 跑法：`cargo test --test pty_pool_bench -- --ignored --nocapture`
//!
//! 数据点对应 spec acceptance：
//! - cold_spawn_baseline → A2 baseline 数据（也跟 frontend baseline 对齐）
//! - warm_hit_with_pool → A1a 核心延迟（IPC→onData 的 backend 部分 · 不含前端 IPC overhead）
//!
//! 注意：`#[ignore]` 因为 spawn 真 shell 进程 · 在 CI 沙盒 / 受限环境可能不稳定 ·
//! 也避免 baseline 数据污染 main test 输出。手动触发跑。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vibestation_core::pty_pool::{PoolConfig, PtyPool, TakeResult};
use vibestation_core::{PtyEvent, PtyEventReceiver, PtyManager, PtySpawnRequest};

const SAMPLES: usize = 10;
const WARMUP_TIMEOUT: Duration = Duration::from_secs(10);
const SPAWN_TIMEOUT: Duration = Duration::from_secs(15);

fn user_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

fn user_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

/// 等待指定 tab_id 的第一次 stdout · 返回从某起点到 stdout 触达的时长。
/// 通过过滤 tab_id 排除 idle 期间的 stdout（idle 用 placeholder tab_id）。
fn wait_first_stdout(
    receiver: &PtyEventReceiver,
    tab_id: &str,
    start: Instant,
    timeout: Duration,
) -> Option<Duration> {
    while start.elapsed() < timeout {
        let remaining = timeout.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            return None;
        }
        match receiver.recv_timeout(remaining) {
            Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => return Some(start.elapsed()),
            Ok(_) => continue, // 非目标 tab_id（idle / 其他 sample）· 继续 drain
            Err(_) => return None,
        }
    }
    None
}

fn print_stats(label: &str, samples: &[f64]) {
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let min = sorted[0];
    let p50 = if n.is_multiple_of(2) {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };
    let p90 = sorted[(n * 9).div_ceil(10).saturating_sub(1).min(n - 1)];
    let max = sorted[n - 1];
    let mean = samples.iter().sum::<f64>() / n as f64;
    let variance = samples
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>()
        / n as f64;
    let stddev = variance.sqrt();

    println!("\n=== {} (n = {}) ===", label, n);
    for (i, x) in samples.iter().enumerate() {
        println!("  [{:>2}] {:>9.2} ms", i + 1, x);
    }
    println!("  --- 排序 ---");
    for (i, x) in sorted.iter().enumerate() {
        println!("  [{:>2}] {:>9.2} ms", i + 1, x);
    }
    println!("  --- 统计 ---");
    println!("  min:    {:>9.2} ms", min);
    println!("  P50:    {:>9.2} ms", p50);
    println!("  mean:   {:>9.2} ms", mean);
    println!("  P90:    {:>9.2} ms", p90);
    println!("  max:    {:>9.2} ms", max);
    println!("  stddev: {:>9.2} ms", stddev);
    println!();
}

/// MVP-20 cold spawn baseline · A2 acceptance 用此与 warm hit 对比
#[test]
#[ignore = "MVP-20 backend benchmark · cargo test --test pty_pool_bench -- --ignored --nocapture"]
fn cold_spawn_baseline() {
    let manager = PtyManager::new();
    let receiver = manager
        .take_event_receiver()
        .expect("event receiver available");

    let mut samples = Vec::new();
    for i in 0..SAMPLES {
        let tab_id = format!("cold-bench-{i}");
        let req = PtySpawnRequest {
            tab_id: tab_id.clone(),
            shell: user_shell(),
            cwd: "/tmp".to_string(),
            cols: 80,
            rows: 24,
        };

        let start = Instant::now();
        manager.spawn(req).expect("cold spawn ok");
        let elapsed = wait_first_stdout(&receiver, &tab_id, start, SPAWN_TIMEOUT)
            .expect("first stdout received within timeout");
        samples.push(elapsed.as_secs_f64() * 1000.0);

        // 清理：kill child · drain 残留 stdout 防污染下一 sample
        manager.kill(&tab_id).ok();
        while receiver.try_recv().is_ok() {}
    }

    print_stats("cold_spawn_baseline · backend spawn→first_stdout", &samples);
}

/// MVP-20 warm hit · A1a 核心延迟（IPC→onData backend 部分 · 不含前端 IPC overhead）
#[test]
#[ignore = "MVP-20 backend benchmark · cargo test --test pty_pool_bench -- --ignored --nocapture"]
fn warm_hit_with_pool() {
    let manager = Arc::new(PtyManager::new());
    let receiver = manager
        .take_event_receiver()
        .expect("event receiver available");
    let pool = PtyPool::new(Arc::clone(&manager), PoolConfig::default());

    let shell_path = PathBuf::from(user_shell());

    let mut samples = Vec::new();
    for i in 0..SAMPLES {
        // 触发 idle 预热 · 等 idle ready
        pool.refill_async(shell_path.clone(), user_home());

        let warmup_start = Instant::now();
        while pool.idle_count() < 1 {
            if warmup_start.elapsed() > WARMUP_TIMEOUT {
                panic!(
                    "idle pool never reached size 1 within {:?} (iter {})",
                    WARMUP_TIMEOUT, i
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        // drain idle 期间累积的 stdout（omz greeting / .zshrc 加载日志）
        // 防止 wait_first_stdout 收到 placeholder tab_id 的非匹配事件占用 timeout
        while receiver.try_recv().is_ok() {}

        let tab_id = format!("warm-bench-{i}");
        let req = PtySpawnRequest {
            tab_id: tab_id.clone(),
            shell: user_shell(),
            cwd: "/tmp".to_string(), // cwd 不同于 idle 的 $HOME · 触发 cd 注入
            cols: 80,
            rows: 24,
        };

        let start = Instant::now();
        match pool.take(&req) {
            TakeResult::Warm(_) => {}
            TakeResult::Cold => panic!("expected warm hit at iter {i} but got Cold"),
        }
        let elapsed = wait_first_stdout(&receiver, &tab_id, start, SPAWN_TIMEOUT)
            .expect("warm hit first stdout received within timeout");
        samples.push(elapsed.as_secs_f64() * 1000.0);

        manager.kill(&tab_id).ok();
        while receiver.try_recv().is_ok() {}
    }

    print_stats(
        "warm_hit_with_pool · backend take→first_stdout (含 cd 注入)",
        &samples,
    );
}

/// 直接控制 PoolConfig 模拟 settings 关 toggle 的 cold path · 验证 spec A2
#[test]
#[ignore = "MVP-20 backend benchmark · cargo test --test pty_pool_bench -- --ignored --nocapture"]
fn cold_path_with_pool_disabled() {
    let manager = Arc::new(PtyManager::new());
    let receiver = manager
        .take_event_receiver()
        .expect("event receiver available");
    let pool = PtyPool::new(
        Arc::clone(&manager),
        PoolConfig {
            enabled: false,
            target_size: 1,
        },
    );

    let mut samples = Vec::new();
    for i in 0..SAMPLES {
        let tab_id = format!("cold-disabled-{i}");
        let req = PtySpawnRequest {
            tab_id: tab_id.clone(),
            shell: user_shell(),
            cwd: "/tmp".to_string(),
            cols: 80,
            rows: 24,
        };

        // pool disable · 必返回 Cold
        match pool.take(&req) {
            TakeResult::Cold => {}
            TakeResult::Warm(_) => panic!("expected Cold at iter {i} but got Warm"),
        }

        // 走 manager.spawn cold path
        let start = Instant::now();
        manager.spawn(req).expect("cold spawn ok");
        let elapsed = wait_first_stdout(&receiver, &tab_id, start, SPAWN_TIMEOUT)
            .expect("first stdout received within timeout");
        samples.push(elapsed.as_secs_f64() * 1000.0);

        manager.kill(&tab_id).ok();
        while receiver.try_recv().is_ok() {}
    }

    print_stats(
        "cold_path_with_pool_disabled · A2 兜底等价 (pool.take 返回 Cold)",
        &samples,
    );
}
