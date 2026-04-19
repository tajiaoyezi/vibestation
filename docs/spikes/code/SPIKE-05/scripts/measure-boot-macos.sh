#!/usr/bin/env bash
# SPIKE-01 · macOS 冷启动耗时测量脚本
# 用法：./scripts/measure-boot-macos.sh [次数，默认 5]
#
# 测量方式：直接 exec bundle 内部 binary，抓 stderr 的 [SPIKE-01] window_ready t=<ms>ms 数值
# 每次测量之间 purge (需 sudo) 不强求；默认靠 macOS 自己的缓存行为 + 2s 间隔

set -euo pipefail

RUNS="${1:-5}"
BUNDLE_DIR="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/target/release/bundle/macos"
APP_NAME="spike-01-tauri.app"
BIN="${BUNDLE_DIR}/${APP_NAME}/Contents/MacOS/spike-01-tauri"

if [[ ! -x "${BIN}" ]]; then
  echo "ERROR: Release binary not found at ${BIN}"
  echo "请先跑：pnpm tauri build"
  exit 1
fi

echo "Measuring cold boot × ${RUNS} (median will be reported)"
echo "Bundle: ${BUNDLE_DIR}/${APP_NAME}"
echo "---"

TIMES=()
for i in $(seq 1 "${RUNS}"); do
  # 启动 app · 抓 stderr · 窗口 ready 后读取 window_ready 行 · 关闭 app
  # 由于 Tauri 启动后会进入事件循环，需用超时信号强制退出
  LOG=$(mktemp)
  # 背景启动 app · 用 nohup 脱离 shell session · stderr 到 LOG
  nohup "${BIN}" >/dev/null 2>"${LOG}" &

  # 等待 window_ready 出现（最多 10s）
  for _ in $(seq 1 100); do
    if grep -q "window_ready" "${LOG}" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done

  # 读取 t=<ms>ms
  MS=$(grep -oE "window_ready t=[0-9]+ms" "${LOG}" | grep -oE "[0-9]+" | head -1 || echo "-1")

  # 关闭 app
  pkill -f "spike-01-tauri" 2>/dev/null || true
  sleep 2

  if [[ "${MS}" == "-1" ]]; then
    echo "Run ${i}: FAILED to capture window_ready"
    cat "${LOG}" | tail -5 | sed 's/^/    /'
  else
    echo "Run ${i}: ${MS}ms"
    TIMES+=("${MS}")
  fi
  rm -f "${LOG}"
done

echo "---"
if [[ ${#TIMES[@]} -eq 0 ]]; then
  echo "No successful measurements."
  exit 1
fi

# 中位数
SORTED=($(printf "%s\n" "${TIMES[@]}" | sort -n))
MID_IDX=$((${#SORTED[@]} / 2))
MEDIAN="${SORTED[${MID_IDX}]}"

# 最小/最大
MIN="${SORTED[0]}"
MAX="${SORTED[$((${#SORTED[@]} - 1))]}"

echo "Samples: ${TIMES[*]}"
echo "Min: ${MIN}ms  Median: ${MEDIAN}ms  Max: ${MAX}ms"
echo ""
if [[ "${MEDIAN}" -lt 2000 ]]; then
  echo "✅ PASS · macOS cold boot median ${MEDIAN}ms < 2000ms"
else
  echo "❌ FAIL · macOS cold boot median ${MEDIAN}ms ≥ 2000ms (spec < 2s)"
fi
