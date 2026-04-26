# Session 20 · 2026-04-26

**session**: 20
**date**: 2026-04-26
**pr_range**: #152-#169（19 PR · 主 agent 主线代码侧 100% 收官）
**theme**: MVP-10 Phase B 完整闭环（Sentry SDK + opt-in modal + endpoint UI + theme realtime）· 2 critical/secondary bug discovered/fixed（modal mount-time webview 虚假 click + theme dual-path）· dispatch §2.13/§2.14 教训规则化 · CLI 自动化 GUI 截图能力边界确立

---

## 主题摘要（按主题维度组织 · 19 PR）

### 1 · MVP-10 Phase B Sentry SDK 完整闭环（11 PR · session 20 核心主线）

ADR-015 accepted 后启动 SDK 编码 · 2 critical/secondary bug discovered/fixed · v0.1 GA blocker 解锁。

- **PR #152**：ADR-015 Telemetry crash stack proposed → accepted by Arbiter（解锁 SDK 编码 · 主 agent · session 20 入口）
- **PR #155**：Phase B SDK 主体（B5 任务 · 主 agent · 4 commits）· `crates/core/src/telemetry.rs` 268 行（ADR-015 §决策约束 · `default_integrations: false` + `send_default_pii: false` + `before_send` 删 contexts.trace · SHA-256 panic hash 防 PII）· `crates/core/tests/telemetry_pii_test.rs` 6 PII 测试 · IPC 3 commands（`telemetry_opt_in_set` / `telemetry_status_get` / `app_version_get`）+ permission · `install_panic_hook` + `try_init_sentry_from_env` · `TelemetryOptInModal` 阻塞 WelcomePage · §B.4 atomic 双门控（19 测试全过）
- **PR #158**：Phase B follow-up · §C.4 收集端点 host UI（`SENTRY_ENDPOINT_HOST` OnceLock + Copy 按钮 + "Not configured" fallback）+ §G.4 H2 regression proof（临时改 `font_family` ts(rename) → tsc FAIL 6 处 · annotation rollback）+ §F runtime evidence CAPTURE-GUIDE.md（4 张 GUI 截图采集步骤）+ `dispatch-prompt-template.md §2.13`（PR #157 round 1 教训规则化）+ 顺手修 PR #156 留下的 2 clippy errors
- **PR #159 🔴 隐藏 critical UX bug fix**：MVP-09 Phase C 错误流 UX 补强 · 发现 19 个 vs-commit-* / vs-toast-* / vs-dialog-* CSS class **完全无定义**（grep 主 styles.css 0 命中）· dialog 在 dev mode **裸 HTML 显示** · 严重 UI degradation · reviewer 当时只看 Rust 测试 + IPC contract · 漏掉 dev mode 启动验证 · 新建 `web/src/panels/CommitBar/styles.css` 363 行（Calm Studio token + scale-in/slide-up 动画）+ Hook stderr "Copy" 按钮 + exit code 显示
- **PR #161 🔴 CRITICAL BUG FIX · v0.1 GA blocker**：Modal mount-time webview 虚假 click guard · 实测发现 `telemetry_opt_in` 启动 12.5 秒内被自动写入 · modal **用户完全看不见** · 5 轮 dev restart + DB watcher 半秒 poll 调试定位 webview 启动 race · WKWebView 把"启动 ready"事件误派发到 modal 内第一个 focusable button（Decline）· SolidJS event delegation 路由到 onClick · 触发虚假 `decide(false)` · 加 200ms `MOUNT_CLICK_GUARD_MS` · 真用户不可能 < 200ms 完成有意点击 · spec §B.1 "首次启动弹对话框 · 阻塞欢迎页" 完全失效 · 顺手交付 §F.01 settings-panel + §F.03 telemetry-opt-in modal 截图（247 KB / 324 KB）
- **PR #162**：§F.02 partial · dark theme single state（349 KB · settings-realtime）· capture 期间发现 secondary dual-path bug · CLI 自动化 GUI 截图能力边界 inventory（`screencapture -l <CGWindowID>` + Swift `CGWindowListCopyWindowInfo` + `cliclick` + `osascript` 能做 vs 不能做）
- **PR #163 🟡 SECONDARY BUG FIX**：theme dual-path · `ThemeSwitch.theme_set` IPC **不 emit `settings_changed`** · status bar click DB 写但 UI 不刷 · violate spec §F.02 "切 theme 后实时生效 · 无重启" · 修复 3 file：(a) `theme.tsx` 加 `listen("settings_changed")` 监听 settings store 推送 + 删 `theme_set` IPC 调用 + onMount 用 `settings_get` 替代 `theme_get`（避免双 IPC 漂移）· (b) `ThemeSwitch.tsx` `handleClick` 双门控调用 setTheme（同步 UI active）+ updateSettings（持久化 + emit · settings_changed event 回环让 ThemeProvider 自动同步）· (c) `AppearanceGroup.tsx` 删除 redundant `themeCtx.setTheme(theme)` 调用 · §F.02 split before/after evidence（02-settings-realtime-after-light.png 381 KB · light theme + Theme radio Light active · 完整 UI 切）

