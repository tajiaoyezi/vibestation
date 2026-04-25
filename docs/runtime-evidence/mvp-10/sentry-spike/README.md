# MVP-10 Sentry Stack Spike

Date: 2026-04-25
Branch: `spike/MVP-10-telemetry-stack-sentry`
Purpose: validate the MVP-10 Phase B telemetry stack candidate before any production SDK integration.

## Conclusion

Recommendation: proceed with [ADR-015](../../../adr/ADR-015-telemetry-stack-sentry.md) as a **proposed** decision for `sentry` 0.47.0, with strict opt-in and sanitized-payload constraints.

This spike did not leave `sentry` in `Cargo.toml` or `Cargo.lock`; the dependency was added only for local validation, then removed.

## Evidence

| File                                                         | Result                                                                                                        |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------- |
| [`step1-sentry-smoke.txt`](./step1-sentry-smoke.txt)         | `sentry` 0.47.0 added temporarily and build/example integration completed locally                             |
| [`step2-pii-sanitization.txt`](./step2-pii-sanitization.txt) | 4 PII sanitization tests passed; emitted event JSON contained only sanitized payload plus Sentry SDK metadata |
| [`step3-cargo-bloat.txt`](./step3-cargo-bloat.txt)           | release example `.text` = 1.8 MiB, file size = 3.2 MiB; final app artifact still needs Phase B/C measurement  |
| [`command-log.md`](./command-log.md)                         | dependency add/remove log and environment notes                                                               |
| [`01-sentry-smoke.png`](./01-sentry-smoke.png)               | screenshot proof for Step 1 output                                                                            |
| [`02-pii-payload.png`](./02-pii-payload.png)                 | screenshot proof for Step 2 output                                                                            |
| [`03-cargo-bloat.png`](./03-cargo-bloat.png)                 | screenshot proof for Step 3 output                                                                            |

## Required Phase B Constraints

- Initialize Sentry only when `telemetry_opt_in == true` and a DSN is configured.
- Keep DSN in environment / local config / GitHub secret; never commit it.
- Set `default_integrations: false` and `send_default_pii: false`.
- Build `CrashReportPayload` in Vibestation first, then send only `version`, `os_type`, and `stack_trace_hash`.
- Add `before_send` as a final whitelist gate.
- Add regression tests that fail if event JSON includes paths, IPs, terminal content, commit hashes, or raw panic text.

## Not Tested

- Real Sentry Web UI receipt was not tested because `SENTRY_DSN` and `SENTRY_AUTH_TOKEN` were absent.
- Final Tauri `.app` / `.dmg` / AppImage artifact size was not measured in this spike.
- Self-hosted Sentry endpoint deployment was not tested.
