//! Vibestation Telemetry · ADR-015 (Sentry SDK + sanitized payload)
//!
//! MVP-10 Phase B 实施。配置约束按 ADR-015 §决策：
//! - `default_integrations: false`（关 backtrace / debug images / process / panic / contexts 默认集成）
//! - `send_default_pii: false`（默认 false · 显式锁定）
//! - `release: vibestation@<version>`（CARGO_PKG_VERSION 编译时注入）
//! - `environment` 显式（"production" / "dev"）· 不 fallback 默认值
//! - `before_send` 删除 `event.contexts.trace`（R-trace 风险 · session profiling）
//!
//! 仅当 `telemetry_opt_in == true` 且 DSN 存在时才 [`init_sentry`]。
//! [`CrashReportPayload`] 仅含 3 字段（version / os_type / stack_trace_hash）·
//! 原始 panic 字符串经 SHA-256 不可逆哈希 · 不会泄漏 PII。

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::OnceLock;
use ts_rs::TS;

/// 全局 Sentry guard · drop 时 flush events · 必须长生命周期（应用整个生命周期）
static SENTRY_GUARD: OnceLock<sentry::ClientInitGuard> = OnceLock::new();

/// 收集端点 host（DSN 的 host 部分 · 不含 public key / project id · §C.4 公开显示）
///
/// 仅在 [`init_sentry`] 成功时填入；`None` 表示 DSN 未配置（telemetry 未真正启用）。
static SENTRY_ENDPOINT_HOST: OnceLock<String> = OnceLock::new();

/// Runtime opt-in 状态（panic hook + IPC 共享）
///
/// 编码：`-1` = 未决策（首次启动 · NULL）· `0` = opt-out · `1` = opt-in。
/// 使用 atomic 而非 Mutex · panic hook 不能 block。
static TELEMETRY_OPT_IN_STATE: AtomicI8 = AtomicI8::new(-1);

/// Crash report 上报 payload · §G.2 锁定 3 字段（白名单防御）
///
/// # 隐私约束（ADR-015 D1）
///
/// 不含 IP / 用户路径 / commit 信息 / 终端内容 / 仓库名。
/// 不含原始 panic 字符串（用 SHA-256 哈希替代 · 不可逆）。
/// 字段白名单由本 struct 定义 + ts-rs 强制 + §C.3.1 unit test 守门。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CrashReportPayload {
    pub version: String,
    pub os_type: String,
    pub stack_trace_hash: String,
}

/// Telemetry opt-in 决策请求（前端 → Rust）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryOptInRequest {
    pub opt_in: bool,
}

/// Telemetry 状态（用于 settings 面板显示）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryStatus {
    /// 当前 opt-in 决策 · `null` = 未决策（首次启动）
    pub opt_in: Option<bool>,
    /// 收集端点描述（DSN 不暴露 · 仅显示自托管 / cloud 二选一）
    pub endpoint_host: String,
    /// 数据收集摘要（用于 Privacy 面板 "View what we collect"）
    pub data_collection_summary: String,
    /// SDK 是否已初始化（仅 opt_in == true 且 DSN 存在时为 true）
    pub initialized: bool,
}

/// 应用版本信息（用于 Privacy 面板调试展示 + crash payload 数据源）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AppVersionInfo {
    pub version: String,
    pub os_type: String,
    pub build_target: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Sentry DSN invalid: {0}")]
    DsnInvalid(String),
    #[error("Telemetry SDK already initialized")]
    AlreadyInitialized,
}

/// 当前应用版本（CARGO_PKG_VERSION 编译时注入）
fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// 操作系统类型 · 粗粒度（防 fingerprint）· 不暴露具体 kernel 版本 / distro
fn os_type() -> String {
    match env::consts::OS {
        "macos" => "macos".to_string(),
        "linux" => "linux".to_string(),
        other => format!("unknown:{other}"),
    }
}

/// 构建目标（编译时由 Cargo 注入 TARGET · 例 `aarch64-apple-darwin`）
fn build_target() -> &'static str {
    option_env!("TARGET").unwrap_or("unknown")
}

/// 从 panic 信息派生 SHA-256 哈希（不可逆 · 防 PII 泄漏）
///
/// 输出 64 字符 hex（SHA-256 = 32 bytes）。
fn hash_stack_trace(panic_info: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(panic_info.as_bytes());
    hasher
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut s, b| {
            use std::fmt::Write;
            write!(s, "{b:02x}").unwrap();
            s
        })
}

