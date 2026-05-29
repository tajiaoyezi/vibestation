use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EnvEntry {
    pub key: String,
    pub value_truncated: String,
    pub is_sensitive_redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EnvPreview {
    pub visible_entries: Vec<EnvEntry>,
    pub filtered_count: u32,
}

#[cfg(not(windows))]
pub const WHITELIST: &[&str] = &["PATH", "HOME", "LANG", "TERM", "SHELL", "USER"];

#[cfg(windows)]
pub const WHITELIST: &[&str] = &[
    "PATH",
    "LANG",
    "TERM",
    "USER",
    "COMSPEC",
    "PATHEXT",
    "USERPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
];

pub const BLACKLIST_PATTERNS: &[&str] = &[
    "KEY",
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "_PAT",
    "_API",
    "OPENAI",
    "ANTHROPIC",
    "CLAUDE",
    "GITHUB_TOKEN",
    "AWS_",
    "GCP_",
];

pub fn filter_env(env: &HashMap<String, String>) -> EnvPreview {
    let mut filtered_count = 0u32;
    let mut visible_entries = Vec::new();

    let mut keys = env.keys().collect::<Vec<_>>();
    keys.sort();

    for key in keys {
        let is_sensitive = is_sensitive_key(key);
        if is_sensitive {
            filtered_count = filtered_count.saturating_add(1);
        }

        if !is_whitelisted(key) || visible_entries.len() >= 10 {
            continue;
        }

        visible_entries.push(EnvEntry {
            key: key.clone(),
            value_truncated: if is_sensitive {
                "[redacted]".to_string()
            } else {
                truncate_value(&env[key])
            },
            is_sensitive_redacted: is_sensitive,
        });
    }

    EnvPreview {
        visible_entries,
        filtered_count,
    }
}

pub fn filtered_env_vars(env: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut keys = env.keys().collect::<Vec<_>>();
    keys.sort();

    keys.into_iter()
        .filter(|key| is_whitelisted(key) && !is_sensitive_key(key))
        .take(10)
        .map(|key| (key.clone(), env[key].clone()))
        .collect()
}

fn is_whitelisted(key: &str) -> bool {
    WHITELIST.contains(&key)
}

fn is_sensitive_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    BLACKLIST_PATTERNS
        .iter()
        .any(|pattern| upper.contains(pattern))
}

