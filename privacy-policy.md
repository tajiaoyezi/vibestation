# Vibestation Privacy Policy

> **Effective date**: 2026-04-26（v0.1.0-alpha 起 · v0.1 GA 发布前可能 minor 调整）
> **Version**: 1.0（initial · GDPR Article 13 minimum 合规）
> **Plain English summary**: Vibestation collects nothing by default. If you opt in, we collect anonymized crash hashes (not the stack trace itself), the app version, and your OS type (macOS or Linux) — that's it. You can opt out anytime in Preferences → Privacy.

本隐私政策按 [GDPR Article 13](https://gdpr-info.eu/art-13-gdpr/) 最小要求编写。Vibestation 是开源项目（[Apache License 2.0](LICENSE)）· 不收集营销数据 · 不卖数据给第三方。本政策只描述 telemetry 功能（默认关闭）的数据处理细节。

---

## 1 · 控制者身份与联系方式（Identity of the Controller）

**控制者（Data Controller）**：Vibestation 项目维护者

- **GitHub 仓库**：[github.com/tajiaoyezi/vibestation](https://github.com/tajiaoyezi/vibestation)（Apache 2.0 开源 · 单人维护 · pre-release 阶段）
- **联系**：通过 GitHub Issue / Discussion · 安全相关走 [SECURITY.md](SECURITY.md) 私有 advisory 流程
- **邮箱**：`privacy@vibestation.dev`（占位 · 域名注册后替换）

GA 发布后（v0.1.0+）会更新为正式控制者信息（如组织化 · 非个人）。

## 2 · 收集的个人数据类别（Categories of Personal Data）

**默认状态**：**不收集任何数据**。Vibestation 安装后 telemetry **opt-in modal 阻塞欢迎页** · 只有用户显式选择 "Accept" 才会启用 telemetry。即使启用 · 收集范围严格限制如下：

### 启用 telemetry（opt-in = true）后收集

| 字段 | 类型 | 示例 | 来源 |
|---|---|---|---|
| `version` | string | `"0.1.0"` | `CARGO_PKG_VERSION` 编译时注入 |
| `os_type` | string | `"macos"` / `"linux"` | `std::env::consts::OS`（粗粒度 · 不暴露 kernel 版本 / distro） |
| `stack_trace_hash` | string | `"a3b8f2..."` (SHA-256 hex) | 从 panic message SHA-256 哈希（不可逆 · 用于聚合相同崩溃） |

**仅此 3 字段**。技术约束在 [`docs/adr/ADR-015-telemetry-stack-sentry.md`](docs/adr/ADR-015-telemetry-stack-sentry.md) §决策 锁定 · payload struct 由 [ts-rs](https://github.com/Aleph-Alpha/ts-rs) 编译时强制（[`crates/core/src/telemetry.rs::CrashReportPayload`](crates/core/src/telemetry.rs)）。

### 永远不收集（即使 opt-in）

- ❌ IP 地址（Sentry SDK `send_default_pii: false` + `default_integrations: false` 关闭所有 PII 默认集成）
- ❌ 用户文件路径（如 `/Users/alice/secret/file.txt`）
- ❌ Git commit hash / commit message / repo 名 / branch 名
- ❌ 终端命令内容（用户 PTY 输入 / 输出）
- ❌ Workspace 路径 / Tab title / scrollback 历史
- ❌ 任何**可识别**到个人的数据（PII）

**技术保证**：原始 panic 字符串经 SHA-256 哈希后只保留 64 位 hex digest · 不可逆推回原始 stack trace。Sentry SDK `before_send` 回调显式删除 `event.contexts.trace`（防 pseudonymous session profiling）。详见 [ADR-015 §决策 R-trace](docs/adr/ADR-015-telemetry-stack-sentry.md)。

## 3 · 收集目的与法律基础（Purposes and Legal Basis）

### 目的

唯一目的：**改进 Vibestation 稳定性**。匿名 crash hash 让我们：

1. 发现高频崩溃模式（同 hash 出现多次 → 优先修复）
2. 关联崩溃到特定 OS / version（识别 platform-specific 问题）
3. **不**用于：营销 / 广告 / 用户行为追踪 / A/B 实验 / 第三方共享

### 法律基础（GDPR）

**用户同意（Consent · GDPR Article 6(1)(a)）**。首次启动通过 telemetry opt-in modal 获取明确 + 自由 + 知情 + 具体的同意：

- **明确**：用户必须 click "Accept" 按钮（modal 阻塞欢迎页 · 不能跳过）
- **自由**：拒绝（"Decline"）后所有功能正常使用 · 无 degradation
- **知情**：modal 详列收集项 + 不收集项 + 链接到本政策
- **具体**：仅同意 telemetry crash report · 不包含其他数据处理

详细 modal UX 见 [MVP-10 spec §B.1 + §C](docs/tasks/MVP-10-settings-telemetry-packaging.md)。

## 4 · 数据保留期（Retention Period）

| 数据 | 保留期 | 说明 |
|---|---|---|
| Crash payload | 90 天 | Sentry 默认保留期 · 90 天后自动删除（Sentry 后台聚合 · 个体记录不保留）|
| 聚合统计 | 不限期 | 例 "macOS 0.1.0 panic 频率" · 已脱敏 · 无法关联个人 |

用户随时可通过 Preferences → Privacy toggle 撤回同意 · 撤回**立即生效**（[spec §B.4](docs/tasks/MVP-10-settings-telemetry-packaging.md)）· 但**已发送到 Sentry 的历史 payload** 因已脱敏（hash · 无 PII）· 无法定向删除单个用户记录。这是技术 design choice · 与隐私优先原则一致（既然没 PII · 也无需保留个人删除能力）。

## 5 · 第三方接收方（Recipients）

### Sentry（Crash Reporting Provider）

启用 telemetry 后 · payload 发送到 [Sentry](https://sentry.io)：

- **托管模式选项**：
  - **Self-hosted**（推荐）：用户 / 组织自建 Sentry 实例 · 数据不出域 · `VIBESTATION_SENTRY_DSN` 指向自建 endpoint
  - **Cloud**（sentry.io）：Sentry 公司托管（位于 US / EU 区域 · 取决于 endpoint）· 受 Sentry [Privacy Policy](https://sentry.io/privacy/) 约束
- **Endpoint 公开**：用户可在 Preferences → Privacy 查看 + 复制当前 telemetry endpoint host（`Collection endpoint` 字段）· 透明可审计
- **DSN 不通过 IPC 传递**：仅 backend 从 `VIBESTATION_SENTRY_DSN` 环境变量读取 · 防 frontend 泄漏 secret

### 不分享给

- ❌ 广告平台（Google Ads / Meta / 等）
- ❌ Analytics 平台（GA / Mixpanel / 等）
- ❌ AI 训练数据（OpenAI / Anthropic / 任何 LLM 厂商）
- ❌ 数据中介 / 数据经纪商
- ❌ 任何第三方营销 / sales / 业务运营公司

## 6 · 用户权利（Your Rights · GDPR Article 15-22）

EU / EEA / UK 用户享有以下权利（其他地区用户视当地法规 · 如 CCPA / PIPL）：

| 权利 | 实现方式 |
|---|---|
| **知情权（Right to be informed）** | 本政策 + telemetry opt-in modal 详列收集项 |
| **访问权（Right of access）** | Sentry web UI 查询（如自托管）· 或邮件 `privacy@vibestation.dev` 申请 export |
| **更正权（Right to rectification）** | 因数据已脱敏（hash · 无 PII）· 无法定向更正个人记录（设计上不存在个人数据） |
| **删除权 / 被遗忘权（Right to erasure）** | toggle off telemetry 阻止未来发送 · 历史 hash 90 天后自动删除（无法定向删除个体记录 · 因无 PII 关联） |
| **限制处理权（Right to restrict processing）** | toggle off telemetry · 立即生效 |
| **可携带权（Right to portability）** | 邮件 `privacy@vibestation.dev` 申请 export（pre-release 阶段 · best-effort）|
| **反对权（Right to object）** | toggle off telemetry · 默认关闭即所有用户都已 "对自动处理反对" |
| **撤回同意权（Right to withdraw consent）** | toggle off telemetry · 立即生效 · 不需重启（[MVP-10 §B.4 spec](docs/tasks/MVP-10-settings-telemetry-packaging.md)） |
| **投诉权（Right to lodge complaint）** | 联系当地 supervisory authority（如德国 BfDI / 英国 ICO / 法国 CNIL）|

## 7 · Cookies / Local Storage / 类似技术

Vibestation 是**桌面应用**（Tauri 2 · 非 web） · **不使用 cookies**。本地存储仅用于：

| 用途 | 路径 | 保留 |
|---|---|---|
| 用户设置（theme / font / git identity / **telemetry opt-in 状态**） | rusqlite DB at `app_local_data_dir()` | 永久（直到用户卸载） |
| Workspace 列表 / Tab scrollback | rusqlite DB 同上 | 永久（直到用户清理） |
| Vite dev mode（仅开发者本地）| LocalStorage | 仅 dev mode |

**用户随时可清除**：删除 DB 文件即可重置（macOS：`~/Library/Application Support/com.vibestation.app/vibestation.db` · Linux：`~/.local/share/com.vibestation.app/vibestation.db`）。删除后下次启动会重新 trigger telemetry opt-in modal。

## 8 · 政策变更（Changes to This Policy）

实质性变更（新增收集字段 / 改第三方接收方 / 改保留期）会：

1. 更新本文件 · `Effective date` 标记新日期
2. CHANGELOG.md `Security` 段记录
3. （v0.2+）应用启动时弹 notice · 用户必须 acknowledge

minor 编辑（typo / 链接修复）不通知。Git history 完整保留所有版本（[github.com/tajiaoyezi/vibestation/commits/main/privacy-policy.md](https://github.com/tajiaoyezi/vibestation/commits/main/privacy-policy.md)）。

## 9 · 联系方式

- **隐私问题 / GDPR 请求**：`privacy@vibestation.dev`（占位 · 域名注册后替换）
- **安全漏洞**：见 [SECURITY.md](SECURITY.md)（GitHub Security Advisory 优先 · 邮件次之）
- **一般 bug / feature**：[GitHub Issue](https://github.com/tajiaoyezi/vibestation/issues)
- **行为准则违规**：见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) §Enforcement

---

**最后更新**：2026-04-26（v1.0 · session 20 · MVP-10 Phase E 非功能文件交付）

**关联文档**：[ADR-015 Telemetry Stack](docs/adr/ADR-015-telemetry-stack-sentry.md) · [SECURITY.md](SECURITY.md) · [LICENSE](LICENSE) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
