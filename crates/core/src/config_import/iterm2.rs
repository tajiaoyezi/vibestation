//! iTerm2 plist 配置 parser（macOS only · linux/windows 直接返回 not found）
//!
//! 路径：~/Library/Preferences/com.googlecode.iterm2.plist
//! 格式：binary plist（默认 · 魔数 bplist00）· fallback text plist
//! 字段：Default Bookmark Guid → 找到 default profile → 提取 Normal Font / Non Ascii Font / ANSI Color N / Shell / Command

use super::{ConfigImportError, ImportSource, ImportedField, RawScanResult};
use std::path::Path;

pub fn scan(home: &Path) -> RawScanResult {
    let path = home.join("Library/Preferences/com.googlecode.iterm2.plist");
    if !path.exists() {
        return RawScanResult {
            source: ImportSource::ITerm2,
            path: None,
            path_exists: false,
            detected_fields: Vec::new(),
            errors: Vec::new(),
        };
    }
    match parse_file(&path) {
        Ok(fields) => RawScanResult {
            source: ImportSource::ITerm2,
            path: Some(path),
            path_exists: true,
            detected_fields: fields,
            errors: Vec::new(),
        },
        Err(e) => RawScanResult {
            source: ImportSource::ITerm2,
            path: Some(path),
            path_exists: true,
            detected_fields: Vec::new(),
            errors: vec![e.to_string()],
        },
    }
}

fn parse_file(path: &Path) -> Result<Vec<ImportedField>, ConfigImportError> {
    use plist::Value;
    let root: Value =
        plist::from_file(path).map_err(|e| ConfigImportError::Plist(e.to_string()))?;
    let dict = root
        .as_dictionary()
        .ok_or_else(|| ConfigImportError::Plist("root is not a dict".to_string()))?;

    // 找 Default Bookmark Guid
    let default_guid = dict
        .get("Default Bookmark Guid")
        .and_then(|v| v.as_string());

    // 找 profiles 数组
    let profiles = dict
        .get("New Bookmarks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ConfigImportError::Plist("New Bookmarks array not found".to_string()))?;

    if profiles.is_empty() {
        return Ok(Vec::new());
    }

    // 选 default profile · 否则取第一个
    let profile = default_guid
        .and_then(|guid| {
            profiles.iter().find(|p| {
                p.as_dictionary()
                    .and_then(|d| d.get("Guid"))
                    .and_then(|g| g.as_string())
                    == Some(guid)
            })
        })
        .or_else(|| profiles.first())
        .and_then(|v| v.as_dictionary())
        .ok_or_else(|| ConfigImportError::Plist("no valid profile found".to_string()))?;

    let mut fields = Vec::new();
    if let Some(font) = profile.get("Normal Font").and_then(|v| v.as_string()) {
        // iTerm2 font format: "JetBrains Mono 14" · 分离 family + size
        if let Some(last_space) = font.rfind(' ') {
            let (family, size_str) = font.split_at(last_space);
            fields.push(ImportedField::FontFamily(family.trim().to_string()));
            if let Ok(size) = size_str.trim().parse::<f32>() {
                fields.push(ImportedField::FontSize(size));
            }
        } else {
            fields.push(ImportedField::FontFamily(font.to_string()));
        }
    }
    if let Some(shell) = profile.get("Command").and_then(|v| v.as_string()) {
        fields.push(ImportedField::Shell(shell.to_string()));
    }
    // ANSI 0-15 colors
    for i in 0u8..=15 {
        let key = format!("Ansi {} Color", i);
        if let Some(color_dict) = profile.get(&key).and_then(|v| v.as_dictionary()) {
            let r = color_dict
                .get("Red Component")
                .and_then(|v| v.as_real())
                .unwrap_or(0.0);
            let g = color_dict
                .get("Green Component")
                .and_then(|v| v.as_real())
                .unwrap_or(0.0);
            let b = color_dict
                .get("Blue Component")
                .and_then(|v| v.as_real())
                .unwrap_or(0.0);
            fields.push(ImportedField::AnsiColor {
                index: i,
                hex: rgb_to_hex(r, g, b),
            });
        }
    }
    Ok(fields)
}

fn rgb_to_hex(r: f64, g: f64, b: f64) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_no_file_not_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let r = scan(tmp.path());
        assert!(!r.path_exists);
    }

    #[test]
    fn scan_empty_profiles_array() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        // 写一个最简 text plist（no profiles）
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>New Bookmarks</key>
    <array/>
