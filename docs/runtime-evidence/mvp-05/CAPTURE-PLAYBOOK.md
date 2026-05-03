# MVP-05 Phase D · Capture Playbook · Arbiter 30-45 min 收口

> **目标**：按本 playbook 一次跑完 · 产出 6 张截图 + 30s 录屏 + F.1-F.6 性能数字 · 主 agent 据此把 MVP-05 spec frontmatter 从 `ready` 翻 `done`。
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

## 🛡 4 Invariant（playbook design intent · Codex review fix · 2026-05-03）

未来改 playbook 任何段前必先对照本 4 条 · 防 Codex 已抓的 3 类 finding 复发：

- **I1 · precondition 必须显式验证 · 不能假设**
  - 违反案例：fixture 起点 ≠ capture 脚本假设的起点 → 6 PNG 都生成但内容错（Codex finding 1）
  - 应用：每个自动化步骤前必须有 setup 步骤把状态归零 · 自动化跑完后必须有 content validation（不只是 file exists/size）
- **I2 · spec 强制 acceptance 不能在 playbook 标"可选" / "optional"**
  - 违反案例：spec §C 要求 vim/nano 编辑状态二次确认 · playbook 标"vim/nano 状态可选" → evidence pass 但用户数据丢失 path 没测（Codex finding 2）
  - 应用：playbook 任何 acceptance 验证步骤的 mandatory/optional 必须等同 spec
- **I3 · playbook 测量 pass 条件必须等价于 spec 的 acceptance 通过条件**
  - 违反案例：spec F.1 要 40 PTY (10 tab × 4 pane) < 500MB · playbook 测 4 pane < 500MB → 实际 40 PTY 可能远超 500MB 但 playbook 标 PASS（Codex finding 3）
  - 应用：playbook 测量公式必须直接等于 spec budget 公式 · 推算 / 抽样必须在 metrics 文件留 per-unit 数字 + 完整推算式
- **I4 · 每个 acceptance 项必须有具体可执行步骤 · 不能"测 1 次（手动）"含糊**
  - 违反案例：A.4 / C.3 / E.2 / E.3 标"测 1 次"但没列按键序列 → Arbiter 跑时随便点几下声称 PASS · evidence 不可复现
  - 应用：每项 acceptance 必须带 step-by-step 按键序列 + 期望结果 + 失败时怎么报告

**自检**：写完任何 playbook 段后 · grep "测 1 次"/"可选"/"optional" → 命中即违反 I2/I4 · 必须补步骤或改强制。

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

### 3.2 · 测 4 pane RSS 3 次

```bash
# 在另一个终端 tab
for i in 1 2 3; do
  echo "=== Run $i ==="
  bash scripts/capture/mvp-05/measure-memory.sh
  echo ""
  sleep 2
done
```

脚本输出（每 run）：

- `Main app PID X: Y MB`（main app · 含 webview · 与 PTY 数无关 · 视为 fixed overhead）
- `Shell PID Z1: A1 MB` × 4（4 个 pane shell）
- `Total: B MB（C shell 进程）+ Main = D MB`

**记录两个数字**（每 run）：

- `MAIN_RSS_MB` = Main app（fixed overhead）
- `PER_PANE_AVG` = (4 个 shell RSS 之和) / 4

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

### 5.5.3 · 路径 2：Solo confirm（1 min）

| 步骤 | 操作 | 期望 |
|---|---|---|
| 1 | ⌘⇧P → Solo | dry-run 预览（同 5.5.2）|
| 2 | 显式选 **Confirm**（点击或键盘选中 + Enter）| 关 3 panes · **保留 focus pane = 右下** |
| 3 | sanity check | 1 pane 占满 · 内容是右下 pane 的 prompt history（`echo idle-pane-d` 输出）· vim/nano process 已被 kill |

**❌ FAIL 信号**：
- 保留的不是原 focus pane（右下）→ Smart Layouts 不尊重 focus
- 多 pane 残留 → close transaction 没 atomic（违反 spec §H.3）

**console 证据**：confirm 后 console 应有 `[mvp-05][F.6] layout_apply solo → DOM commit: XX.XXms` log（同 §5.3）→ 截图 console 存 `docs/runtime-evidence/mvp-05/09-solo-confirm-console.png`

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

### 5.5.5 · 路径 4：AI+Runner confirm（1 min · 重建 fixture）

重新跑 §5.5.1 fixture · 然后：

