#!/usr/bin/env bash
# MVP-08 R-PHASE-E A.6 fixture generator
# 生成临时 git repo · 含 10000 行单文件 · ~5000 行 unstaged diff
# 用于 v0.2 GA 真测大文件滚动帧时长（< 16ms · 见 spec §F.E A.6）
#
# 用法：bash gen-10k-diff.sh <output-dir>
# 退出码：0 = 成功 · 非 0 = 失败（stderr 含具体原因）
#
# 跨平台：macOS（BSD） + Linux（GNU）兼容 · 仅依赖 bash + git + coreutils
# 关联：scripts/fixtures/mvp-08/README.md

set -euo pipefail

# ----------------------------------------------------------------------------
# 参数校验
# ----------------------------------------------------------------------------
if [[ $# -ne 1 ]]; then
    echo "ERROR: 缺少参数 · 用法：$0 <output-dir>" >&2
    echo "示例：$0 /tmp/fixture-10k" >&2
    exit 1
fi

OUTPUT_DIR="$1"

if [[ -z "$OUTPUT_DIR" ]]; then
    echo "ERROR: output-dir 不能为空" >&2
    exit 1
fi

# 防止误删用户重要目录
case "$OUTPUT_DIR" in
    /|/Users|/Users/*|/home|/home/*|"$HOME"|"$HOME"/Documents*|"$HOME"/Desktop*|"$HOME"/Downloads*)
        # 只允许 HOME 下的明确临时路径（如 ~/tmp · ~/scratch）· 否则要求 /tmp 或绝对临时路径
        case "$OUTPUT_DIR" in
            "$HOME"/tmp/*|"$HOME"/scratch/*) ;;
            *)
                echo "ERROR: 输出路径看起来像用户私人目录 · 拒绝写入：$OUTPUT_DIR" >&2
                echo "       推荐使用 /tmp/<name> 或 \$HOME/tmp/<name>" >&2
                exit 1
                ;;
        esac
        ;;
esac

if [[ -e "$OUTPUT_DIR" ]]; then
    echo "ERROR: 输出目录已存在 · 请先删除或换路径：$OUTPUT_DIR" >&2
    echo "       cleanup: rm -rf '$OUTPUT_DIR'" >&2
    exit 1
fi

# 校验依赖
for cmd in git seq awk wc; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "ERROR: 缺少依赖命令：$cmd" >&2
        exit 1
    fi
done

# ----------------------------------------------------------------------------
# 主流程
# ----------------------------------------------------------------------------
echo "[gen-10k-diff] 创建 fixture：$OUTPUT_DIR"

mkdir -p "$OUTPUT_DIR"

# Git init · 用 main 分支（兼容老 git · 不依赖 init.defaultBranch）
cd "$OUTPUT_DIR"
git init --quiet
git checkout -q -b main 2>/dev/null || true  # 已是 main 时跳过

# 局部 git config · 不污染全局
git config user.name "MVP-08 Fixture"
git config user.email "fixture@vibestation.local"
git config commit.gpgsign false

# 写 README（fixture 自身用途说明）
cat > README.md <<'EOF'
# MVP-08 A.6 Fixture · 10k 行 diff

由 `scripts/fixtures/mvp-08/gen-10k-diff.sh` 生成。

## 内容

- `lorem.txt`：单文件 · 10000 行 · 行号 + 短语 pattern
- `git diff`：~5000 行 unstaged 改动（每两行改一行）

## 用途

v0.2 GA 真测 · DevTools Performance 测大文件滚动帧时长（spec MVP-08 §F.E A.6 · < 16ms）。

## 清理

跑完测试后 `rm -rf` 整个目录。
EOF

# 生成 10000 行 lorem.txt
# pattern: 行号 + 简单短语 · 让 diff 算法看到结构 · 但不至于走 binary fallback
# 仅用 awk · 不依赖 python（POSIX 通用 · macOS BSD + Linux GNU 兼容）
awk 'BEGIN {
    phrases[0] = "lorem ipsum dolor sit amet"
    phrases[1] = "consectetur adipiscing elit"
    phrases[2] = "sed do eiusmod tempor incididunt"
    phrases[3] = "ut labore et dolore magna aliqua"
    phrases[4] = "ut enim ad minim veniam"
    phrases[5] = "quis nostrud exercitation ullamco"
    phrases[6] = "laboris nisi ut aliquip ex ea"
    phrases[7] = "commodo consequat duis aute irure"
    for (i = 1; i <= 10000; i++) {
        printf "line %05d - %s\n", i, phrases[i % 8]
    }
}' > lorem.txt

# Sanity check 行数
LINE_COUNT=$(wc -l < lorem.txt | awk '{print $1}')
if [[ "${LINE_COUNT}" -lt 9990 ]]; then
    echo "ERROR: lorem.txt 行数不对 (期望 ~10000 · 实际 ${LINE_COUNT})" >&2
    exit 1
fi

# Initial commit
git add README.md lorem.txt
git commit -q -m "Initial 10k line fixture for MVP-08 A.6"

# 修改前 5000 行（每行后追加 " MODIFIED"）· 不 commit · 形成 unstaged diff
# 用 awk 重写文件 · 保证 macOS BSD + Linux GNU 行为一致
awk 'NR <= 5000 { print $0 " MODIFIED"; next } { print }' lorem.txt > lorem.txt.new
mv lorem.txt.new lorem.txt

# 校验 unstaged diff 行数 (应该 ~10000 行 · 因为 5000 行删 + 5000 行加)
DIFF_LINES=$(git diff lorem.txt | wc -l | awk '{print $1}')
if [[ "${DIFF_LINES}" -lt 4000 ]]; then
    echo "ERROR: unstaged diff 太小 (期望 ~10000 行 diff output · 实际 ${DIFF_LINES})" >&2
    exit 1
fi

# ----------------------------------------------------------------------------
# 报告
# ----------------------------------------------------------------------------
echo "[gen-10k-diff] 完成"
echo "  路径: ${OUTPUT_DIR}"
echo "  lorem.txt 行数: ${LINE_COUNT}"
echo "  git diff 行数: ${DIFF_LINES} (含 +/- markers)"
echo ""
echo "下一步:"
echo "  1. cd '${OUTPUT_DIR}' && git diff lorem.txt | head"
echo "  2. pnpm tauri:dev · 加载该路径作为 workspace · DevTools 测 Diff overlay 滚动帧时长"
echo "  3. 跑完 cleanup: rm -rf '${OUTPUT_DIR}'"

exit 0
