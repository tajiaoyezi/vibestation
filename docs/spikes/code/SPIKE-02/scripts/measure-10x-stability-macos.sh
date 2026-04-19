#!/usr/bin/env bash
# SPIKE-02 · macOS 10x 连续启动稳定性测试
# 用法：./scripts/measure-10x-stability-macos.sh
#
# 测量方式：连续启动 spike-02-tauri 10 次（冷启动）· 抓每次的 window_ready 时间和 panic 信号
# 通过标准：10/10 成功 · 无 panic / 无 crash / 全部能触发 window_ready 事件

set -uo pipefail

RUNS="${1:-10}"
BUNDLE_DIR="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/target/release/bundle/macos"
APP_NAME="spike-02-tauri.app"
BIN="${BUNDLE_DIR}/${APP_NAME}/Contents/MacOS/spike-02-tauri"

if [[ ! -x "${BIN}" ]]; then
  echo "ERROR: Release binary not found at ${BIN}"
  echo "请先跑：pnpm tauri build"
  exit 1
fi

echo "10x Stability Test · SPIKE-02 macOS"
echo "Binary: ${BIN}"
echo "Runs: ${RUNS}"
echo "---"

SUCCESS=0
FAIL=0
FAIL_REASONS=()
TIMES=()

for i in $(seq 1 "${RUNS}"); do
  LOG=$(mktemp)
  nohup "${BIN}" >/dev/null 2>"${LOG}" &

  # 等 window_ready 或 panic · 最多 10s
  OK=0
  for _ in $(seq 1 100); do
    if grep -q "window_ready" "${LOG}" 2>/dev/null; then
      OK=1
      break
    fi
    if grep -qE "(panic|SIGSEGV|thread .* panicked|abort)" "${LOG}" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done

  # 关闭 app
  pkill -f "spike-02-tauri" 2>/dev/null || true
  sleep 1.5

  if [[ "${OK}" == "1" ]]; then
    MS=$(grep -oE "window_ready t=[0-9]+ms" "${LOG}" | grep -oE "[0-9]+" | head -1)
    TIMES+=("${MS}")
    SUCCESS=$((SUCCESS + 1))
    echo "Run ${i}: ✅ OK (${MS}ms)"
  else
    FAIL=$((FAIL + 1))
    REASON=$(tail -3 "${LOG}" | tr '\n' ' ' | head -c 200)
    FAIL_REASONS+=("Run ${i}: ${REASON}")
    echo "Run ${i}: ❌ FAIL"
    echo "    ${REASON}"
  fi
  rm -f "${LOG}"
done

echo "---"
echo "Summary: ${SUCCESS}/${RUNS} success · ${FAIL} fail"

if [[ ${#TIMES[@]} -gt 0 ]]; then
  SORTED=($(printf "%s\n" "${TIMES[@]}" | sort -n))
  MID_IDX=$((${#SORTED[@]} / 2))
  echo "Boot times: ${TIMES[*]}"
  echo "Min: ${SORTED[0]}ms · Median: ${SORTED[${MID_IDX}]}ms · Max: ${SORTED[$((${#SORTED[@]} - 1))]}ms"
fi

if [[ ${#FAIL_REASONS[@]} -gt 0 ]]; then
  echo ""
  echo "Fail reasons:"
  printf '  %s\n' "${FAIL_REASONS[@]}"
fi

echo ""
if [[ "${FAIL}" == "0" ]] && [[ "${SUCCESS}" == "${RUNS}" ]]; then
  echo "✅ PASS · 10/10 连续启动成功 · 无崩溃"
  exit 0
else
  echo "❌ FAIL · ${FAIL}/${RUNS} 次失败"
  exit 1
fi
