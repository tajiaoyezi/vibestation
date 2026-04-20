#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "$0")" && pwd)/common.sh"

RAW_DIR="${SPIKE08_RAW_DIR:-$RAW_DIR_DEFAULT/linux}"
ensure_raw_dir "$RAW_DIR"

cleanup() {
  cleanup_spike08_processes
  if [[ -n "${XVFB_PID:-}" ]]; then
    kill "$XVFB_PID" 2>/dev/null || true
    wait "$XVFB_PID" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

cd "$ROOT_DIR"

if [[ -z "${DISPLAY:-}" ]]; then
  Xvfb :99 -screen 0 1440x900x24 >"$RAW_DIR/xvfb.log" 2>&1 &
  XVFB_PID=$!
  export DISPLAY=:99
  sleep 1
fi

pnpm install --frozen-lockfile
pnpm tauri:build:smoke >"$RAW_DIR/tauri-build-linux.log" 2>&1

APP_PATH="${SPIKE08_APP_PATH:-$ROOT_DIR/src-tauri/target/debug/spike-08-contract-runtime}"

tauri-driver >"$RAW_DIR/tauri-driver.log" 2>&1 &
DRIVER_PID=$!

wait_for_http "http://127.0.0.1:4444/status" 60

set +e
SPIKE08_APP_PATH="$APP_PATH" \
SPIKE08_SCREENSHOT="$RAW_DIR/tauri-driver-smoke.png" \
node ./scripts/tauri-driver-smoke.mjs 2>&1 | tee "$RAW_DIR/tauri-driver-linux-smoke.log"
STATUS=${PIPESTATUS[0]}
set -e

kill "$DRIVER_PID" 2>/dev/null || true
wait "$DRIVER_PID" 2>/dev/null || true

exit "$STATUS"
