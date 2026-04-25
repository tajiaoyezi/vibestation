#!/bin/bash
# SPIKE-01-02 Phase B · Cold Boot Test Runner
# Runs isolated to avoid signal propagation issues
set -uo pipefail

BINARY="${1:-$PWD/target/release/vibestation-app}"
SESSION_TYPE="${2:-x11}"
N="${3:-10}"
LOG_DIR="$PWD/docs/spikes/raw/SPIKE-01-02-phase-B"
TIMESTAMP=$(date +%s)
LOG="${LOG_DIR}/cold-boot-${SESSION_TYPE}-${TIMESTAMP}.csv"

mkdir -p "$LOG_DIR"
echo "iteration,start_ms,end_ms,elapsed_ms,status" > "$LOG"

echo "=== Cold Boot: $SESSION_TYPE × $N ==="

for i in $(seq 1 "$N"); do
    # Kill any lingering
    killall -9 vibestation-app 2>/dev/null || true
    sleep 1

    START=$(date +%s%3N)

    if [ "$SESSION_TYPE" = "wayland" ]; then
        WAYLAND_DISPLAY=wayland-1 XDG_SESSION_TYPE=wayland DISPLAY= setsid "$BINARY" >/dev/null 2>&1 &
    else
        DISPLAY=:0 XDG_SESSION_TYPE=x11 setsid "$BINARY" >/dev/null 2>&1 &
    fi

    PID=$!
    sleep 4

    if kill -0 $PID 2>/dev/null; then
        END=$(date +%s%3N)
        ELAPSED=$((END - START))
        echo "[$i/$N] OK ${ELAPSED}ms"
        echo "$i,$START,$END,$ELAPSED,ok" >> "$LOG"
        kill -TERM $PID 2>/dev/null || true
        sleep 1
    else
        END=$(date +%s%3N)
        ELAPSED=$((END - START))
        echo "[$i/$N] FAIL ${ELAPSED}ms"
        echo "$i,$START,$END,$ELAPSED,fail" >> "$LOG"
    fi
done

# Final cleanup
killall -9 vibestation-app 2>/dev/null || true

echo ""
echo "=== Results ==="
cat "$LOG"
OK_COUNT=$(tail -n +2 "$LOG" | grep -c ",ok" || echo 0)
MEDIAN=$(tail -n +2 "$LOG" | awk -F, '$5=="ok" {print $4}' | sort -n | awk '{a[NR]=$1} END {if(NR%2==1) print a[int(NR/2)+1]; else print (a[NR/2]+a[NR/2+1])/2}')
echo ""
echo "Success: $OK_COUNT/$N"
echo "Median: ${MEDIAN}ms"
echo "Log: $LOG"
