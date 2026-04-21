# MVP-07 Git Log Readonly - Performance Benchmark

## Test Environment
- Machine: macOS (darwin/arm64)
- Repo: vibestation self-repo (~75 commits)
- gix version: 0.70 (max-performance-safe + revparse-regex)
- Date: 2026-04-21

## Unit Test Results
```
running 26 tests
test git_log::tests::file_change_serializes_camel_case ... ok
test git_log::tests::commit_detail_serializes_camel_case ... ok
test git_log::tests::commit_detail_nonexistent_sha_returns_error ... ok
test git_log::tests::format_relative_time_output ... ok
test git_log::tests::git_log_error_display ... ok
test git_log::tests::git_log_entry_serializes_camel_case ... ok
test git_log::tests::git_log_query_request_serializes_camel_case ... ok
test git_log::tests::git_log_query_response_serializes_camel_case ... ok
test git_log::tests::parse_after_date_formats ... ok
test git_log::tests::branch_labels_found_on_main ... ok
test git_log::tests::commit_detail_real_repo ... ok
test git_log::tests::query_non_git_path_returns_not_a_git_repo ... ok
test git_log::tests::query_nonexistent_path_returns_error ... ok
test git_log::tests::query_has_more_correct ... ok
test git_log::tests::query_real_repo_returns_entries ... ok
test git_log::tests::query_pagination_no_overlap ... ok
test git_log::tests::query_filter_by_after_date ... ok
test git_log::tests::query_filter_by_author ... ok
test git_log::tests::query_filter_by_message ... ok

test result: ok. 26 passed; 0 failed; 0 ignored (0.07s)
```

## Verification Commands Passed
- `cargo fmt --all -- --check` ✅
- `cargo clippy --workspace -- -D warnings` ✅
- `cargo test --workspace` ✅ (92 tests, 0 failures)
- `pnpm typecheck` ✅

## H2 Regression Proof
- Added `#[ts(rename = "shaShort")]` → `pnpm typecheck` FAILS (3 type errors)
- Reverted → `pnpm typecheck` PASSES
- Confirms ts-rs bindings prevent type drift between Rust and TypeScript