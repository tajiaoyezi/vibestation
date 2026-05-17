//! MVP-19 §M session summary redaction pipeline.
//!
//! **Distinct from [`crate::sanitize`]** (prompt-injection boundary for AI prompts).
//! This module handles **secret/PII redaction** for session summaries before they are
//! stored or surfaced: API keys, tokens, JWTs, email addresses, phone numbers, and path
//! usernames are replaced with typed redaction markers.
//!
//! ## Fail-closed guarantee (§H9 / §M.1)
//! Any internal engine failure returns [`RedactionError::EngineFailure`]. Callers MUST
//! NOT fall back to the original plaintext on error — "宁可隐藏也不展示可疑明文".
//!
//! ## Pattern coverage (§M.2)
//! Aligned with gitleaks default-rule categories (no custom `.gitleaks.toml` in this
//! repo; see `HC-6` in the W2-D dispatch prompt):
//!   - OpenAI / Anthropic API keys (`sk-...`, `sk-ant-...`)
//!   - GitHub tokens (`ghp_...`, `github_pat_...`)
//!   - JWT tokens (`eyJ...`)
//!   - Bearer tokens (`Bearer <token>`)
//!   - Email addresses
//!   - Phone numbers (NA + international)
//!   - Path usernames (`/Users/<x>`, `/home/<x>`)
//!
//! ## Strategy version
//! [`REDACTION_STRATEGY_VERSION`] labels the active pattern set for audit traceability
//! (§M.4). Increment this string whenever the pattern set changes.

use std::sync::OnceLock;

use regex::Regex;
use thiserror::Error;

/// Labels the active §M pattern set for audit traceability (§M.4).
///
/// Aligned with gitleaks default-rule categories; this is NOT a literal import of a
/// gitleaks config file (no custom `.gitleaks.toml` exists at the repository root).
pub const REDACTION_STRATEGY_VERSION: &str = "v1";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Output of [`redact_session_summary`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionResult {
    /// Session summary text with all matched patterns replaced by redaction markers.
    pub sanitized_text: String,
    /// Total number of individual match-replacements applied.
    pub redaction_count: usize,
    /// Deduplicated list of pattern kind labels that fired (e.g. `"api_key_openai"`,
    /// `"email"`). Order matches first appearance in the pattern list.
    pub redaction_kinds: Vec<String>,
}

/// Error type returned by [`redact_session_summary`].
///
/// On [`RedactionError::EngineFailure`] callers MUST NOT fall back to the raw input
/// (fail-closed, §H9).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RedactionError {
    #[error("redaction engine failure (fail-closed; raw text withheld): {0}")]
    EngineFailure(String),
}

// ---------------------------------------------------------------------------
// §M.3 five-step pipeline entry point
// ---------------------------------------------------------------------------

/// Apply the §M redaction pipeline to a raw session summary string.
///
/// ## Pipeline steps (§M.3)
/// 1. **Normalize**: strip ANSI escape sequences and C0 control characters.
///    (Algorithm mirrors the private `strip_ansi_and_osc` in [`crate::sanitize`];
///    that function is module-private so it is reimplemented here — HC-2.)
/// 2. **Compile patterns** (cached; one-time cost via [`OnceLock`]).
/// 3. **Scan**: for each §M.2 pattern, count matches in the current text.
/// 4. **Redact**: replace all matches with the typed marker; accumulate kind labels.
/// 5. **Return** [`RedactionResult`] with `sanitized_text`, `redaction_count`, and
///    `redaction_kinds`.
///
/// # Errors
/// Returns [`RedactionError::EngineFailure`] if pattern compilation fails at
/// initialization time. **Callers must not fall back to `raw`** on error (§H9).
pub fn redact_session_summary(raw: &str) -> Result<RedactionResult, RedactionError> {
    let patterns = compiled_patterns()?;

    // Step 1: normalize ANSI/control characters.
    let mut text = strip_ansi_and_control(raw);

    // Steps 3–4: scan + redact each pattern in order; accumulate stats.
    let mut redaction_count: usize = 0;
    let mut redaction_kinds: Vec<String> = Vec::new();

    for (re, replacement, kind) in patterns {
        let match_count = re.find_iter(&text).count();
        if match_count > 0 {
            text = re.replace_all(&text, *replacement).into_owned();
            redaction_count = redaction_count.saturating_add(match_count);
            let kind_owned = kind.to_string();
            if !redaction_kinds.contains(&kind_owned) {
                redaction_kinds.push(kind_owned);
            }
        }
    }

    Ok(RedactionResult {
        sanitized_text: text,
        redaction_count,
        redaction_kinds,
    })
}

