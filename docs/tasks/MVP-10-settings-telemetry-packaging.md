---
id: MVP-10
type: mvp
title: 设置面板 + Telemetry opt-in + 打包发布（v0.1 GA）
status: draft
owner:
phase: W11-W12
depends_on: ["MVP-01", "MVP-02", "MVP-03", "MVP-04", "MVP-05", "MVP-06", "MVP-07", "MVP-08", "MVP-09"]
blocks: []
blocked_by: []
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1 · §10.4（非功能）· §5.1（Telemetry）· §10.2（打包大小）
risk_ref: R30
reviewer:
---

# MVP-10: 设置 + Telemetry + 打包发布

> **状态**：`draft`
> **依赖**：所有 MVP-01..09（发布前收尾）
> **v0.1 GA 硬门槛**

---

## 🎯 目标（Goal）

完成 v0.1 发布前的最后一个 MVP：设置面板、Telemetry 首次启动 opt-in 对话框、macOS 公证、Linux AppImage 签名、README/CHANGELOG/SECURITY 就位。

## 📖 背景（Context）

- `CLAUDE.md` #10（A 栏）：Telemetry = **默认关闭 + 首次启动弹 opt-in**（匿名 crash + 版本号，GDPR/CCPA 合规）
- `§10.4 非功能`：LICENSE / NOTICE / CONTRIBUTING / CoC / CHANGELOG / SECURITY / privacy policy 全部就位
- `§9 R30`：Telemetry 隐私合规（默认关 + opt-in）

---

## 🎨 功能范围（Scope）

**Do**：
- 设置面板（Settings app window 或 drawer）：
  - 外观：theme（light/dark/auto）+ font family / size
  - 终端：default shell + pasta 保护 toggle
  - Git：user.name / user.email（从 git config 读取 + 可改）
  - 隐私：Telemetry opt-in toggle + "查看收集内容"链接
- 首次启动 Telemetry opt-in 对话框（MVP-01 启动后、欢迎页前）
- 对话框内容：
  - 收集什么（匿名 crash + 版本号 + OS type 三项，无 IP、无个人内容、无仓库路径）
  - 不收集什么（强调）
  - 接受 / 拒绝 按钮（等宽）+ "Learn more" 链接到 privacy policy
- 用户决策持久化到 rusqlite `app_settings`
- 打包发布：
  - macOS 公证（notarization）+ stapling
  - Linux AppImage + sha256 + GPG 签名（可选）
  - 版本号 `0.1.0`
- 非功能文件：
  - README.md（双语简版 + 对外文案禁区合规）
  - CONTRIBUTING.md
  - CODE_OF_CONDUCT.md（Contributor Covenant 2.1）
  - CHANGELOG.md（Keep a Changelog）
  - SECURITY.md（报告邮箱）
  - privacy-policy.md

**Don't**：
- Telemetry 服务端（收集端点由 CI 期间 Phase 4 做）
- Auto-update 服务端（Tauri plugin 已集成但 update manifest 服务端 v0.2+）
- Windows 打包（v0.4）
- ARM Linux（v0.2）

## 🖼 UI 引用

- 设置面板：参考原型的 modal / drawer（Calm Studio 风格）
- Telemetry 对话框：顶部 icon + 清晰文字 + 两个等宽按钮（拒绝在左，接受在右）

## ✅ Acceptance

### A. 设置面板

- [ ] 菜单 / 快捷键 `⌘,` 打开设置
- [ ] 4 个分组：外观 / 终端 / Git / 隐私
- [ ] 所有改动实时生效（无需重启）
- [ ] 持久化到 rusqlite `app_settings`

### B. Telemetry opt-in 对话框

- [ ] 首次启动（rusqlite 无 telemetry 决策）弹对话框，阻塞欢迎页
- [ ] 对话框列出：收集项 + 不收集项 + 可改设置（设置 → 隐私）
- [ ] 用户选择后写入 rusqlite `telemetry_opt_in: bool`
- [ ] 再次启动不再弹（decision persisted）
- [ ] 设置里改 toggle 立即生效

