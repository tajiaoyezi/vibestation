#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "$0")" && pwd)/common.sh"

RAW_DIR="${SPIKE08_RAW_DIR:-$RAW_DIR_DEFAULT}"
ensure_raw_dir "$RAW_DIR"

cleanup() {
  cleanup_spike08_processes
}

trap cleanup EXIT INT TERM

cd "$ROOT_DIR"

TAURI_LOG="$RAW_DIR/tauri-playwright-dev.log"
RUN_LOG="$RAW_DIR/tauri-playwright-smoke.log"

pnpm tauri:dev:e2e >"$TAURI_LOG" 2>&1 &
TAURI_PID=$!

wait_for_http "http://127.0.0.1:1420" 60

for _ in $(seq 1 60); do
  if [[ -S /tmp/tauri-playwright.sock ]]; then
    break
  fi
  sleep 1
done

set +e
pnpm e2e:tauri 2>&1 | tee "$RUN_LOG"
STATUS=${PIPESTATUS[0]}
set -e

kill "$TAURI_PID" 2>/dev/null || true
wait "$TAURI_PID" 2>/dev/null || true

exit "$STATUS"
