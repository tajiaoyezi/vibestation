# Security Policy

> **English summary**: Report vulnerabilities via GitHub Security Advisory (preferred) or email `security@vibestation.dev`. We aim to respond within 72 hours and coordinate disclosure within 90 days.

Vibestation 团队感谢安全研究人员、用户、贡献者发现并负责任地披露漏洞。本文档描述 Vibestation 项目的安全报告流程、响应时限和披露原则。

## 受支持版本（Supported Versions）

Vibestation 当前为 **pre-release 状态**（v0.1.0-alpha 已发布 macOS-only · v0.1.0 GA 双平台准备中）。安全修复仅向最新主线版本提供。

| 版本 | 状态 | 安全更新 |
|---|---|---|
| 主线 / `main` 分支 | 开发中 | ✅ 提供 |
| v0.1.0-alpha（macOS） | pre-release | 🟡 best-effort（推用户升级到 v0.1.0 GA） |
| < v0.1.0 | 不存在 | — |

GA 发布后（v0.1.0+）本表会更新为标准 N / N-1 支持策略。

## 报告漏洞（Reporting a Vulnerability）

**请勿在 public GitHub Issue 报告安全漏洞。**

### 首选：GitHub Security Advisory（推荐）

1. 访问 [Vibestation GitHub Security Advisory](https://github.com/tajiaoyezi/vibestation/security/advisories/new)
2. 填写标题 + 描述 + 影响范围 + 复现步骤
3. 仓库 maintainer 收到通知后启动私有协作（不公开 · 修复后再 disclose）

### 备选：邮件

发送至 `security@vibestation.dev`（占位 · 域名注册后替换 · 见 `docs/PROGRESS.md` "v0.1 GA 发布策略"）。邮件请包含：

- 漏洞类型（XSS / 路径遍历 / 命令注入 / 信息泄漏 / 加密问题 等）
- 影响范围（哪个 platform / 哪个 component · macOS / Linux · Tauri / IPC / SolidJS / git2）
- 复现步骤（具体到可执行代码 / 命令 · 含 vibestation 版本号 + OS 版本）
- 概念验证（PoC · 可选 · 但更易快速 triage）
- 您希望的署名（披露文档里 · 默认匿名）

### 报告内容建议（提高 triage 速度）

- 影响**哪个版本**（main / v0.1.0-alpha · git commit hash 优先）
- 是否需要**特定权限**（普通用户 / 本地用户 / 已 compromise 的 PTY child）
- 攻击**是否需要用户交互**（自动 vs 用户必须 click）
- 数据**泄漏 / 篡改 / 拒绝服务** 哪类影响

## 响应时限（Response Timeline）

| 阶段 | 目标时限 |
|---|---|
| 初次响应（确认收到 + 分配编号） | 72 小时 |
| Triage（确认是否 bug · 严重度评级） | 7 天 |
| 修复开发 | 30 天（critical / high）· 90 天（medium / low） |
| Coordinated disclosure | 修复 release 后立即公开 advisory · 或最长 90 天后强制 disclosure |

实际时限受 maintainer 可用性影响（pre-release 项目 · 单人维护）· 我们承诺在初次响应内告知您实际预期时间。

## 严重度评级（Severity Classification）

参考 [CVSS 3.1](https://www.first.org/cvss/) 评级：

| 严重度 | CVSS 范围 | 示例 | 处理优先级 |
|---|---|---|---|
| **Critical** | 9.0 - 10.0 | RCE · 任意文件读写 · 完整身份接管 | 立即 |
| **High** | 7.0 - 8.9 | 本地权限提升 · 绕过 sandbox · 信息泄漏（PII） | 30 天 |
| **Medium** | 4.0 - 6.9 | DoS · stored XSS（仅 webview 内） · 限制场景下信息泄漏 | 90 天 |
| **Low** | 0.1 - 3.9 | 错误处理不当 · 非敏感信息泄漏 · 弱默认 | 下个 minor release |

## 协调披露（Coordinated Disclosure）

我们采用 **coordinated disclosure** 模型：

1. 收到报告 · 私下协作修复（不在 public 分支提交带漏洞描述的 commit · 不在 PR 提描述）
2. 修复发布到 release 版本后 · 在 [GitHub Security Advisory](https://github.com/tajiaoyezi/vibestation/security/advisories) 公开 advisory · 含 CVE（如分配）
3. CHANGELOG.md `Security` 段记录修复 · 不含详细 PoC（避免攻击者快速复制）
4. 默认 90 天后强制公开（即使报告者不 ready）· 防 zero-day 长期未修

### 致谢（Hall of Fame）

修复发布的 advisory 会在 [GitHub Security Advisory](https://github.com/tajiaoyezi/vibestation/security/advisories) 致谢报告者（除非您选择匿名）。Vibestation 是 Apache 2.0 开源项目 · 不提供 bug bounty · 但对每个负责任披露表示真诚感谢。

## 不在范围（Out of Scope）

以下问题**不视为安全漏洞**（请走普通 issue / discussion）：

- 第三方依赖的已公开 CVE（请 PR 升级依赖 · 走 `dependabot`）
- 用户自配 telemetry endpoint（`VIBESTATION_SENTRY_DSN`）的 typo / 错配
- 操作系统层面的漏洞（macOS / Linux 内核 · 报给厂商）
- 浏览器内 webview engine 漏洞（报给 wry / Tauri 上游）
- 用户 git hooks（pre-commit）执行任意代码 · 这是 git 的 by-design 行为
- v0.1 阶段 missing security headers / 弱默认（pre-release · 未达 GA）

## 加密 / 数据保护承诺

参考 [`docs/adr/ADR-015-telemetry-stack-sentry.md`](docs/adr/ADR-015-telemetry-stack-sentry.md) 详述 telemetry payload 隐私设计：

- Telemetry **默认关闭** · 首次启动 modal 显式 opt-in（[MVP-10 §B.1](docs/tasks/MVP-10-settings-telemetry-packaging.md)）
- Crash payload **不包含**：IP / 用户文件路径 / commit hash / terminal content / repo 名
- Crash payload **包含**：粗粒度 OS（macos / linux）+ app version + SHA-256 panic hash（不可逆）
- 用户可在 Preferences → Privacy 任意时刻撤回（实时生效 · 不需重启 · [MVP-10 §F.02](docs/tasks/MVP-10-settings-telemetry-packaging.md)）

详细的隐私政策见 [`privacy-policy.md`](privacy-policy.md)。

## 关联

- [Apache License 2.0](LICENSE) · 软件许可
- [`privacy-policy.md`](privacy-policy.md) · 隐私政策（GDPR Article 13 合规）
- [`docs/adr/ADR-015-telemetry-stack-sentry.md`](docs/adr/ADR-015-telemetry-stack-sentry.md) · Telemetry 技术决策（Sentry SDK + 隐私约束）
- [`docs/tasks/MVP-10-settings-telemetry-packaging.md`](docs/tasks/MVP-10-settings-telemetry-packaging.md) · v0.1 GA 发布 task

---

**最后更新**：2026-04-26（session 20 · MVP-10 Phase E 非功能文件交付）
