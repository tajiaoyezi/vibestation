# MVP-05 Phase D · Capture Playbook · Arbiter 30-45 min 收口

> **目标**：按本 playbook 一次跑完 · 产出 **14 张截图（01-06 capture flow + 08-11 destructive path + 12 toast + 13 focus + 14 confirm UI + 15 sole-pane close）+ 30s 录屏 + filled metrics（F.1-F.6 性能数字 + §A inheritance 文本表 + §5.5.3 process check + §D.4/§E.2/§E.3 文本结果）** · 主 agent 据此把 MVP-05 spec frontmatter 从 `ready` 翻 `done`。
>
> **触发条件**：v0.1 GA gate 前 · MVP-05 spec acceptance 21 项中 17 项 GUI 行为 + 6 项性能数字必须 Arbiter 本地实测 · 主 agent（CLI）做不了。
>
> **预计时间**：30-45 min · 含 5 min 前置 + 25-35 min 实测 + 5 min commit。
>
> **关联**：
> - spec：[`docs/tasks/MVP-05-pane-split-single-level.md`](../../tasks/MVP-05-pane-split-single-level.md)
> - metrics：[`metrics-mvp-05.md`](./metrics-mvp-05.md)
> - capture script：[`scripts/capture/mvp-05/capture-phase-d.sh`](../../../scripts/capture/mvp-05/capture-phase-d.sh)
> - measure script：[`scripts/capture/mvp-05/measure-memory.sh`](../../../scripts/capture/mvp-05/measure-memory.sh)
> - ADR-011 runtime evidence location

---

## 🛡 11 Invariant（playbook design intent · Codex round 1+2+3 review fix · 2026-05-03）

未来改 playbook 任何段前必先对照本 11 条 · 防 Codex 已抓的 10 类 finding 复发。**round 1（I1-I4）+ round 2（I5-I8）+ round 3（I9-I11）11 条均经实战验证**。

### Round 1 invariants（基础 4 条）

- **I1 · precondition 必须显式验证 · 不能假设**
  - 违反案例：fixture 起点 ≠ capture 脚本假设的起点 → 6 PNG 都生成但内容错（round 1 finding 1）
  - 应用：每个自动化步骤前必须有 setup 步骤把状态归零 · 自动化跑完后必须有 content validation（不只是 file exists/size）
- **I2 · spec 强制 acceptance 不能在 playbook 标"可选" / "optional"**
  - 违反案例：spec §C 要求 vim/nano 编辑状态二次确认 · playbook 标"vim/nano 状态可选" → evidence pass 但用户数据丢失 path 没测（round 1 finding 2）
  - 应用：playbook 任何 acceptance 验证步骤的 mandatory/optional 必须等同 spec
- **I3 · playbook 测量 pass 条件必须等价于 spec 的 acceptance 通过条件**
  - 违反案例：spec F.1 要 40 PTY (10 tab × 4 pane) < 500MB · playbook 测 4 pane < 500MB → 实际 40 PTY 可能远超 500MB 但 playbook 标 PASS（round 1 finding 3）
  - 应用：playbook 测量公式必须直接等于 spec budget 公式 · 推算 / 抽样必须在 metrics 文件留 per-unit 数字 + 完整推算式
- **I4 · 每个 acceptance 项必须有具体可执行步骤 · 不能"测 1 次（手动）"含糊**
  - 违反案例：A.4 / C.3 / E.2 / E.3 标"测 1 次"但没列按键序列 → Arbiter 跑时随便点几下声称 PASS · evidence 不可复现
  - 应用：每项 acceptance 必须带 step-by-step 按键序列 + 期望结果 + 失败时怎么报告

### Round 2 invariants（深度 4 条 · Codex round 2 finding 抽象）

- **I5 · playbook 跨段引用必须一致 · 不能内部矛盾**
  - 违反案例：§7 commit checklist 列 6 PNG · §9/§11 mandatory 13 PNG · runner 按 §7 走可漏交 mandatory evidence（round 2 finding 1）
  - 应用：每段 evidence 数量 / acceptance 列表必须互相 cross-ref · 写完 grep 跨段 stale 引用（如 "6 PNG" / "6 截图"）· 加 evidence-completeness 验证脚本（§7 ls + grep BLOCK）
- **I6 · 外部脚本输出必须验证 input 正确 · 不能信任脚本默认行为**
  - 违反案例：F.1 假设 measure-memory.sh 数 4 个 pane shell · 但脚本有 fallback 数全局 zsh/bash · 推算被污染（round 2 finding 2）
  - 应用：任何依赖外部脚本输出的测量必须先验证 input · 比如 raw PID/cmd · 不正好预期数即 BLOCK · metrics 表加 raw input 列
- **I7 · evidence 必须证 spec acceptance 真实行为 · 不只"操作发生了"**
  - 违反案例：A.1/A.2 录屏只 echo · 不证 shell/cwd 继承 · A.3 不测 sole-pane close（round 2 finding 3）
  - 应用：spec 每个 acceptance 项的 evidence 必须直接 demonstrate spec 描述的 behavior · 不只是触发了快捷键 · 必须验证状态变化等价 spec 期望
- **I8 · safety-critical evidence 必须直接观察 · 不能间接推断**
  - 违反案例：§5.5.3 Solo confirm 只截 console（timing 证据）· 不证 layout 1 pane / focus 保留 / vim/nano 真死（round 2 finding 4）
  - 应用：data-loss / kill / persistence 类 acceptance 的 evidence 必须含直接观察 artifact（UI 截图 + process check）· timing/log 只能作为补充

### Round 3 invariants（深度 3 条 · Codex round 3 finding 抽象）

- **I9 · validator 必须 fail-closed · 不能匹配 template token**
  - 违反案例：§7 grep `D.4.*PASS\|D.4.*FAIL` 会匹配 template 自身 "D.4 PASS / FAIL"（template 含两个值）· 不证已填（round 3 finding 1）
  - 应用：placeholder 用独特 token（`<TBD>` / `<yes_or_no>` / `<PASS_or_FAIL>`）让 grep BLOCK · 显式判定字段用 `key 判定：(PASS|FAIL)$` 行尾固定模式 · template 文案不能误匹配 filled 形式
- **I10 · 同类 path 必须共享 evidence 严格度**
  - 违反案例：§5.5.3 Solo confirm 加了 process check · §5.5.5 AI+Runner confirm 没加（round 3 finding 2）· 同类 destructive confirm path evidence 不一致
  - 应用：枚举所有同类 path（如 destructive confirm × N · split × N · close × N）· 对每个 path 应用 invariant · 不只对 codex 指的那个 · grep 同类 keyword 找全
- **I11 · 测量来源必须自证 · 不能依赖独立后续验证**
  - 违反案例：F.1 RSS 测量（measure-memory.sh） + 独立 SHELL_COUNT 验证 + 独立 pgrep 列 PID 三步不绑定 · runner 可复制旧 RSS · 后续验证 PID 可能不同（round 3 finding 3）
  - 应用：测量步骤本身必须输出验证字段（PID + RSS + PPID 一起 · 同一组）· 不能"先测后验" · 流程改"先列源 → 直接量"

**自检**（写完任何 playbook 段后跑全部 grep · 任一异常即违反对应 invariant）：

```bash
# I2/I4 自检（含糊步骤 / 可选 mandatory）
grep -nE "测 1 次|状态可选|optional" docs/runtime-evidence/mvp-05/CAPTURE-PLAYBOOK.md
# 命中应只有 §0.5 invariants 说明引用 · 否则违反 I2/I4
```

