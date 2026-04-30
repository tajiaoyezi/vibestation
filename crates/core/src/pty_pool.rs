use crate::pty::{
    effective_shell_for_spawn, resolve_shell, PtyManager, PtySession, PtySpawnRequest,
};
use crossbeam_channel::{self, Sender};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use ts_rs::TS;
use uuid::Uuid;

/// MVP-20 BUG-001 修复 · IPC `tab_pty_spawn` / `pane_pty_spawn` 返回类型
///
/// `warm: true` 表示 PTY 预热池命中 · 前端必须进入 ANSI clear filter 模式
/// 隐藏 cd 注入命令的 zsh ZLE echo + syntax-highlighting 重绘 flash
/// （root cause: zsh 把 stdin 注入当 user input · ZLE 必 echo · syntax-highlighting
///  redraw 让命令行短暂可见 · clear 命令 fork 慢 ~50-100ms 用户能看到）
///
/// 前端实现：见 `web/src/panels/Terminal/PaneTerminal.tsx` 的 ANSI clear filter
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SpawnResult {
    /// true = pool 命中（前端 buffer + ANSI clear filter）
    /// false = cold spawn（前端正常 write）
    pub warm: bool,
}

pub const IDLE_MAX_AGE: Duration = Duration::from_secs(300);
const IDLE_SWEEP_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolConfig {
    pub enabled: bool,
    pub target_size: u8,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_size: 1,
        }
    }
}

pub struct PtyPool {
    idle: Arc<Mutex<VecDeque<IdlePty>>>,
    config: Arc<Mutex<PoolConfig>>,
    manager: Weak<PtyManager>,
    last_refill: Arc<Mutex<Option<(PathBuf, PathBuf)>>>,
    inflight: Arc<AtomicUsize>,
    timer_shutdown_tx: Sender<()>,
    timer_handle: Mutex<Option<JoinHandle<()>>>,
    shutdown: Arc<AtomicBool>,
}

pub struct IdlePty {
    pub session: Arc<PtySession>,
    pub shell: PathBuf,
    pub spawn_at: Instant,
    pub idle_id: String,
}

pub enum TakeResult {
    Warm(IdlePty),
    Cold,
}

impl PtyPool {
    #[must_use]
    pub fn new(manager: Arc<PtyManager>, config: PoolConfig) -> Self {
        Self::new_with_timing(manager, config, IDLE_MAX_AGE, IDLE_SWEEP_INTERVAL)
    }

