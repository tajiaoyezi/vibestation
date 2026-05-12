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
    Macos,
    Linux,
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
    mac_app_paths: &'static [&'static str],
    mac_bins: &'static [&'static str],
    linux_bins: &'static [&'static str],
    linux_desktop_markers: &'static [&'static str],
    term_program_markers: &'static [&'static str],
}

const TERMINALS: &[TerminalDefinition] = &[
    TerminalDefinition {
        id: "ghostty",
        display_name: "Ghostty",
        mac_priority: Some(80),
        linux_priority: Some(80),
        mac_app_paths: &["/Applications/Ghostty.app"],
        mac_bins: &["ghostty"],
        linux_bins: &["ghostty"],
        linux_desktop_markers: &["ghostty"],
        term_program_markers: &["ghostty"],
    },
    TerminalDefinition {
        id: "iterm2",
        display_name: "iTerm2",
        mac_priority: Some(70),
        linux_priority: None,
        mac_app_paths: &["/Applications/iTerm.app", "/Applications/iTerm2.app"],
        mac_bins: &["iterm2", "iterm"],
        linux_bins: &[],
        linux_desktop_markers: &[],
        term_program_markers: &["iterm"],
    },
    TerminalDefinition {
        id: "terminal-app",
        display_name: "Terminal.app",
        mac_priority: Some(60),
        linux_priority: None,
        mac_app_paths: &[
            "/Applications/Terminal.app",
            "/System/Applications/Utilities/Terminal.app",
        ],
        mac_bins: &["terminal"],
        linux_bins: &[],
        linux_desktop_markers: &[],
        term_program_markers: &["apple_terminal", "terminal.app"],
    },
    TerminalDefinition {
        id: "alacritty",
        display_name: "Alacritty",
        mac_priority: Some(50),
        linux_priority: Some(70),
        mac_app_paths: &["/Applications/Alacritty.app"],
        mac_bins: &["alacritty"],
        linux_bins: &["alacritty"],
        linux_desktop_markers: &["alacritty"],
        term_program_markers: &["alacritty"],
    },
    TerminalDefinition {
        id: "gnome-terminal",
        display_name: "GNOME Terminal",
        mac_priority: None,
        linux_priority: Some(60),
        mac_app_paths: &[],
        mac_bins: &[],
        linux_bins: &["gnome-terminal"],
        linux_desktop_markers: &["gnome-terminal", "gnome.terminal", "org.gnome.terminal"],
        term_program_markers: &["gnome terminal", "gnome-terminal", "gnome.terminal"],
    },
    TerminalDefinition {
        id: "konsole",
        display_name: "Konsole",
        mac_priority: None,
        linux_priority: Some(50),
        mac_app_paths: &[],
        mac_bins: &[],
        linux_bins: &["konsole"],
        linux_desktop_markers: &["konsole", "org.kde.konsole"],
        term_program_markers: &["konsole"],
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
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
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
        .flat_map(|terminal| terminal.mac_bins.iter().chain(terminal.linux_bins.iter()))
        .copied()
        .collect::<Vec<_>>();
    bins.sort_unstable();
    bins.dedup();

    bins.into_iter()
        .filter(|bin| command_exists(bin))
        .map(str::to_string)
        .collect()
}

fn command_exists(bin: &str) -> bool {
    Command::new("which")
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
}
