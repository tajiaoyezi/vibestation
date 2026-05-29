use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Macos,
    Linux,
    Windows,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExternalTerminalLaunchRequest {
    pub terminal_id: String,
    pub pane_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExternalTerminalLaunchResult {
    pub success: bool,
    pub failed_reason: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LaunchError {
    #[error("unsupported external terminal combination: {terminal_id} on {platform:?}")]
    UnsupportedCombination {
        terminal_id: String,
        platform: Platform,
    },
    #[error("unknown external terminal: {0}")]
    UnknownTerminal(String),
    #[error("cwd is empty")]
    EmptyCwd,
    #[error("shell is empty")]
    EmptyShell,
}

pub fn current_platform() -> Platform {
    #[cfg(target_os = "macos")]
    {
        Platform::Macos
    }
    #[cfg(target_os = "windows")]
    {
        Platform::Windows
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Platform::Linux
    }
}

pub fn build_launch_command(
    terminal_id: &str,
    cwd: &Path,
    shell: &str,
    platform: Platform,
) -> Result<LaunchCommand, LaunchError> {
    let cwd = cwd.to_string_lossy().trim().to_string();
    if cwd.is_empty() {
        return Err(LaunchError::EmptyCwd);
    }
    let shell = shell.trim();
    if shell.is_empty() {
        return Err(LaunchError::EmptyShell);
    }

    match terminal_id {
        "ghostty" => match platform {
            Platform::Macos => Ok(LaunchCommand {
                program: "open".to_string(),
                args: vec![
                    "-a".to_string(),
                    "Ghostty".to_string(),
                    "--args".to_string(),
                    format!("--working-directory={cwd}"),
                    "-e".to_string(),
                    shell.to_string(),
                ],
            }),
            Platform::Linux => Ok(LaunchCommand {
                program: "ghostty".to_string(),
                args: vec![
                    format!("--working-directory={cwd}"),
                    "-e".to_string(),
                    shell.to_string(),
                ],
            }),
            Platform::Windows => unsupported(terminal_id, platform),
        },
        "iterm2" => match platform {
            Platform::Macos => Ok(LaunchCommand {
                program: "open".to_string(),
                args: vec!["-a".to_string(), "iTerm.app".to_string(), cwd],
            }),
            Platform::Linux | Platform::Windows => unsupported(terminal_id, platform),
        },
        "terminal-app" => match platform {
            Platform::Macos => Ok(LaunchCommand {
                program: "open".to_string(),
                args: vec!["-a".to_string(), "Terminal".to_string(), cwd],
            }),
            Platform::Linux | Platform::Windows => unsupported(terminal_id, platform),
        },
        "alacritty" => match platform {
            Platform::Macos => Ok(LaunchCommand {
                program: "open".to_string(),
                args: vec![
                    "-a".to_string(),
                    "Alacritty".to_string(),
                    "--args".to_string(),
                    "--working-directory".to_string(),
                    cwd,
                    "-e".to_string(),
                    shell.to_string(),
                ],
            }),
            Platform::Linux => Ok(LaunchCommand {
                program: "alacritty".to_string(),
                args: vec![
                    "--working-directory".to_string(),
                    cwd,
                    "-e".to_string(),
                    shell.to_string(),
                ],
            }),
            Platform::Windows => unsupported(terminal_id, platform),
        },
        "gnome-terminal" => match platform {
            Platform::Macos | Platform::Windows => unsupported(terminal_id, platform),
            Platform::Linux => Ok(LaunchCommand {
                program: "gnome-terminal".to_string(),
                args: vec![
                    format!("--working-directory={cwd}"),
                    "--".to_string(),
                    shell.to_string(),
                ],
            }),
        },
        "konsole" => match platform {
            Platform::Macos | Platform::Windows => unsupported(terminal_id, platform),
            Platform::Linux => Ok(LaunchCommand {
                program: "konsole".to_string(),
                args: vec![
                    "--workdir".to_string(),
                    cwd,
                    "-e".to_string(),
                    shell.to_string(),
                ],
            }),
        },
        "windows-terminal" => match platform {
            // wt.exe -d <cwd> <shell> · -d 设工作目录，trailing shell 作为首个 profile 命令。
            Platform::Windows => Ok(LaunchCommand {
                program: "wt.exe".to_string(),
                args: vec!["-d".to_string(), cwd, shell.to_string()],
            }),
            Platform::Macos | Platform::Linux => unsupported(terminal_id, platform),
        },
        "pwsh" => match platform {
            // pwsh.exe -NoExit -WorkingDirectory <cwd> · 进入交互式会话且不退出。
            Platform::Windows => Ok(LaunchCommand {
                program: "pwsh.exe".to_string(),
                args: vec![
                    "-NoExit".to_string(),
                    "-WorkingDirectory".to_string(),
                    cwd,
                ],
            }),
            Platform::Macos | Platform::Linux => unsupported(terminal_id, platform),
        },
        "conhost" => match platform {
            // cmd.exe /D /K cd /d <cwd> · /D 跳过 AutoRun，/K 执行后保留窗口，cd /d 切到 cwd。
            Platform::Windows => Ok(LaunchCommand {
                program: "cmd.exe".to_string(),
                args: vec![
                    "/D".to_string(),
                    "/K".to_string(),
                    format!("cd /d {cwd}"),
                ],
            }),
            Platform::Macos | Platform::Linux => unsupported(terminal_id, platform),
        },
        other => Err(LaunchError::UnknownTerminal(other.to_string())),
    }
}

fn unsupported(terminal_id: &str, platform: Platform) -> Result<LaunchCommand, LaunchError> {
    Err(LaunchError::UnsupportedCombination {
        terminal_id: terminal_id.to_string(),
        platform,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn command(terminal_id: &str, platform: Platform) -> LaunchCommand {
        build_launch_command(
            terminal_id,
            Path::new("/Users/leaf/project"),
            "/bin/zsh",
            platform,
        )
        .unwrap()
    }

    #[test]
    fn macos_ghostty_command_uses_open_with_working_directory_and_shell() {
        let command = command("ghostty", Platform::Macos);

        assert_eq!(command.program, "open");
        assert_eq!(
            command.args,
            [
                "-a",
                "Ghostty",
                "--args",
                "--working-directory=/Users/leaf/project",
                "-e",
                "/bin/zsh"
            ]
        );
    }

    #[test]
    fn macos_iterm2_command_opens_cwd_without_shell_override() {
        let command = command("iterm2", Platform::Macos);

        assert_eq!(command.program, "open");
        assert_eq!(command.args, ["-a", "iTerm.app", "/Users/leaf/project"]);
    }

    #[test]
    fn macos_terminal_app_command_opens_cwd_without_shell_override() {
        let command = command("terminal-app", Platform::Macos);

        assert_eq!(command.program, "open");
        assert_eq!(command.args, ["-a", "Terminal", "/Users/leaf/project"]);
    }

    #[test]
    fn macos_alacritty_command_uses_open_args() {
        let command = command("alacritty", Platform::Macos);

        assert_eq!(command.program, "open");
        assert_eq!(
            command.args,
            [
                "-a",
                "Alacritty",
                "--args",
                "--working-directory",
                "/Users/leaf/project",
                "-e",
                "/bin/zsh"
            ]
        );
    }

    #[test]
    fn linux_ghostty_command_uses_working_directory_and_shell() {
        let command = command("ghostty", Platform::Linux);

        assert_eq!(command.program, "ghostty");
        assert_eq!(
            command.args,
            ["--working-directory=/Users/leaf/project", "-e", "/bin/zsh"]
        );
    }

    #[test]
    fn linux_alacritty_command_uses_working_directory_and_shell() {
        let command = command("alacritty", Platform::Linux);

        assert_eq!(command.program, "alacritty");
        assert_eq!(
            command.args,
            [
                "--working-directory",
                "/Users/leaf/project",
                "-e",
                "/bin/zsh"
            ]
        );
    }

    #[test]
    fn linux_gnome_terminal_command_uses_double_dash_before_shell() {
        let command = command("gnome-terminal", Platform::Linux);

        assert_eq!(command.program, "gnome-terminal");
        assert_eq!(
            command.args,
            ["--working-directory=/Users/leaf/project", "--", "/bin/zsh"]
        );
    }

    #[test]
    fn linux_konsole_command_uses_workdir_and_shell() {
        let command = command("konsole", Platform::Linux);

        assert_eq!(command.program, "konsole");
        assert_eq!(
            command.args,
            ["--workdir", "/Users/leaf/project", "-e", "/bin/zsh"]
        );
    }

    #[test]
    fn unsupported_linux_iterm2_returns_error() {
        let result = build_launch_command(
            "iterm2",
            Path::new("/Users/leaf/project"),
            "/bin/zsh",
            Platform::Linux,
        );

        assert!(matches!(
            result,
            Err(LaunchError::UnsupportedCombination { terminal_id, platform })
                if terminal_id == "iterm2" && platform == Platform::Linux
        ));
    }

    #[test]
    fn unsupported_macos_gnome_terminal_returns_error() {
        let result = build_launch_command(
            "gnome-terminal",
            Path::new("/Users/leaf/project"),
            "/bin/zsh",
            Platform::Macos,
        );

        assert!(matches!(
            result,
            Err(LaunchError::UnsupportedCombination { terminal_id, platform })
                if terminal_id == "gnome-terminal" && platform == Platform::Macos
        ));
    }

    #[test]
    fn unknown_terminal_returns_error() {
        let result = build_launch_command(
            "kitty",
            Path::new("/Users/leaf/project"),
            "/bin/zsh",
            Platform::Linux,
        );

        assert!(matches!(result, Err(LaunchError::UnknownTerminal(id)) if id == "kitty"));
    }

    #[test]
    fn empty_shell_returns_error() {
        let result = build_launch_command(
            "ghostty",
            Path::new("/Users/leaf/project"),
            " ",
            Platform::Linux,
        );

        assert_eq!(result, Err(LaunchError::EmptyShell));
    }

    #[test]
    fn empty_cwd_returns_error() {
        let result = build_launch_command("ghostty", Path::new(""), "/bin/zsh", Platform::Linux);

        assert_eq!(result, Err(LaunchError::EmptyCwd));
    }

    #[test]
    fn paths_with_spaces_remain_single_arguments_for_all_supported_terminals() {
        let cwd = PathBuf::from("/Users/leaf/My Project");

        for (terminal_id, platform) in [
            ("ghostty", Platform::Macos),
            ("iterm2", Platform::Macos),
            ("terminal-app", Platform::Macos),
            ("alacritty", Platform::Macos),
            ("ghostty", Platform::Linux),
            ("alacritty", Platform::Linux),
            ("gnome-terminal", Platform::Linux),
            ("konsole", Platform::Linux),
        ] {
            let command = build_launch_command(terminal_id, &cwd, "/bin/zsh", platform).unwrap();

            assert!(
                command
                    .args
                    .iter()
                    .any(|arg| arg.contains("/Users/leaf/My Project")),
                "{terminal_id:?} on {platform:?} should preserve cwd as one argument"
            );
        }
    }

    #[test]
    fn paths_with_quotes_remain_single_arguments_for_all_supported_terminals() {
        let cwd = PathBuf::from("/Users/leaf/Project \"Quoted\"");

        for (terminal_id, platform) in [
            ("ghostty", Platform::Macos),
            ("iterm2", Platform::Macos),
            ("terminal-app", Platform::Macos),
            ("alacritty", Platform::Macos),
            ("ghostty", Platform::Linux),
            ("alacritty", Platform::Linux),
            ("gnome-terminal", Platform::Linux),
            ("konsole", Platform::Linux),
        ] {
            let command = build_launch_command(terminal_id, &cwd, "/bin/zsh", platform).unwrap();

            assert!(
                command
                    .args
                    .iter()
                    .any(|arg| arg.contains("Project \"Quoted\"")),
                "{terminal_id:?} on {platform:?} should not shell-escape or split quoted cwd"
            );
        }
    }

    #[test]
    fn unicode_paths_remain_single_arguments_for_all_supported_terminals() {
        let cwd = PathBuf::from("/Users/leaf/项目/终端");

        for (terminal_id, platform) in [
            ("ghostty", Platform::Macos),
            ("iterm2", Platform::Macos),
            ("terminal-app", Platform::Macos),
            ("alacritty", Platform::Macos),
            ("ghostty", Platform::Linux),
            ("alacritty", Platform::Linux),
            ("gnome-terminal", Platform::Linux),
            ("konsole", Platform::Linux),
        ] {
            let command = build_launch_command(terminal_id, &cwd, "/bin/zsh", platform).unwrap();

            assert!(
                command
                    .args
                    .iter()
                    .any(|arg| arg.contains("/Users/leaf/项目/终端")),
                "{terminal_id:?} on {platform:?} should preserve unicode cwd"
            );
        }
    }

    // TEST-3.1.2 (AC2) — Windows launch recipes.
    #[test]
    fn test_3_1_2_windows_launch_recipes() {
        let cwd = Path::new(r"C:\Users\leaf\project");

        // windows-terminal → wt.exe with cwd present.
        let wt = build_launch_command("windows-terminal", cwd, "pwsh.exe", Platform::Windows)
            .expect("windows-terminal should be supported on Windows");
        assert!(
            wt.program.contains("wt.exe"),
            "program should be wt.exe, got {}",
            wt.program
        );
        assert!(
            wt.args.iter().any(|arg| arg.contains(r"C:\Users\leaf\project")),
            "wt args should contain cwd, got {:?}",
            wt.args
        );

        // pwsh → pwsh.exe program.
        let pwsh = build_launch_command("pwsh", cwd, "pwsh.exe", Platform::Windows)
            .expect("pwsh should be supported on Windows");
        assert_eq!(pwsh.program, "pwsh.exe");
        assert!(
            pwsh.args.iter().any(|arg| arg.contains(r"C:\Users\leaf\project")),
            "pwsh args should contain cwd, got {:?}",
            pwsh.args
        );

        // conhost/cmd → cmd.exe based recipe.
        let conhost = build_launch_command("conhost", cwd, "cmd.exe", Platform::Windows)
            .expect("conhost should be supported on Windows");
        assert!(
            conhost.program.contains("cmd.exe") || conhost.program.contains("conhost"),
            "conhost recipe should be cmd.exe/conhost based, got {}",
            conhost.program
        );
        assert!(
            conhost.args.iter().any(|arg| arg.contains(r"C:\Users\leaf\project")),
            "conhost args should contain cwd, got {:?}",
            conhost.args
        );
    }

    // TEST-3.1.2 (AC2) — macOS-only terminals are unsupported on Windows.
    #[test]
    fn windows_macos_only_terminals_are_unsupported() {
        let cwd = Path::new(r"C:\Users\leaf\project");

        for terminal_id in ["iterm2", "terminal-app", "gnome-terminal", "konsole"] {
            let result = build_launch_command(terminal_id, cwd, "pwsh.exe", Platform::Windows);
            assert!(
                matches!(
                    result,
                    Err(LaunchError::UnsupportedCombination { platform, .. })
                        if platform == Platform::Windows
                ),
                "{terminal_id} should be UnsupportedCombination on Windows, got {result:?}"
            );
        }
    }

    // TEST-3.1.2 (AC2) — Windows cwd with spaces stays a single argument.
    #[test]
    fn windows_paths_with_spaces_remain_single_arguments() {
        let cwd = PathBuf::from(r"C:\Users\leaf\My Project");

        for terminal_id in ["windows-terminal", "pwsh", "conhost"] {
            let command =
                build_launch_command(terminal_id, &cwd, "pwsh.exe", Platform::Windows).unwrap();
            assert!(
                command
                    .args
                    .iter()
                    .any(|arg| arg.contains(r"C:\Users\leaf\My Project")),
                "{terminal_id} on Windows should preserve cwd with spaces as one arg, got {:?}",
                command.args
            );
        }
    }
}