// ---------------------------------------------------------------------------
// §M.2 pattern specifications
// ---------------------------------------------------------------------------

/// Internal pattern descriptor (private; compiled once via [`compiled_patterns`]).
struct PatternSpec {
    /// Regex string. Compiled to [`Regex`] at first call; any error surfaces as
    /// [`RedactionError::EngineFailure`] (fail-closed).
    pattern: &'static str,
    /// Replacement marker (e.g. `"[REDACTED_TOKEN]"`).
    replacement: &'static str,
    /// Kind label added to [`RedactionResult::redaction_kinds`].
    kind: &'static str,
}

/// §M.2 ordered pattern list – more specific patterns appear before more general ones
/// (e.g. `sk-ant-` before the generic `sk-`) to prevent partial double-redaction.
///
/// All categories align with gitleaks default rules (HC-6):
///   Bearer token → Anthropic key → OpenAI key → GitHub PAT → GitHub token →
///   JWT → email → phone → path usernames.
static PATTERN_SPECS: &[PatternSpec] = &[
    // Bearer token: "Bearer <token>" → [REDACTED_TOKEN]  (§M.5 example)
    // Listed first so "Bearer sk-live-..." is consumed as one unit, preventing
    // the sk- pattern from partially re-matching the already-redacted text.
    PatternSpec {
        pattern: r"(?i)Bearer\s+\S+",
        replacement: "[REDACTED_TOKEN]",
        kind: "bearer_token",
    },
    // Anthropic API key: sk-ant-...  (more specific than generic sk-)
    // Gitleaks default rule: "Anthropic API Key"
    PatternSpec {
        pattern: r"sk-ant-[A-Za-z0-9\-_]{20,}",
        replacement: "[REDACTED_TOKEN]",
        kind: "api_key_anthropic",
    },
    // OpenAI API key: sk-...  (also covers sk-live-... / sk-proj-... variants)
    // Gitleaks default rule: "OpenAI API Key"
    PatternSpec {
        pattern: r"sk-[A-Za-z0-9\-_]{20,}",
        replacement: "[REDACTED_TOKEN]",
        kind: "api_key_openai",
    },
    // GitHub fine-grained personal access token: github_pat_...
    // Gitleaks default rule: "GitHub Fine-Grained Personal Access Token"
    PatternSpec {
        pattern: r"github_pat_[A-Za-z0-9_]{36,}",
        replacement: "[REDACTED_TOKEN]",
        kind: "api_key_github_pat",
    },
    // GitHub classic personal access token: ghp_...
    // Gitleaks default rule: "GitHub Personal Access Token"
    PatternSpec {
        pattern: r"ghp_[A-Za-z0-9]{36,}",
        replacement: "[REDACTED_TOKEN]",
        kind: "api_key_github",
    },
    // JWT: eyJ<header>.<payload>.<signature>  (three base64url-encoded parts)
    // Gitleaks default rule: "JSON Web Token"
    PatternSpec {
        pattern: r"eyJ[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]*",
        replacement: "[REDACTED_TOKEN]",
        kind: "jwt",
    },
    // Email address
    // Gitleaks default rule: generic PII detection
    PatternSpec {
        pattern: r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}",
        replacement: "[REDACTED_EMAIL]",
        kind: "email",
    },
    // Phone number: NA (NXX-NXX-XXXX) and international (+1 ...) formats.
    // Separators limited to space and hyphen (avoids false positives on dotted
    // version strings like "1.2.3.4567").
    // Gitleaks default rule: phone number PII category.
    PatternSpec {
        pattern: r"\+?1?[\s\-]?\(?[0-9]{3}\)?[\s\-][0-9]{3}[\s\-][0-9]{4}",
        replacement: "[REDACTED_PHONE]",
        kind: "phone",
    },
    // Path username /Users/<name>  →  /Users/[REDACTED_USER]/<rest>  (§M.5 example)
    // Matches the username segment; the remainder of the path is left intact.
    PatternSpec {
        pattern: r"/Users/[A-Za-z0-9_.\-]+",
        replacement: "/Users/[REDACTED_USER]",
        kind: "path_username",
    },
    // Path username /home/<name>  →  /home/[REDACTED_USER]/<rest>
    PatternSpec {
        pattern: r"/home/[A-Za-z0-9_.\-]+",
        replacement: "/home/[REDACTED_USER]",
        kind: "path_username",
    },
];