```bash
# I5 自检（跨段 stale 数量引用）
grep -n "6 截图\|6 PNG\|6 张" docs/runtime-evidence/mvp-05/CAPTURE-PLAYBOOK.md
# 命中应只有 §2/§2.4 段内 capture flow 6 张引用 + invariants 说明 + §7/§550 finding 描述 · 顶部目标 + §11 必须用 14 PNG
```

```bash
# I9 自检（metrics template 残留 placeholder · 应只在未填的 metrics-mvp-05.md · §7 validator 自动 catch）
grep -nE "<TBD>|<yes_or_no>|<PASS_or_FAIL>" docs/runtime-evidence/mvp-05/metrics-mvp-05.md
# 在未跑实测时应有 N 处（template 状态）· 跑完实测后应 0 处（§7 BLOCK if 任一残留）
```

```bash
# I10 自检（同类 path 是否共享 evidence · 跨 §5.5.3/§5.5.5 grep）
grep -nE "vim/nano kill confirmed" docs/runtime-evidence/mvp-05/CAPTURE-PLAYBOOK.md docs/runtime-evidence/mvp-05/metrics-mvp-05.md
# 应在 §5.5.3 + §5.5.5 + §7 validator + §0.5 invariants + §11 反馈模板共出现 · 缺一即违反 I10
```

---

## 0 · 前置准备（5 min）

```bash
# 0.1 · 确认 main 最新（不在某个老 branch）
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation
git checkout main && git pull --ff-only origin main

# 0.2 · 创新 branch 收 evidence
git checkout -b docs/mvp-05-phase-d-capture

# 0.3 · 确认依赖
node --version  # ≥ 20
pnpm --version  # 9.x
cargo --version

# 0.4 · 清理旧 evidence（保留 metrics-mvp-05.md + 本 PLAYBOOK · 删 PNG/MOV）
rm -f docs/runtime-evidence/mvp-05/*.png docs/runtime-evidence/mvp-05/*.mov

# 0.5 · 准备 DevTools-friendly build（dev 模式 · 因为 F.2-F.6 需 DevTools）
pnpm install
# 不 build · dev mode 直接 watch
```

---

## 1 · 启动 dev mode + 验证 Solo 起点（5 min · I1）

> **I1 关键**：本段 fixture 起点 = **Solo（1 pane）**· 因为 §2.1 capture-phase-d.sh 假设从 Solo 起按 `⌘\` 横分。若起点是 4 panes · 第一张 01 截图就是 4 panes 状态 + 后续 ⌘\ 触发"已达单层上限"toast · 6 PNG 都生成但内容全错 · Codex finding 1。
>
> §3 内存测量 + §4 60FPS 拖拽 + §5 IPC 时延 + §5.5 destructive path 各段会**单独重建 4 panes fixture**· 不能依赖本段。

```bash
# 1.1 · 启动 dev mode（前台跑 · 输出在终端）
pnpm tauri:dev
```

等窗口出现（约 5-15s 取决于增量 build）。

**1.2 · 验证起点 = Solo（1 pane）**：

App 默认启动 = 1 tab + 1 pane Solo。Sanity check：

- 窗口里**只有 1 个 pane**（无水平 / 垂直分隔条）· 占满整个 main 区
- pane 有 prompt（PTY 起好）· `echo solo-ready` 能输出
- 若窗口有多 pane（前次 session 持久化布局）· 按 `⌘⇧P` → ↑↓ 选 **Solo** → Enter → 二次确认 Enter · 强制归零

**❌ 如果 PTY 没起 / pane 无 prompt** · 不要继续 · webview `Cmd+Option+I` 开 DevTools 看 console errors · 报告主 agent。

**❌ 如果 ⌘⇧P 不响应 / Smart Layouts 菜单不出**· 说明 §C 功能本身坏 · §2 capture 也会失败 · 报告主 agent。

---

## 2 · 6 截图 + 30s 录屏（10 min · I1）

### 2.1 · ⚠️ 自动化 · capture-phase-d.sh（已知不可靠 · 仅作参考 · 强烈推荐 §2.2 完全手动）

> **Codex finding 1（high · 2026-05-03）**：自动化路径有以下已知问题 · 跑出的 evidence 必须按 §2.4 content validation 逐张人工确认 · 否则 6 PNG 都生成但内容错 · 错 evidence 翻 spec done = 假 PASS。
>
> **已知问题清单**：
> 1. 起点不验证：脚本第一张 01 直接截当前画面 · 若不是 Solo 状态（即使 §1 验证过 · 期间任何键盘事件可能改变状态）· 01 内容错
> 2. 模拟键盘不可靠：osascript `keystroke` 在某些 macOS 版本 / accessibility 权限不全时静默失败 · 后续 split 命令 no-op
> 3. 单层上限拒绝：脚本 04 在 02 状态后 ⌘⇧\ · 若 02 已经有右 pane · 可能触发"已达单层上限"toast · 04 截到 toast 不是 2×2
> 4. 无 content validation：脚本只看 file count + size · 不检查每张图实际画面

**前置**（若仍要尝试自动化）：app 窗口在**绝对前台** + Solo 起点（§1.2 验证完）+ accessibility 权限已加（系统设置 → 隐私与安全性 → 辅助功能 → Terminal/iTerm 已勾）。

```bash
# 在另一个终端 tab（不在 pnpm tauri:dev 那个）
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation
bash scripts/capture/mvp-05/capture-phase-d.sh
```

跑完**必走 §2.4 content validation**· 任一项不通过即按 §2.2 完全手动重做。

### 2.2 · ✅ 完全手动 fallback（推荐 · 5-10 min · 100% 可控）

osascript 模拟键盘可能因 app 焦点 / 系统 accessibility 权限 / Tauri 行为偏差失败。**首选 fallback** = 完全手动：

```bash
# 取 vibestation 窗口 ID
WID=$(osascript -e 'tell application "System Events" to id of front window of (first process whose name contains "Vibestation")')
echo "Window ID: $WID"

# 然后手动操作 + 每步截图（macOS Shift+Cmd+5 → window 模式 · 或下面命令）
OUT=docs/runtime-evidence/mvp-05

# 01 Solo（先 Smart Layout → Solo · 退出菜单后截图）
# 手动：app 里⌘⇧P → ↑↓ 选 Solo → Enter → 二次确认 Enter
screencapture -x -l "$WID" "$OUT/01-solo-single-pane.png"

# 02 横分（⌘\）
# 手动：app 里⌘\
screencapture -x -l "$WID" "$OUT/02-horizontal-2-panes.png"

# 03 垂直（先 Solo · 再 ⌘⇧\）
# 手动：⌘⇧P → Solo → Enter Enter · 然后 ⌘⇧\
screencapture -x -l "$WID" "$OUT/03-vertical-2-panes.png"

# 04 2×2（在 02 状态基础上 · ⌘⇧\）
# 手动：⌘⇧P → Solo → Enter Enter · ⌘\ · ⌘⇧\ · click 左 pane · ⌘⇧\
screencapture -x -l "$WID" "$OUT/04-2x2-quad-panes.png"

# 05 Smart Layout 菜单
# 手动：⌘⇧P（菜单弹出后立即截图）
screencapture -x -l "$WID" "$OUT/05-smart-layout-menu.png"

