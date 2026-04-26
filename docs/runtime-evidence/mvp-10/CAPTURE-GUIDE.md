# MVP-10 Phase B · Runtime Evidence Capture Guide

> **目标**：用户（Arbiter）本地 30 min 采集 4 张截图 · 闭合 §F runtime evidence。
> **为什么不能 CI 自动化**：03/04 涉及 DevTools network panel + 无 DSN 时 0 outbound 验证 · 必须 GUI + 人眼。

## 前置

```bash
# 1. clean DB（让 telemetry_opt_in 回到 NULL · 触发首次启动 modal）
rm -f ~/Library/Application\ Support/com.vibestation.app/vibestation.db

# 2. 启动 dev
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation
pnpm tauri:dev
```

## 截图 4 张（命名 + 内容）

### `01-settings-panel.png`（设置面板 4 分组打开）

1. 启动 app · 关闭 telemetry modal（accept 或 decline 都行）
2. 创建 workspace 或打开已有
3. 打开 Preferences（菜单 / `Cmd+,`）
4. **要求**：4 个分组（Appearance / Terminal / Git / Privacy）全部 expanded
5. **§C.4 验证点**：Privacy 分组里能看到 "Collection endpoint" 字段 + Copy 按钮（disabled 灰显，因为 DSN 未配置）
6. `Cmd+Shift+4` 区域截屏到 `docs/runtime-evidence/mvp-10/01-settings-panel.png`

### `02-settings-realtime.png`（改 theme 实时生效）

1. Preferences → Appearance · 把 theme 从 dark 切到 light（或反向）
2. **要求**：app 主体（非 modal）背景色立即变化 · 无需重启
3. 截屏需同时显示：(a) 切换前后对比（split screenshot · 或两次截屏拼接）· 或 (b) 单张但 settings drawer + 主面板同框可见已切换
4. 路径 `02-settings-realtime.png`

### `03-telemetry-opt-in.png`（首次启动 opt-in modal · 阻塞欢迎页）

1. 完全退出 app · 删除 DB（见前置）
2. 重启 `pnpm tauri:dev`
3. **要求**：app 启动后立即弹 telemetry opt-in modal · 全屏 overlay 阻塞 WelcomePage
4. 验证点：(a) 标题 "Help improve Vibestation" · (b) 双栏 We collect / We never collect · (c) Decline + Accept 双按钮
5. 路径 `03-telemetry-opt-in.png`

### `04-telemetry-decline.png`（decline 后 0 outbound proof）

1. 在 03 步骤的 modal 上点 **Decline**
2. 打开 DevTools（webview 右键 → Inspect Element · 或 macOS `View → Open DevTools`）→ Network panel
3. 触发一个 panic（最简单方法：crates/app 加临时 `panic!("test")` · 或在 dev menu 触发已有 crash · 或不触发但保留 idle）
4. **要求**：Network panel 显示 0 个 outbound 请求到 sentry.io / ingest.sentry.io / 任何 telemetry endpoint
5. 截屏含：(a) Network panel filter "ingest" 或 "sentry" · 0 请求 · (b) Settings → Privacy 显示 Telemetry: Disabled
6. 路径 `04-telemetry-decline.png`

## 体积约束（ADR-011 R4）

- 单张推荐 ≤ 500KB · 上限 1MB
- macOS `screencapture` 默认 PNG 通常 200-800KB · 体积过大用 `sips -s format png -s formatOptions 60`

## 完成后

1. `git add docs/runtime-evidence/mvp-10/0[1-4]-*.png`
2. `git commit -m "docs(mvp-10): §F runtime evidence 4 张截图"` · 含 trailer
3. 推到 follow-up PR 分支（或新独立 PR）

## 体积验证

```bash
ls -lh docs/runtime-evidence/mvp-10/*.png
# 累计应 ≤ 4MB（4 × 1MB 上限）
```

## 相关

- spec §runtime evidence 段（line 270-279 of MVP-10 spec）
- ADR-011 R1-R5：runtime evidence 路径 + 体积 + PR body 引用
- 既有先例：`docs/runtime-evidence/mvp-02/0[1-3]-*.jpg` · `docs/runtime-evidence/mvp-03/`
