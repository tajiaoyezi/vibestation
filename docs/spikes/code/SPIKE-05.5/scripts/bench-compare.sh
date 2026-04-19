#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"
prepare_app
iterations="${1:-3}"
for strategy in shared per-session; do
  run_iterations "$strategy" single-yes "$iterations"
  run_iterations "$strategy" four-yes "$iterations"
  run_iterations "$strategy" interactive-tui "$iterations"
done