涉及 PR 总计 7 个（#152 / #155 / #158 / #161 / #162 / #163 + #155 衍生 #159 是 MVP-09 Phase C 但 dual review 漏属同根因）。

### 2 · 多 agent 协作收尾 + Linux 平台 baseline 补强（4 PR · 入口任务）

session 19 末未完成的入口任务在 session 20 早期一波收官。

- **PR #153**：session-history/session-19.md 归档（36 PR · 168 行 · 9 主题分组 · MVP-11 全 done + MVP-05 Phase A/B/C + ADR-006 Ubuntu validated + branch protect 机械化）· 主 agent
- **PR #154**：PROGRESS.md M-2 滚动窗口同步 · session 19 移交至 session-19.md 归档 + session 20 入口段建立 · Ubuntu Claude（D2 任务）
- **PR #156**：MVP-09 §D Criterion bench + §E 集成测试（B4 任务 · Ubuntu Claude · `crates/core/tests/git_ops_integration.rs` · Linux 基线证据 · stage 0.26ms / commit 0.35ms / stage_1k 31.5ms · 远低于 spec 性能要求 380×/1400×/63× 余量）
- **PR #157**：ADR README 索引同步 + 决策表 #10 行（U2 任务 · Ubuntu Kimi · 含 round 2 字节级 self-fix · round 1 Kimi 误覆盖 ADR-015 PR #152 措辞 · round 2 用 `git checkout origin/main -- <file>` 字节级恢复）

### 3 · 教训规则化制度化（2 PR · session 20 重要 deliverable）

session 20 三连 bug + Kimi 误覆盖 round 1 共 4 个事件 · 全部规则化沉淀。

- **PR #158** 内含 `dispatch-prompt-template.md §2.13`：索引同步类 prompt 禁 inline 已被其他 PR 改过的源文件 · 必须 `git checkout origin/main -- <file>`（PR #157 round 1 教训）
- **PR #164** 单 PR 完整新增 §2.14：Reviewer 必须启 dev 模式跑 critical UX path（PR #159 / #161 / #163 三连共同根因）· 触发条件 + 6 步强制做法 + 4 禁止做法 + 4 反模式 · 是全局 rule 15「CI 绿 ≠ runtime 过」在 reviewer 阶段的具体落地 · 与 §2.3 implementer 责任双管齐下

### 4 · 状态文档全面同步（5 PR · session 末整理）

主 agent 主线 100% 收官后 · 把所有 stale 状态描述对齐实际进度 · 让下次 agent 看到精确状态。

- **PR #160**：PROGRESS session 20 mid sync · 补 PR #154-#159 · 8 PR 同步 + MVP v0.1 进度 phase 级别详化
- **PR #165**：PROGRESS session 20 late sync · 补 PR #160-#164 · 14 PR 总同步 + "主 agent 主线代码侧 100% 收官 + 2 critical/secondary bug discovered/fixed"
- **PR #166**：`docs/tasks/README.md` MVP-09/10 状态描述 · 加 session 20 共 7 个相关 PR reference
- **PR #167**：MVP-10 spec phase 表 · Phase A/B 🟡 → ✅ done · 加 5 PR reference（#152/#155/#158/#161/#163）+ Phase C/D/E 精确阻塞条件
- **PR #168**：MVP-09 spec phase 表 · "本 PR" → 准确 PR reference（#116 / #118）· Phase C/D 状态从 ⏳ todo → ✅ done / 🟡 性能 done · 加 PR #156/#159 reference
- **PR #169**：MVP-08 + MVP-05 spec phase 表 minor fix · "本 PR" → PR #112（fs watch）· `[#147e](.../pull/)` 不存在 → PR #151（MVP-05 §F 仪表化）

### 5 · 双 critical/secondary bug 调试方法论沉淀

session 20 最大经验产出。

#### Bug 1（CRITICAL · v0.1 GA blocker · PR #161）

**症状**：删 DB 重启 dev mode · `telemetry_opt_in` 在 12.5 秒内被自动写入 · modal **用户完全看不见**。

**调试**（5 轮 dev restart + DB watcher 半秒 poll）：

