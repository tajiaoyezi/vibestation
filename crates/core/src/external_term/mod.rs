//! External terminal detection, command construction, and env preview.

pub mod detect;
pub mod env_filter;
pub mod launch;

pub use detect::{detect_terminals, ExternalTerminalInfo};
pub use env_filter::{filter_env, filtered_env_vars, EnvEntry, EnvPreview};
pub use launch::{
    build_launch_command, current_platform, ExternalTerminalLaunchRequest,
    ExternalTerminalLaunchResult, LaunchCommand, LaunchError, Platform,
};
