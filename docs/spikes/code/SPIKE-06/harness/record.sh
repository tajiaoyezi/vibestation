#!/usr/bin/env bash
# SPIKE-06 harness · 通用 CLI 录制 wrapper
#
# 用法: ./record.sh <scenario-name>
#
# 功能:
#   1. 查找 scenarios/<name>.sh
#   2. 用 BSD script(1) 录 3 次到 ~/.vibestation-spike-raw/SPIKE-06/<name>-{01,02,03}.raw
#   3. 输出字节数作为 smoke
#
# 输出文件约定: `.raw` 后缀（按 spec §A.5.1 · 防误 commit）
# 存储位置: $HOME/.vibestation-spike-raw/SPIKE-06/（repo worktree 外 · gitignore 天然覆盖不到）

set -euo pipefail

SCENARIO="${1:-}"
if [[ -z "$SCENARIO" ]]; then
  echo "Usage: $0 <scenario-name>" >&2
  echo "  scenarios/ 目录下查找 <scenario-name>.sh" >&2
  exit 1
fi

HARNESS_DIR="$(cd "$(dirname "$0")" && pwd)"
SCENARIO_FILE="${HARNESS_DIR}/scenarios/${SCENARIO}.sh"

if [[ ! -f "$SCENARIO_FILE" ]]; then
  echo "❌ scenario not found: $SCENARIO_FILE" >&2
  echo "   可用 scenarios:" >&2
  ls "${HARNESS_DIR}/scenarios/" 2>/dev/null | sed 's/\.sh$//' | sed 's/^/     - /' >&2
  exit 1
fi

RAW_DIR="${HOME}/.vibestation-spike-raw/SPIKE-06"
mkdir -p "$RAW_DIR"

echo "▶ scenario: $SCENARIO"
echo "  scenario file: $SCENARIO_FILE"
echo "  raw dir: $RAW_DIR"
echo ""

for N in 01 02 03; do
  OUT="${RAW_DIR}/${SCENARIO}-${N}.raw"
  echo "  ▶ run ${N}/3 → $OUT"
  # BSD script(1): -q 安静 · typescript 存 · 子命令退出码透传
  # 注: macOS script 语法 "script [-q] file command ..."
  if script -q "$OUT" bash "$SCENARIO_FILE" > /dev/null 2>&1; then
    SIZE=$(wc -c < "$OUT" | tr -d ' ')
    echo "    ✓ ${SIZE} bytes"
  else
    echo "    ✗ script(1) exited non-zero · 检查 $OUT" >&2
  fi
done

echo ""
echo "✅ recorded: ${RAW_DIR}/${SCENARIO}-*.raw"
echo ""
echo "下一步:"
echo "  1. 脱敏: ./redact.py --input <raw> --output <txt>"
echo "  2. 验证: ./verify.sh <redacted-dir>"
