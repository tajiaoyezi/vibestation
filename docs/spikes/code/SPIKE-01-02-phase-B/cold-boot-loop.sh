#!/bin/bash
# SPIKE-01-02 Phase B · Cold Boot Loop Test
# Usage: ./cold-boot-loop.sh [N] [session_type]
#   N: number of iterations (default: 10)
#   session_type: x11 | wayland (default: x11)

set -euo pipefail

N=${1:-10}
SESSION_TYPE=${2:-x11}
BINARY="${BINARY:-$PWD/target/release/vibestation-app}"
TIMESTAMP=$(date +%s)
LOG_DIR="${LOG_DIR:-$PWD/docs/spikes/raw/SPIKE-01-02-phase-B}"
LOG="${LOG_DIR}/cold-boot-${SESSION_TYPE}-${TIMESTAMP}.csv"

echo "=== Cold Boot Test ==="
echo "  Iterations: $N"
echo "  Session:    $SESSION_TYPE"
echo "  Binary:     $BINARY"
echo "  Log:        $LOG"
echo ""

mkdir -p "$LOG_DIR"
echo "iteration,start_ms,end_ms,elapsed_ms,exit_code" > "$LOG"

for i in $(seq 1 "$N"); do
    echo -n "[$i/$N] Starting... "

    # Kill any lingering instances
    pkill -f vibestation-app 2>/dev/null || true
    sleep 1

    START=$(date +%s%3N)

    if [ "$SESSION_TYPE" = "wayland" ]; then
        # Launch under Weston Wayland compositor
        WAYLAND_DISPLAY=wayland-1 XDG_SESSION_TYPE=wayland "$BINARY" &
    else
        # X11 default
        XDG_SESSION_TYPE=x11 "$BINARY" &
    fi

    PID=$!

    # Wait for window to appear (check via xdotool or just time-based)
    # xterm.js/webview rendering typically stable within 3-5s
    sleep 4

    # Check if process still alive
    if kill -0 $PID 2>/dev/null; then
        END=$(date +%s%3N)
        ELAPSED=$((END - START))
        echo "OK (${ELAPSED}ms)"
        echo "$i,$START,$END,$ELAPSED,0" >> "$LOG"
        kill $PID 2>/dev/null || true
        wait $PID 2>/dev/null || true
    else
        END=$(date +%s%3N)
        ELAPSED=$((END - START))
        echo "CRASH (${ELAPSED}ms)"
        echo "$i,$START,$END,$ELAPSED,1" >> "$LOG"
    fi

    sleep 1
done

echo ""
echo "=== Results ==="
echo "Raw: $LOG"
if [ "$(wc -l < "$LOG")" -gt 1 ]; then
    MEDIAN=$(tail -n +2 "$LOG" | awk -F, '$5==0 {print $4}' | sort -n | awk '{a[NR]=$1} END {print (NR%2==1)?a[int(NR/2)+1]:(a[NR/2]+a[NR/2+1])/2}')
    FAILS=$(tail -n +2 "$LOG" | awk -F, '$5==1 {count++} END {print count+0}')
    echo "Median (success only): ${MEDIAN}ms"
    echo "Failures: ${FAILS}/$N"
fi