### C. Telemetry 实际行为

- [ ] opt-in = false：**不发送任何遥测**（包括 crash report）
- [ ] opt-in = true：发送匿名 crash + 版本号 + OS type
- [ ] crash report 不含：IP / 用户文件路径 / commit 信息 / 终端内容
- [ ] 收集端点 URL 在设置里公开显示

### D. macOS 打包

- [ ] `pnpm tauri build` 在 macOS 产出 signed + notarized .dmg
- [ ] 公证通过（Apple Notary Service）
- [ ] Stapling 完成（dmg 可离线校验）
- [ ] Gatekeeper 开启的干净 mac 上可直接打开（无"无法验证开发者"提示）

### E. Linux 打包

- [ ] AppImage 产出
- [ ] sha256 校验和同时上传
- [ ] Ubuntu 24 Wayland + X11 都可启动
- [ ] （可选）GPG 签名 AppImage

### F. 非功能文件

- [ ] `README.md` 双语（英/中），首屏即懂能做什么；**不提 AI-Aware / Mission Control**（禁区）
- [ ] `CONTRIBUTING.md` 说明 PR 流程 + 代码风格
- [ ] `CODE_OF_CONDUCT.md` Contributor Covenant 2.1
- [ ] `CHANGELOG.md` Keep a Changelog 格式，v0.1.0 条目完整
- [ ] `SECURITY.md` 有效的安全报告邮箱
- [ ] `privacy-policy.md` 公开 + 设置里链接

### G. GitHub Release

- [ ] v0.1.0 tag + Release 页面
- [ ] 上传：mac dmg（x86_64 + aarch64）+ Linux AppImage（x86_64 + aarch64）+ sha256.txt
- [ ] Release notes 来自 CHANGELOG.md

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | Telemetry payload 脱敏 + 设置持久化 |
| 集成 | 设置变更 → rusqlite 写入 → 重启恢复 |
| E2E | 完整首次启动流程（包括 Telemetry 对话框）|
| 手动 QA | 三平台打包验证 + notarization 实机测试 |

## 💾 数据模型变更

扩展 `app_settings`：
```
telemetry_opt_in: Option<bool>     // None = 未决策，弹对话框
paste_protection: bool = true
default_shell: String
```

## ⚠️ 已知风险

- **R30 Telemetry 合规**：GDPR/CCPA 要求默认关 + 透明收集项 + 用户可撤回 → 本 spec 覆盖
- **Apple Developer Program 审批时间**：SPIKE-06 已在 W0 申请，v0.1 发布（W12）时必须已通过；若未通过 → unsigned dmg（有警告，非 block）
- **Notarization 失败常见原因**：entitlements 配置不全 / 代码引用不合规 API → 需要提前 W11 测试通过

## 📝 Notes

- Telemetry 使用 `sentry` 或等价开源方案，收集端点 URL 在 Phase 4 CI workflow 阶段确定（可能用 Plausible self-hosted 或 PostHog free tier）
- MVP-10 的 "privacy-policy.md" **必须过法律 / 合规检查**（即使是个人项目，GDPR 要求清楚声明）

## 🔗 相关

- `CLAUDE.md` #10 · #1（LICENSE）· 对外文案禁区
- `implementation-plan.md` §10.1 · §10.2 · §10.4 · §5.1 · §9 R30
- SPIKE-06 Apple Developer Program 申请
- 上游：MVP-01..09 全部
- 下游：v0.2 push/pull/auto-update

---

**自审四问**：
1. 递归完备性：设置 / Telemetry / 打包 / 非功能 4 类全覆盖 ✅
2. 反向场景：Notarization 失败 / Dev Program 未批 都有 fallback ✅
3. 边界适用性：三平台打包 + GDPR/CCPA 合规明确 ✅
4. YAGNI：auto-update 服务端 / Windows / ARM Linux 都推后 ✅
