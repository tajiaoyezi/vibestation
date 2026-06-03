# MVP-08 Fixture Generators · R-PHASE-E v0.2 真测前置

> **用途**：生成 MVP-08 R-PHASE-E v0.2 真测所需的临时 git fixture · 让 A.6（10k 行 diff 滚动）+ F.3（1k 文件 Status 渲染）可在不污染用户私人 workspace 的前提下精确测量。
>
> **背景**：PR #117 + PR #136 R-PHASE-E round 2 因当前测试 workspace（`ubuntu-claw`）clean · 缺 10k 行 diff 与 1k 文件 changes · A.6 / F.3 推到 v0.2 GA 补齐。本目录脚本提供可复现的 fixture 生成方案 · 由 v0.2 主 agent / 用户本地用 Chrome DevTools Performance panel 跑 trace。
>
> **关联**：
>
> - Spec：[`docs/tasks/MVP-08-diff-and-git-status.md`](../../../docs/tasks/MVP-08-diff-and-git-status.md) §F.E（A.6 / F.3 acceptance）
> - 度量记录：`docs/runtime-evidence/mvp-08/phase-e/metrics-phase-e.md`（未产出 · capture mandate 已 [ADR-023](../../../docs/adr/ADR-023-capture-mandate-removed.md) 移除）
> - PR #136 round 3：限制中标明 v0.2 真测需用 fixture · 本脚本兑现该承诺

## 脚本清单

| 脚本              | 用途                                                         | spec 对应                 |
| ----------------- | ------------------------------------------------------------ | ------------------------- |
| `gen-10k-diff.sh` | 创建临时 git repo · 单文件 10000 行 · ~5000 行 unstaged diff | A.6 · 大文件滚动帧时长    |
| `gen-1k-files.sh` | 创建临时 git repo · 1000 文件 · ~500 unstaged 改动           | F.3 · 1k 文件 Status 渲染 |

## 使用方法

### 基本用法

```bash
# A.6 fixture（10k 行 diff）
bash scripts/fixtures/mvp-08/gen-10k-diff.sh /tmp/fixture-10k

# F.3 fixture（1k 文件）
bash scripts/fixtures/mvp-08/gen-1k-files.sh /tmp/fixture-1k
```

输出目录立即可用。失败时退出码非 0 · 错误信息打到 stderr。

### v0.2 真测完整流程（A.6 示例）

```bash
# 1. 生成 fixture
bash scripts/fixtures/mvp-08/gen-10k-diff.sh /tmp/a6-fixture

# 2. 启动 vibestation 并加载该 fixture 作为 workspace
pnpm tauri:dev
#   App 启动后 · 通过 Workspace Picker 添加 /tmp/a6-fixture 作为 workspace

# 3. 触发 Diff overlay：
#   - 点击 GIT STATUS 中 lorem.txt 的 unstaged 行
#   - 主区 Diff overlay 打开 · 显示 ~5000 行差异

# 4. Chrome DevTools Performance trace：
#   - 在 Diff overlay 内 · 用拖动滚动条或键盘 PageDown 滚动 3 秒
#   - DevTools → Performance → Record → 滚动 → Stop
#   - 看 Frames track · 找最长帧时间（worst-case frame budget）
#   - 重复 3 次 · 取 P99
#   - spec 阈值：< 16ms

# 5. 清理
rm -rf /tmp/a6-fixture
```

F.3（1k 文件 Status 渲染）流程同上 · 把 fixture 换成 `gen-1k-files.sh` 输出 · 触发面板换成 GIT STATUS 的 Refresh 按钮 · DevTools 录 click→DOM commit。

## 输出结构

### `gen-10k-diff.sh <output-dir>`

```text
<output-dir>/
├── .git/                    # git 初始化 · main 分支 · 1 commit
├── README.md                # fixture 自身的简单 README · 标明用途
└── lorem.txt                # 10000 行 · 内容是行号 + 简单 lorem ipsum 风格短语
```

`git diff` 显示前 ~5000 行被修改（每两行改一行的 pattern · 形成 ~5000 行 unstaged diff）。

### `gen-1k-files.sh <output-dir>`

```text
<output-dir>/
├── .git/                    # git 初始化 · main 分支 · 1 commit
├── README.md                # fixture 自身的简单 README
└── files/
    ├── file-0001.txt
    ├── file-0002.txt
    ├── ...
    └── file-1000.txt        # 1000 个文件 · 每个 < 1KB
```

