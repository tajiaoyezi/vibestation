#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "$0")" && pwd)/common.sh"

RAW_DIR="${SPIKE08_RAW_DIR:-$RAW_DIR_DEFAULT}"
ensure_raw_dir "$RAW_DIR"

cd "$ROOT_DIR"

TMP_DIR="$(mktemp -d)"
cp src/App.tsx "$TMP_DIR/App.tsx"

restore() {
  cp "$TMP_DIR/App.tsx" src/App.tsx
  pnpm typecheck >/dev/null
}

trap restore EXIT INT TERM

python3 - <<'PY'
from pathlib import Path

app = Path("src/App.tsx")
text = app.read_text()
target = "const response = await deleteWorkspace({ workspaceId: workspace.id });"
replacement = 'const response = await deleteWorkspace({ workspaceId: legacyWorkspaceId(workspace) ?? "missing-id" });'
if target not in text:
    raise SystemExit("target line not found in App.tsx")
app.write_text(text.replace(target, replacement))
PY

pnpm typecheck >/dev/null

set +e
SPIKE08_TRACE="$RAW_DIR/h2-runtime-regression-trace.zip" \
SPIKE08_SCREENSHOT="$RAW_DIR/h2-runtime-regression.png" \
SPIKE08_RUN_LOG="$RAW_DIR/h2-runtime-regression.log" \
./scripts/run-browser-smoke.sh
STATUS=$?
set -e

if [[ "$STATUS" -eq 0 ]]; then
  echo "Expected browser E2E to fail after runtime regression" >&2
  exit 1
fi
