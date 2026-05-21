//! MVP-18 prompt sanitization boundary.
//!
//! This module treats pane failure text as untrusted display/input text only. It never
//! executes shell metacharacters; callers may append the sanitized text to a draft,
//! but execution stays outside this layer.

use thiserror::Error;

pub const MAX_AI_PROMPT_BYTES: usize = 8 * 1024;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SanitizeCtx {
    pub workspace_root: Option<String>,
}

impl SanitizeCtx {
    #[must_use]
    pub fn new(workspace_root: impl Into<String>) -> Self {
        Self {
            workspace_root: Some(workspace_root.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedPrompt {
    pub text: String,
    pub redaction_count: usize,
    pub truncated: bool,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SanitizeError {
    #[error("prompt contains a NUL byte")]
    NulByte,
}

pub fn sanitize_ai_prompt(raw: &str, ctx: &SanitizeCtx) -> Result<SanitizedPrompt, SanitizeError> {
    if raw.as_bytes().contains(&0) {
        return Err(SanitizeError::NulByte);
    }

    let stripped = strip_ansi_and_osc(raw);
    let (url_redacted, url_redactions) = redact_url_credentials(&stripped);
    let (secret_redacted, secret_redactions) = redact_secret_patterns(&url_redacted);
    let path_sanitized = sanitize_absolute_paths(&secret_redacted, ctx);
    let (text, truncated) = truncate_utf8(&path_sanitized, MAX_AI_PROMPT_BYTES);

    Ok(SanitizedPrompt {
        text,
        redaction_count: url_redactions + secret_redactions,
        truncated,
    })
}

fn strip_ansi_and_osc(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == 0x1b {
            if i + 1 >= bytes.len() {
                i += 1;
                continue;
            }

            match bytes[i + 1] {
                b'[' => {
                    i += 2;
                    while i < bytes.len() && !is_ansi_final_byte(bytes[i]) {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                    continue;
                }
                b']' | b'P' | b'^' | b'_' => {
                    i = skip_terminated_control_string(bytes, i + 2);
                    continue;
                }
                _ => {
                    i += 1;
                    continue;
                }
            }
        }

        out.push(bytes[i]);
        i += 1;
    }

    String::from_utf8(out).unwrap_or_default()
}

fn is_ansi_final_byte(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte)
}

fn skip_terminated_control_string(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        if bytes[i] == 0x07 {
            return i + 1;
        }
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
            return i + 2;
        }
        i += 1;
    }
    i
}

fn redact_url_credentials(input: &str) -> (String, usize) {
    map_non_whitespace_tokens(input, |token| {
        let Some(scheme_end) = token.find("://") else {
            return (token.to_string(), 0);
        };

        let authority_start = scheme_end + 3;
        let authority_end = token[authority_start..]
            .find(['/', '?', '#'])
            .map_or(token.len(), |offset| authority_start + offset);
        let authority = &token[authority_start..authority_end];

        if let Some(at_offset) = authority.find('@') {
            let userinfo = &authority[..at_offset];
            if userinfo.contains(':') {
                let host_start = authority_start + at_offset + 1;
                let redacted = format!(
                    "{}<REDACTED>@{}",
                    &token[..authority_start],
                    &token[host_start..]
                );
                return (redacted, 1);
            }
        }

        (token.to_string(), 0)
    })
}

fn redact_secret_patterns(input: &str) -> (String, usize) {
    let (assignments_redacted, assignment_count) = redact_sensitive_assignments(input);
    let (tokens_redacted, token_count) = redact_standalone_secret_tokens(&assignments_redacted);
    (tokens_redacted, assignment_count + token_count)
}

fn redact_sensitive_assignments(input: &str) -> (String, usize) {
    let mut out = String::with_capacity(input.len());
    let mut count = 0;
    let mut i = 0;

    while i < input.len() {
        let Some(ch) = input[i..].chars().next() else {
            break;
        };

        if is_identifier_start(ch) {
            let key_start = i;
            let mut key_end = i + ch.len_utf8();
            while key_end < input.len() {
                let Some(next) = input[key_end..].chars().next() else {
                    break;
                };
                if !is_identifier_continue(next) {
                    break;
                }
                key_end += next.len_utf8();
            }

            let key = &input[key_start..key_end];
            if is_sensitive_key(key) {
                let mut cursor = key_end;
                while cursor < input.len() {
                    let Some(next) = input[cursor..].chars().next() else {
                        break;
                    };
                    if next != ' ' && next != '\t' {
                        break;
                    }
                    cursor += next.len_utf8();
                }

                if cursor < input.len() {
                    let sep = input[cursor..].chars().next().expect("checked cursor");
                    if sep == '=' || sep == ':' {
                        cursor += sep.len_utf8();
                        while cursor < input.len() {
                            let Some(next) = input[cursor..].chars().next() else {
                                break;
                            };
                            if next != ' ' && next != '\t' {
                                break;
                            }
                            cursor += next.len_utf8();
                        }

                        let value_start = cursor;
                        let (value_end, has_value) = consume_secret_value(input, value_start);
                        if has_value {
                            out.push_str(&input[key_start..value_start]);
                            out.push_str("<REDACTED>");
                            count += 1;
                            i = value_end;
                            continue;
                        }
                    }
                }
            }

            out.push_str(key);
            i = key_end;
            continue;
        }

        out.push(ch);
        i += ch.len_utf8();
    }

    (out, count)
}

fn consume_secret_value(input: &str, start: usize) -> (usize, bool) {
    if start >= input.len() {
        return (start, false);
    }

    let first = input[start..].chars().next().expect("checked start");
    if first == '"' || first == '\'' {
        let mut cursor = start + first.len_utf8();
        while cursor < input.len() {
            let Some(ch) = input[cursor..].chars().next() else {
                break;
            };
            cursor += ch.len_utf8();
            if ch == first {
                return (cursor, true);
            }
        }
        return (cursor, cursor > start + first.len_utf8());
    }

    let mut cursor = start;
    while cursor < input.len() {
        let Some(ch) = input[cursor..].chars().next() else {
            break;
        };
        if ch.is_whitespace() || matches!(ch, ',' | ';') {
            break;
        }
        cursor += ch.len_utf8();
    }

    (cursor, cursor > start)
}

fn redact_standalone_secret_tokens(input: &str) -> (String, usize) {
    map_non_whitespace_tokens(input, |token| {
        let leading_len = token
            .find(|ch: char| ch.is_ascii_alphanumeric() || ch == '<')
            .unwrap_or(token.len());
        let trailing_len = token
            .rfind(|ch: char| ch.is_ascii_alphanumeric() || ch == '>' || ch == '_')
            .map_or(0, |idx| token.len() - idx - 1);
        let core_end = token.len().saturating_sub(trailing_len);
        let core = &token[leading_len..core_end];

        if looks_like_secret_token(core) {
            let redacted = format!("{}<REDACTED>{}", &token[..leading_len], &token[core_end..]);
            return (redacted, 1);
        }

        (token.to_string(), 0)
    })
}

fn looks_like_secret_token(token: &str) -> bool {
    if token == "<REDACTED>" {
        return false;
    }

    let lower = token.to_ascii_lowercase();
    (lower.starts_with("sk-") && token.len() >= 12)
        || (lower.starts_with("ghp_") && token.len() >= 12)
        || (lower.starts_with("github_pat_") && token.len() >= 20)
        || (lower.starts_with("xoxb-") && token.len() >= 16)
        || (token.starts_with("AKIA") && token.len() >= 16)
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}

fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("passwd")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("private_key")
        || lower.contains("access_key")
        || lower == "auth"
        || lower.ends_with("_auth")
}

fn sanitize_absolute_paths(input: &str, ctx: &SanitizeCtx) -> String {
    map_non_whitespace_tokens(input, |token| (sanitize_path_token(token, ctx), 0)).0
}

fn sanitize_path_token(token: &str, ctx: &SanitizeCtx) -> String {
    let Some(path_start) = token.find('/') else {
        if is_denied_windows_path(token) {
            return "<REDACTED_PATH>".to_string();
        }
        return token.to_string();
    };

    let prefix = &token[..path_start];
    let candidate = &token[path_start..];

    if let Some(root) = ctx
        .workspace_root
        .as_deref()
        .filter(|root| !root.is_empty())
    {
        let root = root.trim_end_matches('/');
        if candidate == root {
            return format!("{prefix}.");
        }
        if let Some(rest) = candidate.strip_prefix(&format!("{root}/")) {
            return format!("{prefix}./{rest}");
        }
    }

    if is_denied_system_path(candidate) || is_denied_windows_path(candidate) {
        return format!("{prefix}<REDACTED_PATH>");
    }

    token.to_string()
}

fn is_denied_system_path(path: &str) -> bool {
    ["/etc", "/System", "/usr", "/bin", "/sbin"]
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}/")))
}