fn truncate_value(value: &str) -> String {
    let mut chars = value.chars();
    let prefix = chars.by_ref().take(40).collect::<String>();
    if chars.next().is_some() {
        format!("{prefix}...")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(entries: &[(&str, &str)]) -> HashMap<String, String> {
        entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    // Unix-fixture test (HOME / SHELL in WHITELIST) · Windows WHITELIST differs,
    // Windows coverage in `test_3_1_4_windows_env_whitelist_comspec`.
    #[cfg(not(windows))]
    #[test]
    fn whitelist_entries_pass_through() {
        let preview = filter_env(&env(&[
            ("PATH", "/usr/bin"),
            ("HOME", "/Users/leaf"),
            ("LANG", "en_US.UTF-8"),
            ("TERM", "xterm-256color"),
            ("SHELL", "/bin/zsh"),
            ("USER", "leaf"),
        ]));

        assert_eq!(preview.filtered_count, 0);
        assert_eq!(preview.visible_entries.len(), 6);
        assert!(preview
            .visible_entries
            .iter()
            .all(|entry| !entry.is_sensitive_redacted));
    }

    #[test]
    fn openai_api_key_is_counted_and_hidden() {
        let preview = filter_env(&env(&[("OPENAI_API_KEY", "sk-secret")]));

        assert_eq!(preview.filtered_count, 1);
        assert!(preview.visible_entries.is_empty());
    }

    #[test]
    fn lowercase_blacklist_is_case_insensitive() {
        let preview = filter_env(&env(&[("openai_api_key", "sk-secret")]));

        assert_eq!(preview.filtered_count, 1);
        assert!(preview.visible_entries.is_empty());
    }

    #[test]
    fn key_prefix_is_redacted_conservatively() {
        let preview = filter_env(&env(&[("KEYBOARD_LAYOUT", "us")]));

        assert_eq!(preview.filtered_count, 1);
        assert!(preview.visible_entries.is_empty());
    }

    #[test]
    fn github_token_is_counted_once() {
        let preview = filter_env(&env(&[("GITHUB_TOKEN", "ghp_secret")]));

        assert_eq!(preview.filtered_count, 1);
        assert!(preview.visible_entries.is_empty());
    }

    #[test]
    fn path_value_is_not_scanned_for_sensitive_substrings() {
        let preview = filter_env(&env(&[("PATH", "/home/user/.openai/bin")]));

        assert_eq!(preview.filtered_count, 0);
        assert_eq!(
            preview.visible_entries[0].value_truncated,
            "/home/user/.openai/bin"
        );
    }

    #[test]
    fn random_non_whitelisted_var_is_hidden_without_security_count() {
        let preview = filter_env(&env(&[("RANDOM_VAR", "foo")]));

        assert_eq!(preview.filtered_count, 0);
        assert!(preview.visible_entries.is_empty());
    }

    #[test]
    fn vendor_patterns_are_counted() {
        let preview = filter_env(&env(&[
            ("ANTHROPIC_AUTH_TOKEN", "secret"),
            ("CLAUDE_SESSION", "secret"),
            ("AWS_ACCESS_KEY_ID", "secret"),
            ("GCP_PROJECT_TOKEN", "secret"),
        ]));

        assert_eq!(preview.filtered_count, 4);
        assert!(preview.visible_entries.is_empty());
    }

    // Unix-fixture test (HOME / SHELL in WHITELIST) · Windows WHITELIST differs.
    #[cfg(not(windows))]
    #[test]
    fn visible_entries_are_limited_to_ten_and_sorted() {
        let preview = filter_env(&env(&[
            ("PATH", "/usr/bin"),
            ("HOME", "/Users/leaf"),
            ("LANG", "en_US.UTF-8"),
            ("TERM", "xterm-256color"),
            ("SHELL", "/bin/zsh"),
            ("USER", "leaf"),
        ]));

        assert_eq!(
            preview
                .visible_entries
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>(),
            ["HOME", "LANG", "PATH", "SHELL", "TERM", "USER"]
        );
    }

    #[test]
    fn long_values_are_truncated_to_forty_chars_plus_suffix() {
        let value = "a".repeat(48);
        let preview = filter_env(&env(&[("PATH", &value)]));

        assert_eq!(
            preview.visible_entries[0].value_truncated,
            format!("{}...", "a".repeat(40))
        );
    }

    // TEST-3.1.4 (AC4) — Windows WHITELIST surfaces COMSPEC / PATHEXT.
    #[cfg(windows)]
    #[test]
    fn test_3_1_4_windows_env_whitelist_comspec() {
        let preview = filter_env(&env(&[
            ("COMSPEC", r"C:\Windows\System32\cmd.exe"),
            ("PATHEXT", ".COM;.EXE;.BAT;.CMD"),
            ("USERPROFILE", r"C:\Users\leaf"),
        ]));

        let visible: Vec<&str> = preview
            .visible_entries
            .iter()
            .map(|entry| entry.key.as_str())
            .collect();
        assert!(
            visible.contains(&"COMSPEC"),
            "COMSPEC must be whitelisted on Windows, got {visible:?}"
        );
        assert!(
            visible.contains(&"PATHEXT"),
            "PATHEXT must be whitelisted on Windows, got {visible:?}"
        );
        assert_eq!(preview.filtered_count, 0);
    }

    // TEST-3.1.4 (AC4) — Unix WHITELIST keeps SHELL (zero regression).
    #[cfg(not(windows))]
    #[test]
    fn unix_env_whitelist_keeps_shell() {
        assert!(
            WHITELIST.contains(&"SHELL"),
            "SHELL must stay whitelisted on Unix"
        );

        let preview = filter_env(&env(&[("SHELL", "/bin/zsh")]));
        assert_eq!(
            preview.visible_entries.len(),
            1,
            "SHELL must remain visible on Unix"
        );
        assert_eq!(preview.visible_entries[0].key, "SHELL");
    }
}
