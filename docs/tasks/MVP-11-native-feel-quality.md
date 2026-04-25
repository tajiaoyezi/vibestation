---
id: MVP-11
type: mvp
title: Native Feel Quality · 对标 MUX0 · 治 "web 套壳" 观感
status: ready
owner:
phase: W12+
depends_on: ["MVP-10"]
depends_on_notes: "MVP-10 Phase A 设置面板 UI 已 done（PR #114）· MVP-11 Phase 1/2/3/5 不依赖 MVP-10 Phase B · **MVP-11 Phase 4（Appearance 字段扩展）必须等 MVP-10 Phase B IPC + ts-rs binding 交付后才能实施**（settings_update IPC + AppSettings/SettingsUpdateRequest binding 是 Phase B 产物 · Phase A 仅有 UI mock + AppSettingsStore KV 工具类 · 见 MVP-10 spec §G.1）· 文件域和 MVP-10 Phase B/C/D/E 不冲突 · 可并行。"
blocks: []
blocked_by: []
blocked_note:
estimate: 6d
plan_ref: implementation-plan.md §10.1（功能清单 · v0.1 GA · 用户感知质量补强 · 无直接上位 plan_ref · session 19 用户反馈驱动 · MUX0 对标）
risk_ref: R31（新增·本 MVP 自定义·待 implementation-plan §9 追加）
reviewer: Claude Code（self-review + code-reviewer agent · session 19）
---

# MVP-11: Native Feel Quality · 对标 MUX0

> **状态**：`ready`（session 19 翻转 · self-review + code-reviewer agent 0 CRITICAL · round 2 修 8 处后 · Arbiter broad authorization）
> **依赖**：MVP-10 Phase A 设置面板（已 done · PR #114）· Phase 4 等 MVP-10 Phase B
> **战略依据**：用户 session 19 反馈 "目前程序像 web 套壳 · 不像桌面 app" · 对标 MUX0（`https://github.com/10xChengTu/MUX0`）的 native 观感

---

## 🎯 目标（Goal）

让 Vibestation 在 Tauri 2 + webview 架构限制内 · 视觉和交互**最大化逼近 macOS 原生应用**观感 · 消除 "web 套壳" 违和感。对标参考：Zed / VSCode / Raycast / MUX0（前三者同为 Electron/Tauri · 做到 native 级）。

## 📖 背景（Context）

- 用户 session 19 实测反馈 · "pnpm tauri:dev 启动后像浏览器页面 · 右键是浏览器菜单 · 可以 Cmd+A 全选整页"
- 对标项目 MUX0（186 stars · Swift 72% · libghostty + Metal GPU · 完全原生）· **架构级超越 Vibestation**（Tauri webview）
- 架构限制：**不可能**完全达到 MUX0 的 GPU 原生渲染级别（否则要从 Tauri 切到 Swift/Rust native UI framework · 爆炸性变更）
- 架构内天花板：**Zed / Raycast / VSCode 级**（它们也是 Electron/Tauri · 通过 Vibrancy + 自定义 title bar + 禁浏览器行为 + native menu 做到 native 观感）
- 本 MVP 在架构天花板内推进 · 不触及 #5/#6/#12/#19 决策锁定

**v0.1 GA 必要性**：user-facing quality · 直接影响"这是专业产品"印象 · 建议 v0.1 GA 前完成 · 但可与 MVP-10 Phase B/C/D/E 并行。

---

## 🎨 功能范围（Scope）

**Do**：
- macOS Vibrancy（毛玻璃材质 · native `NSVisualEffectView`）
- 禁用 webview 浏览器行为（右键 inspector / Cmd+R 刷新 / Cmd+- 缩放 / Ctrl+A 全选页面）
- Tauri native context menu（走 AppKit NSMenu · 替换 web div 伪装）
- 自定义 title bar（`titleBarStyle: "Overlay"` + `hiddenTitle: true` · traffic light 融入内容区）
- 对齐 macOS HIG 字体（SF Pro Display UI + SF Mono terminal · Linux fallback Inter + JetBrains Mono）
- Appearance 设置面板扩展（`Background Opacity` / `Window Padding X/Y` / `Cursor Style` / `Cursor Blink` / `Unfocused Pane Opacity` 5 字段 · 对标 MUX0 · **不含 Background Blur**：Tauri `windowEffects.radius` 是窗口圆角不是 blur 强度 · macOS Vibrancy blur 由系统 material 决定不可调）
- Linux / Windows 降级（Linux 纯色兜底 · Windows 推 v0.4）

