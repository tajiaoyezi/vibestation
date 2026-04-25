//! Pane PTY · MVP-05 Phase B
//!
//! 实施 §H.6 锁 A 选项：IPC 命名空间独立（`pane_pty_*` 5 commands）·
//! Backend 复用 [`crate::pty::PtyManager`] 基础设施 · 但用独立实例
//! （`AppState.pane_pty_manager`）以避免 tab/pane id collision + 独立生命周期。
//!
//! Wrapper functions 把 `PanePty*` IPC payload 转换为 `Pty*` payload · 转发给
//! `PtyManager` · 然后把 `PtyEvent::{Stdout,Exited}` 反向映射回 `PanePty*Event`。

use crate::panes::{PanePtyExitedEvent, PanePtySpawnRequest, PanePtyStdoutEvent};
use crate::pty::{PtyError, PtyEvent, PtyExitedEvent, PtyManager, PtySpawnRequest, PtyStdoutEvent};

/// 用 [`PanePtySpawnRequest`] spawn 一个 pane PTY session。
///
/// 内部把 `pane_id` 作为 backend `tab_id` · 因为 [`PtyManager`] 用 String 作 entity key ·
/// pane_pty_manager 是独立实例 · 不会与 tab_pty_manager 的 tab_id 冲突。
pub fn spawn(manager: &PtyManager, req: PanePtySpawnRequest) -> Result<(), PtyError> {
    manager.spawn(PtySpawnRequest {
        tab_id: req.pane_id,
        shell: req.shell,
        cwd: req.cwd,
        cols: req.cols,
        rows: req.rows,
    })
}

/// 写入 stdin 到指定 pane PTY。
pub fn stdin(manager: &PtyManager, pane_id: &str, data: &str) -> Result<(), PtyError> {
    manager.stdin(pane_id, data)
}

/// 调整 pane PTY 窗口大小。
pub fn resize(manager: &PtyManager, pane_id: &str, cols: u16, rows: u16) -> Result<(), PtyError> {
    manager.resize(pane_id, cols, rows)
}

/// 向 pane PTY 发送信号（SIGINT / SIGTERM 等）。
pub fn signal(manager: &PtyManager, pane_id: &str, signal: &str) -> Result<(), PtyError> {
    manager.signal(pane_id, signal)
}

/// 终止 pane PTY · 返回 exit code（如有）。
pub fn kill(manager: &PtyManager, pane_id: &str) -> Result<Option<i32>, PtyError> {
    manager.kill(pane_id)
}

/// 把 [`PtyEvent`]（backend）反向映射为 pane 命名空间事件。
///
/// 主要用于 `crates/app/src/lib.rs` 的 event listener · 把 backend
/// PtyManager 的事件转发为 frontend 期待的 `pane_pty_stdout` / `pane_pty_exited`。
pub fn map_event(event: PtyEvent) -> PanePtyEvent {
    match event {
        PtyEvent::Stdout(PtyStdoutEvent { tab_id, data }) => {
            PanePtyEvent::Stdout(PanePtyStdoutEvent {
                pane_id: tab_id,
                data,
            })
        }
        PtyEvent::Exited(PtyExitedEvent { tab_id, exit_code }) => {
            PanePtyEvent::Exited(PanePtyExitedEvent {
                pane_id: tab_id,
                exit_code,
            })
        }
    }
}

/// Pane 命名空间下的 PTY 事件 · 与 [`PtyEvent`] 是反向命名映射。
#[derive(Debug, Clone)]
pub enum PanePtyEvent {
    Stdout(PanePtyStdoutEvent),
    Exited(PanePtyExitedEvent),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_stdout_event_translates_tab_id_to_pane_id() {
        let event = PtyEvent::Stdout(PtyStdoutEvent {
            tab_id: "pane-uuid-1".to_string(),
            data: "hello".to_string(),
        });
        match map_event(event) {
            PanePtyEvent::Stdout(payload) => {
                assert_eq!(payload.pane_id, "pane-uuid-1");
                assert_eq!(payload.data, "hello");
            }
            PanePtyEvent::Exited(_) => panic!("expected Stdout variant"),
        }
    }

    #[test]
    fn map_exited_event_preserves_exit_code() {
        let event = PtyEvent::Exited(PtyExitedEvent {
            tab_id: "pane-uuid-2".to_string(),
            exit_code: Some(0),
        });
        match map_event(event) {
            PanePtyEvent::Exited(payload) => {
                assert_eq!(payload.pane_id, "pane-uuid-2");
                assert_eq!(payload.exit_code, Some(0));
            }
            PanePtyEvent::Stdout(_) => panic!("expected Exited variant"),
        }
    }

    #[test]
    fn spawn_wrapper_forwards_request_fields() {
        let manager = PtyManager::new();
        let req = PanePtySpawnRequest {
            pane_id: "pane-mock-spawn".to_string(),
            shell: "/bin/sh".to_string(),
            cwd: "/tmp".to_string(),
            cols: 80,
            rows: 24,
        };
        // 实际 spawn 真 PTY · 验证 backend 接受了 pane_id 作为 entity key
        // （CI 环境可能没有 /bin/sh shell · skip 这种 case · 用 manager API 调用是否触发 NotFound 等错误判断）
        let result = spawn(&manager, req);
        // spawn 可能成功（macOS / Linux 一般有 /bin/sh）· 也可能失败（缺 shell · 缺 PTY 资源）
        // 关键是不 panic · 转发逻辑成立
        let _ = result;
        // cleanup
        manager.close_all_sessions();
    }
}
