# MVP-10 §F.04 · Pre-Capture 就绪体检

> **定位**：本文件是 MVP-10 [`CAPTURE-GUIDE.md`](./CAPTURE-GUIDE.md) §F.04（telemetry decline 0 outbound · DevTools network panel）的**前置体检**——主 agent（CLI）能程序化验证的代码侧前置已全跑，结论固化在此。
> **它不是 capture 本身**：§F.04 DevTools Network 面板验证 telemetry decline 后 0 outbound 设计上就是 Arbiter 本人通过 `pnpm tauri:dev` 实跑 + 开 DevTools Network tab + 在 telemetry opt-in dialog 选 "拒绝" + 观察 Network 面板 0 Sentry / 0 outbound 请求，CLI agent 无 webview 能力，不能替代。
> **用途**：Arbiter 跑 §F.04 capture 时（5 min）· 先读本文件 —— 代码侧已 green 的 cargo 测试 + §F.01/02/03 三个已捕证据不必重复跑 / 重新验。
>
> 体检执行：Claude Code 主 agent · 2026-05-20 · 镜像 MVP-19 #384 模式

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| 项                                    | 验证方式                                                       | 结果                                                                                                                                                                                                                                                                                                                                             |
| ------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **app_settings 单元测试**             | `cargo test -p vibestation-core --lib 'app_settings::tests::'` | **9 passed · 0 failed · 0 ignored**（含 set/get roundtrip · update 仅写 provided fields · pool reopen 持久化 · 多 key 独立 · external_term 字段 + pty_pool 字段 roundtrip）                                                                                                                                                                      |
| **Phase A 设置面板 4 分组 done**      | PR #114                                                        | 外观 / 终端 / Git / 隐私 4 分组 SolidJS 组件 + AppSettings KV store + ⌘, 快捷键                                                                                                                                                                                                                                                                  |
| **Phase B Sentry 集成 + opt-in done** | PR #152 / #155 / #158 / #161 / #163                            | ADR-015 accepted · Sentry SDK 编码（`default_integrations: false` + PII SHA-256 hash + before_send 删 trace · 19 测试全过）· §C.4 endpoint UI · §G.4 H2 proof · §B.1 modal mount-time click guard（critical bug fix · webview 启动 race · 200ms guard）· §F.02 theme dual-path fix（ThemeProvider listen settings_changed event · 实时生效闭环） |
| **Phase D Linux AppImage done**       | PR #174                                                        | 7.61 MB AppImage（< 80 MB 余量 10.5×）+ sha256 + Ubuntu 24.04.4 LTS GNOME on X11 启动验证（1920×1080 / 135 KB 截图）                                                                                                                                                                                                                             |
| **既有 §F evidence 3/4 done**         | `ls docs/runtime-evidence/mvp-10/`                             | 4 截图（01-settings-panel · 02-settings-realtime + 02-after-light · 03-telemetry-opt-in）+ CAPTURE-GUIDE.md + h2-regression-proof.{log,md}（§G.4 done）+ phase-d/（Linux AppImage 3 件 evidence）+ sentry-spike/（3 截图 + 3 log + cargo-bloat-install + command-log + README）                                                                  |

### §F evidence 3/4 已 done 详情

| §F    | 项                                                       | 状态  | 文件                                                                |
| ----- | -------------------------------------------------------- | ----- | ------------------------------------------------------------------- |
| §F.01 | 设置面板基本布局 + ⌘, 快捷键                             | ✅    | `01-settings-panel.png`                                             |
| §F.02 | 设置实时生效（主题切换 light/dark）                      | ✅    | `02-settings-realtime.png` + `02-settings-realtime-after-light.png` |
| §F.03 | Telemetry opt-in 对话框首次启动                          | ✅    | `03-telemetry-opt-in.png`                                           |
| §F.04 | Telemetry decline 后 0 outbound（DevTools Network 验证） | 🟡 待 | **缺**（本 PRE-CAPTURE-READINESS 主目标）                           |

---

## ⚠️ 关键 gap 预警

### gap-1 · §F.04 DevTools Network panel 验证未捕获

**坐实**：`ls docs/runtime-evidence/mvp-10/` = 缺 `04-*.png` · CAPTURE-GUIDE.md §F.04 待验。