    #[must_use]
    fn new_with_timing(
        manager: Arc<PtyManager>,
        config: PoolConfig,
        idle_max_age: Duration,
        sweep_interval: Duration,
    ) -> Self {
        let idle = Arc::new(Mutex::new(VecDeque::new()));
        let config = Arc::new(Mutex::new(config));
        let manager = Arc::downgrade(&manager);
        let last_refill = Arc::new(Mutex::new(None));
        let inflight = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));
        let (timer_shutdown_tx, timer_shutdown_rx) = crossbeam_channel::bounded(1);
        let timer_handle = spawn_timer_thread(TimerState {
            idle: Arc::clone(&idle),
            config: Arc::clone(&config),
            manager: Weak::clone(&manager),
            last_refill: Arc::clone(&last_refill),
            inflight: Arc::clone(&inflight),
            shutdown: Arc::clone(&shutdown),
            shutdown_rx: timer_shutdown_rx,
            idle_max_age,
            sweep_interval,
        });

        Self {
            idle,
            config,
            manager,
            last_refill,
            inflight,
            timer_shutdown_tx,
            timer_handle: Mutex::new(Some(timer_handle)),
            shutdown,
        }
    }

    pub fn take(&self, req: &PtySpawnRequest) -> TakeResult {
        if self.shutdown.load(Ordering::SeqCst) {
            return TakeResult::Cold;
        }

        if !lock(&self.config).enabled {
            return TakeResult::Cold;
        }

        let requested_shell =
            effective_shell_for_spawn(&req.shell, std::env::var("SHELL").ok().as_deref());
        let Some(resolved_shell) = resolve_shell(&requested_shell) else {
            return TakeResult::Cold;
        };

        let Some(mut idle) = self.pop_matching_idle(&resolved_shell) else {
            return TakeResult::Cold;
        };
        let Some(manager) = self.manager.upgrade() else {
            lock(&self.idle).push_front(idle);
            return TakeResult::Cold;
        };

        let old_tab_id = idle.session.tab_id_clone();
        let Ok(session) = manager.rename_session(&old_tab_id, req.tab_id.clone()) else {
            lock(&self.idle).push_front(idle);
            return TakeResult::Cold;
        };
        idle.session = session;
        if inject_cd_clear(&idle.session, Path::new(&req.cwd)).is_err() {
            if manager.rename_session(&req.tab_id, old_tab_id).is_ok() {
                idle.session.set_tab_id(idle.idle_id.clone());
                lock(&self.idle).push_front(idle);
            } else {
                let _ = manager.terminate_session(&idle.session);
            }
            return TakeResult::Cold;
        }
        TakeResult::Warm(idle)
    }

    pub fn refill_async(&self, shell: PathBuf, cwd: PathBuf) {
        if self.shutdown.load(Ordering::SeqCst) {
            return;
        }

        *lock(&self.last_refill) = Some((shell.clone(), cwd.clone()));
        self.schedule_refill(shell, cwd);
    }

    pub fn kill_all(&self) {
        let drained: Vec<IdlePty> = lock(&self.idle).drain(..).collect();
        if let Some(manager) = self.manager.upgrade() {
            for idle in drained {
                let _ = manager.terminate_session(&idle.session);
            }
        }
    }

    pub fn apply_config_change(&self, new_config: PoolConfig, current_default_shell: PathBuf) {
        let old_config = {
            let mut config = lock(&self.config);
            let old_config = *config;
            *config = new_config;
            old_config
        };

        if old_config.enabled && !new_config.enabled {
            self.kill_all();
            return;
        }

        if !old_config.enabled && new_config.enabled {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/"));
            self.refill_async(current_default_shell, home);
            return;
        }

        if old_config.target_size != new_config.target_size {
            self.set_size(new_config.target_size);
        }
    }

    pub fn handle_default_shell_change(&self, new_shell: PathBuf) {
        self.kill_all();
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        self.refill_async(new_shell, home);
    }

    pub fn set_size(&self, new_size: u8) {
        lock(&self.config).target_size = new_size;
        self.trim_to_size(new_size as usize);

        let refill = lock(&self.last_refill).clone();
        if let Some((shell, cwd)) = refill {
            self.schedule_refill(shell, cwd);
        }
    }

    #[must_use]
    pub fn idle_count(&self) -> usize {
        lock(&self.idle).len()
    }

    pub fn shutdown(&self) {
        if self.shutdown.swap(true, Ordering::SeqCst) {
            return;
        }
        let _ = self.timer_shutdown_tx.send(());
        if let Some(handle) = lock(&self.timer_handle).take() {
            let _ = handle.join();
        }
        self.kill_all();
    }

    fn pop_matching_idle(&self, shell: &PathBuf) -> Option<IdlePty> {
        let mut idle = lock(&self.idle);
        let candidate = idle.front()?;
        if candidate.shell != *shell {
            return None;
        }
        idle.pop_front()
    }

    fn trim_to_size(&self, target_size: usize) {
        let mut excess = Vec::new();
        {
            let mut idle = lock(&self.idle);
            while idle.len() > target_size {
                if let Some(session) = idle.pop_back() {
                    excess.push(session);
                }
            }
        }

        if let Some(manager) = self.manager.upgrade() {
            for idle in excess {
                let _ = manager.terminate_session(&idle.session);
            }
        }
    }

    fn schedule_refill(&self, shell: PathBuf, cwd: PathBuf) {
        schedule_refill_shared(
            Arc::clone(&self.idle),
            Arc::clone(&self.config),
            Weak::clone(&self.manager),
            Arc::clone(&self.inflight),
            Arc::clone(&self.shutdown),
            shell,
            cwd,
        );
    }
}

impl Drop for PtyPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct TimerState {
    idle: Arc<Mutex<VecDeque<IdlePty>>>,
    config: Arc<Mutex<PoolConfig>>,
    manager: Weak<PtyManager>,
    last_refill: Arc<Mutex<Option<(PathBuf, PathBuf)>>>,
    inflight: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    shutdown_rx: crossbeam_channel::Receiver<()>,
    idle_max_age: Duration,
    sweep_interval: Duration,
}

fn spawn_timer_thread(state: TimerState) -> JoinHandle<()> {
    thread::Builder::new()
        .name("vibestation-pty-pool-timer".to_string())
        .spawn(move || timer_loop(state))
        .expect("spawn vibestation-pty-pool-timer")
}

