//! Tauri WebviewWindow 生命周期管理。

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use thiserror::Error;
use uuid::Uuid;
use vibestation_core::{
    DetachedWindowBounds, PaneDetachAction, PaneDetachCloseResult, PaneDetachOpenResult,
    PaneDetachStateEvent,
};

use crate::pane_detach::state::{
    DetachError as DetachStateError, DetachedPaneMap, DetachedWindowInfo, PaneId,
};

const PANE_DETACH_STATE_CHANGED_EVENT: &str = "pane_detach_state_changed";

#[derive(Debug, Error)]
pub enum WindowManagerError {
    #[error("detach state error: {0}")]
    DetachStateError(#[from] DetachStateError),

    #[error("WebviewWindow creation failed: {reason}")]
    WindowCreationFailed { reason: String },

    #[error("WebviewWindow close failed: {reason}")]
    WindowCloseFailed { reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetachedWindowSpec {
    pub pane_id: PaneId,
    pub window_label: String,
    pub url: String,
    pub title: String,
    pub bounds: DetachedWindowBounds,
    pub registers_close_listener: bool,
}

pub fn new_detached_window_label() -> String {
    format!("pane-detach-{}", Uuid::new_v4().simple())
}

pub fn detached_window_url(pane_id: &str) -> Result<String, WindowManagerError> {
    if pane_id.is_empty() {
        return Err(WindowManagerError::WindowCreationFailed {
            reason: "pane_id is empty".to_string(),
        });
    }
    Ok(format!("index.html?mode=detached&pane={pane_id}"))
}

pub fn detached_window_spec(
    pane_id: impl Into<String>,
    window_label: impl Into<String>,
    bounds: DetachedWindowBounds,
) -> Result<DetachedWindowSpec, WindowManagerError> {
    let pane_id = pane_id.into();
    let window_label = window_label.into();
    if window_label.is_empty() {
        return Err(WindowManagerError::WindowCreationFailed {
            reason: "window_label is empty".to_string(),
        });
    }
    if bounds.width < 200 || bounds.height < 150 {
        return Err(WindowManagerError::WindowCreationFailed {
            reason: format!(
                "bounds too small: {}x{} (min 200x150)",
                bounds.width, bounds.height
            ),
        });
    }

    Ok(DetachedWindowSpec {
        url: detached_window_url(&pane_id)?,
        title: format!("Pane · {pane_id}"),
        pane_id,
        window_label,
        bounds,
        registers_close_listener: true,
    })
}

pub fn open_detached_window(
    app: &AppHandle,
    detached_panes: &DetachedPaneMap,
    pane_id: PaneId,
) -> Result<PaneDetachOpenResult, WindowManagerError> {
    let window_label = new_detached_window_label();
    let bounds = DetachedWindowBounds::default();
    let spec = detached_window_spec(pane_id.clone(), window_label.clone(), bounds.clone())?;
    let info = DetachedWindowInfo::new(window_label.clone(), "default", bounds.clone());

    detached_panes.insert(pane_id.clone(), info)?;

    let build_result =
        WebviewWindowBuilder::new(app, &window_label, WebviewUrl::App(spec.url.into()))
            .title(spec.title)
            .inner_size(bounds.width as f64, bounds.height as f64)
            .position(bounds.x as f64, bounds.y as f64)
            .resizable(true)
            .build();

    let window = match build_result {
        Ok(window) => window,
        Err(error) => {
            detached_panes.remove(&pane_id);
            return Err(WindowManagerError::WindowCreationFailed {
                reason: error.to_string(),
            });
        }
    };

    let app_for_listener = app.clone();
    let listener_label = window_label.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            if let Err(error) = handle_detached_window_destroyed(&app_for_listener, &listener_label)
            {
                eprintln!("[MVP-17] detached window destroy handler failed: {error}");
            }
        }
    });

    emit_state_event(
        app,
        &PaneDetachStateEvent {
            pane_id,
            action: PaneDetachAction::Detached,
            window_label: Some(window_label.clone()),
        },
    );

    Ok(PaneDetachOpenResult {
        window_label,
        initial_bounds: bounds,
    })
}