**Don't**：
- 切换技术栈到 Swift / Rust native UI（爆炸性变更 · 违反决策 #6/#12/#19 · 超出 MVP-11）
- 自绘 terminal 渲染层（当前仍用 xterm.js · Metal GPU 渲染超出范围）
- 完整 agent status icon 系统（MUX0 的差异化 · 但依赖 AI-Aware Pane · v1.0 vision 锁定 · 见 ADR-009）
- Windows 11 Mica 效果（v0.4 Windows 支持后再说）
- 触摸栏 / Dock 徽标 / Notification Center widget（v0.3+）

## 🛠 实施进度

MVP-11 估时 6d · 拆 5 Phase 实施 · Phase 1-4 可多 agent 并行（文件域大部分隔离）· Phase 5 单 agent 收尾：

| Phase | 范围 | 文件域 | 依赖 | 状态 | PR |
|---|---|---|---|---|---|
| **Phase 1 · Vibrancy + 禁 webview 行为** | `tauri.conf.json` 加 `windowEffects` + `transparent: true` + `app.macOSPrivateApi: true`（Tauri 2 通过 conf 启用 · **不是** Cargo feature）· 全局 CSS 半透明 + 禁 `user-select` + terminal/diff override · 前端 keyboard event 禁 Cmd+R / Cmd+- / Ctrl+A（prod only） | `crates/app/tauri.conf.json` · `web/src/index.css` · `web/src/main.tsx` | MVP-10 Phase A（已 done） | ✅ done | — |
| **Phase 2 · 自定义 title bar + Traffic Light 融入** | `titleBarStyle: "Overlay"` + `hiddenTitle: true` + `trafficLightPosition` · 前端加 `.title-bar-drag` 区域（`-webkit-app-region: drag`）· sidebar 延伸到 title bar 区 · Linux `#[cfg(target_os = "macos")]` 分支保留默认 title bar | `crates/app/tauri.conf.json` · `web/src/App.tsx` · `web/src/layouts/*.tsx` · `web/src/index.css` | Phase 1（Vibrancy 生效才能看到 overlay） | ⏳ todo | — |
| **Phase 3 · Native Context Menu + 快捷键** | 新建 `crates/app/src/menu.rs` · Tauri v2 `Menu API`（NSMenu 走 AppKit）· 标签栏右键（Close / Close Others / Rename / Duplicate）· 终端右键（Copy / Paste / Clear）· `⌘T/⌘W/⌘D` 快捷键 · permission toml + capability 引用 | `crates/app/src/menu.rs`（新建）· `crates/app/permissions/menu.toml`（新建）· `crates/app/capabilities/default.json` · `web/src/panels/Terminal/*.tsx` | Phase 1 | ⏳ todo | — |
| **Phase 4 · Appearance 字段对标 MUX0** | 扩展 MVP-10 `AppearanceGroup.tsx` 加 6 字段（Background Opacity / Blur / Padding X / Y / Cursor Style / Cursor Blink）· `app_settings` KV 扩 6 keys · CSS vars 消费 · Unfocused Pane Opacity 单独在 Terminal 组 | `web/src/panels/Settings/AppearanceGroup.tsx`（扩）· `web/src/panels/Settings/TerminalGroup.tsx`（扩 Unfocused Pane）· `crates/core/src/app_settings.rs`（6 新 KV key · YAGNI 无 migration）· `crates/app/src/lib.rs` IPC（复用 MVP-10 `settings_update`） | MVP-10 Phase A（设置面板存在）· Phase 1（Opacity/Blur CSS vars 生效） | ⏳ todo | — |
| **Phase 5 · 字体对齐 HIG** | CSS `font-family` · macOS `"SF Pro Display", system-ui` + `"SF Mono", ui-monospace` · Linux `"Inter", system-ui` + `"JetBrains Mono", monospace` · 不 bundle 字体（走系统字体） | `web/src/styles/typography.css`（新建或扩）· `web/src/index.css` | 无 | ⏳ todo | — |

