use std::sync::Arc;

use sentry::{ClientOptions, Level};
use spike_mvp_10_telemetry::{capture_panic, payload_json, scrub_sentry_event};

fn main() {
    let panic_info = "Vibestation SPIKE-MVP-10 Phase B local smoke";
    let payload = capture_panic(panic_info);
    let payload_json = payload_json(&payload);

    println!("sanitized_payload={payload_json}");

    let Some(dsn) = std::env::var("SENTRY_DSN")
        .ok()
        .filter(|value| !value.is_empty())
    else {
        println!("sentry_smoke_ran=true");
        println!("sentry_send_skipped=true");
        println!("skip_reason=SENTRY_DSN not set");
        return;
    };

    let _sentry_guard = sentry::init((
        dsn,
        ClientOptions {
            default_integrations: false,
            send_default_pii: false,
            release: Some("vibestation-core@0.1.0".into()),
            environment: Some("development".into()),
            before_send: Some(Arc::new(|mut event| {
                scrub_sentry_event(&mut event);
                Some(event)
            })),
            ..Default::default()
        },
    ));

    sentry::capture_message("Vibestation SPIKE-MVP-10 Phase B test", Level::Info);

    println!("sentry_smoke_ran=true");
    println!("sentry_send_attempted=true");
}