/// 从 panic 信息派生 [`CrashReportPayload`]（脱敏 · 仅 3 字段白名单）
///
/// # PII 脱敏保证（ADR-015 R1）
///
/// 即使 `panic_info` 含 PII（用户路径 / commit / IP / 终端内容 / 仓库名）·
/// 输出 payload 仅含：粗粒度 OS + 应用版本 + SHA-256 哈希 · 不会暴露原始内容。
///
/// # 测试
///
/// `crates/core/tests/telemetry_pii_test.rs` 覆盖 3 类 PII 边界（spec §C.3.1）：
/// - 用户路径 + commit hash + IP
/// - 终端命令内容
/// - Git 仓库路径
pub fn capture_panic(panic_info: &str) -> CrashReportPayload {
    CrashReportPayload {
        version: current_version().to_string(),
        os_type: os_type(),
        stack_trace_hash: hash_stack_trace(panic_info),
    }
}

/// 构造 [`TelemetryStatus`]（settings 面板显示用）
///
/// `endpoint_host` 来源（按优先级）：
/// 1. [`init_sentry`] 成功时从 DSN 解析的 host（如 `o123456.ingest.sentry.io`）
/// 2. 未配置 DSN → `"Not configured"`（spec §C.4 · 用户能看到当前未发任何数据）
pub fn build_status(opt_in: Option<bool>) -> TelemetryStatus {
    TelemetryStatus {
        opt_in,
        endpoint_host: SENTRY_ENDPOINT_HOST
            .get()
            .cloned()
            .unwrap_or_else(|| "Not configured".to_string()),
        data_collection_summary: data_collection_summary().to_string(),
        initialized: is_initialized(),
    }
}

/// 构造 [`AppVersionInfo`]（debug 用 + crash payload 字段同源）
pub fn build_version_info() -> AppVersionInfo {
    AppVersionInfo {
        version: current_version().to_string(),
        os_type: os_type(),
        build_target: build_target().to_string(),
    }
}

/// 数据收集摘要 · 公开字符串 · 跟 Privacy 面板 UI 同步
pub fn data_collection_summary() -> &'static str {
    "Anonymous crash hash + app version + OS type. \
     We do not collect: IP, file paths, terminal content, commit messages, or repository names."
}

/// SDK 是否已初始化（仅 opt_in == true 且 DSN 存在时为 true）
pub fn is_initialized() -> bool {
    SENTRY_GUARD.get().is_some()
}

/// 同步 runtime opt-in 状态（panic hook + IPC 共享 · §B.4 实时生效）
pub fn set_runtime_opt_in(value: Option<bool>) {
    let encoded: i8 = match value {
        None => -1,
        Some(false) => 0,
        Some(true) => 1,
    };
    TELEMETRY_OPT_IN_STATE.store(encoded, Ordering::Relaxed);
}

/// 读取 runtime opt-in 状态
pub fn runtime_opt_in() -> Option<bool> {
    match TELEMETRY_OPT_IN_STATE.load(Ordering::Relaxed) {
        1 => Some(true),
        0 => Some(false),
        _ => None,
    }
}

/// 是否应该发送 telemetry · panic hook + `capture_crash_report` 内部用
///
/// 双门控：必须 `opt_in == Some(true)` 且 SDK 已 init（DSN 存在）。
/// §B.4 acceptance · opt-in == false 时立即停止发送（当前 session 已排队的 crash flush 后不再新增）。
pub fn should_send_telemetry() -> bool {
    runtime_opt_in() == Some(true) && is_initialized()
}

/// 初始化 Sentry SDK（按 ADR-015 §决策约束）
///
/// 仅在 `opt_in == true` 且 `dsn` 非空时调用。多次调用幂等
/// （第二次返回 [`TelemetryError::AlreadyInitialized`]）。
///
/// # 隐私约束（ADR-015 §决策）
///
/// - `default_integrations: false`（关 backtrace / debug images / process / panic / contexts 默认集成）
/// - `send_default_pii: false`（默认 false · 显式锁定）
/// - `release: vibestation@<version>`
/// - `environment` 显式
/// - `before_send` 删除 `event.contexts.trace`（R-trace · pseudonymous session profiling 风险 · v0.2 自托管时可重新评估）
///
/// # Errors
///
/// - [`TelemetryError::DsnInvalid`] 若 DSN 格式无效
/// - [`TelemetryError::AlreadyInitialized`] 若已初始化（防重复 init）
pub fn init_sentry(dsn: &str, environment: &str) -> Result<(), TelemetryError> {
    if is_initialized() {
        return Err(TelemetryError::AlreadyInitialized);
    }

    let parsed_dsn: sentry::types::Dsn = dsn
        .parse()
        .map_err(|e: sentry::types::ParseDsnError| TelemetryError::DsnInvalid(e.to_string()))?;

    // §C.4 · 仅暴露 host（不含 public_key / project_id · DSN secret 部分不进 IPC）
    let _ = SENTRY_ENDPOINT_HOST.set(parsed_dsn.host().to_string());

    let options = sentry::ClientOptions {
        dsn: Some(parsed_dsn),
        release: Some(format!("vibestation@{}", current_version()).into()),
        environment: Some(environment.to_string().into()),
        send_default_pii: false,
        default_integrations: false,
        before_send: Some(std::sync::Arc::new(|mut event| {
            // ADR-015 R-trace · 删除 contexts.trace（span_id / trace_id ·
            // pseudonymous session profiling 风险 · v0.2 自托管时可重新评估）
            event.contexts.remove("trace");
            Some(event)
        })),
        ..Default::default()
    };

    let guard = sentry::init(options);
    SENTRY_GUARD
        .set(guard)
        .map_err(|_| TelemetryError::AlreadyInitialized)?;
    Ok(())
}

