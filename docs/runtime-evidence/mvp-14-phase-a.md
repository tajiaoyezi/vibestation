# MVP-14 Phase A Runtime Evidence

## Date
2026-05-07

## Test Results

### Core Tests
```
cargo test --package vibestation-core panes::
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 509 filtered out

cargo test --package vibestation-core pane_service::
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 574 filtered out
```

### Build
```
cargo build -p vibestation-app
Finished dev profile [unoptimized + debuginfo] target(s) in 10.30s
```

### TypeScript Type Check
```
pnpm typecheck
tsc --noEmit (PASS)
```

### New ts-rs Bindings Generated (12/12)
1. ✅ LayoutEnvelope.ts
2. ✅ WorkspaceLayoutState.ts
3. ✅ LayoutPresetKind.ts
4. ✅ LayoutApplyAdvancedRequest.ts
5. ✅ LayoutApplyResult.ts
6. ✅ PaneNavDirection.ts
7. ✅ PaneNavigateRequest.ts
8. ✅ PaneNavigateResult.ts
9. ✅ PaneMaximizeRequest.ts
10. ✅ PaneMaximizeResult.ts
11. ✅ PaneResizeStepRequest.ts
12. ✅ LayoutHistoryEntry.ts

## Changes Summary

### crates/core/src/panes.rs
- Added LayoutEnvelope v1 with version, root, focused_pane_id, updated_at
- Added LayoutPresetKind enum (Solo, AiAndRunner, DualAi, TripleReview, Quad)
- Added PaneLayoutError tagged union with 7 variants
- Added 10 new IPC request/result types with ts-rs derives
- Relaxed MAX_LAYOUT_SPLIT_DEPTH from 2 to 5
- Added ratio clamp [0.05, 0.95]
- Added build_layout_for_preset() for new presets
- Made collect_pane_ids() public
- Added find_split_ratio() helper
- 89 tests (10 new)

### crates/core/src/pane_service.rs
- Added apply_layout_preset_advanced() for new presets
- Added apply_pane_navigate() (simplified DFS-based)
- Added apply_pane_maximize() (session-only)
- Added apply_pane_resize_step() (keyboard resize)
- 24 tests (all passing)

### crates/app/src/pane_layout_advanced.rs
- New file with 4 IPC command handlers:
  - pane_layout_apply_advanced
  - pane_navigate
  - pane_maximize
  - pane_resize_step

### crates/app/src/lib.rs
- Added mod pane_layout_advanced
- Imported new command handlers
- Registered 4 new commands in invoke_handler
- Added new type imports

### crates/app/build.rs
- Added 12 new ts-rs exports
- Updated index.ts with new binding exports

### crates/app/capabilities/default.json
- Added 4 new permissions for advanced layout commands

### crates/app/permissions/pane-layout-advanced.toml
- New permission file with 4 command grants

## Backward Compatibility
- MVP-05 tests still pass
- Old LayoutNode JSON auto-migrates to LayoutEnvelope v1
- validate_mvp_05 preserved as deprecated wrapper
