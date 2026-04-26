//! MVP-10 §C.3.1 · PII 脱敏 unit test 套件（Phase B 实施）
//!
//! 验证 [`vibestation_core::telemetry::capture_panic`] 即使输入含 PII（用户路径 /
//! commit hash / IP / 终端内容 / 仓库名）· 输出 [`CrashReportPayload`] 仍仅含
//! 3 字段白名单（version / os_type / stack_trace_hash）· 原始 PII 经 SHA-256
//! 不可逆哈希 · 不会泄漏。
//!
//! 编译时 ts-rs derive + 字段白名单已锁定 struct 形态 · 本套件提供 runtime 双保险。

use vibestation_core::telemetry::{capture_panic, CrashReportPayload};

/// §C.3.1 · 用户路径 + commit hash + IP（spec 锁定模板）
#[test]
fn capture_panic_strips_user_path_commit_hash_ip() {
    let panic_info = "thread 'main' panicked at 'Failed to read /Users/alice/secret/file.txt: \
                      commit abc1234567890abcdef · IP 192.168.1.42'";
    let payload: CrashReportPayload = capture_panic(panic_info);

    let payload_json = serde_json::to_string(&payload).unwrap();

    // 不含用户路径（绝对路径 / home dir）
    assert!(
        !payload_json.contains("/Users/alice"),
        "payload leaked /Users/alice path: {payload_json}"
    );
    assert!(
        !payload_json.contains("secret"),
        "payload leaked filename: {payload_json}"
    );

    // 不含 commit hash 全文（仅 stack_trace_hash · 是 SHA-256 哈希值 ≠ commit hash）
    assert!(
        !payload_json.contains("abc1234567890abcdef"),
        "payload leaked commit hash: {payload_json}"
    );

    // 不含 IP（IPv4 正则）
    let ip_regex = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
    assert!(
        !ip_regex.is_match(&payload_json),
        "payload leaked IP: {payload_json}"
    );

    // 字段白名单 runtime 双保险（编译时 struct 已锁 + ts-rs derive 强制）
    assert_eq!(payload.version, env!("CARGO_PKG_VERSION"));
    assert!(!payload.os_type.is_empty());
    assert_eq!(payload.stack_trace_hash.len(), 64); // SHA-256 hex = 64 chars
    assert!(payload
        .stack_trace_hash
        .chars()
        .all(|c| c.is_ascii_hexdigit()));
}

/// §C.3.1 · 终端命令内容（rm -rf 等 · 必须脱敏）
#[test]
fn capture_panic_strips_terminal_command_content() {
    let panic_info = "panic at 'parse error in user input: rm -rf ~/Documents'";
    let payload = capture_panic(panic_info);
    let payload_json = serde_json::to_string(&payload).unwrap();

    assert!(
        !payload_json.contains("rm -rf"),
        "payload leaked shell command: {payload_json}"
    );
    assert!(
        !payload_json.contains("Documents"),
        "payload leaked dir name: {payload_json}"
    );
    assert!(
        !payload_json.contains("~/"),
        "payload leaked home path shorthand: {payload_json}"
    );
}

/// §C.3.1 · Git 仓库路径（含项目名 · 必须脱敏）
#[test]
fn capture_panic_strips_repo_path_and_project_name() {
    let panic_info = "panic at '/Users/alice/work/secret-project/.git/HEAD missing'";
    let payload = capture_panic(panic_info);
    let payload_json = serde_json::to_string(&payload).unwrap();

    assert!(
        !payload_json.contains("secret-project"),
        "payload leaked project name: {payload_json}"
    );
    assert!(
        !payload_json.contains("alice"),
        "payload leaked username: {payload_json}"
    );
    assert!(
        !payload_json.contains(".git"),
        "payload leaked .git internal path: {payload_json}"
    );
}

/// 边界 · 空 panic 信息（极端 case · 不应 panic）
#[test]
fn capture_panic_handles_empty_input() {
    let payload = capture_panic("");
    assert_eq!(payload.stack_trace_hash.len(), 64);
    // SHA-256("") = e3b0c44... 已知值
    assert_eq!(
        payload.stack_trace_hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

/// 边界 · 极长 panic 信息（防 buffer 溢出 · SHA-256 始终 64 chars）
#[test]
fn capture_panic_handles_long_input() {
    let long_input: String = "x".repeat(100_000);
    let payload = capture_panic(&long_input);
    assert_eq!(payload.stack_trace_hash.len(), 64);
}

/// 边界 · UTF-8 多字节字符（中文 panic message）
#[test]
fn capture_panic_handles_utf8_multibyte() {
    let panic_info = "崩溃：用户输入了 /Users/张三/secret 文件";
    let payload = capture_panic(panic_info);
    let payload_json = serde_json::to_string(&payload).unwrap();

    assert!(!payload_json.contains("张三"));
    assert!(!payload_json.contains("/Users/"));
    assert!(!payload_json.contains("secret"));
    assert_eq!(payload.stack_trace_hash.len(), 64);
}
