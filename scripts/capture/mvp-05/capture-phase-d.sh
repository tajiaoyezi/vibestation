#!/usr/bin/env bash
# MVP-05 Phase D · 5 截图 + 30s 录屏 capture script (macOS)
#
# Spec §Phase D 要求 ≥ 5 张截图 + 30s 录屏 · 覆盖：
#   01-solo-single-pane.png       Solo 单 pane（默认状态）
#   02-horizontal-2-panes.png     ⌘\ 右分屏后
#   03-vertical-2-panes.png       ⌘⇧\ 下分屏后（独立场景 · 需先 Solo）
#   04-2x2-quad-panes.png         右分 + 下分 = 2x2 4 pane
#   05-smart-layout-menu.png      ⌘⇧P 命令面板（dry-run 预览）
#   06-after-smart-apply.png      Solo 应用后（验证关闭非聚焦 pane）
#   07-flow-recording.mov         30s 完整流程录屏（手工跑）
#
# 前提：
#   1. pnpm tauri:dev 已启动 · 窗口 ready · 在前台
#   2. 至少 1 tab open（pane mode · 即 panesByTabId 有数据）
#   3. macOS · screencapture / osascript 工具链
#
# 用法：bash scripts/capture/mvp-05/capture-phase-d.sh

set -euo pipefail

OUT_DIR="docs/runtime-evidence/mvp-05"
mkdir -p "${OUT_DIR}"

if [[ "$(uname)" != "Darwin" ]]; then
  echo "WARNING: 当前仅 macOS 实施 · Linux capture 留给 kimi-ubuntu24" >&2
  exit 1
fi

echo "确保 vibestation app 已启动 · 窗口在前台 · 至少 1 tab（pane mode）"
echo "按 Enter 继续 capture..."
read -r

# 取 vibestation 窗口 ID（assumes "Vibestation" in window title）
WINDOW_ID="$(osascript -e 'tell application "System Events" to id of front window of (first process whose name contains "Vibestation")' 2>/dev/null || echo "")"

if [[ -z "${WINDOW_ID:-}" ]]; then
  echo "ERROR: 未找到 Vibestation 窗口 · 请确保 app 启动" >&2
  exit 1
fi

# 01 Solo 单 pane（默认状态）
echo "01 / 6 · Solo 单 pane（当前状态）"
sleep 1
screencapture -x -l "${WINDOW_ID}" "${OUT_DIR}/01-solo-single-pane.png"

# 02 横分屏 ⌘\
echo "02 / 6 · 横分屏（⌘\\）"
osascript -e 'tell application "System Events" to keystroke "\\" using command down' 2>/dev/null || true
sleep 1
screencapture -x -l "${WINDOW_ID}" "${OUT_DIR}/02-horizontal-2-panes.png"

# 04 2x2（在 02 基础上 ⌘⇧\）
echo "04 / 6 · 2x2 quad panes（⌘⇧\\）"
osascript -e 'tell application "System Events" to keystroke "|" using {command down, shift down}' 2>/dev/null || true
sleep 1
screencapture -x -l "${WINDOW_ID}" "${OUT_DIR}/04-2x2-quad-panes.png"

# 回到 Solo · 03 vertical 单独再做
echo "  → 应用 Solo 回到单 pane"
osascript -e 'tell application "System Events" to keystroke "p" using {command down, shift down}' 2>/dev/null || true
sleep 1
osascript -e 'tell application "System Events" to keystroke return' 2>/dev/null || true  # 选 Solo
sleep 0.5
osascript -e 'tell application "System Events" to keystroke return' 2>/dev/null || true  # 确认
sleep 1

# 03 vertical（⌘⇧\ on Solo · 直接下分）
echo "03 / 6 · vertical 2 panes（Solo 后 ⌘⇧\\）"
osascript -e 'tell application "System Events" to keystroke "|" using {command down, shift down}' 2>/dev/null || true
sleep 1
screencapture -x -l "${WINDOW_ID}" "${OUT_DIR}/03-vertical-2-panes.png"

# 05 Smart Layouts 命令面板
echo "05 / 6 · Smart Layouts 命令面板（⌘⇧P）"
osascript -e 'tell application "System Events" to keystroke "p" using {command down, shift down}' 2>/dev/null || true
sleep 1
screencapture -x -l "${WINDOW_ID}" "${OUT_DIR}/05-smart-layout-menu.png"

# 06 应用 Solo 后
echo "06 / 6 · 应用 Solo 后"
osascript -e 'tell application "System Events" to keystroke return' 2>/dev/null || true
sleep 0.5
osascript -e 'tell application "System Events" to keystroke return' 2>/dev/null || true
sleep 1
screencapture -x -l "${WINDOW_ID}" "${OUT_DIR}/06-after-smart-apply.png"

echo ""
echo "DONE: 6 张截图 capture 完成 · 输出 ${OUT_DIR}/"
echo ""
echo "07 录屏（手工 · 30s）："
echo "  screencapture -V 30 -x -l ${WINDOW_ID} ${OUT_DIR}/07-flow-recording.mov"
echo "  录屏中手动操作：⌘\\ → ⌘⇧\\ → 拖拽 splitter → ⌘⇧P → Solo apply"