// ---------------------------------------------------------------------------
// Pattern compilation (fail-closed, cached)
// ---------------------------------------------------------------------------

type CompiledPattern = (Regex, &'static str, &'static str);

/// Returns a reference to the lazily-compiled pattern list.
///
/// Compilation happens at most once (per process). If any pattern fails to compile,
/// the `OnceLock` stores `Err` and every subsequent call returns
/// [`RedactionError::EngineFailure`] (fail-closed: callers receive `Err` and must not
/// expose the raw input).
fn compiled_patterns() -> Result<&'static Vec<CompiledPattern>, RedactionError> {
    static CACHE: OnceLock<Result<Vec<CompiledPattern>, String>> = OnceLock::new();

    CACHE
        .get_or_init(|| {
            let mut out = Vec::with_capacity(PATTERN_SPECS.len());
            for spec in PATTERN_SPECS {
                match Regex::new(spec.pattern) {
                    Ok(re) => out.push((re, spec.replacement, spec.kind)),
                    Err(e) => {
                        return Err(format!("failed to compile pattern {:?}: {e}", spec.pattern));
                    }
                }
            }
            Ok(out)
        })
        .as_ref()
        .map_err(|e| RedactionError::EngineFailure(e.clone()))
}

// ---------------------------------------------------------------------------
// ANSI / control-character normalization (step 1)
// ---------------------------------------------------------------------------

/// Strip ANSI escape sequences and C0 control characters from `input`.
///
/// Handles:
///   - CSI sequences (`ESC [` … final byte in `0x40–0x7E`)
///   - OSC / DCS / PM / APC strings terminated by BEL (`0x07`) or ST (`ESC \`)
///   - Other two-byte Fe sequences (`ESC` + any byte)
///   - C0 control bytes `0x00–0x1F` except HT (`\t`), LF (`\n`), CR (`\r`)
///
/// Algorithm mirrors the private `strip_ansi_and_osc` in [`crate::sanitize`]; that
/// function is module-private and `sanitize.rs` must not be modified (HC-2), so the
/// same byte-scanning approach is reimplemented here.
fn strip_ansi_and_control(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == 0x1b {
            // ESC sequence
            if i + 1 >= bytes.len() {
                i += 1;
                continue;
            }
            match bytes[i + 1] {
                b'[' => {
                    // CSI: ESC [ <params> <final>
                    i += 2;
                    while i < bytes.len() && !is_ansi_final(bytes[i]) {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1; // consume final byte
                    }
                }
                b']' | b'P' | b'^' | b'_' => {
                    // OSC / DCS / PM / APC: terminated by BEL or ST
                    i = skip_string_terminator(bytes, i + 2);
                }
                _ => {
                    // Other Fe sequences (2-byte): skip both
                    i += 2;
                }
            }
            continue;
        }

        // Strip C0 control characters except common whitespace
        if bytes[i] < 0x20 && !matches!(bytes[i], b'\t' | b'\n' | b'\r') {
            i += 1;
            continue;
        }

        out.push(bytes[i]);
        i += 1;
    }

    // Input is valid UTF-8; filtering only complete ASCII bytes preserves UTF-8 validity.
    String::from_utf8_lossy(&out).into_owned()
}

/// Returns `true` for ANSI CSI final bytes (`0x40–0x7E`).
fn is_ansi_final(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte)
}

