//! Local PATH fixer shim.
//!
//! Tauri 文档提到 `fix-path-env-rs`，但当前 registry 里无法解析到对应 crate。
//! 这里保留同名 `fix_path_env::fix()` API，行为只做 PATH 修复，不扩散到 async runtime。

use std::io;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::env;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::process::Command;

pub fn fix() -> io::Result<()> {
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Ok(())
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let shell = resolved_shell(env::var("SHELL").ok().as_deref());
        let shell_name = shell.rsplit('/').next().unwrap_or("sh");
        let mut command = Command::new(&shell);
        command.args(shell_command_args(shell_name));

        let output = command.output()?;
        if !output.status.success() {
            return Ok(());
        }

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            env::set_var("PATH", path);
        }
        env::set_var("SHELL", &shell);

        Ok(())
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn resolved_shell(shell: Option<&str>) -> String {
    shell
        .map(str::trim)
        .filter(|shell| !shell.is_empty())
        .unwrap_or(default_shell())
        .to_string()
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn shell_command_args(shell_name: &str) -> &'static [&'static str] {
    match shell_name {
        "bash" | "zsh" | "fish" => &["-l", "-c", "printf %s \"$PATH\""],
        _ => &["-c", "printf %s \"$PATH\""],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_shell_prefers_non_empty_env_value() {
        assert_eq!(
            resolved_shell(Some("/opt/homebrew/bin/fish")),
            "/opt/homebrew/bin/fish"
        );
    }

    #[test]
    fn resolved_shell_falls_back_for_empty_env_value() {
        assert_eq!(resolved_shell(Some("   ")), default_shell());
        assert_eq!(resolved_shell(None), default_shell());
    }

    #[test]
    fn fish_uses_login_shell_for_path_fix() {
        assert_eq!(
            shell_command_args("fish"),
            ["-l", "-c", "printf %s \"$PATH\""]
        );
    }

    #[test]
    fn sh_uses_non_login_shell_for_path_fix() {
        assert_eq!(shell_command_args("sh"), ["-c", "printf %s \"$PATH\""]);
    }
}
