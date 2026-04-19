#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_BIN="$ROOT_DIR/target/debug/bundle/macos/spike-05-5-pty.app/Contents/MacOS/spike-05-5-pty"
BUILD_LOG="$ROOT_DIR/raw-data/build.log"

prepare_app() {
  mkdir -p "$ROOT_DIR/raw-data"
  (
    cd "$ROOT_DIR"
    if [[ ! -d node_modules ]]; then
      PNPM_HOME=/tmp/pnpm-home PNPM_STORE_DIR=/tmp/pnpm-store pnpm install --ignore-workspace --force
    fi
    ./node_modules/.bin/tsc --noEmit
    ./node_modules/.bin/vite build
    CARGO_HOME=/tmp/cargo-home ./node_modules/.bin/tauri build --debug >"$BUILD_LOG" 2>&1 || true
  )

  if [[ ! -x "$APP_BIN" ]]; then
    echo "[SPIKE-05.5] bundled app missing: $APP_BIN" >&2
    cat "$BUILD_LOG" >&2 || true
    exit 1
  fi
}

run_case() {
  local strategy="$1"
  local scenario="$2"
  local run_dir="$3"

  mkdir -p "$run_dir"
  rm -rf "$run_dir"/*

  (
    cd "$ROOT_DIR"
    SPIKE055_STRATEGY="$strategy" \
    SPIKE055_SCENARIO="$scenario" \
    SPIKE055_OUTPUT_DIR="$run_dir" \
    SPIKE055_CLOSE_ON_COMPLETE=1 \
    "$APP_BIN" >"$run_dir/app.log" 2>&1
  )
}

run_iterations() {
  local strategy="$1"
  local scenario="$2"
  local iterations="${3:-3}"
  local scenario_dir="$ROOT_DIR/raw-data/$strategy/$scenario"
  mkdir -p "$scenario_dir"

  for index in $(seq 1 "$iterations"); do
    local run_dir="$scenario_dir/run-$index"
    echo "[SPIKE-05.5] strategy=$strategy scenario=$scenario iteration=$index"
    run_case "$strategy" "$scenario" "$run_dir"
  done
}
