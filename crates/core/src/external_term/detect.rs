use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::process::Command;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExternalTerminalInfo {
    pub id: String,
    pub display_name: String,
    pub detected: bool,
    pub priority_hint: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetectionPlatform {
    // `Macos` is only constructed in the `#[cfg(target_os = "macos")]` branch of
    // `current_detection_platform()`; on Linux/Windows builds it is reachable only
    // via tests, so suppress dead_code off-macOS (variant kept for cross-platform match arms).
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Macos,
    Linux,
    // `Windows` is only constructed in the `#[cfg(target_os = "windows")]` branch of
    // `current_detection_platform()`; on macOS/Linux builds it is reachable only via
    // Windows-gated tests, so suppress dead_code off-Windows (variant kept for cross-platform match arms).
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    Windows,
}

#[derive(Debug, Default)]
pub(crate) struct TerminalDetectionContext {
    pub platform: Option<DetectionPlatform>,
    pub term_program: Option<String>,
    pub mac_app_paths: Vec<String>,
    pub path_bins: Vec<String>,
    pub xdg_default: Option<String>,
}

struct TerminalDefinition {
    id: &'static str,
    display_name: &'static str,
    mac_priority: Option<u8>,
    linux_priority: Option<u8>,
    windows_priority: Option<u8>,
    mac_app_paths: &'static [&'static str],
    mac_bins: &'static [&'static str],
    linux_bins: &'static [&'static str],
    windows_bins: &'static [&'static str],
    linux_desktop_markers: &'static [&'static str],
    term_program_markers: &'static [&'static str],
    /// Always present on Windows regardless of PATH probe (e.g. conhost/cmd ship with the OS).
    windows_builtin: bool,
}

const TERMINALS: &[TerminalDefinition] = &[
    TerminalDefinition {
        id: "ghostty",
        display_name: "Ghostty",
        mac_priority: Some(80),
        linux_priority: Some(80),
        windows_priority: None,
        mac_app_paths: &["/Applications/Ghostty.app"],
        mac_bins: &["ghostty"],
        linux_bins: &["ghostty"],
        windows_bins: &[],
        linux_desktop_markers: &["ghostty"],
        term_program_markers: &["ghostty"],
        windows_builtin: false,
    },
    TerminalDefinition {
        id: "iterm2",
        display_name: "iTerm2",
        mac_priority: Some(70),
        linux_priority: None,
        windows_priority: None,
        mac_app_paths: &["/Applications/iTerm.app", "/Applications/iTerm2.app"],
        mac_bins: &["iterm2", "iterm"],
        linux_bins: &[],
        windows_bins: &[],
        linux_desktop_markers: &[],
        term_program_markers: &["iterm"],
        windows_builtin: false,
    },
    TerminalDefinition {
        id: "terminal-app",
        display_name: "Terminal.app",
        mac_priority: Some(60),
        linux_priority: None,
        windows_priority: None,
        mac_app_paths: &[
            "/Applications/Terminal.app",
            "/System/Applications/Utilities/Terminal.app",
        ],
        mac_bins: &["terminal"],
        linux_bins: &[],
        windows_bins: &[],
        linux_desktop_markers: &[],
        term_program_markers: &["apple_terminal", "terminal.app"],
        windows_builtin: false,
    },
    TerminalDefinition {
        id: "alacritty",
        display_name: "Alacritty",
        mac_priority: Some(50),
        linux_priority: Some(70),
        windows_priority: None,
        mac_app_paths: &["/Applications/Alacritty.app"],
        mac_bins: &["alacritty"],
        linux_bins: &["alacritty"],
        windows_bins: &[],
        linux_desktop_markers: &["alacritty"],
        term_program_markers: &["alacritty"],
        windows_builtin: false,
    },
    TerminalDefinition {
        id: "gnome-terminal",
        display_name: "GNOME Terminal",
        mac_priority: None,
        linux_priority: Some(60),
        windows_priority: None,
        mac_app_paths: &[],
        mac_bins: &[],
        linux_bins: &["gnome-terminal"],
        windows_bins: &[],
        linux_desktop_markers: &["gnome-terminal", "gnome.terminal", "org.gnome.terminal"],
        term_program_markers: &["gnome terminal", "gnome-terminal", "gnome.terminal"],
        windows_builtin: false,
    },
    TerminalDefinition {
        id: "konsole",
        display_name: "Konsole",
        mac_priority: None,
        linux_priority: Some(50),
        windows_priority: None,
        mac_app_paths: &[],
        mac_bins: &[],
        linux_bins: &["konsole"],
        windows_bins: &[],
        linux_desktop_markers: &["konsole", "org.kde.konsole"],
        term_program_markers: &["konsole"],
        windows_builtin: false,
    },
    TerminalDefinition {
        id: "windows-terminal",
        display_name: "Windows Terminal",
        mac_priority: None,
        linux_priority: None,
        windows_priority: Some(80),
        mac_app_paths: &[],
        mac_bins: &[],
        linux_bins: &[],
        windows_bins: &["wt"],
        linux_desktop_markers: &[],
        term_program_markers: &["windows terminal", "windows-terminal", "wt"],
        windows_builtin: false,
    },
    TerminalDefinition {
        id: "pwsh",
        display_name: "PowerShell",
        mac_priority: None,
        linux_priority: None,
        windows_priority: Some(70),
        mac_app_paths: &[],
        mac_bins: &[],
        linux_bins: &[],
        windows_bins: &["pwsh"],
        linux_desktop_markers: &[],
        term_program_markers: &["pwsh", "powershell"],
        windows_builtin: false,
    },
    TerminalDefinition {
        id: "conhost",
        display_name: "Console Host (cmd)",
        mac_priority: None,
        linux_priority: None,
        windows_priority: Some(50),
        mac_app_paths: &[],
        mac_bins: &[],
        linux_bins: &[],
        windows_bins: &["conhost", "cmd"],
        linux_desktop_markers: &[],
        term_program_markers: &[],
        // conhost/cmd ships with Windows, so it is always available as a baseline.
        windows_builtin: true,
    },
];