</dict>
</plist>"#;
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert!(r.detected_fields.is_empty());
        assert!(r.errors.is_empty());
    }

    #[test]
    fn scan_single_profile_extracts_font_shell() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>New Bookmarks</key>
    <array>
        <dict>
            <key>Guid</key>
            <string>profile-1</string>
            <key>Normal Font</key>
            <string>JetBrains Mono 14</string>
            <key>Command</key>
            <string>/bin/zsh</string>
        </dict>
    </array>
</dict>
</plist>"#;
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 3); // family + size + shell
        assert!(r.errors.is_empty());
    }

    #[test]
    fn scan_broken_plist_graceful() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"this is not plist").unwrap();
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert!(r.detected_fields.is_empty());
        assert_eq!(r.errors.len(), 1);
    }

    #[test]
    fn scan_ansi_colors_16() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        // 构建含 16 个 ANSI color 的 plist XML
        let mut xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>New Bookmarks</key>
    <array>
        <dict>
            <key>Guid</key>
            <string>profile-1</string>
            <key>Normal Font</key>
            <string>JetBrains Mono 14</string>"#.to_string();
        for i in 0u8..=15 {
            xml.push_str(&format!(
                "\n            <key>Ansi {} Color</key>\n            <dict>\n                <key>Red Component</key><real>{}</real>\n                <key>Green Component</key><real>{}</real>\n                <key>Blue Component</key><real>{}</real>\n            </dict>",
                i,
                i as f64 / 15.0,
                i as f64 / 15.0,
                i as f64 / 15.0
            ));
        }
        xml.push_str(
            r#"
        </dict>
    </array>
</dict>
</plist>"#,
        );
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        assert!(r.path_exists);
        // family + size + shell(缺) + 16 ansi = 18
        let ansi_count = r
            .detected_fields
            .iter()
            .filter(|f| matches!(f, ImportedField::AnsiColor { .. }))
            .count();
        assert_eq!(ansi_count, 16);
        // 验证 index 0 的 hex
        let first_ansi = r
            .detected_fields
            .iter()
            .find(|f| matches!(f, ImportedField::AnsiColor { index: 0, .. }));
        assert!(
            matches!(first_ansi, Some(ImportedField::AnsiColor { hex, .. }) if hex == "#000000")
        );
        // 验证 index 15 的 hex (1.0 → ff)
        let last_ansi = r
            .detected_fields
            .iter()
            .find(|f| matches!(f, ImportedField::AnsiColor { index: 15, .. }));
        assert!(
            matches!(last_ansi, Some(ImportedField::AnsiColor { hex, .. }) if hex == "#ffffff")
        );
    }

    #[test]
    fn scan_ansi_colors_partial() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>New Bookmarks</key>
    <array>
        <dict>
            <key>Guid</key>
            <string>profile-1</string>
            <key>Ansi 0 Color</key>
            <dict>
                <key>Red Component</key><real>0.1</real>
                <key>Green Component</key><real>0.2</real>
                <key>Blue Component</key><real>0.3</real>
            </dict>
            <key>Ansi 7 Color</key>
            <dict>
                <key>Red Component</key><real>0.7</real>
                <key>Green Component</key><real>0.8</real>
                <key>Blue Component</key><real>0.9</real>
            </dict>
        </dict>
    </array>
