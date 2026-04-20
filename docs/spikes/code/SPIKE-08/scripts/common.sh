#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RAW_DIR_DEFAULT="$ROOT_DIR/../../raw/SPIKE-08"

cleanup_spike08_processes() {
  pkill -f "playwright" 2>/dev/null || true
  pkill -f "tauri-driver" 2>/dev/null || true
  pkill -f "tauri dev" 2>/dev/null || true
  pkill -f "vite" 2>/dev/null || true
}

wait_for_http() {
  local url="$1"
  local attempts="${2:-60}"

  for _ in $(seq 1 "$attempts"); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done

  echo "Timed out waiting for $url" >&2
  return 1
}

ensure_raw_dir() {
  local dir="${1:-$RAW_DIR_DEFAULT}"
  mkdir -p "$dir"
}