| 步骤 | 操作 | 期望 |
|---|---|---|
| 1 | ⌘⇧P → AI+Runner → Confirm | 先降级（关 2 panes 保留 focus + 1 邻居）· 再强制右分屏 50/50 |
| 2 | sanity check | 最终是水平 2 panes 50/50 · 左 pane 是原 focus（右下 idle-pane-d）· 右 pane 是新 spawn shell（无 prompt history）· 左下 nano + 左上 vim process 已 kill |

**❌ FAIL 信号**：
- 不是水平 2 panes / 比例不是 50/50 → §C.3 strict 50/50 acceptance 不达标
- 保留的左 pane 不是原 focus → focus tracking 错

**截图证据**：`docs/runtime-evidence/mvp-05/11-ai-runner-confirm-result.png`

### 5.5.6 · 路径自检（mandatory）

跑完 5.5.2-5.5.5 共 4 路径 · 4 张证据图（08/09/10/11）必须存在：

```bash
ls -la docs/runtime-evidence/mvp-05/{08,09,10,11}-*.png
```

任一缺 / 任一 FAIL 信号触发 → BLOCK §6 metrics 填表 · 报告主 agent。

---

## 6 · 填 metrics-mvp-05.md（5 min）

打开 `docs/runtime-evidence/mvp-05/metrics-mvp-05.md` · 把上面所有"实测"段的空白填上。

实测后状态字段：

- `🟡 工具就位 · 待跑` → `✅ 实测 PASS`（如 F.x P99 < 目标）
- `🟡 工具就位 · 待跑` → `❌ 实测 FAIL`（如 F.x P99 ≥ 目标 · 触发 fallback / 进 spec §⚠️ 已知风险）

---

## 7 · 关闭 app + commit + push（5 min）

```bash
# 7.1 · 关 app（pnpm tauri:dev 终端 Ctrl+C）

# 7.2 · 验证 evidence
ls -la docs/runtime-evidence/mvp-05/
# 应有：6 PNG + 1 MOV + metrics-mvp-05.md + CAPTURE-PLAYBOOK.md (本文件)

# 7.3 · git status 确认
git status
# 应显示：
# - 新增 6 PNG + 1 MOV
# - 修改 metrics-mvp-05.md
# - （已有）CAPTURE-PLAYBOOK.md（本 playbook 在 docs/mvp-05-capture-playbook 分支已 merge · 不显示）

# 7.4 · stage + commit
git add docs/runtime-evidence/mvp-05/
git commit -m "docs(MVP-05): Phase D runtime evidence · 6 截图 + 30s 录屏 + §F 实测数字

按 docs/runtime-evidence/mvp-05/CAPTURE-PLAYBOOK.md 跑完 · §F.1-F.6 全 PASS（或具体说明）

Co-authored-by: Claude Code <noreply@anthropic.com>"

# 7.5 · push
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

### A. 分屏操作（4 项 · 录屏覆盖 + A.4 显式 toast 测）

- [ ] A.1 ⌘\ 右分屏 · 新 pane 继承父 shell + cwd · **录屏 0-5s 段验证**
- [ ] A.2 ⌘⇧\ 下分屏 · 同上
- [ ] A.3 ⌘⌃W 关 pane · 仅剩 1 时关 tab · **录屏 20-25s 段验证**
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

【§5.5 数据丢失确认路径 · mandatory · I2】
- 5.5.2 Solo cancel · default 是 Cancel · ✅ / ❌ · 截图 08 ✅ / ❌
- 5.5.3 Solo confirm · 保留 focus pane · ✅ / ❌ · 截图 09 ✅ / ❌
- 5.5.4 AI+Runner cancel · 显示降级提示 · ✅ / ❌ · 截图 10 ✅ / ❌
- 5.5.5 AI+Runner confirm · 50/50 + 保留 focus · ✅ / ❌ · 截图 11 ✅ / ❌

【步骤化 acceptance · §9 · I4】
- A.4 单层上限 toast · spec 文案 verbatim · ✅ / ❌ · 截图 12 ✅ / ❌
- D.4 比例持久化（重启 app）· ✅ / ❌
- E.1 click focus 高亮 · ✅ / ❌ · 截图 13 ✅ / ❌
- E.2 仅 focus pane 收 keydown · ✅ / ❌
- E.3 yes 持续输出 · 切 focus 不打断 · 增长率 ___% · ✅ / ❌

异常 / 已知坑（如有）：
___
```

主 agent 据此开 PR · 翻 spec done。

**任一 ❌（特别是 §5.5 destructive path 或 F.1 推算超 budget）即 BLOCK · 不能翻 spec done · 必须先报告主 agent 评估 fix path**。