/// 上报 crash report（仅 opt-in == true 且 SDK 已初始化时生效 · 否则 no-op）
///
/// 使用 sentry::capture_message · 仅含脱敏 payload（version / os_type / hash）。
/// 双门控：opt-in == false 或 DSN 缺失 → no-op · 0 网络请求。
/// §B.4 acceptance · 用户 opt-out 后立即停止发送（即使 SDK 已 init）。
pub fn capture_crash_report(payload: &CrashReportPayload) {
    if !should_send_telemetry() {
        return;
    }

    sentry::configure_scope(|scope| {
        scope.set_tag("os_type", &payload.os_type);
        scope.set_tag("version", &payload.version);
        scope.set_tag("stack_trace_hash", &payload.stack_trace_hash);
    });
    sentry::capture_message(
        &format!("crash:{}", payload.stack_trace_hash),
        sentry::Level::Error,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_panic_returns_three_fields_only() {
        let payload = capture_panic("simple panic message");
        // 只能有 3 字段（编译时由 struct 锁 · runtime 双保险）
        let json = serde_json::to_value(&payload).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 3);
        assert!(obj.contains_key("version"));
        assert!(obj.contains_key("osType"));
        assert!(obj.contains_key("stackTraceHash"));
    }

    #[test]
    fn capture_panic_hash_is_sha256_hex() {
        let payload = capture_panic("anything");
        // SHA-256 = 32 bytes = 64 hex chars
        assert_eq!(payload.stack_trace_hash.len(), 64);
        assert!(payload
            .stack_trace_hash
            .chars()
            .all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn capture_panic_hash_is_deterministic() {
        let p1 = capture_panic("same input");
        let p2 = capture_panic("same input");
        assert_eq!(p1.stack_trace_hash, p2.stack_trace_hash);
    }

    #[test]
    fn capture_panic_hash_differs_per_input() {
        let p1 = capture_panic("input one");
        let p2 = capture_panic("input two");
        assert_ne!(p1.stack_trace_hash, p2.stack_trace_hash);
    }

    #[test]
    fn os_type_returns_known_platforms() {
        let os = os_type();
        // 只允许 "macos" / "linux" / "unknown:..." · 防 fingerprint
        assert!(
            os == "macos" || os == "linux" || os.starts_with("unknown:"),
            "unexpected os_type: {os}"
        );
    }

    #[test]
    fn version_matches_cargo_pkg_version() {
        assert_eq!(current_version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn build_status_reports_initialized_false_by_default() {
        // 测试环境 SDK 未 init · initialized 必须 false
        let status = build_status(Some(true));
        assert!(!status.initialized);
        assert_eq!(status.opt_in, Some(true));
    }

    #[test]
    fn build_status_includes_summary() {
        let status = build_status(None);
        assert!(status.data_collection_summary.contains("Anonymous"));
        assert!(status.data_collection_summary.contains("not collect"));
    }

    #[test]
    fn build_status_endpoint_host_defaults_to_not_configured() {
        // §C.4 · DSN 未配置时 endpoint_host == "Not configured"
        // （SENTRY_ENDPOINT_HOST 仅 init_sentry 成功时填入 · 测试环境无 DSN）
        let status = build_status(Some(true));
        assert_eq!(status.endpoint_host, "Not configured");
    }

    #[test]
    fn capture_crash_report_no_op_when_not_initialized() {
        // SDK 未 init · 调 capture_crash_report 应 no-op（不 panic / 不发送）
        let payload = capture_panic("test panic");
        capture_crash_report(&payload);
        // 验证未初始化（默认值）
        assert!(!is_initialized());
    }
}
