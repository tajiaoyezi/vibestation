# MVP-11 Runtime 证据 · 已知限制

## Phase 2 · Title bar overlay（2026-04-25 · Codex）

### 实施状态

- [x] B.1 `titleBarStyle: "Overlay"` + `hiddenTitle: true` + `trafficLightPosition: { x: 20, y: 20 }`
- [x] B.2 `.title-bar-drag` 28px 覆盖顶部 · `data-tauri-drag-region` + 显式 `getCurrentWindow().startDragging()`
- [x] B.3 macOS window drag runtime 验证 2 次
- [x] B.4 Linux 空白区防护：`.title-bar-drag` 仅 `.platform-macos` 显示；Rust title bar setup 仅 `#[cfg(target_os = "macos")]`
- [x] B.5 `02-title-bar-overlay.png` 已保存

### Runtime 验证

环境：macOS Apple Silicon · debug profile · bundle path `/private/tmp/mvp-11-phase-2-work/target/debug/bundle/macos/Vibestation.app`

命令：

```bash
pnpm tauri build --config crates/app/tauri.conf.json --debug --bundles app --no-sign
open -n /private/tmp/mvp-11-phase-2-work/target/debug/bundle/macos/Vibestation.app
```

可视验证：

- `docs/runtime-evidence/mvp-11/02-title-bar-overlay.png`
- 结果：traffic lights 位于内容区左上；系统标题文字隐藏；sidebar 顶部高度延伸到 title bar 区。

拖动验证：

| 状态 | 输入 | WindowServer bounds |
|---|---|---|
| focus 前 | title bar 区 CGEvent drag from `(940,176)` to `(1140,241)` | `X=640,Y=161` → `X=840,Y=226` |
| blur/再激活路径 | Codex 激活后，title bar 区 CGEvent drag from `(1140,241)` to `(1340,306)` | `X=840,Y=226` → `X=1040,Y=291` |

备注：

- `mcp__computer_use__.drag` 未移动 macOS 窗口；改用原生 CGEvent 做最终拖动验证。
- 系统 `screencapture -l` 对该 Tauri 窗口返回 `could not create image from window`（WindowServer `kCGWindowSharingState=0`）；最终 PNG 来自 Codex Computer Use 的临时截图缓存并已复制入本目录。
- Ubuntu 24 GUI 未在本机可用；Linux title bar 只做静态边界验证，见 task spec `R-PHASE-2.linux`。

## Phase 3 · Native Context Menu + 快捷键（PR #124/#126 · 已知限制）

## 实施状态

- [x] C.1 `crates/app/src/menu.rs` 新建 · Tauri v2 Menu API
- [x] C.2 标签栏右键 5 项（Close Tab / Close Other Tabs / Close Tabs to the Right / Rename Tab / Duplicate Tab）
- [x] C.3 终端右键 4 项（Copy / Paste / Clear Terminal / Select All）
- [x] C.4 快捷键 ⌘T/⌘W/⌘D/⌘⇧D/⌘, 全注册
- [x] C.5 permission toml + capability 引用
- [ ] C.6 Linux GTK Menu fallback 实测
- [ ] C.7 Runtime 证据 2 截图

## C.6 · Linux GTK Menu 自动 fallback

**状态**：SKIP · 当前环境无 Ubuntu GUI

Tauri v2 Menu API 跨平台 · Linux 自动走 GTK Menu · 无需额外代码。
实测需 Ubuntu 24 + GTK 桌面环境 · 当前开发机为 headless Linux · 无法运行 GUI 应用。

**Known limitation**：Linux GTK Menu 外观未在目视验证中确认。

## C.7 · Runtime 证据截图

**状态**：待补 · 需 macOS 实机编译运行

| 文件 | 描述 | 状态 |
|---|---|---|
| `03-context-menu-tab.png` | macOS 标签栏右键 · NSMenu 样式 | 待补 |
| `04-context-menu-terminal.png` | macOS 终端右键 · NSMenu 样式 | 待补 |

**阻塞原因**：
1. 本次 Phase 2 只验证 title bar overlay，不重复打开右键菜单截图路径
2. Linux GTK menu 外观仍需 Ubuntu 24 GUI 环境确认

**建议**：Phase 4/closeout 回收时统一补 `03-context-menu-tab.png` / `04-context-menu-terminal.png`。
