# MVP-11 Phase 3 Runtime 证据 · 已知限制

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
1. 当前环境无 Cargo/Rust 工具链 → 无法编译 Tauri 应用
2. 当前环境为 Linux → 无法运行 macOS NSMenu
3. 需 macOS 实机 `pnpm tauri:dev` 启动后右键截图

**建议**：由主 agent 或另一台 macOS 机器 checkout 本分支后编译运行并补截图。