**影响**：spec §F.04（telemetry decline 后 0 outbound · DevTools network panel 验证）是 MVP-10 § F evidence 4/4 最后一项 · 阻塞 v0.1 GA 完整 audit trail（虽然 v0.1.0/v0.1.1 已 ship · §F.04 是 v0.1 完整收尾 evidence）。

**不是 gap 是 deferred**：spec §I 明确「§F.04 telemetry decline 0 outbound · DevTools network panel 验证 · 需 Arbiter 本地 5 min capture」· 短 capture 窗口（5 min · 仅需开 DevTools + 验 0 Sentry 请求）。

### gap-2 · Phase E 非功能 + v0.1.0 tag 已发布（独立窗口）

**坐实**：v0.1.0 / v0.1.1 已 ship · 但 spec Phase E 当前索引仍标 `⏳ todo`（spec 与现实索引滞后）。

**影响**：非阻塞 · spec status 翻 done 等 §F.04 + Phase E 同步翻转。

### Phase C deferred to v0.2（不是 gap · 已决策）

spec §I 明确：**Apple Dev Program $99/y + 2-2 周审批不阻塞 v0.1 alpha** · v0.1 改 unsigned 模式 + README Gatekeeper bypass 指引（`xattr -cr /Applications/Vibestation.app`）· Phase C 公证 / notarization 推 v0.2 · 触发条件：「README 反馈"装不上"超 5 次 / 公开 landing page 上线 / macOS 用户基础超 100 任一即触发」。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 [`CAPTURE-GUIDE.md`](./CAPTURE-GUIDE.md) §F.04：

1. **清掉本地 telemetry_opt_in 决策**（让 dialog 重新弹）：
   - macOS：`rm -f "$HOME/Library/Application Support/com.vibestation.app/vibestation.db"` 或 SQL `DELETE FROM app_settings WHERE key='telemetry_opt_in'`
2. **`pnpm tauri:dev`** 启动应用 · webview 启动 ready · WelcomePage 被 modal 阻塞
3. **Cmd+Opt+I** 开 DevTools · 切到 **Network** tab · 清空 logs · enable "Preserve log"
4. **在 telemetry opt-in modal 点 "拒绝"**
5. **观察 DevTools Network panel**：
   - **预期**：0 Sentry 域请求（`sentry.io` / `*.ingest.sentry.io` 全无 outbound）· 0 outbound HTTP 到任何非 localhost 域
   - **不应**：有任何 `POST /api/{project_id}/envelope/` 请求
6. **截图**：DevTools Network 面板 + telemetry opt-in 状态 显示 "拒绝" → `docs/runtime-evidence/mvp-10/04-decline-0-outbound.png`
7. **PR + R1-R5**：单文件 ≤ 500 KB · PR body Test Plan 必含「§F.04 evidence 已补齐 · `docs/runtime-evidence/mvp-10/04-decline-0-outbound.png`」

### gap-2 / Phase E 同步翻转（可同 PR 做）

如果 Arbiter 5 min capture §F.04 后顺手补 Phase E status：

- spec frontmatter 维持 `ready`（capture 视角）· 但 README 表 + CLAUDE.md 状态字段同步更新「§F.04 evidence 补齐 + Phase E v0.1.0 GA 已发」

---

## 结论

MVP-10 §F.04 验收项中：

- **代码侧 app_settings 9 passed · Phase A/B/D 全 done · §F.01/02/03 evidence 3/4 已捕**（仅 §F.04 缺）✅
- **§F.04 DevTools Network panel 5 min capture**（Arbiter 本地 · CLI 无 webview）🔴
- **Phase C deferred to v0.2**（决策已锁 · 不是 gap）✅
- **Phase E v0.1.0 已 ship 但 spec/README 标 ⏳ todo**（独立同步 · 不阻塞 §F.04）🟡

MVP-10 spec 维持 `ready`（§F.04 + Phase E 同步翻转后 spec status 翻 done）。

**关联**：spec [`docs/tasks/MVP-10-settings-telemetry-packaging.md`](../../tasks/MVP-10-settings-telemetry-packaging.md) §F.04 + Phase E · [`CAPTURE-GUIDE.md`](./CAPTURE-GUIDE.md) §F.04 · ADR-015 telemetry stack sentry · PR #114 Phase A / #152/#155/#158/#161/#163 Phase B / #174 Phase D · `.claude/rules/runtime-evidence-location.md` R1-R5 · 模板镜像 [MVP-19 PRE-CAPTURE-READINESS](../mvp-19/PRE-CAPTURE-READINESS.md) PR #384
