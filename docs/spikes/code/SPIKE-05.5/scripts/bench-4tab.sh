#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"
prepare_app
run_iterations four-yes "${1:-3}"