fn timer_loop(state: TimerState) {
    while !state.shutdown.load(Ordering::SeqCst) {
        match state.shutdown_rx.recv_timeout(state.sweep_interval) {
            Ok(()) | Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                expire_idle(&state);
            }
        }
    }
}

fn expire_idle(state: &TimerState) {
    let now = Instant::now();
    let mut expired = Vec::new();
    {
        let mut idle = lock(&state.idle);
        let mut kept = VecDeque::with_capacity(idle.len());
        while let Some(candidate) = idle.pop_front() {
            if now.duration_since(candidate.spawn_at) >= state.idle_max_age {
                expired.push(candidate);
            } else {
                kept.push_back(candidate);
            }
        }
        *idle = kept;
    }

    if expired.is_empty() {
        return;
    }

    if let Some(manager) = state.manager.upgrade() {
        for idle in expired {
            let _ = manager.terminate_session(&idle.session);
        }
    }

    if let Some((shell, cwd)) = lock(&state.last_refill).clone() {
        schedule_refill_shared(
            Arc::clone(&state.idle),
            Arc::clone(&state.config),
            Weak::clone(&state.manager),
            Arc::clone(&state.inflight),
            Arc::clone(&state.shutdown),
            shell,
            cwd,
        );
    }
}

fn schedule_refill_shared(
    idle: Arc<Mutex<VecDeque<IdlePty>>>,
    config: Arc<Mutex<PoolConfig>>,
    manager: Weak<PtyManager>,
    inflight: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    shell: PathBuf,
    cwd: PathBuf,
) {
    loop {
        if shutdown.load(Ordering::SeqCst) {
            return;
        }

        let config_value = *lock(&config);
        if !config_value.enabled {
            return;
        }

        let target_size = config_value.target_size as usize;
        let current = lock(&idle).len() + inflight.load(Ordering::SeqCst);
        if current >= target_size {
            return;
        }

        inflight.fetch_add(1, Ordering::SeqCst);
        spawn_refill_job_shared(
            Arc::clone(&idle),
            Arc::clone(&config),
            Weak::clone(&manager),
            Arc::clone(&inflight),
            Arc::clone(&shutdown),
            shell.clone(),
            cwd.clone(),
        );
    }
}

fn spawn_refill_job_shared(
    idle: Arc<Mutex<VecDeque<IdlePty>>>,
    config: Arc<Mutex<PoolConfig>>,
    manager: Weak<PtyManager>,
    inflight: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    shell: PathBuf,
    cwd: PathBuf,
) {
    thread::spawn(move || {
        let result = spawn_idle(manager.clone(), shell, cwd);
        if let Some(idle_pty) = result {
            if should_keep_idle(&idle, &config, &shutdown) {
                lock(&idle).push_back(idle_pty);
            } else if let Some(manager) = manager.upgrade() {
                let _ = manager.terminate_session(&idle_pty.session);
            }
        }
        inflight.fetch_sub(1, Ordering::SeqCst);
    });
}

fn spawn_idle(manager: Weak<PtyManager>, shell: PathBuf, cwd: PathBuf) -> Option<IdlePty> {
    let manager = manager.upgrade()?;
    let shell = resolve_shell(&shell.to_string_lossy())?;
    let idle_id = format!("__idle_{}__", Uuid::new_v4());
    let session = manager
        .spawn_registered_session(idle_id.clone(), shell.clone(), cwd, 80, 24)
        .ok()?;

    Some(IdlePty {
        session,
        shell,
        spawn_at: Instant::now(),
        idle_id,
    })
}

fn should_keep_idle(
    idle: &Arc<Mutex<VecDeque<IdlePty>>>,
    config: &Arc<Mutex<PoolConfig>>,
    shutdown: &Arc<AtomicBool>,
) -> bool {
    if shutdown.load(Ordering::SeqCst) {
        return false;
    }

    let config = *lock(config);
    config.enabled && lock(idle).len() < config.target_size as usize
}

