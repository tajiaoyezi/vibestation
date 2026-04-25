#!/bin/bash
# SPIKE-01-02 Phase B · Accurate Cold Boot Test
# Measures time from fork to process entering stable running state
set -uo pipefail

BINARY="${1:-$PWD/target/release/vibestation-app}"
SESSION_TYPE="${2:-x11}"
N="${3:-10}"
LOG_DIR="$PWD/docs/spikes/raw/SPIKE-01-02-phase-B"
TIMESTAMP=$(date +%s)
LOG="${LOG_DIR}/cold-boot-${SESSION_TYPE}-${TIMESTAMP}.csv"

mkdir -p "$LOG_DIR"
echo "iteration,start_ms,stable_ms,elapsed_ms,status" > "$LOG"

echo "=== Cold Boot: $SESSION_TYPE × $N ==="

for i in $(seq 1 "$N"); do
    killall -9 vibestation-app 2>/dev/null || true
    sleep 1

    START=$(date +%s%3N)

    if [ "$SESSION_TYPE" = "wayland" ]; then
        WAYLAND_DISPLAY=wayland-1 XDG_SESSION_TYPE=wayland DISPLAY= "$BINARY" >/dev/null 2>&1 &
    else
        DISPLAY=:0 XDG_SESSION_TYPE=x11 "$BINARY" >/dev/null 2>&1 &
    fi

    PID=$!

    # Wait up to 5s for process to enter stable state (not zombie, multi-threaded)
    STABLE_MS=""
    for attempt in $(seq 1 50); do
        sleep 0.1
        if ! kill -0 $PID 2>/dev/null; then
            break
        fi
        STAT=$(ps -p $PID -o stat= 2>/dev/null | tr -d ' ')
        if [ -n "$STAT" ] && echo "$STAT" | grep -q 'l'; then
            STABLE_MS=$(date +%s%3N)
            break
        fi
    done

    if [ -n "$STABLE_MS" ]; then
        ELAPSED=$((STABLE_MS - START))
        echo "[$i/$N] OK ${ELAPSED}ms"
        echo "$i,$START,$STABLE_MS,$ELAPSED,ok" >> "$LOG"
        kill -TERM $PID 2>/dev/null || true
        sleep 0.5
    else
        END=$(date +%s%3N)
        ELAPSED=$((END - START))
        echo "[$i/$N] FAIL ${ELAPSED}ms"
        echo "$i,$START,$END,$ELAPSED,fail" >> "$LOG"
    fi
done

killall -9 vibestation-app 2>/dev/null || true

echo ""
echo "=== Results ==="
cat "$LOG"
OK_COUNT=$(tail -n +2 "$LOG" | grep -c ",ok" || echo 0)
MEDIAN=$(tail -n +2 "$LOG" | awk -F, '$5=="ok" {print $4}' | sort -n | awk '{a[NR]=$1} END {if(NR%2==1) print a[int(NR/2)+1]; else print (a[NR/2]+a[NR/2+1])/2}')
FAILS=$((N - OK_COUNT))
echo ""
echo "Success: $OK_COUNT/$N"
echo "Failures: $FAILS"
echo "Median: ${MEDIAN}ms"
echo "Log: $LOG"