**下次 agent 起点**：Phase 2 · Phase 1 已落地（Vibrancy + transparent + macos-private-api + CSS semi-transparent + webview 行为禁用）· 继续 title bar overlay + Traffic Light 融入。

**并行调度建议**：
- OpenCode（全栈）· Phase 1 + Phase 4（连贯：Vibrancy 生效后扩 Appearance 字段）· 估时 4d
- Codex（视觉精细）· Phase 2（title bar 跨平台分支）· 估时 2d
- Kimi（本地或远程）· Phase 3（Menu API 独立模块）· 估时 1.5d
- 主 agent 或 Kimi · Phase 5（字体 · 0.5d 小任务）

**依赖关系说明**：
- MVP-11 整体依赖 MVP-10 Phase A done（✅ · PR #114）· 其他 MVP-01..09 已 done
- Phase 1 → Phase 2（Overlay title bar 依赖 Vibrancy 显示 · 否则 overlay 看不到）
- Phase 1 → Phase 4（Appearance 字段的 Opacity/Blur CSS vars 依赖 Phase 1 透明基础）
- Phase 3 独立（只动 menu.rs + Tauri Menu API · 不依赖 Vibrancy）
- Phase 5 独立（纯 CSS）

## 🖼 UI 引用

- **对标 MUX0**：`https://github.com/10xChengTu/MUX0` · 尤其其 Settings → Appearance 6 字段 + Font 3 字段
- **Vibrancy 视觉**：VSCode Vibrancy 扩展的默认 `hudWindow` / Zed 默认设置
- **Title bar 布局**：参考 Raycast / Zed · traffic light 左上 · sidebar 内容延伸到 title bar 区
- **`design/directions/1-calm-studio.html`**：原有 Calm Studio 视觉方向保留 · MVP-11 仅在此基础加 Vibrancy 层（不替换）

## ✅ Acceptance

### A. Phase 1 · Vibrancy + 禁 webview 行为

- [ ] A.1 `tauri.conf.json` 加：`app.macOSPrivateApi: true` + `windows[0].transparent: true` + `windows[0].windowEffects: { effects: ["hudWindow"], state: "followsWindowActiveState", radius: 12 }`（Tauri 2 通过 conf 启用 macos-private-api · **不是** Cargo feature · radius 是窗口圆角 · 不是 blur 强度）
- [ ] A.2 macOS 启动后窗口背景透出桌面壁纸（毛玻璃效果可见）· 截图 `docs/runtime-evidence/mvp-11/01-vibrancy-macos.png`
- [ ] A.3 CSS 全局 `html, body { background: transparent }` · 主 container 半透明背景（light `rgba(250,250,250,0.85)` / dark `rgba(28,28,30,0.75)`）
- [ ] A.4 禁用 webview 行为（release build）：右键无 inspector / Cmd+R 不刷新 / Cmd+- / Cmd+= 不缩放 / Ctrl+A 不全选页面（terminal/diff/editor 单独保留 text selection）
- [ ] A.4.1 Dev build（`debug_assertions`）保留 DevTools 和浏览器行为（方便调试）· 仅 release 禁
- [ ] A.5 Linux 降级：`#[cfg(target_os = "linux")]` 分支不启 transparent · CSS 用纯色 `rgba(bg, 0.98)` 兜底 · 窗口仍正常显示（无黑屏）
- [ ] A.6 MVP-04 Phase F benchmark 不回归：scrollback throughput 回归 < 10%（如果 > 10% · 回退 `transparent: false` · 使用 `windowBackground` 不透明 effect）· 贴 bench 对比数字到 PR body

### B. Phase 2 · Title bar 定制

