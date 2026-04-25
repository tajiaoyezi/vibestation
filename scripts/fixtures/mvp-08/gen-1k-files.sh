#!/usr/bin/env bash
# MVP-08 R-PHASE-E F.3 fixture generator
# 生成临时 git repo · 含 1000 文件 · ~500 文件 unstaged 改动
# 用于 v0.2 GA 真测 1k 文件 Status 列表渲染（< 70ms · 见 spec §F.E F.3）
#
# 用法：bash gen-1k-files.sh <output-dir>
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
    echo "示例：$0 /tmp/fixture-1k" >&2
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
for cmd in git seq find wc awk; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "ERROR: 缺少依赖命令：$cmd" >&2
        exit 1
    fi
done

# ----------------------------------------------------------------------------
# 主流程
# ----------------------------------------------------------------------------
echo "[gen-1k-files] 创建 fixture：$OUTPUT_DIR"

mkdir -p "$OUTPUT_DIR/files"
cd "$OUTPUT_DIR"

# Git init · 用 main 分支
git init --quiet
git checkout -q -b main 2>/dev/null || true

# 局部 git config · 不污染全局
git config user.name "MVP-08 Fixture"
git config user.email "fixture@vibestation.local"
git config commit.gpgsign false

# 写 fixture README
cat > README.md <<'EOF'
# MVP-08 F.3 Fixture · 1k 文件 status

由 `scripts/fixtures/mvp-08/gen-1k-files.sh` 生成。

## 内容

- `files/file-0001.txt` - `files/file-1000.txt`：1000 个小文件（每个 < 1KB）
- 内容是文件名 + 创建 timestamp + 简单内容
- 修改后 ~500 文件 unstaged

## 用途

v0.2 GA 真测 · DevTools Performance 测 1k 文件 Status 面板渲染（spec MVP-08 §F.E F.3 · < 70ms）。

## 清理

跑完测试后 `rm -rf` 整个目录。
EOF

# 生成 1000 文件 · 每个文件内容简单但不重复（避免 git 深度去重）
echo "[gen-1k-files] 生成 1000 文件..."
TIMESTAMP=$(date +%s)

# 用 awk 一次性生成 1000 文件 · 避免 1000 次 fork（macOS spawn 慢）
# 但 awk 写文件需要逐个 close · 简化用 shell loop · 加 batch 进度输出
i=1
while [[ "$i" -le 1000 ]]; do
    # printf 0-padding 到 4 位 · macOS BSD + Linux GNU 都支持
    NAME=$(printf "file-%04d" "$i")
    cat > "files/${NAME}.txt" <<EOL
${NAME}
created=${TIMESTAMP}
seq=${i}
content=lorem ipsum dolor sit amet ${i}
EOL
    i=$((i + 1))
    # 每 200 文件输出一次进度
    if [[ $((i % 200)) -eq 0 ]]; then
        echo "[gen-1k-files]   ${i}/1000 ..."
    fi
done

# Initial commit
git add README.md files/
git commit -q -m "Initial 1000-file fixture for MVP-08 F.3"

# 修改前 500 个文件（追加一行）· 不 commit · 形成 unstaged changes
echo "[gen-1k-files] 修改前 500 文件 (unstaged) ..."
i=1
while [[ "$i" -le 500 ]]; do
    NAME=$(printf "file-%04d" "$i")
    echo "modified=true" >> "files/${NAME}.txt"
    i=$((i + 1))
done

# 校验
FILE_COUNT=$(find files -type f | wc -l | awk '{print $1}')
if [[ "${FILE_COUNT}" -lt 1000 ]]; then
    echo "ERROR: 文件数不对 (期望 1000 · 实际 ${FILE_COUNT})" >&2
    exit 1
fi

CHANGE_COUNT=$(git status -s | wc -l | awk '{print $1}')
if [[ "${CHANGE_COUNT}" -lt 400 ]]; then
    echo "ERROR: 改动数太少 (期望 ~500 · 实际 ${CHANGE_COUNT})" >&2
    exit 1
fi

# ----------------------------------------------------------------------------
# 报告
# ----------------------------------------------------------------------------
echo "[gen-1k-files] 完成"
echo "  路径: ${OUTPUT_DIR}"
echo "  files 总数: ${FILE_COUNT}"
echo "  unstaged 改动数: ${CHANGE_COUNT}"
echo ""
echo "下一步:"
echo "  1. cd '${OUTPUT_DIR}' && git status -s | head"
echo "  2. pnpm tauri:dev · 加载该路径作为 workspace · DevTools 测 GIT STATUS Refresh 渲染时长"
echo "  3. 跑完 cleanup: rm -rf '${OUTPUT_DIR}'"

exit 0
