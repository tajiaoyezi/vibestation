#!/usr/bin/env bash
# SPIKE-01 · Ubuntu 24 冷启动耗时测量脚本（Wayland + X11 通用）
# 用法：
#   Wayland 会话下：./scripts/measure-boot-ubuntu.sh [次数，默认 5]
#   X11 会话下同上（切换 session 通过 GDM 登录界面 · 不在本脚本范围）
#
# 前置：
#   - 已跑过 `pnpm tauri build` 生成 release binary
#   - 系统依赖已装（webkit2gtk-4.1 / libssl-dev / libayatana-appindicator3-dev / librsvg2-dev / patchelf）
#   - Wayland 测试时建议 GNOME 42+ · ibus/fcitx5 已配置中文输入法

set -euo pipefail

RUNS="${1:-5}"
BIN="$(cd "$(dirname "$0")/.." && pwd)/src-tauri/target/release/spike-01-tauri"

if [[ ! -x "${BIN}" ]]; then
  echo "ERROR: Release binary not found at ${BIN}"
  echo "请先跑：pnpm tauri build"
  exit 1
fi

# 检测当前会话类型
SESSION_TYPE="${XDG_SESSION_TYPE:-unknown}"
echo "Session type: ${SESSION_TYPE}"
if [[ "${SESSION_TYPE}" == "wayland" ]]; then
  echo "  WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-unset}"
elif [[ "${SESSION_TYPE}" == "x11" ]]; then
  echo "  DISPLAY=${DISPLAY:-unset}"
fi
echo "---"

echo "Measuring cold boot × ${RUNS} (median will be reported)"
echo "Binary: ${BIN}"
echo "---"

TIMES=()
for i in $(seq 1 "${RUNS}"); do
  LOG=$(mktemp)
  nohup "${BIN}" >/dev/null 2>"${LOG}" &

  for _ in $(seq 1 100); do
    if grep -q "window_ready" "${LOG}" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done

  MS=$(grep -oE "window_ready t=[0-9]+ms" "${LOG}" | grep -oE "[0-9]+" | head -1 || echo "-1")

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
  echo "No successful measurements. Possible causes:"
  echo "  - Wayland/X11 环境变量未就绪"
  echo "  - webkit2gtk 依赖缺失 (sudo apt install libwebkit2gtk-4.1-dev)"
  echo "  - 无图形桌面 (headless SSH 场景不适用)"
  exit 1
fi

SORTED=($(printf "%s\n" "${TIMES[@]}" | sort -n))
MID_IDX=$((${#SORTED[@]} / 2))
MEDIAN="${SORTED[${MID_IDX}]}"
MIN="${SORTED[0]}"
MAX="${SORTED[$((${#SORTED[@]} - 1))]}"

echo "Session: ${SESSION_TYPE}"
echo "Samples: ${TIMES[*]}"
echo "Min: ${MIN}ms  Median: ${MEDIAN}ms  Max: ${MAX}ms"
echo ""
if [[ "${MEDIAN}" -lt 3000 ]]; then
  echo "✅ PASS · Ubuntu ${SESSION_TYPE} cold boot median ${MEDIAN}ms < 3000ms"
else
  echo "❌ FAIL · Ubuntu ${SESSION_TYPE} cold boot median ${MEDIAN}ms ≥ 3000ms (spec < 3s)"
fi