- [ ] B.1 `titleBarStyle: "Overlay"` + `hiddenTitle: true` + `trafficLightPosition: { x: 20, y: 20 }`（macOS only）
- [ ] B.2 前端加 `.title-bar-drag` 区域 · 高度 28px · `-webkit-app-region: drag` · 覆盖顶部 · 内嵌按钮 `-webkit-app-region: no-drag`
- [ ] B.3 用户可以拖动窗口（从 title bar 区 · 不从 sidebar 内容）· 实测 2 次（focus / blur 状态都测 · 注意 Overlay 不 focus 有已知 drag 限制）
- [ ] B.4 Linux 保留默认 title bar（`#[cfg(target_os = "linux")]` 分支不改 `titleBarStyle`）· 窗口正常显示 · 不出现 `.title-bar-drag` 空白区
- [ ] B.5 sidebar 视觉延伸到 title bar 区（traffic light 悬浮在 sidebar 内容之上 · 类 Zed）· 截图 `docs/runtime-evidence/mvp-11/02-title-bar-overlay.png`

### C. Phase 3 · Native Context Menu + 快捷键

- [ ] C.1 新建 `crates/app/src/menu.rs` · 使用 Tauri v2 `tauri::menu::Menu` API · 走 AppKit NSMenu
- [ ] C.2 标签栏右键菜单：`Close Tab` / `Close Other Tabs` / `Close Tabs to the Right` / `Rename Tab` / `Duplicate Tab`（5 项 · 符合 macOS HIG）
- [ ] C.3 终端右键菜单：`Copy` / `Paste` / `Clear Terminal` / `Select All`（4 项）
- [ ] C.4 快捷键注册：`⌘T` 新 tab · `⌘W` 关 tab · `⌘D` 水平 split · `⌘⇧D` 垂直 split
- [x] C.4.0 `⌘,` 打开 Preferences（已落地 · MVP-10 Phase A · `web/src/App.tsx:164` `case ","` · 仅回归验证 · 不重实现）
- [ ] C.5 permission toml + capability 引用（按 tauri-v2-patterns rule）
- [ ] C.6 Linux 降级：Linux GTK Menu 自动 fallback（Tauri v2 Menu API 跨平台 · 无需额外代码）
- [ ] C.7 runtime 证据：macOS 截图 `docs/runtime-evidence/mvp-11/03-context-menu-tab.png` + `04-context-menu-terminal.png`

### D. Phase 4 · Appearance 字段对标 MUX0

- [ ] D.1 扩展 `AppearanceGroup.tsx` 加 6 字段：
  - `Background Opacity`（slider 0-1 · step 0.05 · default 0.85）
  - `Window Padding X`（number 0-20 · default 2）
  - `Window Padding Y`（number 0-20 · default 2）
  - `Cursor Style`（radio `block` / `bar` / `underline` · default `block`）
  - `Cursor Blink`（toggle · default false）
- [ ] D.2 扩展 `TerminalGroup.tsx` 加 `Unfocused Pane Opacity`（slider 0-1 · default 0.7 · 仅多 pane 生效）
- [ ] D.3 `app_settings` 扩 6 KV keys：`bg_opacity` / `window_padding_x` / `window_padding_y` / `cursor_style` / `cursor_blink` / `unfocused_pane_opacity`（YAGNI · 无 migration · **不含 bg_blur** · macOS Vibrancy blur 由系统 material 决定不可调）
- [ ] D.4 IPC：复用 MVP-10 `settings_update`（partial update）· 不新增 IPC command · binding 扩展（`SettingsUpdateRequest` 加 6 字段）· **依赖 MVP-10 Phase B 落地**（见 `depends_on_notes`）
- [ ] D.5 实时生效路径分离（明示 · 防错误实现）：
  - **Opacity / Padding / Cursor / Unfocused Opacity** → 走 CSS var（`--bg-opacity` / `--window-padding-x` / `--window-padding-y` / `--cursor-style` / `--unfocused-opacity`）· DOM commit < 100ms
  - **Vibrancy material 切换**（v0.2+ 才考虑 · 当前固定 hudWindow）→ 走 Tauri `window.setEffects()` 调用 · 不能 CSS
  - 注意：MVP-11 Phase 4 用户**只控 CSS 路径** · `windowEffects.radius` 固定 12（窗口圆角 · 非 blur 强度）
- [ ] D.6 持久化：重启应用后 6 字段值一致（integration test 覆盖）
- [ ] D.7 runtime 证据：3 张截图（Opacity=0.5/0.85/1.0 对比 · 展示毛玻璃强度变化）· `docs/runtime-evidence/mvp-11/05-opacity-variants.png`

