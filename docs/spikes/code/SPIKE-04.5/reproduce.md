# SPIKE-04.5 v2 Reproduce Guide

## Prerequisites

- Rust toolchain (tested with rustc 1.95.0)
- macOS or Linux (tested on macOS with Apple M2 Max, 34.4 GB RAM)

## Build

```bash
cd /tmp/spike-04-5-work-v2/spike-04-5-rusqlite-safety
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release
```

## Run Full Suite (A + B.1–B.5)

```bash
rm -rf /tmp/spike-04-5-test-data-v2
./target/release/spike-04-5
```

Expected output ends with:
```
A  Performance:    FAIL
B.1 Crash recovery: PASS
B.2 Corruption:     PASS
B.3 Migration:      PASS
B.4 Export/Import:  PASS
B.5 Self-check:     PASS

A.3 FAIL (range scan P99=220ms > 50ms threshold) -> Conclusion (B partial)
B.1-5 all PASS -> R27 (silent data corruption) fully closed
A.3 performance: Arbiter decides (accept 220ms / add index / scope downgrade)
```

**Note**: A.3 correctly reports FAIL in v2 (threshold corrected from 50s to 50ms).

## Module Structure

| Module | Lines | Purpose |
|--------|-------|---------|
| `src/main.rs` | ~190 | Orchestration + A/B.1-B.5 test logic |
| `src/manifest.rs` | ~30 | Manifest struct (per_table, last_committed_tx_id), atomic write |
| `src/op_log.rs` | ~40 | OpLogEntry, write/read/update, fsync |
| `src/self_check.rs` | ~50 | Self-check, reconcile forward, silent-loss detection |
| `src/backup_mod.rs` | ~70 | create_backup, create_periodic_backup (retention), update_last_known_good |
| `src/rollback_ui.rs` | ~30 | CLI mock for auto-rollback UI |

## Key Thresholds (v2 corrected)

```rust
let r_pass = r_p99 < 0.005;  // 0.005s = 5ms (spec: < 5ms)
let s_pass = s_p99 < 0.050;  // 0.050s = 50ms (spec: < 50ms)
let w_pass = w_p99 < 60.0;   // 60s (spec: < 60s)
```

## Raw Data

After running, raw data is at `/tmp/spike-04-5-test-data-v2/`.