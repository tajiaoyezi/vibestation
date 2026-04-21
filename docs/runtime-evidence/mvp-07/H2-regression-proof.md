# H2 Regression Proof - ts-rs type drift detection

## Method
1. Added `#[ts(rename = "shaShort")]` to `GitLogEntry.short_sha` in `crates/core/src/git_log.rs`
2. Ran `cargo build -p vibestation-app` to regenerate `.ts` bindings
3. Ran `pnpm typecheck` (tsc --noEmit)

## Result: FAILED (as expected)

```
src/panels/GitLog/GitLogPanel.tsx(229,76): error TS2339: Property 'shortSha' does not exist on type 'GitLogEntry'.
src/panels/GitLog/GitLogPanel.tsx(230,72): error TS2339: Property 'shortSha' does not exist on type 'GitLogEntry'.
src/panels/GitLog/GitLogPanel.tsx(233,57): error TS2339: Property 'shortSha' does not exist on type 'GitLogEntry'.
```

## Revert
1. Removed `#[ts(rename = "shaShort")]` from `GitLogEntry.short_sha`
2. Rebuilt with `cargo build -p vibestation-app`
3. Ran `pnpm typecheck` — PASSED

## Conclusion
ts-rs auto-generated bindings correctly prevent type drift between Rust IPC structs and TypeScript consumers. Any manual rename of a Rust field without updating the frontend will be caught at typecheck time.