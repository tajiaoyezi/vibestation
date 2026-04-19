#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"
prepare_app
run_iterations hol-ipc-saturated "${1:-3}"