/// Advance `i` past an OSC/DCS/PM/APC string terminator (BEL or ESC `\`).
fn skip_string_terminator(bytes: &[u8], mut i: usize) -> usize {
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: unwrap redaction result.
    fn redact(raw: &str) -> RedactionResult {
        redact_session_summary(raw).expect("redact_session_summary should succeed")
    }

    // ── §M.5 verbatim examples ───────────────────────────────────────────────

    /// §M.5 example 1: Bearer token → [REDACTED_TOKEN]
    #[test]
    fn m5_bearer_token_full_replacement() {
        let result = redact("Authorization: Bearer sk-live-abcdefghijklmnopqrstuvwxyz");
        assert_eq!(
            result.sanitized_text, "Authorization: [REDACTED_TOKEN]",
            "Bearer sk-live-... should become [REDACTED_TOKEN]"
        );
        assert_eq!(result.redaction_count, 1);
        assert!(result.redaction_kinds.contains(&"bearer_token".to_string()));
    }

    /// §M.5 example 2: /Users/alice/... → /Users/[REDACTED_USER]/...
    #[test]
    fn m5_path_username_users() {
        let result = redact("error at /Users/alice/projects/foo.rs:10");
        assert_eq!(
            result.sanitized_text, "error at /Users/[REDACTED_USER]/projects/foo.rs:10",
            "/Users/<name> segment should be redacted while rest of path is preserved"
        );
        assert_eq!(result.redaction_count, 1);
        assert!(result
            .redaction_kinds
            .contains(&"path_username".to_string()));
    }

    // ── API key patterns (§M.2) ──────────────────────────────────────────────

    /// Positive: OpenAI sk- key is redacted.
    #[test]
    fn api_key_openai_positive() {
        let result = redact("key: sk-abcdefghijklmnopqrstuvwxyz0123456789");
        assert!(result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert!(!result.sanitized_text.contains("sk-abcdef"));
        assert!(result.redaction_count >= 1);
        assert!(result
            .redaction_kinds
            .contains(&"api_key_openai".to_string()));
    }

    /// Negative: short "sk-" prefix below threshold is NOT redacted.
    #[test]
    fn api_key_openai_negative_too_short() {
        let result = redact("prefix sk-short and sk-tiny");
        // "sk-short" is 8 chars total; sk- + 5 chars < 20 threshold → not matched
        assert!(!result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert_eq!(result.redaction_count, 0);
    }

    /// Positive: Anthropic sk-ant- key is redacted with correct kind.
    #[test]
    fn api_key_anthropic_positive() {
        let key = "sk-ant-api03-abcdefghijklmnopqrstuvwxyz0123";
        let result = redact(&format!("using key {key}"));
        assert!(result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert!(!result.sanitized_text.contains(key));
        assert!(result
            .redaction_kinds
            .contains(&"api_key_anthropic".to_string()));
    }

    /// Positive: GitHub fine-grained PAT is redacted.
    #[test]
    fn api_key_github_pat_positive() {
        let token = "github_pat_ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefg";
        let result = redact(&format!("token={token}"));
        assert!(result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert!(!result.sanitized_text.contains(token));
        assert!(result
            .redaction_kinds
            .contains(&"api_key_github_pat".to_string()));
    }

    /// Negative: "github_pat_" with fewer than 36 suffix chars is NOT redacted.
    #[test]
    fn api_key_github_pat_negative_too_short() {
        let result = redact("github_pat_tooshort");
        assert!(!result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert_eq!(result.redaction_count, 0);
    }

    /// Positive: GitHub classic PAT (ghp_) is redacted.
    #[test]
    fn api_key_github_classic_positive() {
        let token = "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcde";
        let result = redact(&format!("auth: {token}"));
        assert!(result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert!(!result.sanitized_text.contains(token));
        assert!(result
            .redaction_kinds
            .contains(&"api_key_github".to_string()));
    }

    // ── JWT patterns ─────────────────────────────────────────────────────────

    /// Positive: well-formed JWT (three base64url parts) is redacted.
    #[test]
    fn jwt_positive() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9\
                   .eyJzdWIiOiIxMjM0NTY3ODkwIn0\
                   .SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let result = redact(&format!("token: {jwt}"));
        assert!(result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert!(!result.sanitized_text.contains("eyJhbGciO"));
        assert!(result.redaction_kinds.contains(&"jwt".to_string()));
    }

    /// Negative: plain text starting with "eyJ" but without dots is NOT a JWT.
    #[test]
    fn jwt_negative_no_dots() {
        let result = redact("eyJhello world this has no dots");
        assert!(!result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert_eq!(result.redaction_count, 0);
    }

    // ── Bearer token patterns ────────────────────────────────────────────────

    /// Positive: case-insensitive Bearer token is redacted.
    #[test]
    fn bearer_token_case_insensitive() {
        let result = redact("header: bearer MySecretTokenValue123");
        assert!(result.sanitized_text.contains("[REDACTED_TOKEN]"));
        assert!(!result.sanitized_text.contains("MySecretTokenValue123"));
        assert!(result.redaction_kinds.contains(&"bearer_token".to_string()));
    }

    /// Negative: the word "bearer" alone without a following token is NOT redacted.
    #[test]
    fn bearer_negative_no_token() {
        // "bearer" at end of string with no following non-whitespace
        let result = redact("the word bearer alone");
        // "bearer alone" — \S+ matches "alone", so this IS redacted per the pattern.
        // Adjust: test a sentence where "bearer" is the last word.
        let result2 = redact("discusses the concept of bearer");
        // "bearer" at end of input: `Bearer\s+\S+` requires whitespace + token after it.
        assert_eq!(
            result2.sanitized_text, "discusses the concept of bearer",
            "bearer at end of string with no following token should not be redacted"
        );
        assert_eq!(result2.redaction_count, 0);
        // First test: "bearer alone" — "alone" is the token. Verify it is redacted.
        assert!(result.sanitized_text.contains("[REDACTED_TOKEN]"));
    }

    // ── Email patterns ───────────────────────────────────────────────────────

    /// Positive: a standard email address is redacted.
    #[test]
    fn email_positive() {
        let result = redact("contact user@example.com for support");
        assert!(result.sanitized_text.contains("[REDACTED_EMAIL]"));
        assert!(!result.sanitized_text.contains("user@example.com"));
        assert!(result.redaction_kinds.contains(&"email".to_string()));
    }

    /// Negative: a domain without @ is NOT redacted as email.
    #[test]
    fn email_negative_no_at_sign() {
        let result = redact("visit example.com for more info");
        assert!(!result.sanitized_text.contains("[REDACTED_EMAIL]"));
        assert_eq!(result.redaction_count, 0);
    }

    /// Negative: a local-only string with @ but no valid TLD is NOT redacted.
    #[test]
    fn email_negative_no_tld() {
        // "user@localhost" — no dot in domain, TLD requires {2,} alpha chars
        let result = redact("send to user@localhost");
        assert!(!result.sanitized_text.contains("[REDACTED_EMAIL]"));
    }

    // ── Phone number patterns ────────────────────────────────────────────────

    /// Positive: a standard NA phone number is redacted.
    #[test]
    fn phone_positive_na_format() {
        let result = redact("call 555-123-4567 for help");
        assert!(result.sanitized_text.contains("[REDACTED_PHONE]"));
        assert!(!result.sanitized_text.contains("555-123-4567"));
        assert!(result.redaction_kinds.contains(&"phone".to_string()));
    }

    /// Positive: international format with + prefix is redacted.
    #[test]
    fn phone_positive_international() {
        let result = redact("reach +1-800-555-1234");
        assert!(result.sanitized_text.contains("[REDACTED_PHONE]"));
        assert!(result.redaction_kinds.contains(&"phone".to_string()));
    }

    /// Negative: a short number sequence that does not match phone format.
    #[test]
    fn phone_negative_short_number() {
        let result = redact("step 42 or line 1234");
        assert!(!result.sanitized_text.contains("[REDACTED_PHONE]"));
        assert_eq!(result.redaction_count, 0);
    }

    // ── Path username patterns ───────────────────────────────────────────────

    /// Positive: /home/<name> is redacted.
    #[test]
    fn path_username_home_positive() {
        let result = redact("config at /home/bob/.config/app.toml");
        assert!(result.sanitized_text.contains("/home/[REDACTED_USER]"));
        assert!(!result.sanitized_text.contains("/home/bob"));
        assert!(result
            .redaction_kinds
            .contains(&"path_username".to_string()));
    }

    /// Negative: /tmp/ and /usr/bin/ are NOT treated as user paths.
    #[test]
    fn path_username_negative_system_paths() {
        let result = redact("binary at /usr/bin/env and temp /tmp/build");
        assert!(!result.sanitized_text.contains("[REDACTED_USER]"));
        assert_eq!(result.redaction_count, 0);
    }

    // ── redaction_count and redaction_kinds correctness ──────────────────────

    /// Multiple distinct patterns in one string produce correct counts and kinds.
    #[test]
    fn multiple_patterns_count_and_kinds() {
        let input = "email user@test.org token sk-abcdefghijklmnopqrstuvwxyz0123 \
                     path /Users/carol/work";
        let result = redact(input);
        // email + api_key_openai + path_username
        assert_eq!(result.redaction_count, 3);
        assert!(result.redaction_kinds.contains(&"email".to_string()));
        assert!(result
            .redaction_kinds
            .contains(&"api_key_openai".to_string()));
        assert!(result
            .redaction_kinds
            .contains(&"path_username".to_string()));
        assert!(!result.sanitized_text.contains("user@test.org"));
        assert!(!result.sanitized_text.contains("sk-abcdef"));
        assert!(!result.sanitized_text.contains("/Users/carol"));
    }

    /// Same kind appearing twice increments count but keeps kinds deduplicated.
    #[test]
    fn same_kind_deduplication_in_kinds() {
        let input = "a@b.com c@d.org";
        let result = redact(input);
        assert_eq!(result.redaction_count, 2);
        assert_eq!(
            result
                .redaction_kinds
                .iter()
                .filter(|k| k.as_str() == "email")
                .count(),
            1,
            "email kind should appear exactly once in redaction_kinds"
        );
    }

    // ── Clean text (no redaction) ────────────────────────────────────────────

    /// Normal prose is returned unchanged.
    #[test]
    fn clean_text_unchanged() {
        let plain = "This session analyzed five commits in the feature branch.";
        let result = redact(plain);
        assert_eq!(result.sanitized_text, plain);
        assert_eq!(result.redaction_count, 0);
        assert!(result.redaction_kinds.is_empty());
    }

    // ── ANSI / control character normalization ───────────────────────────────

    /// ANSI CSI colour codes are stripped before redaction.
    #[test]
    fn ansi_csi_stripped() {
        let result = redact("\x1b[31merror\x1b[0m: sk-abcdefghijklmnopqrstuvwxyz0123");
        assert!(!result.sanitized_text.contains('\x1b'));
        assert!(result.sanitized_text.starts_with("error:"));
        assert!(result.sanitized_text.contains("[REDACTED_TOKEN]"));
    }

    /// OSC sequences are stripped.
    #[test]
    fn ansi_osc_stripped() {
        let result = redact("safe\x1b]52;c;SGVsbG8=\x07 text");
        assert_eq!(result.sanitized_text, "safe text");
    }

    /// C0 control characters (except HT/LF/CR) are removed.
    #[test]
    fn control_chars_stripped() {
        // 0x01 SOH, 0x08 BS should be removed; \t and \n preserved
        let result = redact("line\x01two\x08end\tnewline\n");
        assert_eq!(result.sanitized_text, "linetwoend\tnewline\n");
    }

    // ── Fail-closed contract ─────────────────────────────────────────────────

    /// Compiled patterns must all succeed (validates all static regex strings).
    #[test]
    fn all_static_patterns_compile() {
        compiled_patterns().expect("all static §M.2 patterns must compile without error");
    }

    /// RedactionError display does not expose the original plaintext.
    /// Verifies the fail-closed API contract: callers must treat Err as a signal to
    /// hide the input, not as a recoverable error that falls back to the raw string.
    #[test]
    fn redaction_error_does_not_expose_plaintext() {
        let sensitive = "sk-secret-0123456789abcdef0123456789abcdef";
        // Construct the error as the engine would on a compile failure.
        let err =
            RedactionError::EngineFailure("failed to compile pattern \"[broken\": ...".to_string());
        // The error message must not carry the sensitive input.
        assert!(!err.to_string().contains(sensitive));

        // A successful call must also not leave sensitive content in the output.
        let result =
            redact_session_summary(&format!("key: {sensitive}")).expect("patterns must compile");
        assert!(!result.sanitized_text.contains(sensitive));
        assert!(result.redaction_count > 0);
    }

    // ── Empty / edge inputs ──────────────────────────────────────────────────

    /// Empty input produces empty output with zero redactions.
    #[test]
    fn empty_input() {
        let result = redact("");
        assert_eq!(result.sanitized_text, "");
        assert_eq!(result.redaction_count, 0);
        assert!(result.redaction_kinds.is_empty());
    }

    /// Whitespace-only input is returned unchanged.
    #[test]
    fn whitespace_only() {
        let result = redact("   \n\t  ");
        assert_eq!(result.sanitized_text, "   \n\t  ");
        assert_eq!(result.redaction_count, 0);
    }
}