1. T+0~12s: DB 0 rows（modal mount 之前）
2. T+12s: telemetry_opt_in 突然 = false / true · DB 1 row
3. backend 加 DIAG `eprintln!` 显示 `telemetry_opt_in_set` IPC 被调（仅 1 次）
4. App.tsx `<Show when={!telemetryDecided()}>` 整体 disable · IPC 0 调用 · 确认源在 modal 内
5. button onClick 改 NOOP（保留 Decline）· IPC 仍触发 · DB 写 **false** · → Decline 被自动 click（modal 第一个 focusable element）

**根因**：webview 启动 + modal mount 同帧 · macOS WKWebView 把"启动 ready"事件误派发到 modal 内第一个 focusable button · SolidJS event delegation 路由到 onClick · 触发虚假 `decide(false)`。

**修复**：200ms `MOUNT_CLICK_GUARD_MS`：

```ts
let mountedAt = 0;
onMount(() => { mountedAt = performance.now(); });
const isEarlyClick = (): boolean => {
  if (mountedAt === 0) return true;
  return performance.now() - mountedAt < 200;
};
const decide = async (optIn: boolean): Promise<void> => {
  if (submitting()) return;
  if (isEarlyClick()) return;  // 丢弃 mount 同帧 webview 虚假 click
  // ... existing logic
};
```

#### Bug 2（SECONDARY · spec §F.02 violation · PR #163）

**症状**：status bar `ThemeSwitch` click light icon · DB 写 `theme=light` 但 UI 仍 dark 渲染。

**调试**：通过 capture §F.02 split before/after 时观察到 status bar click 不实时刷 UI · 但 Prefs panel radio click 实时刷。

**根因**：双 IPC 路径分离：

| 来源 | IPC | emit settings_changed | UI 实时刷新 |
|---|---|---|---|
| Status bar `ThemeSwitch` | `theme_set` | ❌ | ❌ |
| Prefs `AppearanceGroup` radio | `settings_update` | ✓ | ✓ |

**修复**：

1. ThemeProvider 加 `listen("settings_changed")` 监听 · 同步 internal theme signal
2. ThemeSwitch.handleClick 双门控调用 setTheme（同步 UI active）+ updateSettings（持久化 + emit）
3. AppearanceGroup 删 redundant `themeCtx.setTheme(theme)` 重复调用

### 6 · CLI 自动化 GUI 截图能力边界确立

session 20 主 agent 第一次大规模做 GUI 截图采集 · 实测能力边界 · 写入 PR #162 + LAST-SESSION-STATE-2026-04-26.md：

#### ✅ 能做

- macOS `screencapture -x -l <CGWindowID>` 静默截屏
- Swift `CGWindowListCopyWindowInfo` 拿 CGWindowID（不依赖 AppleScript · 后者 -1728 错误）
- AppleScript click app menu items（Vibestation > "Settings…"）
- cliclick 像素 click 大 button（Accept · X close · radio button via sweep）
- AppleScript / cliclick keystroke 全局快捷键（Cmd+T → menu accelerator）
- DB watcher 半秒粒度 detect 写入（用于 bug 调试 · `sqlite3 SELECT` poll）

#### ❌ 不能（精度 / 路径问题）

- 小 radio button 精确 click（受 retina + window shadow + 多 radio adjacent 影响 · sweep 易命中相邻）
- webview 内部组件 keyboard shortcut（⌘\ ⌘⇧\ ⌘⇧P 不 split pane · 因 webview 不接收 macOS app 级 key event）
- terminal pane shell 自动 spawn（PaneTerminal 不 mount · 即使 ⌘T 创建 tab）
- DevTools UI（network panel filter · Performance trace · console snapshot）
- 复杂实机交互（流式 LLM stream + Ctrl+C 残帧检验）

#### 核心限制

webview 内部组件的 keyboard event 路径跟 macOS app 级 key event 分离 · cliclick / osascript keystroke 只到 webview 外壳 · 不到 SolidJS 内部 handler。这是所有 webview app 通用限制。

#### 主 agent 已交付的 §F evidence

3/4 done：

- ✓ 01-settings-panel.png（247 KB · 4 分组 + §C.4 endpoint UI）
- ✓ 02-settings-realtime.png + 02-settings-realtime-after-light.png（349 KB + 381 KB · split before/after）
- ✓ 03-telemetry-opt-in.png（324 KB · modal 阻塞 WelcomePage）
- ⏳ 04-telemetry-decline.png（DevTools network 0 outbound · CLI 完全不能 · 必须 Arbiter）

---

## 特色（session 20 教训沉淀 + critical bug rescue）

### 1 · 救下 v0.1 GA blocker

