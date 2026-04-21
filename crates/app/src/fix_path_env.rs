//! Local PATH fixer shim.
//!
//! Tauri 文档提到 `fix-path-env-rs`，但当前 registry 里无法解析到对应 crate。
//! 这里保留同名 `fix_path_env::fix()` API，行为只做 PATH 修复，不扩散到 async runtime。

use std::env;
use std::io;
use std::process::Command;

pub fn fix() -> io::Result<()> {
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        return Ok(());
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let shell = env::var("SHELL").unwrap_or_else(|_| default_shell().to_string());
        let shell_name = shell.rsplit('/').next().unwrap_or("sh");
        let mut command = Command::new(&shell);

        match shell_name {
            "bash" | "zsh" => {
                command.args(["-l", "-c", "printf %s \"$PATH\""]);
            }
            _ => {
                command.args(["-c", "printf %s \"$PATH\""]);
            }
        }

        let output = command.output()?;
        if !output.status.success() {
            return Ok(());
        }

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            env::set_var("PATH", path);
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn default_shell() -> &'static str {
    "/bin/zsh"
}

#[cfg(all(not(target_os = "macos"), target_os = "linux"))]
fn default_shell() -> &'static str {
    "/bin/bash"
}
