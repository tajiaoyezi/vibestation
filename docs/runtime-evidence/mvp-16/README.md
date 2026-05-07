# MVP-16 Runtime Evidence

## Scope

Phase A covers backend-only delivery for `rebase_ops`:

- `crates/core/src/rebase_ops.rs` backend state machine for rebase, interactive rebase, merge, cherry-pick, conflict resolution, and crash recovery detection.
- Tauri IPC registration and permissions for the Phase A command surface.
- SQLite migration for `rebase_state` persistence.
- 18 new `ts-rs` bindings generated under `web/src/bindings/`.

Phase B covers the frontend UI surface:

- `web/src/panels/RebaseEditor/` interactive rebase plan editor.
- `web/src/panels/Diff/3way/` conflict resolver.
- `web/src/components/ConflictBanner/` active conflict banner with recovery variant prop.
- `web/src/dialogs/MergeDialog/` and `web/src/dialogs/CherryPickDialog/`.
- `web/src/panels/GitLog/contextMenu.tsx` Git Log right-click actions.

Runtime screenshots for Phase B were intentionally skipped per user instruction on 2026-05-07. This README records that waiver so the PR report does not imply screenshot evidence was captured.

## Evidence

- `01-cargo-test-output.txt`: `cargo test --workspace` output captured on 2026-05-06.
- `02-ts-rs-bindings-list.txt`: globbed binding file list for `Rebase*.ts`, `Cherry*.ts`, `Merge*.ts`, `Conflict*.ts`, and `CrashRecovery*.ts`.
- Phase B UI verification uses command gates and source review in this PR. Screenshot files are not part of the Phase B evidence set by user request.

The binding-list glob contains 20 matching files because it also includes two pre-existing bindings, `ConflictFile.ts` and `MergeConflictInfo.ts`. The Phase A diff adds 18 new binding files.

## Gates

- `pnpm install --frozen-lockfile`: completed without lockfile changes.
- `pnpm lint`: passed.
- `pnpm typecheck`: passed.
- `cargo test --workspace`: passed.
