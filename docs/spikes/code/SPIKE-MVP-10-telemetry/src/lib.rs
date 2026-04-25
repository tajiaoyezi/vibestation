use std::borrow::Cow;

use sentry::protocol::Event;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const VIBESTATION_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReportPayload {
    pub version: String,
    pub os_type: String,
    pub stack_trace_hash: String,
}

pub fn capture_panic(panic_info: &str) -> CrashReportPayload {
    CrashReportPayload {
        version: VIBESTATION_VERSION.to_string(),
        os_type: std::env::consts::OS.to_string(),
        stack_trace_hash: stable_64_hex_hash(panic_info),
    }
}

pub fn payload_json(payload: &CrashReportPayload) -> String {
    payload_value(payload).to_string()
}

pub fn payload_value(payload: &CrashReportPayload) -> Value {
    json!({
        "version": payload.version,
        "os_type": payload.os_type,
        "stack_trace_hash": payload.stack_trace_hash,
    })
}

pub fn event_from_payload(payload: &CrashReportPayload) -> Event<'static> {
    Event {
        message: Some(payload_json(payload)),
        release: Some(Cow::Borrowed("vibestation-core@0.1.0")),
        environment: Some(Cow::Borrowed("development")),
        ..Default::default()
    }
}

pub fn scrub_sentry_event(event: &mut Event<'static>) {
    event.contexts.remove("trace");
    event.user = None;
    event.request = None;
    event.breadcrumbs = Default::default();
    event.exception = Default::default();
    event.stacktrace = None;
    event.threads = Default::default();
    event.modules.clear();
    event.tags.clear();
    event.extra.clear();
}

pub fn sanitized_sentry_event(panic_info: &str) -> Event<'static> {
    let payload = capture_panic(panic_info);
    let mut event = event_from_payload(&payload);
    scrub_sentry_event(&mut event);
    event
}

pub fn sanitized_sentry_event_json(panic_info: &str) -> String {
    serde_json::to_string(&sanitized_sentry_event(panic_info))
        .expect("sanitized sentry event should serialize")
}

fn stable_64_hex_hash(input: &str) -> String {
    format!("{:x}", Sha256::digest(input.as_bytes()))
}
