use sentry::protocol::{Context, Map};
use serde_json::json;
use spike_mvp_10_telemetry::{
    capture_panic, event_from_payload, payload_json, sanitized_sentry_event_json,
    scrub_sentry_event,
};

fn assert_no_pii(serialized: &str) {
    for forbidden in [
        "/Users/alice",
        "alice",
        "secret",
        "abc1234567890abcdef",
        "192.168.1.42",
        "rm -rf",
        "Documents",
        "secret-project",
        ".git",
        "span_id",
        "trace_id",
        "\"trace\"",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "payload leaked forbidden fragment {forbidden:?}: {serialized}"
        );
    }
}

#[test]
fn capture_panic_strips_pii() {
    let panic_info = "thread 'main' panicked at 'Failed to read /Users/alice/secret/file.txt: \
                      commit abc1234567890abcdef · IP 192.168.1.42'";
    let payload = capture_panic(panic_info);
    let payload_json = payload_json(&payload);

    assert_no_pii(&payload_json);
    assert_eq!(payload.version, "0.1.0");
    assert!(!payload.os_type.is_empty());
    assert_eq!(payload.stack_trace_hash.len(), 64);
    assert!(payload
        .stack_trace_hash
        .chars()
        .all(|ch| ch.is_ascii_hexdigit()));
}

#[test]
fn capture_panic_handles_terminal_content() {
    let panic_info = "panic at 'parse error in user input: rm -rf ~/Documents'";
    let payload = capture_panic(panic_info);
    let payload_json = payload_json(&payload);

    assert_no_pii(&payload_json);
    assert_eq!(payload.stack_trace_hash.len(), 64);
}

#[test]
fn capture_panic_handles_repo_path() {
    let panic_info = "fatal: failed to read /Users/alice/work/secret-project/.git/HEAD";
    let payload = capture_panic(panic_info);
    let payload_json = payload_json(&payload);

    assert_no_pii(&payload_json);
    assert_eq!(payload.stack_trace_hash.len(), 64);
}

#[test]
fn sanitized_event_excludes_pii_fields() {
    let panic_info = "panic at /Users/alice/secret/file.txt after rm -rf ~/Documents \
                      in repo /Users/alice/work/secret-project/.git with IP 192.168.1.42 \
                      and commit abc1234567890abcdef";
    let payload = capture_panic(panic_info);
    let mut event = event_from_payload(&payload);
    let mut trace = Map::new();

    trace.insert("span_id".to_string(), json!("5eb65e2df9b71af0"));
    trace.insert(
        "trace_id".to_string(),
        json!("88b6f25de8069f69a565db20770c069c"),
    );
    event
        .contexts
        .insert("trace".to_string(), Context::Other(trace));

    scrub_sentry_event(&mut event);
    let event_json = serde_json::to_string(&event).unwrap();
    let message = event.message.as_deref().unwrap();

    assert_no_pii(&event_json);
    assert!(message.contains("\"version\":\"0.1.0\""));
    assert!(message.contains("\"os_type\""));
    assert!(message.contains("\"stack_trace_hash\""));

    let sanitized_json = sanitized_sentry_event_json(panic_info);
    assert_no_pii(&sanitized_json);
}
