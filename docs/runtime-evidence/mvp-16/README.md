# MVP-16 Phase A Runtime Evidence

## Scope

Phase A covers backend-only delivery for `rebase_ops`:

- `crates/core/src/rebase_ops.rs` backend state machine for rebase, interactive rebase, merge, cherry-pick, conflict resolution, and crash recovery detection.
- Tauri IPC registration and permissions for the Phase A command surface.
- SQLite migration for `rebase_state` persistence.
- 18 new `ts-rs` bindings generated under `web/src/bindings/`.

Phase A does not include GUI implementation. Rebase editor, conflict banner, 3-way diff UI, merge dialog, cherry-pick dialog, screenshots, and desktop runtime capture remain Phase B-D scope.

## Evidence

- `01-cargo-test-output.txt`: `cargo test --workspace` output captured on 2026-05-06.
- `02-ts-rs-bindings-list.txt`: globbed binding file list for `Rebase*.ts`, `Cherry*.ts`, `Merge*.ts`, `Conflict*.ts`, and `CrashRecovery*.ts`.

The binding-list glob contains 20 matching files because it also includes two pre-existing bindings, `ConflictFile.ts` and `MergeConflictInfo.ts`. The Phase A diff adds 18 new binding files.

## Gates

- `pnpm install --frozen-lockfile`: completed without lockfile changes.
- `pnpm lint`: passed.
- `pnpm typecheck`: passed.
- `cargo test --workspace`: passed.
