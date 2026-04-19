#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"
prepare_app
run_iterations single-yes "${1:-3}"
run_iterations interactive-top "${1:-3}"