#[cfg(windows)]
fn is_denied_windows_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("c:\\windows")
        || lower.starts_with("c:\\program files")
        || lower.starts_with("\\\\")
}

#[cfg(not(windows))]
fn is_denied_windows_path(_path: &str) -> bool {
    false
}

fn truncate_utf8(input: &str, max_bytes: usize) -> (String, bool) {
    if input.len() <= max_bytes {
        return (input.to_string(), false);
    }

    let mut end = max_bytes;
    while !input.is_char_boundary(end) {
        end -= 1;
    }

    (input[..end].to_string(), true)
}

fn map_non_whitespace_tokens(
    input: &str,
    mut map: impl FnMut(&str) -> (String, usize),
) -> (String, usize) {
    let mut out = String::with_capacity(input.len());
    let mut count = 0;
    let mut i = 0;

    while i < input.len() {
        let Some(ch) = input[i..].chars().next() else {
            break;
        };

        if ch.is_whitespace() {
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        let start = i;
        i += ch.len_utf8();
        while i < input.len() {
            let Some(next) = input[i..].chars().next() else {
                break;
            };
            if next.is_whitespace() {
                break;
            }
            i += next.len_utf8();
        }

        let (mapped, redactions) = map(&input[start..i]);
        out.push_str(&mapped);
        count += redactions;
    }

    (out, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sanitize(raw: &str) -> SanitizedPrompt {
        sanitize_ai_prompt(raw, &SanitizeCtx::default()).expect("sanitize should succeed")
    }

    #[test]
    fn strips_ansi_csi_sequences() {
        let out = sanitize("\u{1b}[31merror\u{1b}[0m: failed");

        assert_eq!(out.text, "error: failed");
        assert!(!out.truncated);
    }

    #[test]
    fn strips_osc52_clipboard_sequences() {
        let out = sanitize("safe\u{1b}]52;c;SGVsbG8=\u{7} text");

        assert_eq!(out.text, "safe text");
    }

    #[test]
    fn rejects_nul_byte() {
        let err = sanitize_ai_prompt("safe\0unsafe", &SanitizeCtx::default()).unwrap_err();

        assert_eq!(err, SanitizeError::NulByte);
    }

    #[test]
    fn truncates_to_eight_kibibytes() {
        let raw = "a".repeat(MAX_AI_PROMPT_BYTES + 16);
        let out = sanitize(&raw);

        assert_eq!(out.text.len(), MAX_AI_PROMPT_BYTES);
        assert!(out.truncated);
    }

    #[test]
    fn redacts_common_secret_patterns_and_counts_hits() {
        let out = sanitize(
            "OPENAI_API_KEY=sk-test1234567890 token: ghp_1234567890abcdef password=hunter2",
        );

        assert!(out.text.contains("OPENAI_API_KEY=<REDACTED>"));
        assert!(out.text.contains("token: <REDACTED>"));
        assert!(out.text.contains("password=<REDACTED>"));
        assert_eq!(out.redaction_count, 3);
        assert!(!out.text.contains("hunter2"));
    }

    #[test]
    fn redacts_url_credentials() {
        let out = sanitize("fetch https://alice:secret@example.com/repo.git");

        assert_eq!(out.text, "fetch https://<REDACTED>@example.com/repo.git");
        assert_eq!(out.redaction_count, 1);
    }

    #[test]
    fn redacts_system_absolute_paths() {
        let out = sanitize("see /etc/passwd and /usr/bin/env");

        assert_eq!(out.text, "see <REDACTED_PATH> and <REDACTED_PATH>");
    }

    #[test]
    fn rewrites_workspace_absolute_paths_to_relative() {
        let ctx = SanitizeCtx::new("/Users/leaf/project");
        let out = sanitize_ai_prompt("error at /Users/leaf/project/src/lib.rs:10", &ctx)
            .expect("sanitize should succeed");

        assert_eq!(out.text, "error at ./src/lib.rs:10");
    }
}