/// 给 idle PTY 注入 cd + clear · 让它从 $HOME 切到目标 workspace。
/// 返回 Err 时调用方应走 cold spawn（路径不可安全 escape）。
fn inject_cd_clear(session: &PtySession, target_cwd: &Path) -> Result<(), String> {
    let cmd = cd_clear_command(target_cwd)?;
    session
        .write_input(&cmd)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn cd_clear_command(target_cwd: &Path) -> Result<String, String> {
    let path_str = target_cwd.to_string_lossy();
    let escaped = if !path_str.contains('\'') {
        format!("'{path_str}'")
    } else if !path_str.contains('"') {
        format!("\"{path_str}\"")
    } else {
        return Err("path contains both quotes".into());
    };
    Ok(format!("cd -- {escaped}; clear\n"))
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pty::{PtyEvent, PtyEventReceiver};
    use std::time::Duration;

    fn manager_and_pool(target_size: u8) -> (Arc<PtyManager>, PtyPool) {
        let manager = Arc::new(PtyManager::new());
        let pool = PtyPool::new(
            Arc::clone(&manager),
            PoolConfig {
                enabled: true,
                target_size,
            },
        );
        (manager, pool)
    }

    fn manager_and_pool_with_timing(
        target_size: u8,
        idle_max_age: Duration,
        sweep_interval: Duration,
    ) -> (Arc<PtyManager>, PtyPool) {
        let manager = Arc::new(PtyManager::new());
        let pool = PtyPool::new_with_timing(
            Arc::clone(&manager),
            PoolConfig {
                enabled: true,
                target_size,
            },
            idle_max_age,
            sweep_interval,
        );
        (manager, pool)
    }

    fn request(tab_id: &str, shell: &str) -> PtySpawnRequest {
        request_with_cwd(tab_id, shell, "/tmp")
    }

    fn request_with_cwd(tab_id: &str, shell: &str, cwd: &str) -> PtySpawnRequest {
        PtySpawnRequest {
            tab_id: tab_id.to_string(),
            shell: shell.to_string(),
            cwd: cwd.to_string(),
            cols: 80,
            rows: 24,
        }
    }

    fn first_idle_id(pool: &PtyPool) -> Option<String> {
        lock(&pool.idle).front().map(|idle| idle.idle_id.clone())
    }

    fn wait_until<F>(timeout: Duration, mut condition: F)
    where
        F: FnMut() -> bool,
    {
        let started = Instant::now();
        while started.elapsed() < timeout {
            if condition() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(condition(), "condition did not become true before timeout");
    }

    fn recv_stdout_for(events: &PtyEventReceiver, tab_id: &str, expected: &str, timeout: Duration) {
        let started = Instant::now();
        let mut output = String::new();

        while started.elapsed() < timeout {
            let remaining = timeout
                .checked_sub(started.elapsed())
                .unwrap_or_else(|| Duration::from_millis(1));
            match events.recv_timeout(remaining) {
                Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
                    output.push_str(&event.data);
                    if output.contains(expected) {
                        return;
                    }
                }
                Ok(_) => continue,
                Err(error) => panic!("event wait failed for {tab_id}: {error}"),
            }
        }

        panic!("timed out waiting for {expected:?} from {tab_id}; output={output:?}");
    }

    #[test]
    fn take_warm_hit_with_matching_shell() {
        let (_manager, pool) = manager_and_pool(1);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);

        match pool.take(&request("tab-warm", "/bin/sh")) {
            TakeResult::Warm(idle) => {
                assert_eq!(idle.session.tab_id_clone(), "tab-warm");
                assert_eq!(idle.shell, PathBuf::from("/bin/sh"));
            }
            TakeResult::Cold => panic!("expected warm PTY"),
        }
        assert_eq!(pool.idle_count(), 0);
    }

    #[test]
    fn take_cold_when_pool_empty() {
        let (_manager, pool) = manager_and_pool(1);
        assert!(matches!(
            pool.take(&request("tab-cold", "/bin/sh")),
            TakeResult::Cold
        ));
    }

    #[test]
    fn take_cold_when_shell_mismatch() {
        let (_manager, pool) = manager_and_pool(1);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);

        assert!(matches!(
            pool.take(&request("tab-mismatch", "/bin/bash")),
            TakeResult::Cold
        ));
        assert_eq!(pool.idle_count(), 1);
        pool.kill_all();
    }

    #[test]
    fn refill_async_eventually_fills_pool() {
        let (_manager, pool) = manager_and_pool(1);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);
        pool.kill_all();
    }

    #[test]
    fn set_size_grow_triggers_refill() {
        let (_manager, pool) = manager_and_pool(1);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);

        pool.set_size(2);
        wait_until(Duration::from_secs(3), || pool.idle_count() == 2);
        pool.kill_all();
    }

    #[test]
    fn set_size_shrink_kills_excess() {
        let (_manager, pool) = manager_and_pool(2);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 2);

        pool.set_size(1);
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);
        pool.kill_all();
    }

    #[test]
    fn kill_all_drains_idle() {
        let (_manager, pool) = manager_and_pool(2);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 2);

        pool.kill_all();
        assert_eq!(pool.idle_count(), 0);
    }

    #[test]
    fn idle_expire_after_max_age() {
        let (_manager, pool) =
            manager_and_pool_with_timing(1, Duration::from_millis(80), Duration::from_millis(20));
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);
        let original_idle_id = first_idle_id(&pool).expect("idle PTY should exist");

        wait_until(Duration::from_secs(3), || {
            pool.idle_count() == 1 && first_idle_id(&pool).is_some_and(|id| id != original_idle_id)
        });
        pool.kill_all();
    }

    #[test]
    fn apply_config_disable_kills_all() {
        let (_manager, pool) = manager_and_pool(2);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 2);

        pool.apply_config_change(
            PoolConfig {
                enabled: false,
                target_size: 2,
            },
            PathBuf::from("/bin/sh"),
        );

        assert_eq!(pool.idle_count(), 0);
        assert!(!lock(&pool.config).enabled);
    }

    #[test]
    fn handle_default_shell_change_kills_old_idle() {
        let (_manager, pool) = manager_and_pool(1);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);
        let old_idle_id = first_idle_id(&pool).expect("idle PTY should exist");

        pool.handle_default_shell_change(PathBuf::from("/bin/sh"));

        wait_until(Duration::from_secs(3), || {
            pool.idle_count() == 1 && first_idle_id(&pool).is_some_and(|id| id != old_idle_id)
        });
        pool.kill_all();
    }

    #[test]
    fn shutdown_drains_idle_and_blocks_refill() {
        let (_manager, pool) = manager_and_pool(1);
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);

        pool.shutdown();
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        thread::sleep(Duration::from_millis(100));

        assert_eq!(pool.idle_count(), 0);
    }

    #[test]
    fn inject_cd_clear_normal_path() {
        assert_eq!(
            cd_clear_command(Path::new("/tmp/my-project")).unwrap(),
            "cd -- '/tmp/my-project'; clear\n"
        );
    }

    #[test]
    fn inject_cd_clear_path_with_single_quote() {
        assert_eq!(
            cd_clear_command(Path::new("/tmp/it's-here")).unwrap(),
            "cd -- \"/tmp/it's-here\"; clear\n"
        );
    }

    #[test]
    fn inject_cd_clear_path_with_double_quote() {
        assert_eq!(
            cd_clear_command(Path::new("/tmp/has\"quote")).unwrap(),
            "cd -- '/tmp/has\"quote'; clear\n"
        );
    }

    #[test]
    fn inject_cd_clear_path_with_both_quotes_returns_err() {
        assert_eq!(
            cd_clear_command(Path::new("/tmp/it's\"both")).unwrap_err(),
            "path contains both quotes"
        );
    }

    #[test]
    fn take_warm_with_cwd_injects_cd() {
        let (manager, pool) = manager_and_pool(1);
        let events = manager
            .take_event_receiver()
            .expect("event receiver should be available once");
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);

        assert!(matches!(
            pool.take(&request_with_cwd("tab-cd", "/bin/sh", "/tmp")),
            TakeResult::Warm(_)
        ));
        manager.stdin("tab-cd", "pwd\nexit\n").unwrap();

        recv_stdout_for(&events, "tab-cd", "/tmp", Duration::from_secs(5));
    }

    #[test]
    fn pty_session_rename_changes_emitted_tab_id() {
        let (manager, pool) = manager_and_pool(1);
        let events = manager
            .take_event_receiver()
            .expect("event receiver should be available once");
        pool.refill_async(PathBuf::from("/bin/sh"), PathBuf::from("/tmp"));
        wait_until(Duration::from_secs(3), || pool.idle_count() == 1);

        assert!(matches!(
            pool.take(&request("tab-renamed", "/bin/sh")),
            TakeResult::Warm(_)
        ));
        manager
            .stdin("tab-renamed", "printf 'renamed-ok\\n'\nexit\n")
            .unwrap();

        recv_stdout_for(&events, "tab-renamed", "renamed-ok", Duration::from_secs(5));
    }
}
