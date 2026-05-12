//! Tauri WebviewWindow 生命周期管理。
//!
//! 本文件是 **session 29 Phase B skeleton**。session 30 将完成：
//! - 实际 WebviewWindow builder + 异步创建
//! - close event listener · DetachedPaneMap idempotent remove
//! - 异常关闭路径（kill -9 / IPC channel close · spec D.5）
//! - WebviewWindow bounds 实时同步（拖动后取最新 bounds）
//!
//! 当前 stub 行为：
//! - `create_detached_window` 返回 placeholder error `WindowCreationFailed { reason: "skeleton · session 30 实施" }`
//! - `close_detached_window` 返回 OK · 实际不操作 Tauri runtime
//!
//! 该 stub 让以下能力立即可用（session 29）：
//! - ts-rs binding 自动 export 到前端 · Phase C OpenCode mock IPC 接通
//! - IPC handler 在 lib.rs 注册 · permission / capability 配齐
//! - DetachedPaneMap 状态机单测全过（core 层 18 单测 + 9 export 验证）

use thiserror::Error;
use vibestation_core::{DetachError, DetachedWindowBounds};

#[derive(Debug, Error)]
pub enum WindowManagerError {
    #[error("detach state error: {0}")]
    DetachStateError(#[from] DetachError),

    #[error("WebviewWindow creation failed: {reason}")]
    WindowCreationFailed { reason: String },

    #[error("WebviewWindow close failed: {reason}")]
    WindowCloseFailed { reason: String },
}

/// 创建新的 detached WebviewWindow · skeleton 实现。
///
/// session 30 替换为：
/// ```ignore
/// use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
/// let url = WebviewUrl::App(format!("index.html?mode=detached&pane={pane_id}").into());
/// let window = WebviewWindowBuilder::new(app, &window_label, url)
///     .inner_size(bounds.width as f64, bounds.height as f64)
///     .position(bounds.x as f64, bounds.y as f64)
///     .title(format!("Pane · {pane_id}"))
///     .build()?;
/// ```
pub fn create_detached_window(
    pane_id: &str,
    window_label: &str,
    bounds: &DetachedWindowBounds,
) -> Result<(), WindowManagerError> {
    if pane_id.is_empty() {
        return Err(WindowManagerError::WindowCreationFailed {
            reason: "pane_id is empty".to_string(),
        });
    }
    if window_label.is_empty() {
        return Err(WindowManagerError::WindowCreationFailed {
            reason: "window_label is empty".to_string(),
        });
    }
    // bounds 合理性 sanity check（min 200×150 · 防 Tauri 2 builder panic）
    if bounds.width < 200 || bounds.height < 150 {
        return Err(WindowManagerError::WindowCreationFailed {
            reason: format!(
                "bounds too small: {}x{} (min 200x150)",
                bounds.width, bounds.height
            ),
        });
    }

    // session 30 替换 skeleton return
    Err(WindowManagerError::WindowCreationFailed {
        reason: "skeleton · session 30 实施 WebviewWindowBuilder".to_string(),
    })
}

/// 关闭 detached WebviewWindow · skeleton 实现。
///
/// session 30 替换为：
/// ```ignore
/// let window = app.get_webview_window(window_label)
///     .ok_or_else(|| WindowManagerError::WindowCloseFailed { reason: "window not found".into() })?;
/// window.close()?;
/// ```
pub fn close_detached_window(window_label: &str) -> Result<(), WindowManagerError> {
    if window_label.is_empty() {
        return Err(WindowManagerError::WindowCloseFailed {
            reason: "window_label is empty".to_string(),
        });
    }

    // session 30 替换 skeleton return
    // 当前返回 OK · 让 IPC handler / DetachedPaneMap remove 路径可测
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_rejects_empty_pane_id() {
        let bounds = DetachedWindowBounds::default();
        let err = create_detached_window("", "pane-detach-aaa", &bounds).unwrap_err();
        match err {
            WindowManagerError::WindowCreationFailed { reason } => {
                assert!(reason.contains("pane_id is empty"));
            }
            other => panic!("expected WindowCreationFailed · got {other:?}"),
        }
    }

    #[test]
    fn create_rejects_empty_window_label() {
        let bounds = DetachedWindowBounds::default();
        let err = create_detached_window("pane-1", "", &bounds).unwrap_err();
        match err {
            WindowManagerError::WindowCreationFailed { reason } => {
                assert!(reason.contains("window_label is empty"));
            }
            other => panic!("expected WindowCreationFailed · got {other:?}"),
        }
    }

    #[test]
    fn create_rejects_too_small_bounds() {
        let bounds = DetachedWindowBounds {
            x: 0,
            y: 0,
            width: 100, // < 200
            height: 80,
        };
        let err = create_detached_window("pane-1", "pane-detach-aaa", &bounds).unwrap_err();
        match err {
            WindowManagerError::WindowCreationFailed { reason } => {
                assert!(reason.contains("bounds too small"));
                assert!(reason.contains("100x80"));
            }
            other => panic!("expected WindowCreationFailed · got {other:?}"),
        }
    }

    #[test]
    fn create_valid_input_skeleton_returns_session_30_marker() {
        // skeleton 阶段验证：valid input 但仍返回 session 30 marker · session 30 翻成 Ok
        let bounds = DetachedWindowBounds::default();
        let err = create_detached_window("pane-1", "pane-detach-aaa", &bounds).unwrap_err();
        match err {
            WindowManagerError::WindowCreationFailed { reason } => {
                assert!(reason.contains("skeleton"));
                assert!(reason.contains("session 30"));
            }
            other => panic!("expected skeleton marker · got {other:?}"),
        }
    }

    #[test]
    fn close_rejects_empty_window_label() {
        let err = close_detached_window("").unwrap_err();
        match err {
            WindowManagerError::WindowCloseFailed { reason } => {
                assert!(reason.contains("empty"));
            }
            other => panic!("expected WindowCloseFailed · got {other:?}"),
        }
    }

    #[test]
    fn close_valid_label_returns_ok_skeleton() {
        // skeleton 阶段：close 返回 OK · session 30 替换为真实 Tauri close
        assert!(close_detached_window("pane-detach-aaa").is_ok());
    }
}
