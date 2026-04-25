# MVP-11 Phase 4 · Appearance 字段扩展 · Metrics

## D.1 AppeareanceGroup 扩 6 字段

| 字段 | 控件类型 | 范围 | 默认值 | 状态 |
|---|---|---|---|---|
| Background Opacity | slider | 0-1, step 0.05 | 0.85 | ✅ |
| Background Blur | number input | 0-100 | 20 | ✅ |
| Window Padding X | number input | 0-20 | 2 | ✅ |
| Window Padding Y | number input | 0-20 | 2 | ✅ |
| Cursor Style | radio | block/bar/underline | block | ✅ |
| Cursor Blink | toggle | - | false | ✅ |

## D.2 TerminalGroup 扩 Unfocused Pane Opacity

| 字段 | 控件类型 | 范围 | 默认值 | 状态 |
|---|---|---|---|---|
| Unfocused Pane Opacity | slider | 0-1, step 0.05 | 0.7 | ✅ |

## D.3 app_settings 扩 7 KV

| key | default | 类型 |
|---|---|---|
| bg_opacity | "0.85" | f32 |
| bg_blur | "20" | u32 |
| window_padding_x | "2" | u32 |
| window_padding_y | "2" | u32 |
| cursor_style | "block" | String |
| cursor_blink | "false" | bool |
| unfocused_pane_opacity | "0.7" | f32 |

无 migration · YAGNI · app_settings 表 MVP-03 已建

## D.4 IPC contract

| Rust struct | ts-rs 绑定文件 | 状态 |
|---|---|---|
| AppSettings | AppSettings.ts | ✅ |
| SettingsUpdateRequest | SettingsUpdateRequest.ts | ✅ |

- settings_update IPC 已实现（复用 MVP-10 设计 · 无新增 command）
- settings_get + settings_update 两 command
- emit "settings_changed" event → 前端 SolidJS store 更新

## D.5 实时生效路径

CSS var 设置路径（< 100ms）:
1. UI onChange → invoke("settings_update", { bgOpacity: val })
2. Rust 侧写 KV → 返回 AppSettings + emit "settings_changed"
3. 前端 SolidJS store 更新 → applyCssVars()
4. document.documentElement.style.setProperty("--bg-opacity", val)

## D.6 持久化

- Rust 侧 app_settings KV 表 · INSERT ... ON CONFLICT DO UPDATE
- 重启后 settings_get → 读 KV · fallback default

## G.3 H2 regression proof

- 临时将 AppSettings.bg_opacity 改为 background_opacity
- cargo build → FAIL: `struct AppSettings has no field named bg_opacity`
- pnpm typecheck → FAIL: `Property 'bgOpacity' does not exist on type 'AppSettings'`
- Rust 编译 + TypeScript 类型检查双验证

## Benchmark 回归检查

- git_status bench: 无回归（~55µs · 与 baseline 持平）