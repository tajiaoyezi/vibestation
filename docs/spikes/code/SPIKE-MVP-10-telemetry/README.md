# SPIKE-MVP-10-telemetry · 实测源码

## 来源

- 交付 agent：Codex CLI
- 产出时间：2026-04-25
- Review accept：tajiaoyezi（待定）
- 原始归档：本目录（Round 2 · PR #120）

## 复现命令

```bash
cd docs/spikes/code/SPIKE-MVP-10-telemetry
cargo build --release --example sentry_smoke
SENTRY_DSN="https://..." cargo run --release --example sentry_smoke
cargo test --release
```

## 结构

```text
SPIKE-MVP-10-telemetry/
├── Cargo.toml · Cargo.lock
├── examples/
│   └── sentry_smoke.rs
├── src/
│   └── lib.rs
└── tests/
    └── sentry_pii_spike.rs
```

## 关键结论溯源

- Step 1 cargo build pass · 见 `docs/runtime-evidence/mvp-10/sentry-spike/step1-sentry-smoke.txt`
- Step 2 PII 4 测试 · 见 `tests/sentry_pii_spike.rs` + `docs/runtime-evidence/mvp-10/sentry-spike/step2-pii-sanitization.txt`
- Step 3 cargo bloat · 见 `docs/runtime-evidence/mvp-10/sentry-spike/step3-cargo-bloat.txt`

## 注意

- 本目录是独立 Cargo workspace，`[workspace]` 空表用于防止 Cargo 把它当作主 workspace member。
- 本目录代码作为 Spike 证据归档，不直接进入生产实现。
- `before_send` smoke 会显式删除 `event.contexts.trace`；Phase B 正式实现必须保留对应单元测试。
