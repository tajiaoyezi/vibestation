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
//! 当前内置（v0.1）：Cmd+, · Cmd+T · Cmd+W · Cmd+D · Cmd+Shift+D

/// canonical 化一个快捷键字符串
///
/// 输入容忍：`+` / `|` 分隔 modifier · 大小写不敏感 · 接受 unicode 符号（⌘ / ⌥ / ⌃ / ⇧）
///
/// 输出固定：modifier 按 `Cmd > Ctrl > Alt > Shift` 排序 · key 大写 · `+` 连接
///
/// 异常输入（空 / 全 modifier 无 key）→ 返回原始 trim 后字符串（不破坏数据）
#[must_use]
pub fn canonicalize_keybinding(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 把 unicode modifier 符号替换为带 `+` 的 ASCII（输入可能是 `⌘T` · 没有 + 分隔）·
    // 替换后保证后续 split 能把 modifier 和 key 分开。
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

    let mut has_cmd = false;
    let mut has_ctrl = false;
    let mut has_alt = false;
    let mut has_shift = false;
    let mut key: Option<String> = None;

    for tok in tokens {
        match classify_token(tok) {
            TokenKind::Cmd => has_cmd = true,
            TokenKind::Ctrl => has_ctrl = true,
            TokenKind::Alt => has_alt = true,
            TokenKind::Shift => has_shift = true,
            TokenKind::Key(k) => {
                // 多个 key 时取最后一个（容忍 `Cmd+T+U` 这种异常输入 · key=U）
                key = Some(k);
            }
        }
    }

    let mut parts: Vec<String> = Vec::with_capacity(5);
    if has_cmd {
        parts.push("Cmd".to_string());
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
    Cmd,
    Ctrl,
    Alt,
    Shift,
    Key(String),
}

fn classify_token(tok: &str) -> TokenKind {
    let lower = tok.to_lowercase();
    match lower.as_str() {
        "cmd" | "command" | "meta" | "super" | "win" | "windows" | "⌘" => TokenKind::Cmd,
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

/// Vibestation 内置快捷键 + 对应 action（v0.1 · 来源 `crates/app/src/menu.rs`）
///
/// 任何冲突检测都基于此清单的 canonical form 比较
#[must_use]
pub fn vibestation_builtins() -> Vec<(String, &'static str)> {
    let raw: Vec<(&str, &str)> = vec![
        ("Cmd+,", "preferences"),
        ("Cmd+T", "tabs.create"),
        ("Cmd+W", "tabs.close"),
        ("Cmd+D", "pane.split.horizontal"),
        ("Cmd+Shift+D", "pane.split.vertical"),
    ];
    raw.into_iter()
        .map(|(k, a)| (canonicalize_keybinding(k), a))
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

/// 检测一组导入快捷键和 Vibestation 内置的冲突
///
/// 返回的 `vibe_key` / `source_key` 都是 canonical form
#[must_use]
pub fn detect_conflicts(imported: &[(String, String)], // (key, action)
) -> Vec<ConflictHit> {
    let builtins = vibestation_builtins();
    let mut hits = Vec::new();
    for (raw_key, source_action) in imported {
        let canonical = canonicalize_keybinding(raw_key);
        if canonical.is_empty() {
            continue;
        }
        for (vibe_canonical, vibe_action) in &builtins {
            if &canonical == vibe_canonical {
                hits.push(ConflictHit {
                    vibe_key: vibe_canonical.clone(),
                    source_key: canonical.clone(),
                    vibe_action: (*vibe_action).to_string(),
                    source_action: source_action.clone(),
                });
            }
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_simple_lowercase() {
        assert_eq!(canonicalize_keybinding("cmd+t"), "Cmd+T");
    }

    #[test]
    fn canonical_unicode_modifier() {
        assert_eq!(canonicalize_keybinding("⌘T"), "Cmd+T");
        assert_eq!(canonicalize_keybinding("⌘+t"), "Cmd+T");
    }

    #[test]
    fn canonical_meta_alias_for_cmd() {
        assert_eq!(canonicalize_keybinding("Meta+t"), "Cmd+T");
        assert_eq!(canonicalize_keybinding("super+t"), "Cmd+T");
        assert_eq!(canonicalize_keybinding("command+t"), "Cmd+T");
    }

    #[test]
    fn canonical_modifier_order_normalized() {
        // 输入顺序混乱 · 输出按 Cmd > Ctrl > Alt > Shift
        assert_eq!(canonicalize_keybinding("shift+cmd+t"), "Cmd+Shift+T");
        assert_eq!(
            canonicalize_keybinding("alt+ctrl+shift+a"),
            "Ctrl+Alt+Shift+A"
        );
    }

    #[test]
    fn canonical_pipe_separator_alacritty() {
        // Alacritty 用 `Command|Shift` 表达多 modifier
        assert_eq!(canonicalize_keybinding("Command|Shift+T"), "Cmd+Shift+T");
    }

    #[test]
    fn canonical_ctrl_alias() {
        assert_eq!(canonicalize_keybinding("control+c"), "Ctrl+C");
        assert_eq!(canonicalize_keybinding("⌃c"), "Ctrl+C");
    }

    #[test]
    fn canonical_alt_alias() {
        assert_eq!(canonicalize_keybinding("option+f"), "Alt+F");
        assert_eq!(canonicalize_keybinding("⌥f"), "Alt+F");
    }

    #[test]
    fn canonical_function_key_preserved_uppercase() {
        assert_eq!(canonicalize_keybinding("cmd+F1"), "Cmd+F1");
        assert_eq!(canonicalize_keybinding("F11"), "F11");
    }

    #[test]
    fn canonical_named_key_titlecase() {
        assert_eq!(canonicalize_keybinding("cmd+tab"), "Cmd+Tab");
        assert_eq!(canonicalize_keybinding("alt+ESCAPE"), "Alt+Escape");
        assert_eq!(
            canonicalize_keybinding("shift+backspace"),
            "Shift+Backspace"
        );
    }

    #[test]
    fn canonical_punctuation_key() {
        // `,` 字符不变（非字母）
        assert_eq!(canonicalize_keybinding("cmd+,"), "Cmd+,");
    }

    #[test]
    fn canonical_empty_input() {
        assert_eq!(canonicalize_keybinding(""), "");
        assert_eq!(canonicalize_keybinding("   "), "");
    }

    #[test]
    fn canonical_only_modifier_no_key() {
        // 只 modifier · 无 key · 仍生成 modifier 序列
        assert_eq!(canonicalize_keybinding("cmd+shift"), "Cmd+Shift");
    }

    #[test]
    fn vibe_builtins_canonicalized() {
        let builtins = vibestation_builtins();
        // 5 条 · 全是 canonical form
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
        let imported = vec![("cmd+t".to_string(), "new_tab".to_string())];
        let hits = detect_conflicts(&imported);
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
}
