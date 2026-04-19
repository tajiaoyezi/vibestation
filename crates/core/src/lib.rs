//! Vibestation 核心业务逻辑
//!
//! 本 crate 包含与 UI / 桌面框架无关的业务核心：workspace 管理、git 操作、PTY、AI-aware
//! pane 联动（v1.0 vision）等。保持纯 Rust · 可独立 `cargo test` · 不依赖 Tauri。
//!
//! Phase A（MVP-01）scope：仅占位 · 真实业务在 MVP-02..20 填充。

/// 应用版本号 · 从 workspace Cargo.toml 注入（运行时亦可走 Tauri API）。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Phase A 占位函数 · 作为 workspace 联通性的最小证据 · 后续 MVP 会替换。
#[must_use]
pub fn greet() -> &'static str {
    "Vibestation core online"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_returns_fixed_message() {
        assert_eq!(greet(), "Vibestation core online");
    }

    #[test]
    fn version_is_non_empty() {
        assert!(
            !VERSION.is_empty(),
            "CARGO_PKG_VERSION should be injected by Cargo"
        );
    }
}
