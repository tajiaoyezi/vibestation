#!/usr/bin/env bash
# SPIKE-06 verify · 脱敏后目录跑 gitleaks 扫描
# 用途: 确保脱敏后 0 敏感值 · spec §A.5.3 硬要求
#
# Usage: ./verify.sh <redacted-dir>

set -euo pipefail

DIR="${1:-}"
if [[ -z "$DIR" ]]; then
  echo "Usage: $0 <redacted-dir>" >&2
  echo "  e.g. $0 ../../raw/SPIKE-06/" >&2
  exit 1
fi

if [[ ! -d "$DIR" ]]; then
  echo "❌ not a directory: $DIR" >&2
  exit 1
fi

echo "▶ verify: $DIR"
echo ""

if ! command -v gitleaks > /dev/null; then
  echo "⚠️  gitleaks not installed"
  echo "   Install: brew install gitleaks"
  echo "   Skipping scan · PR 2 前必须装跑 zero-hit"
  echo ""
  echo "Fallback: 手工 grep 常见 pattern（不替代 gitleaks）"
  echo "  - sk-[A-Za-z0-9]{40,}  (OpenAI key)"
  echo "  - sk-ant-[A-Za-z0-9_-]{20,} (Anthropic key)"
  echo "  - eyJ[A-Za-z0-9_-]+\\.[A-Za-z0-9_-]+\\.[A-Za-z0-9_-]+  (JWT)"
  echo "  - /Users/[a-zA-Z0-9_.-]+  (path leak · except /Users/USER)"

  # 简易 fallback grep（不 block）
  MATCH=0
  for pattern in \
    'sk-[A-Za-z0-9]\{40,\}' \
    'sk-ant-[A-Za-z0-9_-]\{20,\}' \
    'eyJ[A-Za-z0-9_-]\+\.[A-Za-z0-9_-]\+\.[A-Za-z0-9_-]\+' \
    '/Users/[^/USER ]'
  do
    HITS=$(grep -rE "$pattern" "$DIR" 2>/dev/null | grep -v '/Users/USER' || true)
    if [[ -n "$HITS" ]]; then
      echo "   ⚠️  grep fallback hit pattern '$pattern':"
      echo "$HITS" | head -3 | sed 's/^/      /'
      MATCH=$((MATCH + 1))
    fi
  done

  if [[ $MATCH -eq 0 ]]; then
    echo "   ✓ grep fallback 未命中 · 但这不是权威结论 · 装 gitleaks 再跑"
  else
    echo "   ✗ grep fallback 命中 $MATCH 个 pattern · 查上面输出"
    exit 2
  fi

  exit 0
fi

# 正式 gitleaks 扫描
CONFIG="$(cd "$(dirname "$0")/.." && pwd)/.gitleaks.toml"
if [[ -f "$CONFIG" ]]; then
  CONFIG_ARG="--config $CONFIG"
else
  CONFIG_ARG=""
fi

echo "gitleaks version:"
gitleaks version
echo ""

echo "▶ gitleaks detect --source '$DIR' --no-git --verbose --redact $CONFIG_ARG"
if gitleaks detect --source "$DIR" --no-git --verbose --redact $CONFIG_ARG; then
  echo ""
  echo "✅ gitleaks zero-hit: $DIR 通过"
else
  echo ""
  echo "❌ gitleaks 命中 · 查上面输出 · PR 2 前必须 zero-hit"
  exit 2
fi