pub fn detect_terminals() -> Vec<ExternalTerminalInfo> {
    let ctx = TerminalDetectionContext {
        platform: current_detection_platform(),
        term_program: env::var("TERM_PROGRAM").ok(),
        mac_app_paths: detect_existing_macos_apps(),
        path_bins: detect_path_bins(),
        xdg_default: detect_xdg_default_terminal(),
    };
    detect_terminals_with_context(&ctx)
}

pub(crate) fn detect_terminals_with_context(
    ctx: &TerminalDetectionContext,
) -> Vec<ExternalTerminalInfo> {
    let Some(platform) = ctx.platform else {
        return Vec::new();
    };
    let term_program = ctx
        .term_program
        .as_deref()
        .map(normalize_hint)
        .unwrap_or_default();

    let mut terminals = TERMINALS
        .iter()
        .enumerate()
        .filter_map(|(index, definition)| {
            let base_priority = match platform {
                DetectionPlatform::Macos => definition.mac_priority?,
                DetectionPlatform::Linux => definition.linux_priority?,
                DetectionPlatform::Windows => definition.windows_priority?,
            };
            let detected = match platform {
                DetectionPlatform::Macos => {
                    any_match(&ctx.mac_app_paths, definition.mac_app_paths)
                        || any_match(&ctx.path_bins, definition.mac_bins)
                }
                DetectionPlatform::Linux => {
                    any_match(&ctx.path_bins, definition.linux_bins)
                        || ctx.xdg_default.as_deref().is_some_and(|desktop| {
                            let normalized = normalize_hint(desktop);
                            definition
                                .linux_desktop_markers
                                .iter()
                                .any(|marker| normalized.contains(&normalize_hint(marker)))
                        })
                }
                DetectionPlatform::Windows => {
                    definition.windows_builtin || any_match(&ctx.path_bins, definition.windows_bins)
                }
            };

            if !detected {
                return None;
            }

            let has_hint = !term_program.is_empty()
                && definition
                    .term_program_markers
                    .iter()
                    .any(|marker| term_program.contains(&normalize_hint(marker)));
            let priority_hint = base_priority.saturating_add(if has_hint { 50 } else { 0 });
            Some((
                index,
                ExternalTerminalInfo {
                    id: definition.id.to_string(),
                    display_name: definition.display_name.to_string(),
                    detected: true,
                    priority_hint,
                },
            ))
        })
        .collect::<Vec<_>>();

    terminals.sort_by(|(left_index, left), (right_index, right)| {
        right
            .priority_hint
            .cmp(&left.priority_hint)
            .then_with(|| left_index.cmp(right_index))
    });
    terminals
        .into_iter()
        .map(|(_, terminal)| terminal)
        .collect()
}

