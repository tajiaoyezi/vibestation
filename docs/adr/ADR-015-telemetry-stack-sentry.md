# ADR-015: Telemetry crash stack = Sentry SDK + sanitized payload

**状态**：proposed
**日期**：2026-04-25
**决策者**：Codex CLI（Spike 作者）· reviewer 待定 · tajiaoyezi（Arbiter 待拍板）
**对应 `CLAUDE.md` 决策表**：#10（Telemetry 默认关闭 + opt-in 的实施子决策）
**对应 Spike**：[MVP-10 §H.1.1](../tasks/MVP-10-settings-telemetry-packaging.md)

---

## 背景与问题（Context and Problem Statement）

`CLAUDE.md` #10 已锁定 Telemetry 必须默认关闭、首次启动 opt-in、只收集匿名 crash + 版本号并满足 GDPR/CCPA 合规。MVP-10 Phase B 还缺一个实现层决策：使用成熟 crash SDK，还是自建 HTTP endpoint。

不先锁定技术栈会导致 Phase B 同时做 UI、持久化、PII 脱敏和 SDK 选型，评审面过大。MVP-10 spec 因此要求 Phase B 编码前先做 30 min Spike，并输出本 ADR。

## 决策驱动因素（Decision Drivers）

- **D1 · 隐私约束优先**：默认不初始化遥测；用户 opt-in 后也不能发送仓库路径、终端内容、IP、commit 信息或原始 panic 文本。
- **D2 · crash 场景成熟度**：v0.1 只需要 crash report，不需要产品 analytics。
- **D3 · 自托管 / 数据主权**：未来可使用自托管 endpoint，避免被闭源云服务锁死。
- **D4 · 依赖和体积可控**：新增 SDK 对 release artifact 的影响必须可量化，且不得威胁 AppImage < 80 MB 目标。
- **D5 · 可回退**：若 endpoint、隐私或体积风险失控，能退回自建 HTTP POST。

## 考虑的选项（Considered Options）

### 选项 A · `sentry` crate 0.47.0

Rust 原生 SDK，crash 生态成熟，支持 self-hosted Sentry。Spike 验证了本地集成、PII 白名单 payload、`default_integrations = false` 下的事件形态，以及 release example 的 cargo-bloat 结果。

### 选项 B · 自建 HTTP POST

依赖最少、数据主权最强，但需要自建收集端、聚合 UI、符号化和 retry/backoff 行为。对 v0.1 来说工作量明显超过 crash opt-in 本身。

### 选项 C · Plausible self-hosted

偏 analytics，不是 crash-first 工具。即使自托管，仍需自行补 crash payload、去重和错误聚合。

### 选项 D · PostHog free tier

功能完整但偏产品 analytics，默认是云端数据出域；对当前 crash-only 需求过重。

## 决策（Decision Outcome）

**选择（proposed）**：选项 A · `sentry` crate 0.47.0，但只作为 opt-in crash transport 使用。

**拟定配置约束**：

- 只有 `telemetry_opt_in == true` 且 DSN 存在时才初始化 Sentry；opt-out 或缺 DSN 时不初始化，视为 telemetry disabled。
- DSN 来自环境变量 / GitHub Actions secret / 本地未提交配置，禁止写入仓库。
- `ClientOptions` 必须显式设置：
  - `default_integrations: false`
  - `send_default_pii: false`
  - `release: Some("vibestation@<version>")`
  - `environment` 显式来自 app setting 或构建环境
  - `before_send` 保留为最终防线，只允许白名单字段通过
- Vibestation 先构造 `CrashReportPayload { version, os_type, stack_trace_hash }`，再上报；禁止把原始 panic 字符串、终端内容、repo path 或 git metadata 传入 SDK。
- MVP-10 Phase B 可以先实现 SDK 初始化 + mock/test transport；真实 endpoint 实收事件必须在 Phase B done 前或 Phase C release gate 前补测。

## Spike 证据