pub fn close_detached_window(
    app: &AppHandle,
    detached_panes: &DetachedPaneMap,
    window_label: &str,
) -> Result<PaneDetachCloseResult, WindowManagerError> {
    if window_label.is_empty() {
        return Err(WindowManagerError::WindowCloseFailed {
            reason: "window_label is empty".to_string(),
        });
    }

    let Some(event) = reattach_closed_window(detached_panes, window_label)? else {
        return Err(WindowManagerError::WindowCloseFailed {
            reason: format!("window_label {window_label} not found"),
        });
    };

    if let Some(window) = app.get_webview_window(window_label) {
        window
            .close()
            .map_err(|error| WindowManagerError::WindowCloseFailed {
                reason: error.to_string(),
            })?;
    }

    emit_state_event(app, &event);

    Ok(PaneDetachCloseResult {
        pane_id: event.pane_id,
    })
}

pub fn handle_detached_window_destroyed(
    app: &AppHandle,
    window_label: &str,
) -> Result<Option<PaneDetachStateEvent>, WindowManagerError> {
    let detached_panes = app.state::<DetachedPaneMap>();
    let event = reattach_closed_window(detached_panes.inner(), window_label)?;
    if let Some(event) = event.as_ref() {
        emit_state_event(app, event);
    }
    Ok(event)
}

pub fn reattach_closed_window(
    detached_panes: &DetachedPaneMap,
    window_label: &str,
) -> Result<Option<PaneDetachStateEvent>, WindowManagerError> {
    Ok(detached_panes
        .remove_by_label(window_label)
        .map(|(pane_id, _)| PaneDetachStateEvent {
            pane_id,
            action: PaneDetachAction::Attached,
            window_label: None,
        }))
}

fn emit_state_event(app: &AppHandle, event: &PaneDetachStateEvent) {
    if let Err(error) = app.emit(PANE_DETACH_STATE_CHANGED_EVENT, event) {
        eprintln!("[MVP-17] emit pane_detach_state_changed failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pane_detach::state::{DetachedPaneMap, DetachedWindowInfo};
    use vibestation_core::{PaneDetachAction, PaneDetachStateEvent};

    #[test]
    fn label_uuid_is_unique_and_prefixed() {
        let first = new_detached_window_label();
        let second = new_detached_window_label();

        assert!(first.starts_with("pane-detach-"));
        assert!(second.starts_with("pane-detach-"));
        assert_ne!(first, second);
    }

    #[test]
    fn window_url_targets_detached_mode_and_pane_id() {
        let url = detached_window_url("pane-1").expect("valid url");

        assert_eq!(url, "index.html?mode=detached&pane=pane-1");
    }

    #[test]
    fn window_spec_uses_default_bounds() {
        let bounds = DetachedWindowBounds::default();
        let spec =
            detached_window_spec("pane-1", "pane-detach-aaa", bounds.clone()).expect("valid spec");

        assert_eq!(spec.bounds, bounds);
        assert_eq!(spec.bounds.width, 800);
        assert_eq!(spec.bounds.height, 600);
        assert_eq!(spec.bounds.x, 40);
        assert_eq!(spec.bounds.y, 40);
    }

    #[test]
    fn window_spec_registers_destroyed_listener() {
        let bounds = DetachedWindowBounds::default();
        let spec = detached_window_spec("pane-1", "pane-detach-aaa", bounds).expect("valid spec");

        assert!(spec.registers_close_listener);
    }

    #[test]
    fn destroyed_window_removes_map_entry_and_returns_attached_event() {
        let map = DetachedPaneMap::new();
        map.insert(
            "pane-1".to_string(),
            DetachedWindowInfo::new(
                "pane-detach-aaa",
                "workspace-1",
                DetachedWindowBounds::default(),
            ),
        )
        .expect("insert");

        let event = reattach_closed_window(&map, "pane-detach-aaa")
            .expect("reattach")
            .expect("event emitted");

        assert_eq!(
            event,
            PaneDetachStateEvent {
                pane_id: "pane-1".to_string(),
                action: PaneDetachAction::Attached,
                window_label: None,
            }
        );
        assert!(map.get(&"pane-1".to_string()).is_none());
    }

    #[test]
    fn destroyed_missing_window_label_is_graceful_noop() {
        let map = DetachedPaneMap::new();

        let event = reattach_closed_window(&map, "pane-detach-missing").expect("graceful");

        assert!(event.is_none());
    }
}