fn current_detection_platform() -> Option<DetectionPlatform> {
    #[cfg(target_os = "macos")]
    {
        Some(DetectionPlatform::Macos)
    }
    #[cfg(target_os = "linux")]
    {
        Some(DetectionPlatform::Linux)
    }
    #[cfg(target_os = "windows")]
    {
        Some(DetectionPlatform::Windows)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

fn detect_existing_macos_apps() -> Vec<String> {
    TERMINALS
        .iter()
        .flat_map(|terminal| terminal.mac_app_paths.iter())
        .filter(|path| Path::new(path).exists())
        .map(|path| (*path).to_string())
        .collect()
}

fn detect_path_bins() -> Vec<String> {
    let mut bins = TERMINALS
        .iter()
        .flat_map(|terminal| {
            terminal
                .mac_bins
                .iter()
                .chain(terminal.linux_bins.iter())
                .chain(terminal.windows_bins.iter())
        })
        .copied()
        .collect::<Vec<_>>();
    bins.sort_unstable();
    bins.dedup();

    bins.into_iter()
        .filter(|bin| command_exists(bin))
        .map(str::to_string)
        .collect()
}

/// PATH-lookup probe binary: `where` on Windows, `which` on Unix.
fn command_exists_probe() -> &'static str {
    #[cfg(windows)]
    {
        "where"
    }
    #[cfg(not(windows))]
    {
        "which"
    }
}

fn command_exists(bin: &str) -> bool {
    Command::new(command_exists_probe())
        .arg(bin)
        .status()
        .is_ok_and(|status| status.success())
}

fn detect_xdg_default_terminal() -> Option<String> {
    if current_detection_platform() != Some(DetectionPlatform::Linux) {
        return None;
    }

    let output = Command::new("xdg-mime")
        .args(["query", "default", "x-scheme-handler/terminal"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn any_match(actual: &[String], expected: &[&str]) -> bool {
    expected.iter().any(|expected_value| {
        actual
            .iter()
            .any(|actual_value| actual_value.eq_ignore_ascii_case(expected_value))
    })
}

fn normalize_hint(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(platform: DetectionPlatform) -> TerminalDetectionContext {
        TerminalDetectionContext {
            platform: Some(platform),
            ..TerminalDetectionContext::default()
        }
    }

    fn ids(terminals: &[ExternalTerminalInfo]) -> Vec<&str> {
        terminals
            .iter()
            .map(|terminal| terminal.id.as_str())
            .collect()
    }

    #[test]
    fn macos_detects_ghostty_by_app_path() {
        let mut ctx = ctx(DetectionPlatform::Macos);
        ctx.mac_app_paths
            .push("/Applications/Ghostty.app".to_string());

        assert_eq!(ids(&detect_terminals_with_context(&ctx)), ["ghostty"]);
    }

    #[test]
    fn macos_detects_iterm2_by_which_path() {
        let mut ctx = ctx(DetectionPlatform::Macos);
        ctx.path_bins.push("iterm2".to_string());

        assert_eq!(ids(&detect_terminals_with_context(&ctx)), ["iterm2"]);
    }

    #[test]
    fn macos_detects_terminal_app_by_app_path() {
        let mut ctx = ctx(DetectionPlatform::Macos);
        ctx.mac_app_paths
            .push("/Applications/Terminal.app".to_string());

        assert_eq!(ids(&detect_terminals_with_context(&ctx)), ["terminal-app"]);
    }

    #[test]
    fn macos_detects_alacritty_by_which_path() {
        let mut ctx = ctx(DetectionPlatform::Macos);
        ctx.path_bins.push("alacritty".to_string());

        assert_eq!(ids(&detect_terminals_with_context(&ctx)), ["alacritty"]);
    }

    #[test]
    fn macos_priority_orders_ghostty_before_iterm_before_terminal_before_alacritty() {
        let mut ctx = ctx(DetectionPlatform::Macos);
        ctx.mac_app_paths = vec![
            "/Applications/Alacritty.app".to_string(),
            "/Applications/Terminal.app".to_string(),
            "/Applications/iTerm.app".to_string(),
            "/Applications/Ghostty.app".to_string(),
        ];

        assert_eq!(
            ids(&detect_terminals_with_context(&ctx)),
            ["ghostty", "iterm2", "terminal-app", "alacritty"]
        );
    }

    #[test]
    fn linux_detects_ghostty_by_which_path() {
        let mut ctx = ctx(DetectionPlatform::Linux);
        ctx.path_bins.push("ghostty".to_string());

        assert_eq!(ids(&detect_terminals_with_context(&ctx)), ["ghostty"]);
    }

    #[test]
    fn linux_detects_alacritty_by_xdg_default() {
        let mut ctx = ctx(DetectionPlatform::Linux);
        ctx.xdg_default = Some("Alacritty.desktop".to_string());

        assert_eq!(ids(&detect_terminals_with_context(&ctx)), ["alacritty"]);
    }

    #[test]
    fn linux_detects_gnome_terminal_by_which_path() {
        let mut ctx = ctx(DetectionPlatform::Linux);
        ctx.path_bins.push("gnome-terminal".to_string());

        assert_eq!(
            ids(&detect_terminals_with_context(&ctx)),
            ["gnome-terminal"]
        );
    }

    #[test]
    fn linux_detects_konsole_by_xdg_default() {
        let mut ctx = ctx(DetectionPlatform::Linux);
        ctx.xdg_default = Some("org.kde.konsole.desktop".to_string());

        assert_eq!(ids(&detect_terminals_with_context(&ctx)), ["konsole"]);
    }

    #[test]
    fn linux_priority_orders_ghostty_alacritty_gnome_konsole() {
        let mut ctx = ctx(DetectionPlatform::Linux);
        ctx.path_bins = vec![
            "konsole".to_string(),
            "gnome-terminal".to_string(),
            "alacritty".to_string(),
            "ghostty".to_string(),
        ];

        assert_eq!(
            ids(&detect_terminals_with_context(&ctx)),
            ["ghostty", "alacritty", "gnome-terminal", "konsole"]
        );
    }

    #[test]
    fn term_program_hint_promotes_iterm2_over_ghostty() {
        let mut ctx = ctx(DetectionPlatform::Macos);
        ctx.mac_app_paths = vec![
            "/Applications/Ghostty.app".to_string(),
            "/Applications/iTerm.app".to_string(),
        ];
        ctx.term_program = Some("iTerm.app".to_string());

        assert_eq!(
            ids(&detect_terminals_with_context(&ctx)),
            ["iterm2", "ghostty"]
        );
    }

    #[test]
    fn term_program_hint_mapping_covers_five_terminals() {
        let cases = [
            (
                DetectionPlatform::Linux,
                "Ghostty",
                "ghostty",
                vec!["ghostty", "alacritty"],
                None,
            ),
            (
                DetectionPlatform::Macos,
                "iTerm.app",
                "iterm2",
                vec!["ghostty", "iterm2"],
                None,
            ),
            (
                DetectionPlatform::Macos,
                "Apple_Terminal",
                "terminal-app",
                vec!["ghostty", "terminal"],
                None,
            ),
            (
                DetectionPlatform::Linux,
                "Alacritty",
                "alacritty",
                vec!["ghostty", "alacritty"],
                None,
            ),
            (
                DetectionPlatform::Linux,
                "GNOME Terminal",
                "gnome-terminal",
                vec!["ghostty", "gnome-terminal"],
                Some("org.kde.konsole.desktop"),
            ),
        ];

        for (platform, term_program, expected_id, bins, xdg_default) in cases {
            let mut ctx = ctx(platform);
            ctx.path_bins = bins.into_iter().map(str::to_string).collect();
            ctx.mac_app_paths = ctx
                .path_bins
                .iter()
                .filter_map(|bin| match bin.as_str() {
                    "ghostty" => Some("/Applications/Ghostty.app".to_string()),
                    "iterm2" => Some("/Applications/iTerm.app".to_string()),
                    "terminal" => Some("/Applications/Terminal.app".to_string()),
                    "alacritty" => Some("/Applications/Alacritty.app".to_string()),
                    _ => None,
                })
                .collect();
            ctx.xdg_default = xdg_default.map(str::to_string);
            ctx.term_program = Some(term_program.to_string());

            assert_eq!(
                detect_terminals_with_context(&ctx)
                    .first()
                    .map(|terminal| terminal.id.as_str()),
                Some(expected_id),
                "TERM_PROGRAM={term_program} should promote {expected_id}"
            );
        }
    }

    #[test]
    fn empty_detection_returns_empty_vec_without_panicking() {
        let ctx = ctx(DetectionPlatform::Linux);

        assert!(detect_terminals_with_context(&ctx).is_empty());
    }

    // TEST-3.1.1 (AC1) — Windows detect returns non-empty list with wt / pwsh.
    #[test]
    fn test_3_1_1_windows_detect_returns_wt_pwsh() {
        let mut ctx = ctx(DetectionPlatform::Windows);
        ctx.path_bins = vec!["wt".to_string(), "pwsh".to_string()];

        let result = detect_terminals_with_context(&ctx);
        let detected = ids(&result);
        assert!(!detected.is_empty(), "Windows detection must be non-empty");
        assert!(
            detected.contains(&"windows-terminal"),
            "expected windows-terminal in {detected:?}"
        );
        assert!(detected.contains(&"pwsh"), "expected pwsh in {detected:?}");
    }

    // TEST-3.1.1 (AC1) — conhost is a built-in baseline always detected on Windows.
    #[test]
    fn windows_conhost_is_detected_as_builtin_baseline() {
        let ctx = ctx(DetectionPlatform::Windows);

        // No path_bins injected; conhost/cmd ships with Windows so it is always present.
        let result = detect_terminals_with_context(&ctx);
        let detected = ids(&result);
        assert!(
            detected.contains(&"conhost"),
            "conhost must always be detected on Windows, got {detected:?}"
        );
    }

    // TEST-3.1.1 (AC1) — a not-installed terminal must not appear.
    #[test]
    fn windows_uninstalled_terminal_is_absent() {
        let ctx = ctx(DetectionPlatform::Windows);

        // pwsh not in path_bins => must not be reported as detected.
        let result = detect_terminals_with_context(&ctx);
        let detected = ids(&result);
        assert!(
            !detected.contains(&"pwsh"),
            "pwsh must be absent when not installed, got {detected:?}"
        );
    }

    // TEST-3.1.5 (AC5) — command_exists probe selection: where on Windows, which on Unix.
    #[test]
    fn test_3_1_5_command_exists_platform_probe() {
        assert_eq!(
            command_exists_probe(),
            expected_probe_for_current_platform()
        );
    }

    #[cfg(windows)]
    fn expected_probe_for_current_platform() -> &'static str {
        "where"
    }

    #[cfg(not(windows))]
    fn expected_probe_for_current_platform() -> &'static str {
        "which"
    }
}
