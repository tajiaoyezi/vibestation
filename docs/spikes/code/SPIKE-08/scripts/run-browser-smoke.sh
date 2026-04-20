#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "$0")" && pwd)/common.sh"

RAW_DIR="${SPIKE08_RAW_DIR:-$RAW_DIR_DEFAULT}"
ensure_raw_dir "$RAW_DIR"

cleanup() {
  cleanup_spike08_processes
}

trap cleanup EXIT INT TERM

VITE_LOG="$RAW_DIR/vite-browser.log"
TRACE_PATH="${SPIKE08_TRACE:-$RAW_DIR/playwright-browser-trace.zip}"
SCREENSHOT_PATH="${SPIKE08_SCREENSHOT:-$RAW_DIR/playwright-browser-final.png}"
RUN_LOG="${SPIKE08_RUN_LOG:-$RAW_DIR/playwright-browser-smoke.log}"

cd "$ROOT_DIR"
pnpm dev >"$VITE_LOG" 2>&1 &
VITE_PID=$!

wait_for_http "http://127.0.0.1:1420" 60

set +e
SPIKE08_TRACE="$TRACE_PATH" \
SPIKE08_SCREENSHOT="$SCREENSHOT_PATH" \
node ./scripts/browser-e2e.mjs 2>&1 | tee "$RUN_LOG"
STATUS=${PIPESTATUS[0]}
set -e

kill "$VITE_PID" 2>/dev/null || true
wait "$VITE_PID" 2>/dev/null || true

exit "$STATUS"
