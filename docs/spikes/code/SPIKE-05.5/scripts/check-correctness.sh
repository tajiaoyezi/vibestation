#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"
prepare_app
strategy="${1:-shared}"
iterations="${2:-1}"
run_iterations "$strategy" correctness "$iterations"
