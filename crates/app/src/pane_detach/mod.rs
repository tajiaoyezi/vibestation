//! MVP-17 Phase B · Pane Detach App 层 · Tauri WebviewWindow 生命周期
//!
//! 核心业务逻辑（IPC binding · DetachedPaneMap · 错误 enum）在 `vibestation_core::pane_detach`。
//!
//! 本模块仅包含 Tauri-specific 实现：
//! - `window_manager` · WebviewWindow 创建 / 关闭 / close listener 注册
//!
//! 实际 WebviewWindow 生命周期（Tauri 2 API · 含异步 builder + listen close
//! event）将在 session 30 完成。本 session（29）落 skeleton + IPC handler stub ·
//! 让 ts-rs binding 全 export · 前端 / Phase C OpenCode 可 mock IPC 开工。

pub mod window_manager;

#[allow(unused_imports)]
pub use window_manager::{close_detached_window, create_detached_window, WindowManagerError};
