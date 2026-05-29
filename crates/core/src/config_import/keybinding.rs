//! Keybinding canonical form 算法 + Vibestation 内置冲突检测
//!
//! spec §H.3 锁定：`Modifier 按 Cmd > Ctrl > Alt > Shift 排序 + 大写 Key`
//!
//! 例：
//!   - `⌘T` → `Cmd+T`
//!   - `Cmd+T` → `Cmd+T`
//!   - `cmd+t` → `Cmd+T`
//!   - `Meta+Shift+t` → `Cmd+Shift+T`（Meta 视作 Cmd 别名）
//!   - `Command|Shift+t` → `Cmd+Shift+T`（Alacritty pipe 风格 mods）
//!
//! 内置冲突来源：`crates/app/src/menu.rs` 的 `MenuItem::with_id(..., Some("Cmd+X"))`
//! 当前内置（v0.1）：macOS = Cmd+, / Cmd+T / Cmd+W / Cmd+D / Cmd+Shift+D ·
//! Windows/Linux = Ctrl+, / Ctrl+T / Ctrl+W / Ctrl+D / Ctrl+Shift+D（task-3.3）
//!
//! task-3.3（平台感知）：主修饰键（cmd/command/meta/super/win/windows/⌘）分类为
//! 中性 [`TokenKind::PrimaryMod`]，canonical 落地时按 [`KeyPlatform`] 决定 `Cmd`（Mac）
//! 或 `Ctrl`（Windows/Linux）。`tokenize`/`canonicalize_key` 算法本身平台无关，不变。

/// 平台标识（内部枚举 · 不 ts-rs 导出）· 决定主修饰键 canonical 名。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyPlatform {
    /// macOS：主修饰键 canonical 为 `Cmd`（与 Ctrl 是两个独立修饰键）。
    ///
    /// 运行期仅在 macOS 构造（[`KeyPlatform::current`]）· 非 macOS 编译时仅测试构造
    /// （`_for(KeyPlatform::Mac)` 锁 Mac 分支零回归）· 故非 macOS 非测试视角 dead ·
    /// cfg-gate `allow(dead_code)` 抑制（语义：跨平台双分支变体 · 非真死代码）。
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Mac,
    /// Windows + Linux：主修饰键 canonical 为 `Ctrl`（无独立 Cmd 键）。
    Other,
}

impl KeyPlatform {
    /// 当前编译目标平台。
    #[must_use]
    pub(crate) fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            KeyPlatform::Mac
        }
        #[cfg(not(target_os = "macos"))]
        {
            KeyPlatform::Other
        }
    }

    /// 主修饰键的 canonical 名：Mac → `"Cmd"` · Other → `"Ctrl"`。
    fn primary_modifier(self) -> &'static str {
        match self {
            KeyPlatform::Mac => "Cmd",
            KeyPlatform::Other => "Ctrl",
        }
    }
}

/// canonical 化一个快捷键字符串（运行期取当前平台 · 兼容既有 caller）。
///
/// 输入容忍：`+` / `|` 分隔 modifier · 大小写不敏感 · 接受 unicode 符号（⌘ / ⌥ / ⌃ / ⇧）
///
/// 输出固定：modifier 按 `Cmd > Ctrl > Alt > Shift` 排序 · key 大写 · `+` 连接
/// （主修饰键 canonical 名按平台 · 见 [`canonicalize_keybinding_for`]）
///
/// 异常输入（空 / 全 modifier 无 key）→ 返回原始 trim 后字符串（不破坏数据）
#[must_use]
pub fn canonicalize_keybinding(input: &str) -> String {
    canonicalize_keybinding_for(input, KeyPlatform::current())
}