### E. Phase 5 · 字体对齐 HIG

- [ ] E.1 `web/src/styles/typography.css` 定义 `--font-ui` + `--font-mono` CSS var
  ```css
  :root {
    --font-ui: -apple-system, "SF Pro Display", system-ui, "Inter", "Segoe UI", sans-serif;
    --font-mono: ui-monospace, "SF Mono", "JetBrains Mono", "Menlo", "Consolas", monospace;
  }
  ```
- [ ] E.2 全局 `body { font-family: var(--font-ui); }` · terminal/diff/commit `font-family: var(--font-mono);`
- [ ] E.3 不 bundle 字体（走系统字体 · bundle size 零增）
- [ ] E.4 Linux 测试：系统装 Inter 的显示 Inter · 没装的 fallback system-ui（Ubuntu 24 默认 Ubuntu font）· 截图 Ubuntu VM `docs/runtime-evidence/mvp-11/06-linux-font.png`（可选 · Ubuntu 非主线）
- [ ] E.5 MVP-10 Phase A 的 Font Family 设置 override typography.css 默认（用户显式选字体优先）

## 🧪 测试策略

| 层次 | 范围 |
|---|---|
| 单元 | CSS vars 消费（JS 读 `getComputedStyle` 验证）+ IPC binding 7 字段往返 |
| 集成 | 设置变更 → rusqlite 写入 → 进程重启 → 读取一致（7 字段 full roundtrip） |
| E2E | 完整 flow：打开 Preferences → 改 Opacity 到 0.5 → 看到窗口立刻变透 → 重启 app → 值保持 |
| 手动 QA | 3 平台启动（macOS 14 · Ubuntu 24 · Windows 11 skip）· 拖窗口 / 右键菜单 / 快捷键 |
| Runtime 证据 | ≥ 6 张截图 + 1 段 30s 录屏（完整 Appearance 调节 demo）|

## 📸 Runtime 证据要求

按 [ADR-011](../adr/ADR-011-runtime-evidence-location.md) + `.claude/rules/runtime-evidence-location.md` · MVP-11 实施 PR 必须提交以下证据到 `docs/runtime-evidence/mvp-11/`（进 git · ADR-011 R1-R5）：

- `01-vibrancy-macos.png`（macOS 窗口 Vibrancy 效果 · 能看到桌面壁纸透出）
- `02-title-bar-overlay.png`（title bar overlay · traffic light 融入 sidebar）
- `03-context-menu-tab.png`（标签栏右键 native menu）
- `04-context-menu-terminal.png`（终端右键 native menu）
- `05-opacity-variants.png`（Opacity 0.5 / 0.85 / 1.0 三档对比）
- `06-linux-font.png`（Ubuntu 24 字体 fallback · 可选）
- `07-webview-disabled-behaviors.mp4`（30s 录屏 · 展示 Cmd+R / Cmd+- / 右键 无效）
- `00-linux-fallback.png`（A.5 Ubuntu 24 启动验证 · 主线 macOS 之外可选 · 若 Ubuntu 环境就绪推荐补 · v0.1 GA macOS-only 策略下可 SKIP + 记 known limitation）
- `metrics-mvp-11.md`（MVP-04 Phase F bench 回归对比 · Phase 1 前后 scrollback throughput）

单目录总体积 ≤ 10 MB（ADR-011 R4）· 超则压缩。

## 💾 数据模型变更

`app_settings` 表当前结构（MVP-03 已建）· MVP-11 Phase 4 扩 7 KV key · **不新建 migration**（YAGNI · 对齐 MVP-10 pattern）：

| key | value 编码 | default | 含义 |
|---|---|---|---|
| `bg_opacity` | `"0.85"`（string · f32 解析）| `"0.85"` | 窗口半透明度（影响 container bg alpha） |
| `window_padding_x` | `"2"` | `"2"` | 窗口内容 X 边距（px） |
| `window_padding_y` | `"2"` | `"2"` | 窗口内容 Y 边距（px） |
| `cursor_style` | `"block"` / `"bar"` / `"underline"` | `"block"` | 终端光标形状 |
| `cursor_blink` | `"true"` / `"false"` | `"false"` | 终端光标闪烁 |
| `unfocused_pane_opacity` | `"0.7"` | `"0.7"` | 非聚焦 pane 透明度 |

