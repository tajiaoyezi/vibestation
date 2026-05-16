#!/usr/bin/env bash
# SPIKE-07.5 · 结构化模式 corpus 录制脚本（§E.9 可复现 deliverable）
#
# 录 6 场景 × 2 CLI × 3 take = 36 条 `.structured.jsonl`（未脱敏 → /tmp · 脱敏后 → corpus/）
# Phase 0 probe 定型命令：
#   claude -p --output-format stream-json --verbose --include-hook-events --no-session-persistence "<P>" < /dev/null
#   codex  exec --json "<P>" < /dev/null
#
# 场景构造（decision-grade 可追溯 · prompt 全 generic 无密钥 · redaction-safe by construction）：
#   happy_path        : 简单 prompt · 正常响应
#   long_stream       : 长输出 prompt（多 delta / 长 message）
#   mixed_ansi_json   : 响应内容含 ANSI 转义 + JSON（结构化外层仍干净 · 测内容提取）
#   network_error     : BASE_URL 指向 127.0.0.1:1（connection refused · 确定性网络错误）
#   auth_fail         : 无效 API key env（best-effort · OAuth 登录态可能忽略 env · 见 §限制）
#   interrupt_residual: 长 prompt 后台启动 → 4s 后 SIGTERM → 捕获部分结构化事件（spec §C.1 重定义）
#
# 用法：bash record_corpus.sh <out_dir>   (默认 /tmp/spike075-raw)
set -u
CLAUDE=/Users/leaf/.local/bin/claude
CODEX=/Users/leaf/.nvm/versions/node/v24.14.1/bin/codex
OUT="${1:-/tmp/spike075-raw}"
mkdir -p "$OUT"

CLAUDE_BASE=(-p --output-format stream-json --verbose --include-hook-events --no-session-persistence)

# 3 take 用 3 个不同 prompt（覆盖响应多样性 · 对齐 SPIKE-06 3-take 习惯）
declare -a HAPPY=("用一句中文说明什么是 Git rebase。" "用一句话说明什么是 TCP 三次握手。" "用一句话说明什么是 Rust 的所有权。")
declare -a LONG=("列出 20 个常用 Git 命令，每个用一句话说明用途。" "列出 20 个 Linux 命令，每个一句话说明。" "列出 20 个 Rust 标准库类型，每个一句话说明。")
declare -a MIXED=('输出一个示例：先打印含 ANSI 颜色码的一行（用 \033[31m 红 \033[0m），然后输出 JSON {"lang":"zh","ok":true}。' '给一段含 ANSI 转义序列的文本，再附 JSON 配置 {"a":1,"b":[2,3]}。' '演示 ANSI \033[1m 粗体 \033[0m 后跟 JSON 数组 [{"x":1},{"y":2}]。')

rec_claude() { # scenario take prompt [extra-env]
  local sc="$1" tk="$2" pr="$3"
  echo "  claude/$sc/$tk ..."
  env ${4:-} "$CLAUDE" "${CLAUDE_BASE[@]}" "$pr" < /dev/null \
    > "$OUT/claude_${sc}_${tk}.structured.jsonl" 2> "$OUT/claude_${sc}_${tk}.err"
  echo "    exit=$? lines=$(wc -l < "$OUT/claude_${sc}_${tk}.structured.jsonl")"
}
rec_codex() { # scenario take prompt [extra-env]
  local sc="$1" tk="$2" pr="$3"
  echo "  codex/$sc/$tk ..."
  env ${4:-} "$CODEX" exec --json "$pr" < /dev/null \
    > "$OUT/codex_${sc}_${tk}.structured.jsonl" 2> "$OUT/codex_${sc}_${tk}.err"
  echo "    exit=$? lines=$(wc -l < "$OUT/codex_${sc}_${tk}.structured.jsonl")"
}

for t in 1 2 3; do
  i=$((t-1))
  echo "== take $t =="
  rec_claude happy_path "$t" "${HAPPY[$i]}"
  rec_codex  happy_path "$t" "${HAPPY[$i]}"
  rec_claude long_stream "$t" "${LONG[$i]}"
  rec_codex  long_stream "$t" "${LONG[$i]}"
  rec_claude mixed_ansi_json "$t" "${MIXED[$i]}"
  rec_codex  mixed_ansi_json "$t" "${MIXED[$i]}"
  # network_error：BASE_URL → 拒绝连接（确定性）
  rec_claude network_error "$t" "hi" "ANTHROPIC_BASE_URL=http://127.0.0.1:1"
  rec_codex  network_error "$t" "hi" "OPENAI_BASE_URL=http://127.0.0.1:1 OPENAI_API_KEY=sk-x"
  # auth_fail：无效 key（best-effort）
  rec_claude auth_fail "$t" "hi" "ANTHROPIC_API_KEY=sk-ant-invalid000 ANTHROPIC_BASE_URL=https://api.anthropic.com"
  rec_codex  auth_fail "$t" "hi" "OPENAI_API_KEY=sk-invalid000"
done

# interrupt_residual：长 prompt 后台 → 4s SIGTERM → 部分结构化事件（spec §C.1 重定义）
for t in 1 2 3; do
  echo "== interrupt take $t =="
  "$CLAUDE" "${CLAUDE_BASE[@]}" "详细解释 Git 内部对象模型，至少 800 字。" < /dev/null \
    > "$OUT/claude_interrupt_residual_${t}.structured.jsonl" 2> "$OUT/claude_interrupt_residual_${t}.err" &
  cp=$!; sleep 4; kill -TERM "$cp" 2>/dev/null; wait "$cp" 2>/dev/null
  echo "  claude/interrupt/$t lines=$(wc -l < "$OUT/claude_interrupt_residual_${t}.structured.jsonl")"
  "$CODEX" exec --json "详细解释 TCP 拥塞控制算法，至少 800 字。" < /dev/null \
    > "$OUT/codex_interrupt_residual_${t}.structured.jsonl" 2> "$OUT/codex_interrupt_residual_${t}.err" &
  xp=$!; sleep 4; kill -TERM "$xp" 2>/dev/null; wait "$xp" 2>/dev/null
  echo "  codex/interrupt/$t lines=$(wc -l < "$OUT/codex_interrupt_residual_${t}.structured.jsonl")"
done

echo "DONE · $(ls "$OUT"/*.structured.jsonl 2>/dev/null | wc -l) jsonl in $OUT"
