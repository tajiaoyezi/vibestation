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

## 1 · 启动 dev mode + 创 4 panes fixture（5 min）

```bash
# 1.1 · 启动 dev mode（前台跑 · 输出在终端）
pnpm tauri:dev
```

等窗口出现（约 5-15s 取决于增量 build）。

**1.2 · 创 1 tab + 4 panes 2×2 布局**（手动）：

| 操作 | 快捷键 | 结果 |
|---|---|---|
| 1. App 启动 | — | 默认 1 tab + 1 pane（Solo） |
| 2. 横分 | `⌘\` | 2 panes 水平 |
| 3. 在右 pane 下分 | `⌘⇧\` | 3 panes（左 1 + 右 2 上下） |
| 4. focus 左 pane（点击）| 鼠标 click | focus 高亮迁移 |
| 5. 在左 pane 下分 | `⌘⇧\` | 4 panes 2×2 |

**Sanity check**：4 panes 2×2 布局 · 每个 pane 有 prompt · `echo hi` 应该能跑。

如果某个 pane 没 prompt（PTY 没起）· **不要继续**· 在 webview Cmd+Option+I 开 DevTools 看 console errors · 报告主 agent。

---

## 2 · 6 截图 + 30s 录屏（10 min）

### 2.1 · 自动化 · capture-phase-d.sh

**前置**：app 窗口在前台 · 4 panes fixture 已就位。

```bash
# 在另一个终端 tab（不在 pnpm tauri:dev 那个）
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation
bash scripts/capture/mvp-05/capture-phase-d.sh
```

按提示按 Enter 后 · 脚本会用 osascript 模拟键盘自动跑 6 截图：

- `01-solo-single-pane.png`
- `02-horizontal-2-panes.png`（脚本会先 ⌘\ · 但本步前置已 4 panes · 可能干扰 · 见 fallback）
- `03-vertical-2-panes.png`
- `04-2x2-quad-panes.png`
- `05-smart-layout-menu.png`
- `06-after-smart-apply.png`

### 2.2 · 自动化失败 · 手动 fallback

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

**验证**：`ls -la docs/runtime-evidence/mvp-05/*.png` 应该 6 张 · 单张 50-500 KB（按 ADR-011 R4）。

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

---

## 3 · F.1 内存测量（5 min）

**前置**：4 panes 2×2 fixture 仍在 · app 跑着。

```bash
# 跑 3 次 · 每次取一个 RSS 总数
for i in 1 2 3; do
  echo "=== Run $i ==="
  bash scripts/capture/mvp-05/measure-memory.sh
  echo ""
  sleep 2
done
```

**记录数字**（写在 metrics-mvp-05.md F.1 段）：

```
Run 1: ___ MB（脚本输出 "Total: ___ MB"）
Run 2: ___ MB
Run 3: ___ MB
P99 ≈ max(R1, R2, R3) = ___ MB
```

**通过条件**：P99 < 500MB（spec §F.1 总上限 · 10 tab × 4 pane fixture 推算）· 单 pane ≈ 10MB（SPIKE-05 基线）· 4 pane ≈ 40MB · 实测 < 500MB 即 PASS。

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

### A. 分屏操作（4 项 · 录屏覆盖）

- [ ] A.1 ⌘\ 右分屏 · 新 pane 继承父 shell + cwd · **录屏 0-5s 段验证**
- [ ] A.2 ⌘⇧\ 下分屏 · 同上
- [ ] A.3 ⌘⌃W 关 pane · 仅剩 1 时关 tab · **录屏 20-25s 段验证**
- [ ] A.4 已在右分状态再 ⌘\ → toast 拒绝 · 测 1 次（手动）

### B. 单层嵌套规则（4 项 · 单元测试覆盖 · CLI 已验）

- [x] B.1 4 合法布局 · 单元测试 `panes::tests::*` PASS
- [x] B.2 2 非法布局 · `h2_invalid_3horizontal_layout_is_rejected_*` PASS
- [x] B.3 6 用例覆盖 · 单元测试通过 382 全 PASS
- [x] B.4 非法操作回滚 · `split_atomicity_*` 6 case 全 PASS

### C. Smart Layouts（4 项 · 截图 05 + 06 + 录屏 25-30s 覆盖）

- [ ] C.1 命令面板提供 Solo + AI+Runner（截图 05 验证）
- [ ] C.2 Solo 二次确认 · 测 1 次（手动 · vim/nano 状态可选）
- [ ] C.3 AI+Runner 强制右分屏 50/50 · 2×2 时降级 · 测 1 次
- [ ] C.4 dry-run 预览（截图 05 验证）

### D. 分隔条（5 项 · 录屏 5-15s 覆盖）

- [ ] D.1 拖拽水平 splitter · **录屏 5-10s 验证**
- [ ] D.2 拖拽垂直 splitter · **录屏 10-15s 验证**
- [ ] D.3 双击复位 50/50 · **录屏 15-20s 验证**
- [ ] D.4 比例持久化（重启 app 后 ratio 一致 · 测 1 次手动 · 关 app · 重启 · 看 ratio）
- [ ] D.5 60FPS · **F.2/F.3 实测 P99 < 16ms PASS**

### E. Focus（3 项 · 截图 04 + 手动）

- [ ] E.1 click pane → focus 高亮（截图 04 看边框 · 手动 click 各 pane 验证）
- [ ] E.2 仅 focus pane 收 keydown（手动测 · pane A focus + ls · pane B 不变）
- [ ] E.3 yes 持续输出（手动测 · pane A 跑 yes · 切 focus B · pane A 输出不停）

### F. 性能（6 项 · §3-§5 实测覆盖）

- [ ] F.1 内存 < 500MB · §3 实测 P99
- [ ] F.2 水平 60FPS · §4.2 实测 P99
- [ ] F.3 垂直 60FPS · §4.3 实测 P99
- [ ] F.4 ⌘\ < 150ms · §5.1 实测 P99
- [ ] F.5 ⌘⌃W < 100ms · §5.2 实测 P99
- [ ] F.6 Smart Layouts < 200ms · §5.3 实测 P99

---

## 10 · Fallback / 已知坑

| 坑 | 症状 | 解 |
|---|---|---|
| osascript "not allowed assistive access" | capture-phase-d.sh 第一行报错 | 系统设置 → 隐私与安全性 → 辅助功能 · 加 Terminal / iTerm |
| capture-phase-d.sh osascript 模拟键盘没反应 | 截图都是同一画面 | app 窗口没 focus · 或 Tauri 不响应 OS-level 模拟键盘 · 走 §2.2 完全手动 fallback |
| F.1 measure-memory.sh `pgrep` 找不到 vibestation-app | "未找到进程"报错 | release build 进程名可能不同 · 试 `ps aux \| grep -i vibe` 找 PID · 改脚本 PROCESS_NAME 参数 |
| F.2/F.3 拖拽帧时长 > 16ms | 实测 FAIL | 先确认 dev mode（dev 比 release 慢）· release build 实测一次 · 仍 FAIL 报告主 agent · spec §⚠️ 已知风险加技术债 |
| F.4/F.5/F.6 console 没输出 | 仪表化 log 不见 | webview console 切 "All levels" · 含 info · 看 PR #151 仪表化代码（Terminal.tsx） |
| 录屏 > 10MB | ADR-011 R4 超限 | ffmpeg compress 见 §2.3 |

---

## 11 · 跑完反馈给主 agent 的格式

跑完后回复主 agent · 模板：

```
MVP-05 Phase D capture done · branch: docs/mvp-05-phase-d-capture

§3 F.1 内存：P99 = ___ MB · PASS / FAIL
§4.2 F.2 横拖：P99 = ___ ms · PASS / FAIL
§4.3 F.3 竖拖：P99 = ___ ms · PASS / FAIL
§5.1 F.4 ⌘\：P99 = ___ ms · PASS / FAIL
§5.2 F.5 ⌘⌃W：P99 = ___ ms · PASS / FAIL
§5.3 F.6 Smart Layouts：P99 = ___ ms · PASS / FAIL

截图：6 张 PNG · 录屏：1 MOV · metrics-mvp-05.md 已填

异常 / 已知坑（如有）：
___
```

主 agent 据此开 PR · 翻 spec done。
