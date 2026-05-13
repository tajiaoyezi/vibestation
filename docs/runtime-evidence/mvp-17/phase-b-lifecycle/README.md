# MVP-17 Phase B lifecycle dev-mode evidence

Date: 2026-05-13

Dev mode was started with:

```bash
pnpm tauri:dev
```

Result:

- The Tauri app launched successfully.
- The backend `pane_detach_open` / `pane_detach_close` lifecycle is covered by Rust unit and integration tests in this PR.
- The requested UI screenshot sequence cannot be captured on current `origin/main` without widening this PR into Phase C frontend wiring.

Blocker:

- `web/src/lib/pane-detach.ts` exposes `detachPane`, `closeDetachedPane`, `listDetachedPanes`, and `initPaneDetachStateListener`.
- Current frontend code does not call `initPaneDetachStateListener`.
- Current frontend code does not render `DetachedPlaceholder` from `detachedPanes()`.
- Current frontend code does not wire `detachPane()` into a visible menu, shortcut handler, or pane action.

The raw dev-mode startup log tail is archived in `dev-mode-blocker.raw.log`.