# 06 应用 Solo 后
# 手动：05 状态下选 Solo → Enter Enter
screencapture -x -l "$WID" "$OUT/06-after-smart-apply.png"
```

**File-level 验证**：`ls -la docs/runtime-evidence/mvp-05/*.png` 应该 6 张 · 单张 50-500 KB（按 ADR-011 R4）。

**⚠️ File-level 验证不充分 · 必须走 §2.4 content validation**。

### 2.3 · 30s 录屏

**前置**：4 panes 2×2 fixture 就位 · app 在前台。

```bash
WID=$(osascript -e 'tell application "System Events" to id of front window of (first process whose name contains "Vibestation")')
screencapture -V 30 -x -l "$WID" docs/runtime-evidence/mvp-05/07-flow-recording.mov
```

**录屏 30s 内手动操作**（提前演练 1 次）：

- 0-5s：4 panes 2×2 · 每 pane 各跑一条 `echo hello-pane-N`
- 5-10s：拖动水平 splitter（左右晃 2 次）
- 10-15s：拖动垂直 splitter（上下晃 2 次）
- 15-20s：双击垂直 splitter 复位 50/50
- 20-25s：⌘⌃W 关右下 pane
- 25-30s：⌘⇧P → Smart Layouts → Solo → Enter Enter

**验证**：`ls -la docs/runtime-evidence/mvp-05/07-flow-recording.mov` · ≤ 10 MB（ADR-011 R4 上限）· 若超 10 MB · 用 `ffmpeg -i 07-flow-recording.mov -vcodec h264 -crf 28 07-flow-recording-compressed.mov && mv 07-flow-recording-compressed.mov 07-flow-recording.mov`。

### 2.4 · 6 截图 content validation（mandatory · I1）

> **Codex finding 1 直接修复**：file count + size 不能证明 content 正确。下表 6 项 mandatory · 任一不过即重截。

逐张打开（macOS Quick Look · 选中文件按空格）确认：

| 文件 | 期望 content | 失败信号 |
|---|---|---|
| `01-solo-single-pane.png` | **1 pane 占满 main 区** · 无任何分隔条 · pane 有 prompt | 多 pane / 看到分隔条 → 起点错 · 重做 §1.2 + §2.2 |
| `02-horizontal-2-panes.png` | **2 panes 左右排列** · 中间 1 条垂直分隔条 · 两 pane 都有 prompt | 1 pane / 3+ panes / 上下排列 → 重做 |
| `03-vertical-2-panes.png` | **2 panes 上下排列** · 中间 1 条水平分隔条 · 两 pane 都有 prompt | 1 pane / 3+ panes / 左右排列 → 重做 |
| `04-2x2-quad-panes.png` | **4 panes 2×2 排列** · 1 条垂直 + 1 条水平分隔条 · 4 pane 都有 prompt | < 4 panes / 出现"已达单层上限"toast → 重做 |
| `05-smart-layout-menu.png` | **Smart Layouts 命令面板可见** · 含 Solo + AI+Runner 两选项 · 选中项含 dry-run 预览（"将关闭 N 个 Pane"或类似文案）| 菜单未弹 / 无 dry-run 文案 → 重做 |
| `06-after-smart-apply.png` | **1 pane 占满**（Solo apply 后）· 与 01 视觉一致 · 但保留了 apply 前 focus pane 的 prompt history | 多 pane / 看不出 apply 已执行 → 重做 |

**逐张过完 = §2 截图段通过** · 否则不能进 §3。

> **未过 → grep "重做" 提示**：自动化路径已知不可靠（见 §2.1 4 个已知问题）· 重做时直接走 §2.2 完全手动 · 不要再试 §2.1 自动化。

---

## 3 · F.1 内存测量（5-10 min · I3）

> **Codex finding 3 直接修复**：spec §F.1 真实 budget 是 **40 PTY (10 tab × 4 pane) < 500MB** · 不是"4 pane < 500MB"。本段重新分解 · 既记 per-pane 也记 40 PTY 推算 total · pass 条件 = 推算 total < 500MB。
>
> **为什么不真测 40 PTY**：用户机器创 10 tab × 4 pane fixture 工作量过大（40 个 ⌘N + ⌘\ 操作 + 40 PTY 同时跑可能耗光本机 RAM 阻碍调试）· 按 Codex 第二种建议（per-pane 推算）· 已通过 spec §F.1 acceptance（spec 自己就说 "推算"）。

### 3.1 · 4 pane fixture（重建 · 每段独立）

**前置**：app 跑着 · §1.2 验证 Solo 起点。手动重建 4 panes 2×2：

| 步骤 | 操作 |
|---|---|
| 1 | 先 ⌘⇧P → Solo → Enter Enter（强制归零）|
| 2 | ⌘\ → 2 panes 水平 |
| 3 | click 右 pane focus |
| 4 | ⌘⇧\ → 3 panes（左 1 + 右 2）|
| 5 | click 左 pane focus |
| 6 | ⌘⇧\ → 4 panes 2×2 |

每个 pane 跑 `echo pane-N`（让 PTY 真有进程 · 不是 idle shell）。

### 3.2 · 测 4 pane RSS 3 次（mandatory · I6 + I11 自证流程 · Codex round 3 finding 3 fix）

> **Codex round 3 finding 3 修复**：原本"先跑 measure-memory.sh × 3 取 RSS · 再独立 SHELL_COUNT 验证 · 再独立 pgrep 列 PID"三步不绑定 · runner 可复制第一步 RSS · 第二步通过 · 第三步显示 4 PID 但和测 RSS 的 PID 不同。本段改"先列 PID + cmd · 再 ps -o 取 RSS · 同一组"自证流程 · 不再用 measure-memory.sh（避免 global fallback 风险）。

**自证流程**（每 run · 同一组 PID 测 RSS · I11）：

```bash
# 在另一个终端 tab
for i in 1 2 3; do
  echo "=== Run $i ==="

  # Step 1 · 取 main app PID + RSS
  MAIN_PID=$(pgrep -f vibestation-app | head -1)
  if [[ -z "$MAIN_PID" ]]; then
    echo "❌ BLOCK · 找不到 vibestation-app 进程 · app 可能没启动"
    exit 1
  fi
  MAIN_RSS_KB=$(ps -o rss= -p $MAIN_PID | tr -d ' ')
  MAIN_RSS_MB=$((MAIN_RSS_KB / 1024))
  echo "MAIN_PID=$MAIN_PID · RSS=${MAIN_RSS_MB} MB"

  # Step 2 · 列 main app spawn 的 child shell（必须正好 4 个 · 否则 BLOCK · I6）
  SHELL_PIDS=$(pgrep -P $MAIN_PID -f "zsh|bash|sh" | tr '\n' ' ')
  SHELL_COUNT=$(echo $SHELL_PIDS | wc -w | tr -d ' ')
  echo "Pane shell PIDs (main app child): $SHELL_PIDS"
  echo "Shell count: $SHELL_COUNT（期望 4）"
  if [[ $SHELL_COUNT -ne 4 ]]; then
    echo "❌ BLOCK · main app spawn child 不是 4 个 · 可能某 pane PTY 没起 / 某 shell 死 / app 不通过 fork"
    echo "  MAIN_PID 详情：$(ps -o pid,ppid,command -p $MAIN_PID)"
    echo "  child 详情："
    for pid in $SHELL_PIDS; do
      echo "    PID $pid: $(ps -o pid=,ppid=,command= -p $pid)"
    done
    exit 1
  fi

  # Step 3 · 用同一组 PID 取 RSS（I11 自证 · 不另起 measure-memory.sh）
  TOTAL_SHELL_RSS_KB=0
  PID_RSS_PAIRS=""
  for pid in $SHELL_PIDS; do
    RSS_KB=$(ps -o rss= -p $pid | tr -d ' ')
    if [[ -z "$RSS_KB" || "$RSS_KB" -eq 0 ]]; then
      echo "❌ BLOCK · PID $pid RSS 读不到（进程可能死了）"
      exit 1
    fi
    RSS_MB=$((RSS_KB / 1024))
    TOTAL_SHELL_RSS_KB=$((TOTAL_SHELL_RSS_KB + RSS_KB))
    PID_RSS_PAIRS="$PID_RSS_PAIRS $pid:${RSS_MB}MB"
  done
  TOTAL_SHELL_RSS_MB=$((TOTAL_SHELL_RSS_KB / 1024))
  PER_PANE_AVG_MB=$((TOTAL_SHELL_RSS_MB / 4))

  # Step 4 · 推算 40 PTY total（与 spec budget 直接对照 · I3）
  EXTRAPOLATED_40_PTY=$((MAIN_RSS_MB + PER_PANE_AVG_MB * 40))
  echo "Per-pane avg: ${PER_PANE_AVG_MB} MB"
  echo "Extrapolated 40 PTY: MAIN ${MAIN_RSS_MB} + PER_PANE ${PER_PANE_AVG_MB} × 40 = ${EXTRAPOLATED_40_PTY} MB"
  echo "vs spec budget 500 MB: $([ $EXTRAPOLATED_40_PTY -lt 500 ] && echo "PASS" || echo "FAIL")"
  echo ""
  echo "→ 记 metrics §F.1 表 Run $i 行：MAIN_PID=$MAIN_PID · RSS=${MAIN_RSS_MB}MB · 4 shell:$PID_RSS_PAIRS · PER_PANE_AVG=${PER_PANE_AVG_MB}MB · 推算=${EXTRAPOLATED_40_PTY}MB"
  echo ""
  sleep 2
done
```

**I11 自证特性**：
- Step 2 和 Step 3 用**同一组 SHELL_PIDS** · 不存在"测 RSS 用 PID-A · 验证用 PID-B"漏洞
- Step 2 严格验证 main app child（不走全局 fallback · 不依赖 measure-memory.sh 默认行为）
- Step 4 直接输出 PASS/FAIL · 与 spec budget 等价

**注意**：原本依赖 `scripts/capture/mvp-05/measure-memory.sh` · round 3 fix 后**不再用**（脚本有 global zsh/bash fallback 风险 · 见 I6 历史案例）· 但 script 留 repo 作 reference。

### 3.3 · 推算 40 PTY total + 写 metrics

**公式**（在 metrics-mvp-05.md F.1 段）：

```
单次推算 = MAIN_RSS_MB + PER_PANE_AVG × 40

P99 = max(R1, R2, R3) 的推算值
```

**填表**（metrics-mvp-05.md F.1 替换原"实测"段为下表）：

```
| Run | MAIN_RSS_MB | 4 pane 总 shell RSS | PER_PANE_AVG | 推算 40 PTY = MAIN + PER_PANE × 40 |
|---|---|---|---|---|
| 1 | ___ | ___ | ___ | ___ MB |
| 2 | ___ | ___ | ___ | ___ MB |
| 3 | ___ | ___ | ___ | ___ MB |
| **P99** | — | — | — | **___ MB** |
```

**通过条件**（spec §F.1 真实 budget · I3）：

- ✅ PASS：P99 推算 40 PTY total **< 500 MB**
- ❌ FAIL：P99 推算 ≥ 500 MB · 不要简单标 PASS · 报告主 agent + spec §⚠️ 已知风险加技术债 + 评估是否真测 10 tab × 4 pane fixture 复核

**注意**：实测 4 pane 单 run < 500MB **不等于** PASS（这是 Codex finding 3 抓的 bug）· 必须按推算公式判断。

---

## 4 · F.2/F.3 60FPS 拖拽（10 min · DevTools）

**前置**：4 panes 2×2 · app 在 dev mode（dev 模式才能开 DevTools）。

### 4.1 · 开 DevTools

webview 内 `Cmd + Option + I` · 切到 **Performance** tab。

### 4.2 · F.2 横拖（3 次取 P99）

每次：

1. DevTools Performance · 点红圆 ● Record
2. 在 app 里**左右拖动水平 splitter** 1 秒（左右各晃 1 次）
3. 点 ⏹ Stop
4. 看 **Frames** 行 · 找 1s 内最长帧时长（hover 看 ms 数）
5. 记录 → 重复 3 次 · 取最大值

**记录**：

```
F.2 Run 1 P99 帧时长: ___ ms
F.2 Run 2: ___ ms
F.2 Run 3: ___ ms
F.2 P99 ≈ ___ ms（< 16ms PASS）
```

### 4.3 · F.3 竖拖（3 次取 P99）

同 4.2 · 但拖**垂直 splitter** 上下 1 秒。

```
F.3 Run 1: ___ ms
F.3 Run 2: ___ ms
F.3 Run 3: ___ ms
F.3 P99 ≈ ___ ms
```

**通过条件**：F.2/F.3 P99 < 16ms · 即 60FPS。

---

## 5 · F.4/F.5/F.6 IPC 时延（5-10 min · DevTools console）

**前置**：仍在 dev mode · DevTools console 已开。

### 5.1 · F.4 ⌘\ → 新 pane DOM commit

每次按 `⌘\` 后 · console 自动输出：

```
[mvp-05][F.4] pane_split horizontal → DOM commit: XX.XXms
```

**测 3 次** · 在 app 里：

1. 先 ⌘⇧P → Solo → Enter Enter（回到单 pane）
2. ⌘\ · 看 console
3. ⌘⇧P → Solo → Enter Enter
4. ⌘\ · 看 console
5. 重复

**记录**：

```
F.4 Run 1: ___ ms
F.4 Run 2: ___ ms
F.4 Run 3: ___ ms
F.4 P99 ≈ ___ ms（< 150ms PASS）
```

### 5.2 · F.5 ⌘⌃W → 重排 DOM commit

**前置**：先创 4 panes 2×2 · 然后逐个 ⌘⌃W 关。

每次 ⌘⌃W 后看 console：

```
[mvp-05][F.5] pane_close → DOM commit: XX.XXms
```

**记录**：

```
F.5 Run 1: ___ ms
F.5 Run 2: ___ ms
F.5 Run 3: ___ ms
F.5 P99 ≈ ___ ms（< 100ms PASS）
```

### 5.3 · F.6 Smart Layouts apply

**前置**：4 panes 2×2 · ⌘⇧P → Solo → Enter Enter · 看 console：

```
[mvp-05][F.6] layout_apply solo → DOM commit: XX.XXms
```

**测 3 次** · 每次都先重建 4 panes 2×2 fixture · 再 apply Solo。

**记录**：

```
F.6 Run 1: ___ ms
F.6 Run 2: ___ ms
F.6 Run 3: ___ ms
F.6 P99 ≈ ___ ms（< 200ms PASS）
```

---

## 5.5 · 数据丢失确认路径测试（mandatory · I2 · 5-8 min）

> **Codex finding 2 直接修复**：spec §C.2 + C.3 要求 Smart Layouts 关闭含 vim/nano 编辑状态的 pane 前必须二次确认 · default cancel · 防用户数据丢失。这是用户数据安全 path · **mandatory · 不能标"可选"**。
>
> **本段 4 路径必须全测 + 留 evidence**（截图或 console log）· 任一不过即 BLOCK spec done。

### 5.5.1 · 准备 fixture（2 min）

重建 4 panes 2×2（步骤同 §3.1）· 此时：

| Pane | 跑 |
|---|---|
| 左上 | `vim /tmp/mvp05-test.txt` · 进入 insert 模式 · 输入 "data-A" · **不保存** |
| 左下 | `nano /tmp/mvp05-test-nano.txt` · 输入 "data-B" · **不保存** |
| 右上 | `echo idle-pane-c`（普通 idle shell · 无 editing state） |
| 右下 | `echo idle-pane-d`（同上 · 这个 pane 是 focused · click 它）|

**Sanity check**：左上 pane 显示 vim 状态栏（`-- INSERT --`）· 左下显示 nano 底部命令栏（`^G Get Help` 等）· 右下边框高亮（focus）。

**记录 vim/nano PID（mandatory · §5.5.3 + §5.5.5 process check 用 · I8 + I10）**：

```bash
# 在另一终端跑（不在 fixture 那 4 个 pane 内）
VIM_PID=$(pgrep -f "vim /tmp/mvp05-test.txt" | head -1)
NANO_PID=$(pgrep -f "nano /tmp/mvp05-test-nano.txt" | head -1)
echo "VIM_PID=$VIM_PID NANO_PID=$NANO_PID"
# 写到 metrics §5.5.3 + §5.5.5 段（每段都要 · 5.5.5 重建 fixture 时 PID 不同）
```

### 5.5.2 · 路径 1：Solo cancel（1 min）

| 步骤 | 操作 | 期望 |
|---|---|---|
| 1 | ⌘⇧P → ↑↓ 选 **Solo** | dry-run 预览菜单 · 显示"将关闭 3 个 Pane"或类似文案 + 列出 vim/nano editing state 警告 |
| 2 | 看 Confirm 按钮 | 默认按钮 = **取消**（红色 Confirm 不应是 default · 按 Enter 不应直接执行）|
| 3 | 按 Esc 或选 Cancel | 菜单关闭 · **4 panes 2×2 完整保留**· 左上 vim 仍 INSERT · 左下 nano 仍在 |

**❌ FAIL 信号**：
- 默认按 Enter 直接关 panes（vim 状态丢失）→ spec §C.2 acceptance 不达标
- 没显示 dry-run 预览或没列编辑状态警告 → spec §C.2 acceptance 不达标
- Cancel 后 pane 数量不是 4 → 状态泄漏

**截图证据**：dry-run 预览菜单截图 → 存 `docs/runtime-evidence/mvp-05/08-solo-cancel-preview.png`

### 5.5.3 · 路径 2：Solo confirm（2 min · I8 直接观察 evidence）

> **Codex round 2 finding 4（medium）直接修复**：原本只要 console 截图（09）显示 F.6 timing · 但 console log 不证明 layout 真变 1 pane / focus pane 真保留 / vim/nano process 真被 kill。本路径重写：要 console 截图（09 timing 证据）+ UI 截图（14 layout 直接观察）+ process check（命令行直接观察 vim/nano 已死）。

| 步骤 | 操作 | 期望 |
|---|---|---|
| 1 | ⌘⇧P → Solo | dry-run 预览（同 5.5.2）|
| 2 | 显式选 **Confirm**（点击或键盘选中 + Enter）| 关 3 panes · **保留 focus pane = 右下** |
| 3 | sanity check UI | 1 pane 占满 · 内容是右下 pane 的 prompt history（`echo idle-pane-d` 输出）· vim/nano UI 已消失 |
| 4 | terminal 跑 process check | `ps aux \| grep -E "vim /tmp/mvp05|nano /tmp/mvp05" \| grep -v grep` → **应无输出**（process 真被 kill）|

**❌ FAIL 信号**：
- 保留的不是原 focus pane（右下）→ Smart Layouts 不尊重 focus
- 多 pane 残留 → close transaction 没 atomic（违反 spec §H.3）
- ps 还能看到 vim/nano process → PTY kill 没传到子进程 · spec §H.3 不达标

**Evidence（mandatory · I8 · 3 样齐）**：

1. **console 截图 09**（F.6 timing 证据）：confirm 后 DevTools console 应有 `[mvp-05][F.6] layout_apply solo → DOM commit: XX.XXms` log（同 §5.3）→ 存 `09-solo-confirm-console.png`
2. **UI 截图 14**（layout 状态直接观察 · I8）：confirm 完成后窗口截图 · 显示 1 pane 占满 + prompt history 含 `echo idle-pane-d` 输出 → 存 `14-solo-confirm-ui-result.png`
3. **process check 文本**（vim/nano kill 直接观察 · I8）：步骤 4 的 ps 输出（应空）+ exit code 1 → metrics-mvp-05.md §5.5.3 段记 "ps 输出: (空) · vim/nano kill confirmed"

### 5.5.4 · 路径 3：AI+Runner cancel（1 min · 重建 fixture）

重新跑 §5.5.1 fixture · 然后：

| 步骤 | 操作 | 期望 |
|---|---|---|
| 1 | ⌘⇧P → 选 **AI+Runner** | dry-run 预览 · 显示"2×2 → AI+Runner 需先降级 · 将关闭 N 个 pane"+ 列编辑状态警告 |
| 2 | Esc 取消 | 菜单关 · **4 panes 完整保留** |

**❌ FAIL 信号**：
- 没显示降级提示（spec §C.3 要求）→ acceptance 不达标
- Cancel 后状态变化 → 状态泄漏

**截图证据**：`docs/runtime-evidence/mvp-05/10-ai-runner-cancel-preview.png`

### 5.5.5 · 路径 4：AI+Runner confirm（2 min · 重建 fixture · I10 同类 path 共享 evidence · Codex round 3 finding 2 fix）

> **Codex round 3 finding 2 直接修复**：原本只截 layout 11 · 与 §5.5.3 同类 destructive confirm 不一致 · screenshot 不证 vim/nano kill。本路径加 PID 记录 + post-confirm process check（同 §5.5.3 I8 模式）。

**前置**：重新跑 §5.5.1 fixture（含 vim + nano + idle pane c/d）· **重新记 VIM_PID + NANO_PID**（与 §5.5.3 不同进程 · PID 必不同）· 写 metrics §5.5.5 段。

| 步骤 | 操作 | 期望 |
|---|---|---|
| 1 | ⌘⇧P → AI+Runner → Confirm | 先降级（关 2 panes 保留 focus + 1 邻居）· 再强制右分屏 50/50 |
| 2 | sanity check UI | 最终是水平 2 panes 50/50 · 左 pane 是原 focus（右下 idle-pane-d · 含 prompt history）· 右 pane 是新 spawn shell（无 prompt history）|
| 3 | terminal 跑 process check | `ps -p $VIM_PID -p $NANO_PID -o pid=,command= 2>/dev/null` → **应空**（exit code 1 · vim/nano process 真被 kill）|

**❌ FAIL 信号**：
- 不是水平 2 panes / 比例不是 50/50 → §C.3 strict 50/50 acceptance 不达标
- 保留的左 pane 不是原 focus → focus tracking 错
- ps 还能看到 vim/nano PID → PTY kill 没传到 child（spec §H.3 不达标 · v0.2 vim/nano 数据丢失风险）

**Evidence（mandatory · I8 + I10 · 2 样齐）**：

1. **UI 截图 11**（layout 直接观察）：confirm 后窗口截图 · 显示水平 2 panes 50/50 + 左 pane 含 prompt history → 存 `11-ai-runner-confirm-result.png`
2. **process check 文本**（vim/nano kill 直接观察 · 同 §5.5.3 模式）：步骤 3 ps 输出（应空）→ metrics §5.5.5 段记 "vim/nano kill confirmed"

### 5.5.6 · 路径自检（mandatory · I5 跨段一致）

跑完 5.5.2-5.5.5 共 4 路径 · **5 张证据图（08/09/10/11/14）+ process check 文本** 必须存在：

```bash
ls -la docs/runtime-evidence/mvp-05/{08,09,10,11,14}-*.png
# 5 张 PNG 都应在 · 缺一即 BLOCK
grep -q "vim/nano kill confirmed" docs/runtime-evidence/mvp-05/metrics-mvp-05.md \
  || echo "❌ BLOCK · §5.5.3 process check 文本未记录"
```

任一缺 / 任一 FAIL 信号触发 → BLOCK §6 metrics 填表 · 报告主 agent。

---

## 6 · 填 metrics-mvp-05.md（5 min）

打开 `docs/runtime-evidence/mvp-05/metrics-mvp-05.md` · 把上面所有"实测"段的空白填上。

实测后状态字段：

- `🟡 工具就位 · 待跑` → `✅ 实测 PASS`（如 F.x P99 < 目标）
- `🟡 工具就位 · 待跑` → `❌ 实测 FAIL`（如 F.x P99 ≥ 目标 · 触发 fallback / 进 spec §⚠️ 已知风险）

---

## 7 · 关闭 app + evidence-completeness 验证 + commit + push（5-10 min · I5）

> **Codex round 2 finding 1（high · 2026-05-03）直接修复**：原 §7 只列 6 PNG + 1 MOV · 与 §9/§11 mandatory 的 13 PNG + 1 MOV + filled metrics + D.4/E.2/E.3 文本结果不一致 · runner 按 §7 commit 可绕过 §5.5/A.4/D.4/E 全部 mandatory acceptance evidence。本段重写 · evidence-completeness 验证不齐则 BLOCK commit。

```bash
# 7.1 · 关 app（pnpm tauri:dev 终端 Ctrl+C · 等所有 webview 关）

# 7.2 · evidence-completeness 验证（mandatory · I5 · 不齐即 BLOCK commit）
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation
OUT=docs/runtime-evidence/mvp-05
EXPECTED_PNGS=(01-solo-single-pane 02-horizontal-2-panes 03-vertical-2-panes 04-2x2-quad-panes 05-smart-layout-menu 06-after-smart-apply 08-solo-cancel-preview 09-solo-confirm-console 10-ai-runner-cancel-preview 11-ai-runner-confirm-result 12-single-level-toast 13-focus-bottom-left 14-solo-confirm-ui-result 15-sole-pane-close-tab)
MISSING=0
for name in "${EXPECTED_PNGS[@]}"; do
  if [[ ! -f "$OUT/$name.png" ]]; then
    echo "❌ MISSING: $OUT/$name.png"
    MISSING=$((MISSING+1))
  fi
done
[[ ! -f "$OUT/07-flow-recording.mov" ]] && echo "❌ MISSING: $OUT/07-flow-recording.mov" && MISSING=$((MISSING+1))
if [[ $MISSING -gt 0 ]]; then
  echo "❌ BLOCK · $MISSING evidence file(s) missing · 不能 commit · 回头补"
  exit 1
fi
echo "✓ 14 PNG + 1 MOV 全在"

# 7.3 · I9 placeholder 残留检查（fail-closed · 不能匹配 template token · Codex round 3 finding 1 修复）
PLACEHOLDER_HITS=$(grep -cE "<TBD>|<yes_or_no>|<PASS_or_FAIL>" "$OUT/metrics-mvp-05.md" || true)
if [[ $PLACEHOLDER_HITS -gt 0 ]]; then
  echo "❌ BLOCK · metrics-mvp-05.md 仍有 $PLACEHOLDER_HITS 处 placeholder（<TBD> / <yes_or_no> / <PASS_or_FAIL>）· 必须全填实测值"
  grep -nE "<TBD>|<yes_or_no>|<PASS_or_FAIL>" "$OUT/metrics-mvp-05.md" | head -10
  exit 1
fi
EMPTY_NUMERIC=$(grep -cE "___ MB|___ ms|___%" "$OUT/metrics-mvp-05.md" || true)
if [[ $EMPTY_NUMERIC -gt 0 ]]; then
  echo "❌ BLOCK · metrics-mvp-05.md 仍有 $EMPTY_NUMERIC 处数字未填（___ MB / ___ ms / ___%）"
  exit 1
fi

# 7.4 · 显式判定字段强模式 grep（I9 · 行尾固定 · 不能匹配 "<PASS_or_FAIL>" template）
# 必须严格匹配 "<key> 判定：PASS" 或 "<key> 判定：FAIL" 单一值（不是 "PASS / FAIL" 模板）
require_judgment() {
  local key="$1"
  if ! grep -qE "^${key} 判定：(PASS|FAIL)$" "$OUT/metrics-mvp-05.md"; then
    echo "❌ BLOCK · metrics 缺 ${key} 判定 · 必须显式 '${key} 判定：PASS' 或 'FAIL' 单值（行尾固定 · 不是模板）"
    exit 1
  fi
}
require_judgment "A.1"
require_judgment "A.2"
require_judgment "A.3.1"
require_judgment "A.3.2"
require_judgment "§5.5.3"
require_judgment "§5.5.5"
require_judgment "D.4"
require_judgment "E.2"
require_judgment "E.3"

# 7.4.1 · §5.5.3 + §5.5.5 process check 显式确认（I8 + I10 · 同类 path 共享）
grep -qE "^§5\.5\.3 vim/nano kill confirmed: (yes|no)$" "$OUT/metrics-mvp-05.md" || { echo "❌ BLOCK · §5.5.3 vim/nano kill 字段缺"; exit 1; }
grep -qE "^§5\.5\.5 vim/nano kill confirmed: (yes|no)$" "$OUT/metrics-mvp-05.md" || { echo "❌ BLOCK · §5.5.5 vim/nano kill 字段缺（I10 同类 path · Codex round 3 finding 2）"; exit 1; }

# 7.4.2 · E.3 增长率显式数字（不是 <TBD>%）
grep -qE "增长率（[^）]+）: [0-9]+(\.[0-9]+)?%$" "$OUT/metrics-mvp-05.md" || { echo "❌ BLOCK · E.3 增长率必须显式数字%"; exit 1; }

echo "✓ metrics 字段全填 + 9 项判定 PASS/FAIL 显式 + §5.5.3/5.5.5 process check + E.3 增长率"

# 7.5 · git status 确认
git status
# 应显示：
# - 新增 14 PNG + 1 MOV
# - 修改 metrics-mvp-05.md（含 F.1 PID 列 + A.1/A.2 inheritance 表 + D.4/E.2/E.3 结果）

# 7.6 · stage + commit
git add docs/runtime-evidence/mvp-05/
git commit -m "docs(MVP-05): Phase D runtime evidence · 14 截图 + 30s 录屏 + §F 实测数字 + acceptance 全勾

按 docs/runtime-evidence/mvp-05/CAPTURE-PLAYBOOK.md 跑完
- §F.1 推算 40 PTY: ___ MB · PASS/FAIL
- §F.2-F.6 性能数字全填
- §5.5 destructive path 4 路径全测（08-11 + 14 截图）
- §9 acceptance 21 项全勾（A/B/C/D/E/F + B/H 已 CLI 验证）
- A.1/A.2 inheritance 文本验证 + A.3 sole-pane close（截图 15）

Co-authored-by: Claude Code <noreply@anthropic.com>"

# 7.7 · push
git push -u origin docs/mvp-05-phase-d-capture
```

---

## 8 · 通知主 agent 翻 spec done

跑完后告诉主 agent："MVP-05 Phase D capture done · 见 docs/mvp-05-phase-d-capture 分支"。主 agent 会：

1. 开 PR review evidence
2. 改 `docs/tasks/MVP-05-pane-split-single-level.md`：
   - frontmatter `status: ready` → `done`
   - acceptance 21 项逐项 `[ ]` → `[x]`（按本 playbook 实测结果）
3. merge

---

## 9 · Acceptance 21 项自检清单

跑完后 · 逐项确认（spec §Acceptance）：

### A. 分屏操作（4 项 · I7 真证 spec 行为 · 录屏 + 文本表 + 截图 15）

> **Codex round 2 finding 3（medium · I7）直接修复**：原本 A.1/A.2 标"录屏 0-5s 验证"+ A.3 标"录屏 20-25s 验证"· 但录屏只跑 echo · 不证"新 pane 继承父 shell + cwd"（spec §A.1/A.2 acceptance）· 不测"仅剩 1 时关 tab"（spec §A.3 acceptance）。本段重写 4 项 · 全 mandatory + 具体步骤 + 直接 evidence。

- [ ] A.1 ⌘\ 右分屏 + 继承父 shell + cwd（mandatory · I7 · 2 min）：
  - 步骤 1：⌘⇧P → Solo（归零 1 pane）
  - 步骤 2：在 Solo pane 跑 `cd /tmp && echo "PARENT pwd=$(pwd) shell=$SHELL"` → 记输出 e.g. `PARENT pwd=/tmp shell=/bin/zsh`
  - 步骤 3：⌘\ 右分屏 · 新 pane 自动 spawn shell + 显示 prompt
  - 步骤 4：在新（右）pane 跑 `echo "CHILD pwd=$(pwd) shell=$SHELL"` → 记输出
  - 期望：CHILD pwd == PARENT pwd（`/tmp`）+ CHILD shell == PARENT shell（`/bin/zsh` 或对应）
  - 失败信号：CHILD pwd 是 `$HOME` 不是 `/tmp` → 没继承 cwd · 违反 spec §A.1
  - Evidence：metrics-mvp-05.md §A.1 段记 PARENT/CHILD 输出对照 + PASS/FAIL
- [ ] A.2 ⌘⇧\ 下分屏 + 继承（mandatory · I7 · 2 min）：
  - 同 A.1 步骤 · 但步骤 3 用 `⌘⇧\` 下分屏
  - 同 A.1 期望 + 失败信号
  - Evidence：metrics-mvp-05.md §A.2 段记
- [ ] A.3 ⌘⌃W 关 pane + sole-pane → tab close（mandatory · I7 · 3 min · 拆 2 子测）：
  - **A.3.1 多 pane 关单 pane**：4 panes 2×2 fixture · click 右下 pane focus · ⌘⌃W → 期望 3 panes 剩 + tab 仍存
  - **A.3.2 sole-pane close → tab close**：⌘⇧P → Solo（归零 1 pane）· tab bar 当前应只有 1 个 tab · ⌘⌃W → 期望 tab 关闭（tab bar 空 / 自动新建空 tab / app 行为按 spec 决定 · 见 spec §A.3）
  - 失败信号：A.3.2 时 tab 不关 → spec §A.3 "仅剩 1 时关 tab" 不达标
  - Evidence：A.3.2 触发前后窗口截图（before：1 pane + 1 tab · after：tab 关闭后状态）→ 存 `15-sole-pane-close-tab.png`（含 before/after 标注 · 或 2 张 15a/15b）
- [ ] A.4 单层上限 toast（mandatory · I4 步骤化 · 1 min）：
  - 步骤 1：⌘⇧P → Solo → Enter Enter（归零）
  - 步骤 2：⌘\ → 2 panes 水平
  - 步骤 3：focus 在右 pane（默认 split 后 focus 跳新 pane）· 再次 ⌘\
  - 期望：toast 显示 **"Pane 已达单层上限"** + 文案 **"v0.2 将支持任意嵌套"**（spec §A.4 文案 verbatim · 持续 ≥ 3s）
  - 截图证据：toast 显示瞬间截图 → `docs/runtime-evidence/mvp-05/12-single-level-toast.png`

### B. 单层嵌套规则（4 项 · 单元测试覆盖 · CLI 已验）

- [x] B.1 4 合法布局 · 单元测试 `panes::tests::*` PASS
- [x] B.2 2 非法布局 · `h2_invalid_3horizontal_layout_is_rejected_*` PASS
- [x] B.3 6 用例覆盖 · 单元测试通过 382 全 PASS
- [x] B.4 非法操作回滚 · `split_atomicity_*` 6 case 全 PASS

### C. Smart Layouts（4 项 · mandatory · I2 + I4 · §5.5 destructive path 已覆盖 4 路径）

- [ ] C.1 命令面板提供 Solo + AI+Runner（截图 05 验证 · §2.4 content validation 已确认菜单可见 + 含两选项）
- [ ] C.2 Solo 二次确认 mandatory（**§5.5.2 + §5.5.3 已测 cancel + confirm 两路径** · vim/nano editing state warning 必须显示 · 截图 08 + 09 已存）
- [ ] C.3 AI+Runner 50/50 + 2×2 降级 mandatory（**§5.5.4 + §5.5.5 已测 cancel + confirm 两路径** · 截图 10 + 11 已存）
- [ ] C.4 dry-run 预览 mandatory（**§5.5.2 + §5.5.4 截图 08 + 10 已含 dry-run 文案 "将关闭 N 个 Pane"+ 编辑状态警告**）

### D. 分隔条（5 项 · 录屏 5-15s 覆盖 + D.4 显式重启测）

- [ ] D.1 拖拽水平 splitter · **录屏 5-10s 验证**
- [ ] D.2 拖拽垂直 splitter · **录屏 10-15s 验证**
- [ ] D.3 双击复位 50/50 · **录屏 15-20s 验证**
- [ ] D.4 比例持久化（mandatory · I4 步骤化 · 2 min）：
  - 步骤 1：4 panes 2×2 fixture · 拖水平 splitter 到 30%/70%（非默认）
  - 步骤 2：拖垂直 splitter 到 70%/30%（非默认）
  - 步骤 3：完全关 app（pnpm tauri:dev Ctrl+C · 等所有 webview 关）
  - 步骤 4：重新 `pnpm tauri:dev` · 等窗口起来
  - 期望：4 panes 2×2 布局保留 + ratio 仍是 30/70 + 70/30（不是默认 50/50）
  - 失败信号：布局丢失 / ratio 复位 → DB 持久化错 · BLOCK
- [ ] D.5 60FPS · **F.2/F.3 实测 P99 < 16ms PASS**（§4 已测）

### E. Focus（3 项 · mandatory · I4 步骤化 · 截图 04 + 显式手动）

- [ ] E.1 click pane focus 高亮（mandatory · 1 min）：
  - 步骤 1：4 panes 2×2 fixture
  - 步骤 2：依次 click 4 个 pane · 每次看边框
  - 期望：每次 click 后 · **仅被 click pane 边框高亮（主色 1px solid）**· 其他 3 pane 边框非高亮
  - 失败信号：多 pane 同时高亮 / 边框颜色不变 → focus tracking 坏
  - 截图证据：4 panes 中 click 左下 pane 后截图 → `docs/runtime-evidence/mvp-05/13-focus-bottom-left.png`
- [ ] E.2 仅 focus pane 收 keydown（mandatory · 1 min）：
  - 步骤 1：4 panes 2×2 fixture · 4 pane 都跑 `clear` · 屏幕都干净
  - 步骤 2：click 左上 pane（focus）· 输入 `echo from-top-left`
  - 步骤 3：sanity check 4 pane 内容
  - 期望：左上 pane 显示 `echo from-top-left` 输出 · **其他 3 pane 屏幕完全干净**（无字符泄漏）
  - 失败信号：其他 pane 出现 e/c/h/o 等字符 → keydown 未隔离
- [ ] E.3 yes 持续输出 · 切 focus 不打断（mandatory · 2 min）：
  - 步骤 1：4 panes 2×2 fixture
  - 步骤 2：click 左上 pane focus · 跑 `yes "alive-A" > /tmp/mvp05-yes-A.log` 后台输出
  - 步骤 3：观察 5s · 看 /tmp/mvp05-yes-A.log 行数（`wc -l /tmp/mvp05-yes-A.log` 在另一终端）
  - 步骤 4：click 右下 pane focus · 等 5s
  - 步骤 5：再 `wc -l /tmp/mvp05-yes-A.log` · 行数应**远大于** step 3 的数（增长率不应明显下降 · ≥ 80% 之前 rate）
  - 步骤 6：在 yes 那个 pane（左上）click + Ctrl+C 停掉 yes
  - 期望：focus 切走后 · 左上 pane 的 yes 进程持续输出 · 行数线性增长
  - 失败信号：yes log 行数停止增长 / 增长率 < 50% → PTY 被 focus 切换暂停
  - console 证据：步骤 3 + 5 的 wc -l 输出 → 文本贴 metrics-mvp-05.md E.3 段 + 计算增长率

### F. 性能（6 项 · §3-§5 实测覆盖 · F.1 用 40 PTY 推算公式 · I3）

- [ ] F.1 推算 40 PTY total < 500MB · **§3.3 推算公式 P99**（不是简单 4 pane RSS）
- [ ] F.2 水平 60FPS · §4.2 实测 P99 < 16ms
- [ ] F.3 垂直 60FPS · §4.3 实测 P99 < 16ms
- [ ] F.4 ⌘\ < 150ms · §5.1 实测 P99
- [ ] F.5 ⌘⌃W < 100ms · §5.2 实测 P99
- [ ] F.6 Smart Layouts < 200ms · §5.3 实测 P99

---

## 10 · Fallback / 已知坑

| 坑 | 症状 | 解 |
|---|---|---|
| osascript "not allowed assistive access" | capture-phase-d.sh 第一行报错 | 系统设置 → 隐私与安全性 → 辅助功能 · 加 Terminal / iTerm |
| capture-phase-d.sh 模拟键盘没反应 | 截图都是同一画面 | app 窗口没 focus · 或 Tauri 不响应 OS-level 模拟键盘 · 走 §2.2 完全手动 fallback · §2.4 content validation 必跑 |
| §2.4 content validation 任一项 FAIL | 截图 content 错（如 01 不是 Solo / 04 不是 2×2 / 05 没菜单）| 重做 §1.2 起点 + §2.2 完全手动 · 不要再试 §2.1 自动化 |
| F.1 推算 40 PTY ≥ 500MB | spec §F.1 budget 超 | **不要标 PASS** · 报告主 agent · 评估真测 10 tab × 4 pane fixture / 改 PTY 架构 / 改 budget |
| F.1 measure-memory.sh `pgrep` 找不到 vibestation-app | "未找到进程"报错 | release build 进程名可能不同 · 试 `ps aux \| grep -i vibe` 找 PID · 改脚本 PROCESS_NAME 参数 |
| F.2/F.3 拖拽帧时长 > 16ms | 实测 FAIL | 先确认 dev mode（dev 比 release 慢）· release build 实测一次 · 仍 FAIL 报告主 agent · spec §⚠️ 已知风险加技术债 |
| F.4/F.5/F.6 console 没输出 | 仪表化 log 不见 | webview console 切 "All levels" · 含 info · 看 PR #151 仪表化代码（Terminal.tsx） |
| §5.5 destructive path · default 是 Confirm 不是 Cancel | 按 Enter 直接关 panes（vim 数据丢） | spec §C.2 acceptance FAIL · BLOCK · 报告主 agent fix UI 默认按钮 |
| §5.5 destructive path · 没显示 vim/nano editing state warning | dry-run 预览缺数据丢失警告 | spec §C.2 acceptance FAIL · BLOCK · 报告主 agent fix UI 警告渲染 |
| §5.5 AI+Runner 不强制 50/50 | 比例不是 50/50 | spec §C.3 acceptance FAIL · BLOCK · 报告主 agent fix layout apply 逻辑 |
| 录屏 > 10MB | ADR-011 R4 超限 | ffmpeg compress 见 §2.3 |

---

## 11 · 跑完反馈给主 agent 的格式

跑完后回复主 agent · 模板（Codex review fix 后更新 · 含 §2.4 content validation + §5.5 destructive path 4 路径 + I3 推算公式）：

```
MVP-05 Phase D capture done · branch: docs/mvp-05-phase-d-capture

【6 截图 content validation · §2.4】
- 01 Solo · ✅ / ❌
- 02 Horizontal 2-pane · ✅ / ❌
- 03 Vertical 2-pane · ✅ / ❌
- 04 2×2 quad · ✅ / ❌
- 05 Smart Layouts menu (含 dry-run + warning) · ✅ / ❌
- 06 Solo apply 后 · ✅ / ❌
- 07 30s 录屏 · ___ MB · ✅ / ❌

【F.1 推算 40 PTY · §3.3 · I3】
- MAIN_RSS = ___ MB
- PER_PANE_AVG = ___ MB
- 推算 P99 = MAIN + PER_PANE × 40 = ___ MB
- vs spec budget 500MB · PASS / FAIL

【F.2-F.6 性能 · §4-§5】
§4.2 F.2 横拖：P99 = ___ ms · PASS / FAIL（< 16ms）
§4.3 F.3 竖拖：P99 = ___ ms · PASS / FAIL
§5.1 F.4 ⌘\：P99 = ___ ms · PASS / FAIL（< 150ms）
§5.2 F.5 ⌘⌃W：P99 = ___ ms · PASS / FAIL（< 100ms）
§5.3 F.6 Smart Layouts：P99 = ___ ms · PASS / FAIL（< 200ms）

【§5.5 数据丢失确认路径 · mandatory · I2 + I8 + I10 同类 path 共享 evidence】
- 5.5.2 Solo cancel · default 是 Cancel · ✅ / ❌ · 截图 08 ✅ / ❌
- 5.5.3 Solo confirm · 保留 focus pane · ✅ / ❌ · console 截图 09 ✅ / ❌ · UI 截图 14 ✅ / ❌ · process check (vim/nano kill) ✅ / ❌
- 5.5.4 AI+Runner cancel · 显示降级提示 · ✅ / ❌ · 截图 10 ✅ / ❌
- 5.5.5 AI+Runner confirm · 50/50 + 保留 focus · ✅ / ❌ · 截图 11 ✅ / ❌ · process check (vim/nano kill · I10 与 5.5.3 一致) · ✅ / ❌

【步骤化 acceptance · §9 · I4 + I7】
- A.1 ⌘\ 继承父 shell+cwd · PARENT/CHILD pwd 一致 · ✅ / ❌
- A.2 ⌘⇧\ 继承父 shell+cwd · ✅ / ❌
- A.3.1 多 pane 关单 pane · ✅ / ❌
- A.3.2 sole-pane close → tab close · ✅ / ❌ · 截图 15 ✅ / ❌
- A.4 单层上限 toast · spec 文案 verbatim · ✅ / ❌ · 截图 12 ✅ / ❌
- D.4 比例持久化（重启 app）· ✅ / ❌
- E.1 click focus 高亮 · ✅ / ❌ · 截图 13 ✅ / ❌
- E.2 仅 focus pane 收 keydown · ✅ / ❌
- E.3 yes 持续输出 · 切 focus 不打断 · 增长率 ___% · ✅ / ❌

【evidence 完整性 · §7 §I5】
- §7.2 ls 14 PNG + 1 MOV check · ✅ / ❌
- §7.3 metrics empty fields check · ✅ / ❌
- §7.4 D.4/E.2/E.3 PASS/FAIL grep check · ✅ / ❌
- 全 ✅ 才能 commit · 任一 ❌ BLOCK

异常 / 已知坑（如有）：
___
```

主 agent 据此开 PR · 翻 spec done。

**任一 ❌（特别是 §5.5 destructive path 或 F.1 推算超 budget 或 §7 evidence-completeness）即 BLOCK · 不能翻 spec done · 必须先报告主 agent 评估 fix path**。