</dict>
</plist>"#;
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        assert!(r.path_exists);
        let ansi_count = r
            .detected_fields
            .iter()
            .filter(|f| matches!(f, ImportedField::AnsiColor { .. }))
            .count();
        assert_eq!(ansi_count, 2);
    }

    #[test]
    fn scan_rgb_to_hex_conversion() {
        assert_eq!(rgb_to_hex(1.0, 0.0, 0.0), "#ff0000");
        assert_eq!(rgb_to_hex(0.0, 1.0, 0.0), "#00ff00");
        assert_eq!(rgb_to_hex(0.0, 0.0, 1.0), "#0000ff");
        assert_eq!(rgb_to_hex(0.0, 0.0, 0.0), "#000000");
        assert_eq!(rgb_to_hex(1.0, 1.0, 1.0), "#ffffff");
        // 0.1, 0.2, 0.3 → 0.1*255=25.5→26(0x1a), 0.2*255=51.0→51(0x33), 0.3*255=76.5→77(0x4d)
        assert_eq!(rgb_to_hex(0.1, 0.2, 0.3), "#1a334d");
    }

    // ─── /review-pr round 5 regression: Default Bookmark Guid 多 profile 路径 ────

    /// 多 profile + Default Bookmark Guid 命中**非第一个** profile · 必须按 GUID 取
    /// （现实用户配置都有多 profile · 不能 fallback 第一个）
    #[test]
    fn scan_selects_profile_by_default_bookmark_guid() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Default Bookmark Guid</key>
    <string>profile-2</string>
    <key>New Bookmarks</key>
    <array>
        <dict>
            <key>Guid</key>
            <string>profile-1</string>
            <key>Normal Font</key>
            <string>SF Mono 12</string>
            <key>Command</key>
            <string>/bin/bash</string>
        </dict>
        <dict>
            <key>Guid</key>
            <string>profile-2</string>
            <key>Normal Font</key>
            <string>JetBrains Mono 16</string>
            <key>Command</key>
            <string>/bin/zsh</string>
        </dict>
    </array>
</dict>
</plist>"#;
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert!(r.errors.is_empty());
        // 必须取 profile-2（GUID 命中）· 不是 profile-1
        let has_jetbrains = r
            .detected_fields
            .iter()
            .any(|f| matches!(f, ImportedField::FontFamily(name) if name == "JetBrains Mono"));
        let has_zsh = r
            .detected_fields
            .iter()
            .any(|f| matches!(f, ImportedField::Shell(s) if s == "/bin/zsh"));
        assert!(
            has_jetbrains,
            "GUID profile-2 应该用 JetBrains Mono · 不是 profile-1 SF Mono · fields={:?}",
            r.detected_fields
        );
        assert!(has_zsh, "GUID profile-2 应该用 /bin/zsh · 不是 /bin/bash");
    }

    /// 无 Default Bookmark Guid · fallback 第一个 profile（兼容性 · spec §B）
    #[test]
    fn scan_falls_back_to_first_profile_when_no_guid() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>New Bookmarks</key>
    <array>
        <dict>
            <key>Normal Font</key>
            <string>Menlo 11</string>
            <key>Command</key>
            <string>/bin/sh</string>
        </dict>
        <dict>
            <key>Normal Font</key>
            <string>Hack 14</string>
        </dict>
    </array>
</dict>
</plist>"#;
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        assert!(r.path_exists);
        // 无 Default Bookmark Guid · 取第一个 profile（Menlo / /bin/sh）
        let has_menlo = r
            .detected_fields
            .iter()
            .any(|f| matches!(f, ImportedField::FontFamily(name) if name == "Menlo"));
        assert!(
            has_menlo,
            "无 GUID · 应该 fallback 第一个 profile Menlo · fields={:?}",
            r.detected_fields
        );
    }

    /// Default Bookmark Guid 指向不存在的 profile · 应该 graceful fallback（不 panic）
    #[test]
    fn scan_default_guid_missing_falls_back() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Default Bookmark Guid</key>
    <string>nonexistent-guid</string>
    <key>New Bookmarks</key>
    <array>
        <dict>
            <key>Guid</key>
            <string>profile-x</string>
            <key>Normal Font</key>
            <string>Courier New 13</string>
        </dict>
    </array>
</dict>
</plist>"#;
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        assert!(r.path_exists);
        // GUID 找不到 · 应该 fallback 第一个 profile（Courier New）· 不 panic
        let has_courier = r
            .detected_fields
            .iter()
            .any(|f| matches!(f, ImportedField::FontFamily(name) if name == "Courier New"));
        assert!(
            has_courier,
            "GUID missing · 应该 fallback profile-x Courier New · fields={:?}",
            r.detected_fields
        );
    }
}
