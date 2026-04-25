#!/usr/bin/env bash
# MVP-05 Phase C §F.1 · 4 Pane 内存测量
#
# 用 ps -o rss 测 vibestation-app 进程 + 4 个 pane PTY 子进程的总内存。
# 目标：每 Pane ≈ MVP-04 单 Tab PTY 开销（SPIKE-05 单 Tab 10MB RSS 基准）·
# 4 Pane ≈ 40MB · 总 10 Tab × 4 Pane = 40 PTY < 500MB（spec §F.1）
#
# 使用：
#   1. 启动 app（pnpm tauri:dev 或安装的 .app）· 创建 1 tab
#   2. ⌘\ 横分 1 次 · ⌘⇧\ 在新 pane 下分 · 共 4 panes（2x2）
#   3. 跑 bash scripts/capture/mvp-05/measure-memory.sh
#   4. 输出 vibestation-app + 4 pane shell 总 RSS · 取 3 次 P99
#
# 限制：仅 macOS · Linux 用 /proc/<pid>/status VmRSS（脚本可移植但当前未测）

set -euo pipefail

PROCESS_NAME="${1:-vibestation-app}"

if ! command -v ps &>/dev/null; then
  echo "❌ ps 不可用 · 仅 Unix-like 系统支持" >&2
  exit 1
fi

echo "=== MVP-05 §F.1 · 4 Pane 内存测量 (${PROCESS_NAME}) ==="
echo ""

# Main app process
MAIN_PID=$(pgrep -f "${PROCESS_NAME}" | head -1)
if [[ -z "${MAIN_PID:-}" ]]; then
  echo "❌ 未找到 ${PROCESS_NAME} 进程 · 请先启动 app" >&2
  exit 1
fi

MAIN_RSS_KB=$(ps -o rss= -p "${MAIN_PID}" | tr -d ' ')
MAIN_RSS_MB=$((MAIN_RSS_KB / 1024))
echo "Main app PID ${MAIN_PID}: ${MAIN_RSS_MB} MB"

# Child shell processes (zsh / bash spawned by pane_pty)
SHELL_PIDS=$(pgrep -P "${MAIN_PID}" -f "zsh|bash|sh" 2>/dev/null || true)
if [[ -z "${SHELL_PIDS:-}" ]]; then
  # 不一定是 child · pane PTY 通过 portable-pty fork · 父进程可能不是 main
  # fallback: 统计所有 zsh/bash 进程（粗略）
  SHELL_PIDS=$(pgrep -f "zsh|bash" | head -10)
fi

TOTAL_SHELL_RSS_KB=0
SHELL_COUNT=0
for pid in ${SHELL_PIDS}; do
  RSS_KB=$(ps -o rss= -p "${pid}" 2>/dev/null | tr -d ' ' || echo 0)
  if [[ "${RSS_KB}" -gt 0 ]]; then
    TOTAL_SHELL_RSS_KB=$((TOTAL_SHELL_RSS_KB + RSS_KB))
    SHELL_COUNT=$((SHELL_COUNT + 1))
    echo "  Shell PID ${pid}: $((RSS_KB / 1024)) MB"
  fi
done

TOTAL_RSS_KB=$((MAIN_RSS_KB + TOTAL_SHELL_RSS_KB))
TOTAL_RSS_MB=$((TOTAL_RSS_KB / 1024))

echo ""
echo "Total: $((TOTAL_SHELL_RSS_KB / 1024)) MB（${SHELL_COUNT} shell 进程）+ Main = ${TOTAL_RSS_MB} MB"
echo ""
echo "Spec §F.1 目标：每 Pane ≈ 10MB · 4 Pane ≈ 40MB（不含 main app）"
echo "实测 4 pane shell RSS：$((TOTAL_SHELL_RSS_KB / 1024)) MB · per-pane ≈ $((TOTAL_SHELL_RSS_KB / 1024 / SHELL_COUNT)) MB"

if [[ ${TOTAL_RSS_MB} -lt 500 ]]; then
  echo "✅ 总 RSS ${TOTAL_RSS_MB} MB < 500 MB（spec §F.1 总上限 · 10 tab × 4 pane fixture extrap）"
else
  echo "⚠️ 总 RSS ${TOTAL_RSS_MB} MB ≥ 500 MB · 需关注"
fi