Rust `AppSettings` struct 扩 6 字段 · ts-rs binding 自动更新 · `SettingsUpdateRequest` partial update 扩 6 `Option<>` 字段。

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)

### G.1 Binding 复用（不新建 · 复用 MVP-10）

| Rust struct | 用途 | 变化 |
|---|---|---|
| `AppSettings`（MVP-10）| 全量 settings 查询 | **扩 6 字段**（bg_opacity / window_padding_x / window_padding_y / cursor_style / cursor_blink / unfocused_pane_opacity · 不含 bg_blur） |
| `SettingsUpdateRequest`（MVP-10）| partial update | **扩 6 `Option<>` 字段** |

### G.2 derive 扩展（示例）

```rust
// crates/core/src/app_settings.rs
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // ... MVP-10 原有 8 字段（theme / font_family / font_size / default_shell / paste_protection / telemetry_opt_in / git_user_name / git_user_email）
    #[ts(type = "number")]
    pub bg_opacity: f32,
    #[ts(type = "number")]
    pub window_padding_x: u32,
    #[ts(type = "number")]
    pub window_padding_y: u32,
    pub cursor_style: String,
    pub cursor_blink: bool,
    #[ts(type = "number")]
    pub unfocused_pane_opacity: f32,
}
```

### G.3 H2 regression proof

实施 PR 时执行：将 `AppSettings.bg_opacity` 临时改为 `background_opacity` · 运行 `pnpm typecheck` 必须 FAIL · 证明 bindings 与前端 import 强关联 · 截图 `docs/runtime-evidence/mvp-11/h2-regression-proof.png` · 验证后恢复原名。

## §H. MVP-11 决策锁定

### H.1 · Vibrancy material 默认值

- **锁定** `"hudWindow"`（macOS · 视觉最像专业 IDE · 文档调研通过）
- **候选记录**：
  - `"sidebar"`（MUX0 同款 · 更浅）
  - `"underWindowBackground"`（最深 · 可能过度）
- **用户可改 vs 不可改**：Phase 4 `Background Opacity`（CSS rgba alpha）是用户可调的"视觉强度"代理 · macOS Vibrancy blur 由系统 material 决定不可调（Tauri/AppKit 都没有 API 调 NSVisualEffectView blur 强度）· `windowEffects.radius` 是窗口圆角 · 不是 blur · 已固定 12 不暴露 · v0.2+ 才考虑加 material 切换 UI
- **禁止**：使用 deprecated material（`light` / `dark` / `appearanceBased` / `mediumLight` / `ultraDark`）

### H.2 · Title bar 策略

- **锁定 macOS** `"titleBarStyle": "Overlay"` + `hiddenTitle: true`（trading off：drag region 复杂度 < native 观感收益）
- **锁定 Linux**：不改 `titleBarStyle`（GTK 不支持 Overlay · 保留默认 chrome）
- **Windows 推 v0.4**：Mica 效果需 Windows 11 · v0.1 GA Windows 不支持

### H.3 · 禁用 webview 行为的 dev/prod 策略

- **锁定** `import.meta.env.PROD` 判断 · release build 才禁
- **禁止**：release build 保留右键 DevTools（避免用户意外看到 web inspector · 破坏 "native app" 观感）
- **允许**：dev build 完全保留浏览器行为（方便调试 · 符合用户 session 18 测试 `./target/debug/` 能开控制台的诉求）

### H.4 · Native context menu 跨平台降级

- **锁定** Tauri v2 `tauri::menu::Menu` API · macOS 自动走 NSMenu · Linux 走 GTK Menu · Windows 走 Win32 menu
- **禁止**：用 web div 模拟 context menu（破坏 native 观感 · 违反 MVP-11 目标）
- **禁止**：为每个 OS 手写 menu 实现（Tauri 已封装 · 重复造轮子）

### H.5 · 字体选择策略

