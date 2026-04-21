//! Vibestation 核心业务逻辑
//!
//! 本 crate 包含与 UI / 桌面框架无关的业务核心：workspace 管理、git 操作、PTY、AI-aware
//! pane 联动（v1.0 vision）等。保持纯 Rust · 可独立 `cargo test` · 不依赖 Tauri。

pub mod app_settings;
pub mod config_import;
pub mod db;
pub mod layout;
pub mod pty;
pub mod tabs;
pub mod workspace;

pub use app_settings::AppSettingsStore;
pub use layout::{LayoutState, LayoutStore};
pub use pty::{
    PtyError, PtyEvent, PtyEventReceiver, PtyExitedEvent, PtyManager, PtySpawnRequest,
    PtyStdoutEvent, PTY_EVENT_QUEUE_CAPACITY,
};
pub use tabs::{
    TabCloseRequest, TabCreateRequest, TabListResponse, TabRenameRequest, TabState, TabsDao,
};
pub use workspace::{WorkspaceMetadata, WorkspaceStore};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn greet() -> &'static str {
    "Vibestation core online"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_returns_fixed_message() {
        assert_eq!(greet(), "Vibestation core online");
    }

    #[test]
    fn version_is_non_empty() {
        assert!(
            !VERSION.is_empty(),
            "CARGO_PKG_VERSION should be injected by Cargo"
        );
    }
}