`git status -s` 显示 ~500 文件变化（500 modified · 余下 500 仍 clean · 形成可观察的 Staged + Unstaged 混合场景）。

## 设计约束

| 约束                 | 说明                                                                |
| -------------------- | ------------------------------------------------------------------- |
| 跨平台               | macOS（BSD coreutils） + Linux（GNU coreutils）双兼容               |
| 0 外部依赖           | 仅依赖 `bash` + `git` + `seq` / `wc` / `find` / `awk`（POSIX 通用） |
| 可丢弃               | 输出目录是临时 fixture · 跑完 v0.2 trace 后 `rm -rf`                |
| 不污染用户 workspace | 输出路径由调用方指定 · 默认推荐 `/tmp/<name>`                       |
| 退出码语义           | 0 = 成功 · 非 0 = 失败 + stderr 错误信息                            |
| set -euo pipefail    | 严格 bash 模式 · 任何命令失败立即 abort                             |

## 反模式（不要做）

| 反模式                                                 | 正确做法                                        |
| ------------------------------------------------------ | ----------------------------------------------- |
| 在用户主 workspace（`~/`）生成 fixture                 | 用 `/tmp/` 等临时目录 · 跑完 `rm -rf`           |
| 把 fixture 进 git（commit 到本 repo）                  | fixture 是临时产物 · 不进 repo · 仅脚本进 repo  |
| 依赖 python / node / 自定义工具                        | 仅 bash + git + coreutils · 任何 \*nix 系统可跑 |
| 在脚本里硬编码 macOS-only 命令（如 `seq` 的 BSD 行为） | 显式测试两端 · 用 POSIX 子集                    |

## 自测流程

每次修改本目录脚本后 · 按下面 4 步自测：

```bash
# 1. 跑 10k 脚本
bash scripts/fixtures/mvp-08/gen-10k-diff.sh /tmp/fixture-10k-test
echo "exit code: $?"

# 2. 验证 10k fixture
test -d /tmp/fixture-10k-test/.git || { echo "FAIL: no .git"; exit 1; }
LINES=$(wc -l < /tmp/fixture-10k-test/lorem.txt)
test "$LINES" -ge 9990 || { echo "FAIL: not 10k lines (got $LINES)"; exit 1; }
DIFF_LINES=$(git -C /tmp/fixture-10k-test diff | wc -l)
test "$DIFF_LINES" -ge 4000 || { echo "FAIL: diff too small (got $DIFF_LINES)"; exit 1; }
echo "10k fixture OK · $LINES lines · $DIFF_LINES diff lines"

# 3. 跑 1k 脚本
bash scripts/fixtures/mvp-08/gen-1k-files.sh /tmp/fixture-1k-test
echo "exit code: $?"

# 4. 验证 1k fixture
test -d /tmp/fixture-1k-test/.git || { echo "FAIL: no .git"; exit 1; }
FILE_COUNT=$(find /tmp/fixture-1k-test/files -type f | wc -l)
test "$FILE_COUNT" -ge 1000 || { echo "FAIL: not 1000 files (got $FILE_COUNT)"; exit 1; }
CHANGES=$(git -C /tmp/fixture-1k-test status -s | wc -l)
test "$CHANGES" -ge 400 || { echo "FAIL: changes too few (got $CHANGES)"; exit 1; }
echo "1k fixture OK · $FILE_COUNT files · $CHANGES changes"

# 5. 清理
rm -rf /tmp/fixture-10k-test /tmp/fixture-1k-test
```

## 何时升级本脚本

| 触发                                                                            | 动作                                                                   |
| ------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| MVP-08 spec 改 A.6 / F.3 阈值（如 10k → 50k 行）                                | 改脚本 line/file 数量 + 同步本 README + metrics-phase-e.md v0.2 复现段 |
| v0.2 真测发现 fixture pattern 不真实（如 lorem ipsum 太规律 · diff 算法走捷径） | 引入更现实的 pattern（如真实代码 sample · 多类型 hunk）                |
| 加新 acceptance（如 binary diff · large file fallback 真测）                    | 新增 `gen-binary-diff.sh` / `gen-large-file.sh` 脚本 + 索引到本 README |

---

**Maintained by**: Vibestation 项目 agent · session 19+ · 2026-04-25 · Claude Code
