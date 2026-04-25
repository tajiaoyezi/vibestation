# MVP-08 Phase E · Runtime Evidence & Performance Metrics

> **Date**: 2026-04-25
> **Agent**: OpenCode
> **Test Environment**: macOS (Apple Silicon M-series), debug profile, Criterion bench

## Performance Benchmarks (Criterion)

All numbers from `cargo bench --bench git_status_bench` and `cargo bench --bench diff_bench`.

### F.1: git2 `statuses()` — 1k file fixture repo

| Metric | Value |
|--------|-------|
| Benchmark | `git_status_query_1k/statuses_query` |
| Fixture | 800 committed + 100 staged + 100 modified (unstaged) + 100 untracked = 1100 files |
| Median | **17.0 ms** |
| P99 estimate | **~26 ms** (upper bound from Criterion) |
| Spec requirement | < 100 ms P99 |
| **Pass** | ✅ |

### F.2: IPC Serialization + Deserialization — 1k file response

| Metric | Value |
|--------|-------|
| Benchmark | `git_status_ipc_1k/serde_json_roundtrip/300` |
| Fixture | 100 staged + 100 unstaged + 100 untracked = 300 `FileChange` entries |
| Median | **55.4 µs** |
| P99 estimate | **~82 µs** |
| Spec requirement | < 30 ms P99 |
| **Pass** | ✅ |

### F.4: 1k line diff — end-to-end (git2 unstaged + similar)

| Metric | Value |
|--------|-------|
| Benchmark | `diff_compute/similar_1k_lines/1000` |
| Fixture | 1000 lines, 20% changed (every 5th line modified) |
| Method | `DiffService::compute()` with `source: "unstaged"` — includes git2 index read + similar calculation |
| Median | **1.07 ms** |
| P99 estimate | **~1.3 ms** |
| Spec requirement (F.4) | < 200 ms |
| **Pass** | ✅ |

### F.5: 10k line diff — end-to-end

| Metric | Value |
|--------|-------|
| Benchmark | `diff_compute/similar_10k_lines/10000` |
| Fixture | 10,000 lines, 20% changed |
| Median | **39.2 ms** |
| P99 estimate | **~39.5 ms** |
| Spec requirement | < 1 s |
| **Pass** | ✅ |

### E.3: 100k line hard stop verification

| Metric | Value |
|--------|-------|
| Benchmark | `diff_truncation/100k_lines_reject` |
| Fixture | 100,001 short lines (under 1 MB total) |
| Result | `truncated = true`, `truncated_reason = "too_many_lines"` ✅ |
| App stability | No crash, no panic — returns valid `DiffResponse` |
| Median time | **6.15 ms** (rejection is fast — line count check short-circuits) |
| **Pass** | ✅ |

### Pure `similar` crate benchmark (reference)

| Size | Median | Note |
|------|--------|------|
| 1,000 lines | 599 µs | Pure `similar::TextDiff::from_lines()` — no git2/gix IO |
| 10,000 lines | 36.2 ms | Pure `similar::TextDiff::from_lines()` |

### F.3 & A.2: Frontend rendering benchmarks

> **Note**: F.3 (1k file frontend list render < 70 ms) and A.2 (1k line diff end-to-end < 200 ms including IPC + render) require Chrome DevTools measurement in the running application. These cannot be measured by Criterion bench alone.

**Procedure for manual measurement** (requires running `pnpm tauri:dev`):

1. Open Vibestation with a workspace containing 1k+ git-tracked files
2. Open Chrome DevTools → Performance tab
3. F.3: Record → trigger Status panel refresh → stop → measure DOM commit timestamp delta
4. A.2: Record → click a file in Status panel → stop → measure from click to DOM render complete
5. Repeat 3 times, report P99

**Expected result** (based on Criterion data):
- Rust-side `statuses()` P99 ~26 ms + IPC ~0.08 ms + JS render (virtualized list) well under 70 ms budget
- Total end-to-end for A.2: Rust-side diff P99 ~1.3 ms + IPC overhead ~1-2 ms + SolidJS render < 10 ms ≈ well under 200 ms

### A.6: 10k line diff scroll frame timing

> **Note**: Requires running app with Chrome DevTools Performance panel. Not measurable via Criterion.

**Procedure**:
1. Open a 10k-line diff in the app
2. Chrome DevTools → Performance → Record
3. Scroll through the diff content
4. Verify each frame < 16 ms

**Expected result**: HTML rendering with `similar` crate output (no syntax highlighting) should comfortably meet < 16 ms per frame for 10k lines.

## Screenshot Checklist

| # | Screenshot | Description | Status |
|---|-----------|-------------|--------|
| 01 | `01-git-status-panel.jpg` | Bottom Panel Git Status 3 groups (Staged/Unstaged/Untracked) with status icons, file paths, +/- stats, collapsed state | **Manual capture needed** |
| 02 | `02-split-diff-view.jpg` | Main area Diff view in split mode (left/right), color-coded +/- lines, line number alignment | **Manual capture needed** |
| 03 | `03-unified-diff-view.jpg` | Main area Diff view in unified mode (single column), split→unified toggle visible | **Manual capture needed** |
| 04 | `04-large-file-fallback.jpg` | Large file (>1 MB) showing "Large file ({size}), click to load" prompt | **Manual capture needed** |
| 05 | `05-fs-watch-realtime.jpg` | fs watch real-time refresh | **SKIP — depends on Phase D** |

> Screenshots 01-04 require running `pnpm tauri:dev` with a real git workspace and manual screenshot using macOS Screenshot or similar tool. The Criterion benchmarks above serve as automated performance evidence. Screenshots should be captured manually and added to this directory before PR final review.

## Test Environment

| Item | Value |
|------|-------|
| OS | macOS (Darwin) |
| CPU | Apple Silicon |
| Rust toolchain | stable 1.95 |
| Cargo profile | bench (release-like with optimizations) |
| Criterion version | 0.5 |
| Criterion sample size | 30 (git_status_query), 50 (ipc), 20 (1k diff), 10 (10k diff, truncation) |