# MVP-05 Phase C §F · 性能测量记录

> **状态**：测量框架就位 · 实际数字待 Arbiter 本地 capture（v0.1 GA 前）
> **测量基础设施**：performance.now() inline instrumentation（PR #147e）+ measure-memory.sh + capture-phase-d.sh
> **生成时间**：session 19 末（2026-04-25）

---

## §F 测量目标（spec MVP-05 §F）

| ID | 指标 | 目标 P99 | 测量方法 | 状态 |
|----|------|---------|---------|------|
| F.1 | 4 Pane 内存 RSS | < 500MB（10 tab × 4 pane fixture extrap） | `scripts/capture/mvp-05/measure-memory.sh` | 🟡 工具就位 · 待跑 |
| F.2 | 拖拽水平 splitter 60FPS | 帧时长 < 16ms | DevTools Performance 1s 录制 · 帧时长统计 | 🟡 PR #147 已 60FPS rAF · 实测 P99 待 capture |
| F.3 | 拖拽垂直 splitter 60FPS | 同 F.2 | 同 F.2 · 垂直方向独立 | 🟡 同上 |
| F.4 | ⌘\ → 新 pane DOM commit | < 150ms | inline performance.now() 在 handlePaneSplit · console.info 输出 | ✅ 仪表化 done · 实测数字待 capture |
| F.5 | ⌘⌃W → 重排 DOM commit | < 100ms | inline performance.now() 在 handlePaneClose · console.info 输出 | ✅ 仪表化 done · 实测数字待 capture |
| F.6 | Smart Layouts apply | < 200ms | inline performance.now() 在 handleSmartLayoutApply · console.info 输出 | ✅ 仪表化 done · 实测数字待 capture |

---

## 测量手册（Arbiter 本地 30 min · 跑完后填实测数字）

### F.1 4 Pane 内存

```bash
# 1. 启动 app（推荐 release build · pnpm tauri build --debug --no-bundle）
pnpm tauri:build:smoke

# 2. 在 app 里：手动创建 1 tab → ⌘\ 横分 → ⌘⇧\ 在新 pane 下分（共 4 panes 2x2）

# 3. 跑测量脚本
bash scripts/capture/mvp-05/measure-memory.sh

# 4. 重复 3 次取 P99 · 填入下方 "实测" 段
```

**实测**（待填）：

```
Run 1: ___ MB
Run 2: ___ MB
Run 3: ___ MB
P99 ≈ max ≈ ___ MB
```

通过条件：< 500MB · 单 pane ≈ 10MB（SPIKE-05 基线）

### F.2 / F.3 拖拽 splitter 60FPS

DevTools Performance 面板手动测：

1. 启动 app · `pnpm tauri:dev`（不要 release build · DevTools 仅 dev 模式可用）
2. 创建 4 panes 2x2 layout
3. 在 webview 里 `Cmd+Option+I` 打开 DevTools
4. Performance tab → 录制 1s
5. **横拖**水平 splitter（左右拖拽 1s）· 停录
6. 看 Frames 行：每帧 < 16ms 即 60FPS · 取 P99 帧时长
7. 重复 3 次 · 横/竖各测一次（F.2 / F.3）

**实测**（待填）：

```
F.2 Run 1: P99 帧时长 ___ ms
F.2 Run 2: ___ ms
F.2 Run 3: ___ ms
F.2 P99 ≈ ___ ms（< 16ms PASS）

F.3 Run 1: ___ ms
F.3 Run 2: ___ ms
F.3 Run 3: ___ ms
F.3 P99 ≈ ___ ms
```

### F.4 / F.5 / F.6 IPC 操作时延

```bash
# 1. 启动 app（dev 模式 · 因为需要 DevTools console）
pnpm tauri:dev

# 2. 在 webview Cmd+Option+I 打开 DevTools console

# 3. 执行操作 · console 自动输出 [mvp-05][F.x] xxx → DOM commit: ___ms
#    F.4: ⌘\ 横分屏 3 次（取 3 次后 P99）
#    F.5: ⌘⌃W 关 pane 3 次
#    F.6: ⌘⇧P → Solo apply 3 次

# 4. 复制 console 输出 · 填入下方 "实测" 段
```

**实测**（待填）：

```
F.4 ⌘\ → DOM commit:
  Run 1: [mvp-05][F.4] pane_split horizontal → DOM commit: ___ ms
  Run 2: ___ ms
  Run 3: ___ ms
  P99 ≈ ___ ms（< 150ms PASS）

F.5 ⌘⌃W → DOM commit:
  Run 1: ___ ms
  Run 2: ___ ms
  Run 3: ___ ms
  P99 ≈ ___ ms（< 100ms PASS）

F.6 Smart Layouts apply → DOM commit:
  Run 1: ___ ms
  Run 2: ___ ms
  Run 3: ___ ms
  P99 ≈ ___ ms（< 200ms PASS）
```

---

## 实测局限性

session 19 末仪表化完成 · 实际跑 + 填数字留 Arbiter 本地（约 30 min）。原因：

1. session 已极长（32+ PR · 主 agent 注意力分散）
2. F.2 / F.3 DevTools Performance 面板交互需手动 · 自动化 osascript 触达 webview DevTools 困难
3. 4 pane fixture 设置需手动操作（⌘\ ⌘⇧\ 多次按键）· capture-phase-d.sh 提供自动化但首次跑需调试

**v0.1 GA gate 前推荐**：Arbiter 跑一次完整 §F 测量 · 填上述空白 · 截图存 `docs/runtime-evidence/mvp-05/F-perf-screenshots/`。

---

## Phase D · runtime evidence 截图

| 文件 | 内容 | 状态 |
|---|---|---|
| `01-solo-single-pane.png` | Solo 单 pane（默认状态） | 🟡 capture 脚本就位 |
| `02-horizontal-2-panes.png` | ⌘\ 右分屏后 | 🟡 |
| `03-vertical-2-panes.png` | Solo 后 ⌘⇧\ 下分屏 | 🟡 |
| `04-2x2-quad-panes.png` | 右分 + 下分 = 2×2 4 panes | 🟡 |
| `05-smart-layout-menu.png` | ⌘⇧P 命令面板（dry-run 预览） | 🟡 |
| `06-after-smart-apply.png` | Solo apply 后单 pane | 🟡 |
| `07-flow-recording.mov` | 30s 完整流程录屏 | 🟡 手工 |

跑：`bash scripts/capture/mvp-05/capture-phase-d.sh`

录屏单独跑：`screencapture -V 30 -x -l <window-id> docs/runtime-evidence/mvp-05/07-flow-recording.mov`

---

## 关联

- spec：[`docs/tasks/MVP-05-pane-split-single-level.md`](../../tasks/MVP-05-pane-split-single-level.md) §F
- 仪表化代码：`web/src/panels/Terminal/Terminal.tsx` handlers F.4 / F.5 / F.6
- 测量脚本：`scripts/capture/mvp-05/measure-memory.sh` · `scripts/capture/mvp-05/capture-phase-d.sh`
- ADR-011 runtime evidence location · `docs/runtime-evidence/mvp-05/` 进 git