- **锁定**：不 bundle 字体 · 走系统字体优先级链（SF Pro Display → Inter → system-ui → sans-serif）
- **禁止**：bundle Google Fonts 或自托管字体（bundle size 爆 · 违反 §10.2）
- **允许**：MVP-10 Phase A 的 `Font Family` 设置 override 默认（用户显式选字体优先）

## ⚠️ 已知风险

- **R31（新增）Native Feel Quality 回归**：Phase 1 `transparent: true` 可能降低终端滚屏性能 → MVP-04 Phase F bench 回归 · 测试覆盖 A.6 硬门槛（> 10% 回归则回退）
- **Linux Vibrancy 缺失**：Linux WebKitGTK 不支持 Vibrancy → 明示 "Linux 降级为纯色" · 不阻塞
- **Title bar Overlay drag bug**：Tauri 官方 issue #4316 · 不 focus 时 drag 失败 → Acceptance B.3 明示 "注意已知限制"
- **macOS Private API 审核风险**：`tauri.conf.json` 的 `app.macOSPrivateApi: true`（Tauri 2 通过 conf 启用 · 不是 Cargo feature · 见 A.1）可能触发 Mac App Store 拒审 → Vibestation v0.1 不上 App Store（只走 .dmg + notarization · MVP-10）· 无影响
- **字体加载延迟**：系统字体 `SF Pro Display` 在新机首次加载可能 100-200ms → 全局 `font-display: swap` CSS 兜底

## 📝 Notes

- MVP-11 是"美学 + 交互质量"MVP · 不引入新功能 · 不改 IPC contract（除 G.2 的 AppSettings 7 字段扩展）
- v0.1 GA 建议包含 MVP-11 · 否则"web 套壳"第一印象损害产品定位
- 若时间紧 · 可以分两波发布：
  - v0.1.0-alpha：只 Phase 1 + Phase 3（Vibrancy + native menu · 最立竿见影）
  - v0.1.0 GA：Phase 2 + Phase 4 + Phase 5 补齐
- v0.2+ 考虑 `Agents` tab（MUX0 差异化）· 但依赖打破 ADR-009 AI-Aware Pane v1.0 锁定 · 不在 MVP-11 范围

## 🔗 相关

- `CLAUDE.md` #5/#6/#12/#19 决策锁定（本 MVP 不触及）
- MVP-10 Phase A（设置面板 · Phase 4 依赖）
- ADR-009（AI-Aware Pane v1.0 锁定 · 本 MVP 不触及）
- `design/directions/1-calm-studio.html`（视觉原型 · 保留不改）
- 对标项目 MUX0：`https://github.com/10xChengTu/MUX0`
- Spike 笔记：`spike-tmp/local-notes/MVP-11-vibrancy-spike-notes.md`
- 上游：MVP-10 Phase A（已 done · PR #114）
- 下游：v0.2 `Agents` tab（待 ADR-009 松动 · 远期）

---

## 自审四问

1. **递归完备性**：5 Phase 各自 Acceptance（A/B/C/D/E）齐全 ✅ · 每 Phase 有文件域声明 + 依赖 + 估时 ✅ · runtime 证据要求 6 截图 + 1 录屏 + metrics ✅ · 清单自身在清单中 ✅
2. **反向场景**：`transparent: true` 性能回归 → A.6 硬门槛 · > 10% 回退 ✅ · Linux Vibrancy 缺失 → 降级纯色 ✅ · Overlay drag bug → B.3 明示限制 ✅ · 字体缺失 → font-display: swap 兜底 ✅ · Title bar Overlay Linux 不兼容 → B.4 `#[cfg]` 分支保留默认 ✅
3. **边界适用性**：macOS 主线全覆盖 · Linux 降级策略每 Phase 明示 · Windows 推 v0.4 · dev/prod 行为分离（H.3）· dark/light mode CSS media query 覆盖 ✅
4. **YAGNI**：无新 IPC command（复用 MVP-10 `settings_update`）· 无新 migration（app_settings KV 扩）· 不 bundle 字体（系统字体）· 不切技术栈（Tauri 2 内推进）· 不加 Agents tab（v1.0 锁定）· Windows Mica 推 v0.4 ✅