证据目录：[docs/runtime-evidence/mvp-10/sentry-spike](../runtime-evidence/mvp-10/sentry-spike/README.md)

| 步骤              | 结果                                                                                                                                                                                                          |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Step 1 · SDK 集成 | `cargo add sentry` + `cargo build`/example 编译通过；本轮无 DSN/Auth Token，未验证 Web UI 实收事件                                                                                                            |
| Step 2 · PII 脱敏 | 4 个测试通过；payload 只保留 `version` / `os_type` / `stack_trace_hash`；`default_integrations = false` 时捕获事件无路径、终端内容、IP、commit 信息                                                           |
| Step 3 · 体积     | `cargo bloat --release --crates -n 30 --package vibestation-core` 对 `sentry_smoke` example 显示 `.text` 1.8 MiB、file size 3.2 MiB；Sentry/transport 依赖可接受，但最终 AppImage/dmg 仍需 release build 复测 |
| Step 5 · 清理     | `cargo remove sentry` 后恢复 `Cargo.toml` / `Cargo.lock`；本 ADR 不把 SDK 依赖带入正式代码                                                                                                                    |

本地源码确认：

- `sentry-core` `ClientOptions::default()` 中 `send_default_pii` 默认为 `false`，`default_integrations` 默认为 `true`。
- `sentry` `apply_defaults` 会在 `default_integrations == true` 时加入 backtrace、debug images、contexts、panic、process stacktrace 等集成。
- 结论：正式实现必须显式关闭 default integrations，并由 Vibestation 自己控制白名单 payload。

## 后果（Consequences）

### 正面

- 用成熟 SDK 处理事件 envelope、transport、release/environment 元数据，避免 v0.1 自建 crash 后端。
- 可 self-hosted，符合数据主权要求。
- SDK 未初始化即可天然满足 opt-out 0 发送。
- `before_send` + `CrashReportPayload` 双层白名单能把 PII 控制放在 Rust 侧。

### 负面

- 默认 feature 临时集成拉入 81 个依赖，含 `reqwest` / `hyper` / `tokio` / TLS 相关包；Phase B 正式引入时需要审查 feature set。
- 本轮没有真实 DSN，未验证 Sentry Web UI 实收事件。
- `default_integrations` 默认开启，未来维护者若删掉显式配置会重新打开 PII 风险面。

### 风险

- **R1 · 默认集成被误开**：用单元测试断言 `default_integrations = false` 且 event JSON 不含路径 / IP / terminal content；在代码旁保留短注释。
- **R2 · DSN 泄漏**：DSN 只走 secret/env/local config；secret scan 必须覆盖。
- **R3 · sentry.io 数据出域**：若 Arbiter 不接受云端出域，Phase B 只允许 self-hosted endpoint；否则 fallback 到自建 HTTP POST。
- **R4 · bundle 体积超预算**：Phase B 引入依赖后必须对最终 Tauri artifact 复测；若增量不可接受，先裁剪 features，再 fallback 自建 HTTP POST。
- **R5 · endpoint 未实测**：Phase B done 前补一条真实 endpoint smoke；没有凭证时记录为 release blocker，而不是默默通过。

## 与 `implementation-plan.md` 的映射

- 对应章节：§5.1 Telemetry、§10.4 非功能、§14.4 telemetry / crash 合规
- 对应风险：R30 崩溃上报 / telemetry 合规

## 相关（Links）

- `CLAUDE.md` 决策表：#10
- Task：[MVP-10 设置面板 + Telemetry opt-in + 打包发布](../tasks/MVP-10-settings-telemetry-packaging.md)
- Runtime evidence：[docs/runtime-evidence/mvp-10/sentry-spike](../runtime-evidence/mvp-10/sentry-spike/README.md)
- Local SDK source checked:
  - `/Users/leaf/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sentry-core-0.47.0/src/clientoptions.rs`
  - `/Users/leaf/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sentry-0.47.0/src/defaults.rs`

---

**修订历史**：

- 2026-04-25 · 初版 proposed · Codex CLI
