# MVP-15 Phase D §F · vitest bench evidence

> 本目录覆盖 Phase D §F 的 vitest bench 子集与 §C fixture 生成证据。完整 DevTools Performance / Chrome Memory / visual regression 仍留 Arbiter playbook。

## 运行平台

| 字段 | 值 |
|---|---|
| OS | macOS 26.3.1 (Build 25D771280a) |
| Arch | arm64 |
| CPU | Apple M2 Max |
| Node | v24.14.1 |
| pnpm | 9.15.9 |
| Vitest | 4.1.5 |
| Bench 时间 | 2026-05-12T20:47:50+0800 |

## Bench 数据 vs spec 目标

| Spec | vitest bench 子集 | 实测 P99 / 估算 | 目标 | 余裕 / 状态 | Raw log |
|---|---|---:|---:|---:|---|
| F.1 | 1MB TypeScript Shiki parse（cache miss · 5 samples · 不含 IPC/DOM/首屏） | 1,900.52 ms | N/A（完整首屏 < 300ms 留 DevTools） | 子集量化；不作为首屏 PASS/FAIL | `shiki-parse-1mb.raw.log` |
| F.2 | `< 1MB` sync scheduler path | 0.0004 ms | < 16 ms | 40,000x | `scheduler-10mb-three-tier.raw.log` |
| F.2 | `1MB-10MB` requestIdleCallback scheduler path | 2.7077 ms | < 16 ms | 5.9x | `scheduler-10mb-three-tier.raw.log` |
| F.2 | `>=10MB` Worker fail fallback requestIdleCallback path | 1.4725 ms | < 50 ms | 34.0x | `scheduler-10mb-three-tier.raw.log` |
| F.3 | `setShikiTheme` signal + cached 1MB highlight | 0.0140 ms | < 50 ms | 3,571x | `theme-switch.raw.log` |
| F.4 | `ShikiAdapter.highlight()` same-key LRU cache hit | 0.0013 ms | < 5 ms | 3,846x | `lru-cache-hit.raw.log` |
| F.5 | 10 distinct 1MB TypeScript variants · heap delta after GC | 55.54 MB | < 100 MB | 1.8x | `memory-10x1mb.raw.log` |

## Fixture 证据

- Regenerate command: `./scripts/fixtures/generate-syntax-highlight-fixtures.sh`
- Seed: `mvp-15-phase-d-v1`
- Tracked fixtures:
  - `web/tests/fixtures/syntax-highlight/1mb-typescript.ts` · 1,048,636 bytes
  - `web/tests/fixtures/syntax-highlight/10mb-typescript.ts` · 10,486,082 bytes
  - `web/tests/fixtures/syntax-highlight/1mb-rust.rs` · 1,048,894 bytes
  - `web/tests/fixtures/syntax-highlight/1mb-python.py` · 1,048,877 bytes
  - `web/tests/fixtures/syntax-highlight/1mb-go.go` · 1,048,764 bytes
  - `web/tests/fixtures/syntax-highlight/1mb-java.java` · 1,048,712 bytes
- Local-only fixture: `web/tests/fixtures/syntax-highlight/50mb-typescript.ts` · 52,429,461 bytes · ignored by `.gitignore`.
- Repro check: repeated script run produced identical `shasum` values for all tracked fixtures and the local-only 50MB fixture.

## jsdom vs Chrome DevTools 差异

1. F.1 只测 Shiki parse cache-miss cost，不含 Tauri IPC、diff compute、DOM commit、first viewport paint，也不证明 `1MB diff 首屏 < 300ms`。
2. F.2 使用 mocked Worker failure 来量化 `scheduleHighlight()` fallback overhead，避免 Vitest/jsdom 等待 real Worker timeout。完整 long task trace 仍需 Chrome DevTools Performance。
3. F.3/F.4 只覆盖 signal/cache 命中路径，不覆盖真实 Diff DOM repaint。
4. F.5 使用 Node `process.memoryUsage().heapUsed` + `--execArgv=--expose-gc` 估算 jsdom heap delta；这不等同 Chrome DevTools Memory snapshot。10 个 1MB variant 触发 LRU 50MB cap，最终 cache 保留 6 entries / 46.07MB。

## 复跑命令

```bash
pnpm -C web exec vitest bench tests/utils/shiki/bench/shiki-parse.bench.ts --run --reporter verbose
pnpm -C web exec vitest bench tests/utils/shiki/bench/scheduler.bench.ts --run --reporter verbose
pnpm -C web exec vitest bench tests/utils/shiki/bench/theme-switch.bench.ts --run --reporter verbose
pnpm -C web exec vitest bench tests/utils/shiki/bench/lru-cache-hit.bench.ts --run --reporter verbose
pnpm -C web exec vitest bench tests/utils/shiki/bench/memory.bench.ts --run --reporter verbose --execArgv=--expose-gc
```