/// 平台参数化的 canonical 化（纯函数 · 可对两平台分别测试）。
///
/// 主修饰键（PrimaryMod）按 `platform.primary_modifier()` 落地为 `Cmd`（Mac）或
/// `Ctrl`（Other）。在 Other 平台上 PrimaryMod 与显式 Ctrl 都映射到 Ctrl，会合并
/// （`Cmd+Ctrl+T` → `Ctrl+T`）· 符合 Windows/Linux 无独立 Cmd 键的实际语义。
#[must_use]
pub(crate) fn canonicalize_keybinding_for(input: &str, platform: KeyPlatform) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 把 unicode modifier 符号替换为带 `+` 的 ASCII（输入可能是 `⌘T` · 没有 + 分隔）·
    // 替换后保证后续 split 能把 modifier 和 key 分开。⌘ → Cmd 仍归 PrimaryMod（平台中性）。
    let normalized = trimmed
        .replace('⌘', "Cmd+")
        .replace('⌃', "Ctrl+")
        .replace('⌥', "Alt+")
        .replace('⇧', "Shift+");

    // 按 `+` / `|` 分（Alacritty 用 `Command|Shift+T` · Ghostty 用 `cmd+shift+t`）
    let tokens: Vec<&str> = normalized
        .split(['+', '|'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if tokens.is_empty() {
        return trimmed.to_string();
    }

    let primary_name = platform.primary_modifier();
    let mut has_cmd = false; // PrimaryMod 落到 Mac=Cmd · Other 与 Ctrl 合并
    let mut has_ctrl = false;
    let mut has_alt = false;
    let mut has_shift = false;
    let mut key: Option<String> = None;

    for tok in tokens {
        match classify_token(tok) {
            TokenKind::PrimaryMod => match platform {
                KeyPlatform::Mac => has_cmd = true,
                KeyPlatform::Other => has_ctrl = true, // 合并进 Ctrl（无独立 Cmd）
            },
            TokenKind::Ctrl => has_ctrl = true,
            TokenKind::Alt => has_alt = true,
            TokenKind::Shift => has_shift = true,
            TokenKind::Key(k) => {
                // 多个 key 时取最后一个（容忍 `Cmd+T+U` 这种异常输入 · key=U）
                key = Some(k);
            }
        }
    }

    // 排序规则不变（spec §H.3 锁定）：主修饰键 > Ctrl > Alt > Shift ·
    // Mac 上主修饰键名 = Cmd 排最前；Other 上主修饰键已合并进 Ctrl。
    let mut parts: Vec<String> = Vec::with_capacity(5);
    if has_cmd {
        parts.push(primary_name.to_string()); // Mac only（Other 时 has_cmd 恒 false）
    }
    if has_ctrl {
        parts.push("Ctrl".to_string());
    }
    if has_alt {
        parts.push("Alt".to_string());
    }
    if has_shift {
        parts.push("Shift".to_string());
    }
    if let Some(k) = key {
        parts.push(k);
    }

    if parts.is_empty() {
        return trimmed.to_string();
    }

    parts.join("+")
}

enum TokenKind {
    /// 主修饰键（cmd/command/meta/super/win/windows/⌘）· canonical 名按平台落地。
    PrimaryMod,
    Ctrl,
    Alt,
    Shift,
    Key(String),
}

fn classify_token(tok: &str) -> TokenKind {
    let lower = tok.to_lowercase();
    match lower.as_str() {
        "cmd" | "command" | "meta" | "super" | "win" | "windows" | "⌘" => TokenKind::PrimaryMod,
        "ctrl" | "control" | "⌃" => TokenKind::Ctrl,
        "alt" | "option" | "opt" | "⌥" => TokenKind::Alt,
        "shift" | "⇧" => TokenKind::Shift,
        _ => TokenKind::Key(canonicalize_key(tok)),
    }
}

/// key 标准化：
/// - 单字符（含标点 / 单字母 / 数字 / unicode 字符）→ 大写形式
/// - F1-F12 / F13-F24 → 保留全大写
/// - 其余多字符 named key（Tab / Escape / Backspace / Enter 等）→ titlecase（首字母大写 · 其余小写）
fn canonicalize_key(tok: &str) -> String {
    if tok.chars().count() == 1 {
        return tok.to_uppercase();
    }
    // F1-F24 模式（F + 数字）保留全大写
    if is_function_key(tok) {
        return tok.to_uppercase();
    }
    // 其余 → titlecase
    let mut chars = tok.chars();
    let Some(first) = chars.next() else {
        return tok.to_string();
    };
    let mut result = first.to_uppercase().collect::<String>();
    result.push_str(&chars.as_str().to_lowercase());
    result
}

fn is_function_key(tok: &str) -> bool {
    let lower = tok.to_lowercase();
    if !lower.starts_with('f') {
        return false;
    }
    let rest = &lower[1..];
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

/// Vibestation 内置快捷键 + 对应 action（v0.1 · 来源 `crates/app/src/menu.rs`）·
/// 运行期取当前平台（兼容既有 caller）。
///
/// 任何冲突检测都基于此清单的 canonical form 比较
#[must_use]
pub fn vibestation_builtins() -> Vec<(String, &'static str)> {
    vibestation_builtins_for(KeyPlatform::current())
}

/// 平台参数化的内置快捷键表（纯函数 · 可对两平台分别测试）。
///
/// raw 表用 `Cmd+X` 表达主修饰键 · canonical 化时按 `platform` 落地为 `Cmd`（Mac）
/// 或 `Ctrl`（Other / Windows + Linux）。
#[must_use]
pub(crate) fn vibestation_builtins_for(platform: KeyPlatform) -> Vec<(String, &'static str)> {
    let raw: Vec<(&str, &str)> = vec![
        ("Cmd+,", "preferences"),
        ("Cmd+T", "tabs.create"),
        ("Cmd+W", "tabs.close"),
        ("Cmd+D", "pane.split.horizontal"),
        ("Cmd+Shift+D", "pane.split.vertical"),
    ];
    raw.into_iter()
        .map(|(k, a)| (canonicalize_keybinding_for(k, platform), a))
        .collect()
}

/// 一对冲突描述（IPC 边界外的内部表示 · 不带 user_choice 字段）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictHit {
    pub vibe_key: String,
    pub source_key: String,
    pub vibe_action: String,
    pub source_action: String,
}

/// 检测一组导入快捷键和 Vibestation 内置的冲突（运行期取当前平台 · 兼容既有 caller）
///
/// 返回的 `vibe_key` / `source_key` 都是 canonical form
///
/// 同一 canonical key 多次出现时（如配置文件多次绑定同一快捷键到不同 action）·
/// 只保留**第一个** ConflictHit · 避免 skipped_conflicts 出现重复条目（review round 5 fix）
#[must_use]
pub fn detect_conflicts(imported: &[(String, String)], // (key, action)
) -> Vec<ConflictHit> {
    detect_conflicts_for(imported, KeyPlatform::current())
}

/// 平台参数化的冲突检测（纯函数 · 可对两平台分别测试）。
///
/// 靠 [`canonicalize_keybinding_for`] + [`vibestation_builtins_for`] 的平台一致性 ——
/// Windows 上导入 `Ctrl+T`（或 `win+t`）与内置 `Ctrl+T` 正确命中（之前因强制规范化
/// 成 `Cmd` 而漏判）。
#[must_use]
pub(crate) fn detect_conflicts_for(
    imported: &[(String, String)], // (key, action)
    platform: KeyPlatform,
) -> Vec<ConflictHit> {
    use std::collections::HashMap;
    let builtins = vibestation_builtins_for(platform);
    let mut hits: HashMap<String, ConflictHit> = HashMap::new();
    for (raw_key, source_action) in imported {
        let canonical = canonicalize_keybinding_for(raw_key, platform);
        if canonical.is_empty() {
            continue;
        }
        for (vibe_canonical, vibe_action) in &builtins {
            if &canonical == vibe_canonical {
                // 仅在该 canonical key 还没记录时插入 · 防同 key 多次绑定产生重复 hit
                hits.entry(canonical.clone())
                    .or_insert_with(|| ConflictHit {
                        vibe_key: vibe_canonical.clone(),
                        source_key: canonical.clone(),
                        vibe_action: (*vibe_action).to_string(),
                        source_action: source_action.clone(),
                    });
            }
        }
    }
    hits.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // task-3.3：以下原 no-arg 用例锁 **macOS** canonical（`Cmd+...`）· 改用显式
    // `_for(KeyPlatform::Mac)` 使其在任意宿主（含 Windows CI）确定性验证 Mac 分支零回归
    // （spec §8 R1：现有 macOS 用例锁 Mac 分支不变）· 平台映射差异另由 TEST-3.3.* 覆盖。

    #[test]
    fn canonical_simple_lowercase() {
        assert_eq!(
            canonicalize_keybinding_for("cmd+t", KeyPlatform::Mac),
            "Cmd+T"
        );
    }

    #[test]
    fn canonical_unicode_modifier() {
        assert_eq!(canonicalize_keybinding_for("⌘T", KeyPlatform::Mac), "Cmd+T");
        assert_eq!(
            canonicalize_keybinding_for("⌘+t", KeyPlatform::Mac),
            "Cmd+T"
        );
    }

    #[test]
    fn canonical_meta_alias_for_cmd() {
        assert_eq!(
            canonicalize_keybinding_for("Meta+t", KeyPlatform::Mac),
            "Cmd+T"
        );
        assert_eq!(
            canonicalize_keybinding_for("super+t", KeyPlatform::Mac),
            "Cmd+T"
        );
        assert_eq!(
            canonicalize_keybinding_for("command+t", KeyPlatform::Mac),
            "Cmd+T"
        );
    }

    #[test]
    fn canonical_modifier_order_normalized() {
        // 输入顺序混乱 · 输出按 Cmd > Ctrl > Alt > Shift（Mac 分支）
        assert_eq!(
            canonicalize_keybinding_for("shift+cmd+t", KeyPlatform::Mac),
            "Cmd+Shift+T"
        );
        // 纯 Ctrl/Alt/Shift 排序平台无关 · 仍用 no-arg 验证当前宿主
        assert_eq!(
            canonicalize_keybinding("alt+ctrl+shift+a"),
            "Ctrl+Alt+Shift+A"
        );
    }

    #[test]
    fn canonical_pipe_separator_alacritty() {
        // Alacritty 用 `Command|Shift` 表达多 modifier（Mac 分支）
        assert_eq!(
            canonicalize_keybinding_for("Command|Shift+T", KeyPlatform::Mac),
            "Cmd+Shift+T"
        );
    }

    #[test]
    fn canonical_ctrl_alias() {
        // 显式 Ctrl 平台无关 · no-arg 即可
        assert_eq!(canonicalize_keybinding("control+c"), "Ctrl+C");
        assert_eq!(canonicalize_keybinding("⌃c"), "Ctrl+C");
    }

    #[test]
    fn canonical_alt_alias() {
        // Alt 平台无关
        assert_eq!(canonicalize_keybinding("option+f"), "Alt+F");
        assert_eq!(canonicalize_keybinding("⌥f"), "Alt+F");
    }

    #[test]
    fn canonical_function_key_preserved_uppercase() {
        assert_eq!(
            canonicalize_keybinding_for("cmd+F1", KeyPlatform::Mac),
            "Cmd+F1"
        );
        // F11 无 modifier · 平台无关
        assert_eq!(canonicalize_keybinding("F11"), "F11");
    }

    #[test]
    fn canonical_named_key_titlecase() {
        assert_eq!(
            canonicalize_keybinding_for("cmd+tab", KeyPlatform::Mac),
            "Cmd+Tab"
        );
        // alt/shift 平台无关
        assert_eq!(canonicalize_keybinding("alt+ESCAPE"), "Alt+Escape");
        assert_eq!(
            canonicalize_keybinding("shift+backspace"),
            "Shift+Backspace"
        );
    }

    #[test]
    fn canonical_punctuation_key() {
        // `,` 字符不变（非字母）· Mac 分支
        assert_eq!(
            canonicalize_keybinding_for("cmd+,", KeyPlatform::Mac),
            "Cmd+,"
        );
    }

    #[test]
    fn canonical_empty_input() {
        assert_eq!(canonicalize_keybinding(""), "");
        assert_eq!(canonicalize_keybinding("   "), "");
    }

    #[test]
    fn canonical_only_modifier_no_key() {
        // 只 modifier · 无 key · 仍生成 modifier 序列（Mac 分支）
        assert_eq!(
            canonicalize_keybinding_for("cmd+shift", KeyPlatform::Mac),
            "Cmd+Shift"
        );
    }

    #[test]
    fn vibe_builtins_canonicalized() {
        // Mac 分支：内置全 Cmd+X canonical（Other 分支由 TEST-3.3.2 覆盖）
        let builtins = vibestation_builtins_for(KeyPlatform::Mac);
        assert_eq!(builtins.len(), 5);
        let keys: Vec<&str> = builtins.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"Cmd+,"));
        assert!(keys.contains(&"Cmd+T"));
        assert!(keys.contains(&"Cmd+W"));
        assert!(keys.contains(&"Cmd+D"));
        assert!(keys.contains(&"Cmd+Shift+D"));
    }

    #[test]
    fn detect_conflict_cmd_t_match() {
        // Mac 分支：cmd+t 命中内置 Cmd+T（Other 分支由 TEST-3.3.3 覆盖）
        let imported = vec![("cmd+t".to_string(), "new_tab".to_string())];
        let hits = detect_conflicts_for(&imported, KeyPlatform::Mac);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].vibe_key, "Cmd+T");
        assert_eq!(hits[0].source_key, "Cmd+T");
        assert_eq!(hits[0].vibe_action, "tabs.create");
        assert_eq!(hits[0].source_action, "new_tab");
    }

    #[test]
    fn detect_conflict_cmd_shift_d() {
        // Alacritty pipe 风格命中 split.vertical
        let imported = vec![("Command|Shift+D".to_string(), "split_pane".to_string())];
        let hits = detect_conflicts(&imported);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].vibe_action, "pane.split.vertical");
    }

    #[test]
    fn detect_conflict_no_match() {
        let imported = vec![
            ("cmd+shift+p".to_string(), "command_palette".to_string()),
            ("ctrl+r".to_string(), "search".to_string()),
        ];
        let hits = detect_conflicts(&imported);
        assert!(hits.is_empty());
    }

    #[test]
    fn detect_conflict_multiple_imports_one_hit() {
        let imported = vec![
            ("cmd+shift+p".to_string(), "command_palette".to_string()),
            ("⌘T".to_string(), "spawn_tab".to_string()), // 命中 Cmd+T
            ("F11".to_string(), "fullscreen".to_string()),
        ];
        let hits = detect_conflicts(&imported);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].source_action, "spawn_tab");
    }

    #[test]
    fn detect_conflict_empty_keys_skipped() {
        let imported = vec![
            ("".to_string(), "noop".to_string()),
            ("   ".to_string(), "ws".to_string()),
        ];
        let hits = detect_conflicts(&imported);
        assert!(hits.is_empty());
    }

    // ─── task-3.3 · 平台感知主修饰键（Cmd on macOS · Ctrl on Windows/Linux）─────

    /// TEST-3.3.1（AC1）：`win`/`super`/`meta` 主修饰键按平台落地 ——
    /// Other（Windows/Linux）→ Ctrl · Mac → Cmd。
    #[test]
    fn test_3_3_1_canonicalize_primary_mod_per_platform() {
        // Other 平台：主修饰键 → Ctrl
        assert_eq!(
            canonicalize_keybinding_for("win+t", KeyPlatform::Other),
            "Ctrl+T"
        );
        assert_eq!(
            canonicalize_keybinding_for("super+t", KeyPlatform::Other),
            "Ctrl+T"
        );
        assert_eq!(
            canonicalize_keybinding_for("meta+t", KeyPlatform::Other),
            "Ctrl+T"
        );
        assert_eq!(
            canonicalize_keybinding_for("command+t", KeyPlatform::Other),
            "Ctrl+T"
        );
        // Mac 平台：主修饰键 → Cmd
        assert_eq!(
            canonicalize_keybinding_for("win+t", KeyPlatform::Mac),
            "Cmd+T"
        );
        assert_eq!(
            canonicalize_keybinding_for("meta+t", KeyPlatform::Mac),
            "Cmd+T"
        );
        assert_eq!(canonicalize_keybinding_for("⌘T", KeyPlatform::Mac), "Cmd+T");
    }

    /// TEST-3.3.2（AC2）：`vibestation_builtins_for(Other)` 全为 `Ctrl+...` ·
    /// `Mac` 全为 `Cmd+...`。
    #[test]
    fn test_3_3_2_builtins_ctrl_on_other() {
        let other = vibestation_builtins_for(KeyPlatform::Other);
        let other_keys: Vec<&str> = other.iter().map(|(k, _)| k.as_str()).collect();
        assert!(other_keys.contains(&"Ctrl+,"));
        assert!(other_keys.contains(&"Ctrl+T"));
        assert!(other_keys.contains(&"Ctrl+W"));
        assert!(other_keys.contains(&"Ctrl+D"));
        assert!(other_keys.contains(&"Ctrl+Shift+D"));
        assert!(
            other_keys.iter().all(|k| !k.contains("Cmd")),
            "Other 平台内置不应含 Cmd · 实际={other_keys:?}"
        );
        let mac = vibestation_builtins_for(KeyPlatform::Mac);
        let mac_keys: Vec<&str> = mac.iter().map(|(k, _)| k.as_str()).collect();
        assert!(mac_keys.contains(&"Cmd+T"));
        assert!(mac_keys.contains(&"Cmd+Shift+D"));
    }

    /// TEST-3.3.3（AC3）：Windows/Linux 上导入 `Ctrl+T` 与内置 `Ctrl+T` 冲突命中
    /// （之前因强制规范化成 Cmd 而漏判）。
    #[test]
    fn test_3_3_3_detect_conflicts_ctrl_t_windows() {
        let hits = detect_conflicts_for(
            &[("Ctrl+T".to_string(), "new_tab".to_string())],
            KeyPlatform::Other,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].vibe_key, "Ctrl+T");
        assert_eq!(hits[0].source_key, "Ctrl+T");
        assert_eq!(hits[0].vibe_action, "tabs.create");
        assert_eq!(hits[0].source_action, "new_tab");
    }

    /// TEST-3.3.3b（AC3）：Windows 上导入 `win+t`（终端原始）也命中内置 `Ctrl+T`。
    #[test]
    fn test_3_3_3b_detect_conflicts_win_t_windows() {
        let hits = detect_conflicts_for(
            &[("win+t".to_string(), "new_tab".to_string())],
            KeyPlatform::Other,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].vibe_key, "Ctrl+T");
        assert_eq!(hits[0].vibe_action, "tabs.create");
    }

    /// TEST-3.3.4（AC4）：macOS 语义零回归 ——
    /// `Meta+t`/`⌘T` → `Cmd+T` · `cmd+t` 命中内置 `Cmd+T`。
    #[test]
    fn test_3_3_4_macos_cmd_semantics_unchanged() {
        assert_eq!(
            canonicalize_keybinding_for("Meta+t", KeyPlatform::Mac),
            "Cmd+T"
        );
        assert_eq!(canonicalize_keybinding_for("⌘T", KeyPlatform::Mac), "Cmd+T");
        let hits = detect_conflicts_for(
            &[("cmd+t".to_string(), "new_tab".to_string())],
            KeyPlatform::Mac,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].vibe_key, "Cmd+T");
        assert_eq!(hits[0].vibe_action, "tabs.create");
    }

    /// TEST-3.3.5（AC5）：`tokenize`/`canonicalize_key` 算法平台无关 ——
    /// key 部分大写 / F1-F24 / named key titlecase 不受平台影响。
    #[test]
    fn test_3_3_5_key_titlecase_platform_invariant() {
        for plat in [KeyPlatform::Mac, KeyPlatform::Other] {
            // key 部分始终大写 T
            assert!(canonicalize_keybinding_for("Cmd+Shift+t", plat).ends_with("Shift+T"));
            // F1-F24 全大写
            assert!(canonicalize_keybinding_for("cmd+F11", plat).ends_with("F11"));
            // named key titlecase
            assert!(canonicalize_keybinding_for("alt+ESCAPE", plat).ends_with("Escape"));
            assert!(canonicalize_keybinding_for("shift+tab", plat).ends_with("Tab"));
        }
    }

    /// TEST-3.3.R3（spec §8 R3）：Windows 上 `Cmd+Ctrl+T` 合并为单 `Ctrl+T`
    /// （Windows 无独立 Cmd 键 · 容错合并 · 显式断言非 bug）。
    #[test]
    fn test_3_3_r3_primary_mod_ctrl_merge_on_other() {
        assert_eq!(
            canonicalize_keybinding_for("Cmd+Ctrl+T", KeyPlatform::Other),
            "Ctrl+T"
        );
    }

    /// 排序规则不变（spec §H.3 锁定）：Mac 上 Cmd > Ctrl > Alt > Shift。
    #[test]
    fn test_3_3_sort_order_unchanged_mac() {
        assert_eq!(
            canonicalize_keybinding_for("shift+cmd+t", KeyPlatform::Mac),
            "Cmd+Shift+T"
        );
        assert_eq!(
            canonicalize_keybinding_for("alt+ctrl+shift+a", KeyPlatform::Mac),
            "Ctrl+Alt+Shift+A"
        );
    }
}
