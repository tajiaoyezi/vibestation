#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_BIN="$ROOT_DIR/src-tauri/target/debug/bundle/macos/spike-05-pty.app/Contents/MacOS/spike-05-pty"
BUILD_LOG="$ROOT_DIR/raw-data/build.log"

prepare_app() {
  mkdir -p "$ROOT_DIR/raw-data"
  (
    cd "$ROOT_DIR"
    ./node_modules/.bin/tsc --noEmit
    ./node_modules/.bin/vite build
    CARGO_HOME=/tmp/cargo-home ./node_modules/.bin/tauri build --debug >"$BUILD_LOG" 2>&1 || true
  )

  if [[ ! -x "$APP_BIN" ]]; then
    echo "[SPIKE-05] bundled app missing: $APP_BIN" >&2
    cat "$BUILD_LOG" >&2 || true
    exit 1
  fi
}

run_case() {
  local scenario="$1"
  local run_dir="$2"

  mkdir -p "$run_dir"
  rm -rf "$run_dir"/*

  (
    cd "$ROOT_DIR"
    SPIKE05_SCENARIO="$scenario" \
    SPIKE05_OUTPUT_DIR="$run_dir" \
    SPIKE05_CLOSE_ON_COMPLETE=1 \
    "$APP_BIN" >"$run_dir/app.log" 2>&1
  )
}

run_iterations() {
  local scenario="$1"
  local iterations="${2:-3}"
  local scenario_dir="$ROOT_DIR/raw-data/$scenario"
  mkdir -p "$scenario_dir"

  for index in $(seq 1 "$iterations"); do
    local run_dir="$scenario_dir/run-$index"
    echo "[SPIKE-05] scenario=$scenario iteration=$index"
    run_case "$scenario" "$run_dir"
  done
}
