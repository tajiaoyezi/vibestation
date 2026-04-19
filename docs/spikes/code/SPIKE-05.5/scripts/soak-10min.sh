#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"
prepare_app
run_iterations soak-10min "${1:-1}"
run_iterations hidden-5min "${2:-1}"
