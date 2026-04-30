use crate::pty::{
    effective_shell_for_spawn, resolve_shell, PtyManager, PtySession, PtySpawnRequest,
};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;
use std::time::Instant;
use uuid::Uuid;

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
        Self {
            idle: Arc::new(Mutex::new(VecDeque::new())),
            config: Arc::new(Mutex::new(config)),
            manager: Arc::downgrade(&manager),
            last_refill: Arc::new(Mutex::new(None)),
            inflight: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn take(&self, req: &PtySpawnRequest) -> TakeResult {
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
            return TakeResult::Cold;
        };

        let old_tab_id = idle.session.tab_id_clone();
        let Ok(session) = manager.rename_session(&old_tab_id, req.tab_id.clone()) else {
            lock(&self.idle).push_front(idle);
            return TakeResult::Cold;
        };
        idle.session = session;
        TakeResult::Warm(idle)
    }

    pub fn refill_async(&self, shell: PathBuf, cwd: PathBuf) {
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
        loop {
            let config = *lock(&self.config);
            if !config.enabled {
                return;
            }

            let target_size = config.target_size as usize;
            let current = self.idle_count() + self.inflight.load(Ordering::SeqCst);
            if current >= target_size {
                return;
            }

            self.inflight.fetch_add(1, Ordering::SeqCst);
            self.spawn_refill_job(shell.clone(), cwd.clone());
        }
    }

    fn spawn_refill_job(&self, shell: PathBuf, cwd: PathBuf) {
        let idle = Arc::clone(&self.idle);
        let config = Arc::clone(&self.config);
        let manager = Weak::clone(&self.manager);
        let inflight = Arc::clone(&self.inflight);

        thread::spawn(move || {
            let result = spawn_idle(manager.clone(), shell, cwd);
            if let Some(idle_pty) = result {
                if should_keep_idle(&idle, &config) {
                    lock(&idle).push_back(idle_pty);
                } else if let Some(manager) = manager.upgrade() {
                    let _ = manager.terminate_session(&idle_pty.session);
                }
            }
            inflight.fetch_sub(1, Ordering::SeqCst);
        });
    }
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

fn should_keep_idle(idle: &Arc<Mutex<VecDeque<IdlePty>>>, config: &Arc<Mutex<PoolConfig>>) -> bool {
    let config = *lock(config);
    config.enabled && lock(idle).len() < config.target_size as usize
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

    fn request(tab_id: &str, shell: &str) -> PtySpawnRequest {
        PtySpawnRequest {
            tab_id: tab_id.to_string(),
            shell: shell.to_string(),
            cwd: "/tmp".to_string(),
            cols: 80,
            rows: 24,
        }
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