如果不是为做 §F.03 telemetry modal 截图 · 主 agent 不会 fresh DB 启动 dev mode · 不会发现 modal 自动 dismiss bug。这个 bug 在 PR #155 merge 时所有测试通过 · reviewer 没启 dev mode · 用户在生产环境也不会注意（因为决策已被自动写）但 spec §B.1 隐私关键 path 完全失效。**capture screenshot 顺手 catch v0.1 GA blocker** · session 20 最大意外收获。

### 2 · 教训规则化双管齐下

session 20 三连 bug 共同根因：reviewer 把 "CI 绿 = PR 可 merge" 等同。dispatch §2.3（implementer）+ §2.14（reviewer · session 20 新增）双管齐下 · 防未来类似 webview race / dual IPC path / CSS missing 类问题。

### 3 · CLI agent 主导 + 用户授权 self-merge 模式稳态

session 20 后期 PR #160-#169（10 PR）几乎全部走 self-merge 模式：主 agent 写代码 + self-review + Arbiter approval（PR body trailer "继续工作"）+ 直接 merge。这是 v2-D.1 单人项目模式的极致 · session 19 还是 Arbiter 显式 approve · session 20 用户授权延伸到"按推荐执行"+"继续"。Audit trail 完整：每个 PR body trailer 三行齐 + 关联 commit message。

### 4 · 状态文档全面对齐

session 20 末 5 PR 状态同步（#160 / #165 / #166 / #167 / #168 / #169）让下次 agent 入门即可拿到精确状态：

- PROGRESS.md "MVP v0.1 进度" 行从过度简化（"3/10 done"）→ phase 级精确（"MVP-09 Phase A/B/C done · Phase D 性能 done · 截图待 GUI"）
- README tasks 状态行 PR reference 全列
- spec 内部 phase 表"本 PR" / `[#147e](.../pull/)` 等 stale ref 全清

### 5 · 团队进一步收缩 + 主 agent 高效产出

session 19 团队 5 → 3 人（Codex + OpenCode 离开）· session 20 实际只用：

- 主 agent（Claude Code · Opus 4.7）：13 PR
- Ubuntu Claude（远程独立电脑）：3 PR（D2 / B4 / 部分入口）
- Ubuntu Kimi（远程独立电脑）：1 PR（U2 ADR README sync）

Sub-agent 后台并发模式 session 20 未启用 · 主 agent CLI 工作高效。

### 6 · ADR 增长

15 ADR 不变 · 但 ADR-015 从 proposed → accepted 翻转（PR #152）· 解锁 MVP-10 Phase B SDK 编码。

### 7 · v2-D.1 PR body trailer 100% 合规

session 20 全部 19 PR · 三行 trailer（Implemented by / Reviewed by / Arbiter approval）齐 · 无缺失。Arbiter approval 一行内含具体用户授权语境（"开始"/"继续执行"/"直接 self-merge 模式"等）· 准确反映授权链路。

### 8 · Author 归属 100% 防御性 unset 合规

session 20 全部 19 PR 主 agent 实施部分 author 字段 = `Claude Code <noreply@anthropic.com>`· 无跨 agent 错归。session 18 末事故未在 session 19/20 复现。

---

## 遗留进入 session 21

### 主线（GUI capture · Arbiter 本地 1 小时一次性闭合）

- **MVP-04 §I 22 张截图 + 2 段 30s 录屏**（zsh/bash/fish + Claude/Codex CLI 实机 · cargo test 已 7 PASS / 15 ignore-runtime · 仅缺 GUI 录屏）
- **MVP-05 Phase D `metrics-mvp-05.md` 实测 + 4-7 张截图**（capture-phase-d.sh 已就位 · 30 min）
- **MVP-09 Phase D runtime evidence**（stage/commit 流程截图）
- **MVP-10 §F.04 0 outbound**（DevTools network panel · CLI 完全不能 · 必须 Arbiter）

完成后所有 spec phase 表全 ✅ · 仅剩 spec frontmatter status 翻 done（v2-D.1 self-review + Arbiter approval 流程）。

### off-mainline

- **MVP-10 Phase C/D/E 打包**（macOS 公证 / Linux AppImage / GitHub Release）· 等 SPIKE-06 §B Apple Dev Program approve
- **dead code cleanup**：`crates/app/src/lib.rs::theme_set` IPC handler + capability + permission（PR #163 后已无 frontend 调用方 · 推 v0.2 cleanup PR · 跨 Rust + capability + permission · 风险 > 价值 · v0.1 GA 不强制做）

### 文档

- **session-history/session-20.md 归档**（本 PR · M-2 滚动窗口规则 · session 末整理）
- **PROGRESS.md M-2 滚动**（session 20 移交至 session-20.md · session 21 入口段建立）

---

← 当前进度见 [docs/PROGRESS.md](../PROGRESS.md)
