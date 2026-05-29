//! iTerm2 plist 配置 parser（macOS only · linux/windows 直接返回 not found）
//!
//! 路径：~/Library/Preferences/com.googlecode.iterm2.plist
//! 格式：binary plist（默认 · 魔数 bplist00）· fallback text plist
//! 字段：Default Bookmark Guid → 找到 default profile → 提取 Normal Font / Non Ascii Font / ANSI Color N / Shell / Command

use super::{ImportSource, RawScanResult};
#[cfg(target_os = "macos")]
use super::{ConfigImportError, ImportedField};
use std::path::Path;

pub fn scan(home: &Path) -> RawScanResult {
    // task-3.2 AC3：iTerm2 是 macOS 独占产品 · 非 macOS（Windows/Linux）直接短路 ·
    // 不构造任何 `Library/Preferences/...` 路径（即便恰好存在同名文件也不解析）。
    #[cfg(not(target_os = "macos"))]
    {
        let _ = home;
        RawScanResult {
            source: ImportSource::ITerm2,
            path: None,
            path_exists: false,
            detected_fields: Vec::new(),
            errors: Vec::new(),
        }
    }
    #[cfg(target_os = "macos")]
    {
        scan_macos(home)
    }
}

/// macOS 专属扫描实现（plist 解析）· 仅 macOS 编译 · 见 [`scan`] 短路。
#[cfg(target_os = "macos")]
fn scan_macos(home: &Path) -> RawScanResult {
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
        Ok((fields, warnings)) => RawScanResult {
            source: ImportSource::ITerm2,
            path: Some(path),
            path_exists: true,
            detected_fields: fields,
            // Issue #206 Advisory #4 修复：parser warnings（ANSI RGB 缺失等非致命）
            // 流到 errors 字段 · 让 UI 显示拒绝原因 · 不再 silent default 到 #000000
            errors: warnings,
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

/// 返回 (fields, warnings) · warnings 是非致命字段级 reject 原因
/// （例 ANSI N Color 缺 R/G/B 分量 · 不应 silent default 到 #000000）
///
/// macOS 专属：仅在 macOS 编译（非 macOS 平台 [`scan`] 短路 · 此函数不可达）。
#[cfg(target_os = "macos")]
fn parse_file(path: &Path) -> Result<(Vec<ImportedField>, Vec<String>), ConfigImportError> {
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
        return Ok((Vec::new(), Vec::new()));
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
    let mut warnings: Vec<String> = Vec::new();
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
            // Issue #206 Advisory #4 修复：任一 R/G/B 分量缺失 · skip 整个 color 并推 warning
            // · 不再 silent default 到 0.0（会让缺失分量的颜色变 #000000 · 用户感知 ANSI 配色错乱）
            let r = color_dict.get("Red Component").and_then(|v| v.as_real());
            let g = color_dict.get("Green Component").and_then(|v| v.as_real());
            let b = color_dict.get("Blue Component").and_then(|v| v.as_real());
            match (r, g, b) {
                (Some(r), Some(g), Some(b)) => {
                    fields.push(ImportedField::AnsiColor {
                        index: i,
                        hex: rgb_to_hex(r, g, b),
                    });
                }
                _ => {
                    let missing: Vec<&str> = [
                        r.is_none().then_some("Red"),
                        g.is_none().then_some("Green"),
                        b.is_none().then_some("Blue"),
                    ]
                    .into_iter()
                    .flatten()
                    .collect();
                    warnings.push(format!(
                        "Ansi {} Color missing components ({}) · skipped (avoid silent #000000 fallback)",
                        i,
                        missing.join(", ")
                    ));
                }
            }
        }
    }
    Ok((fields, warnings))
}

/// macOS 专属：仅 [`parse_file`] 调用 · 同 cfg-gate 防非 macOS dead_code 警告。
#[cfg(target_os = "macos")]
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

    /// TEST-3.2.3（AC3）：非 macOS 平台（Windows/Linux）· iTerm2 macOS 独占 ·
    /// `scan` 必须短路返回 `path_exists=false` + 空 `detected_fields` + 空 `errors` +
    /// `path=None`（不构造任何 `Library/Preferences/...` 路径）· 即便 home 下恰好存在
    /// 同名 plist 文件也不解析（短路语义优先）。
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_3_2_3_iterm2_non_macos_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        // 故意在 home 下放一个合法 plist · 验证非 macOS 仍短路不解析
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
            <string>JetBrains Mono 14</string>
        </dict>
    </array>
</dict>
</plist>"#;
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        assert!(!r.path_exists, "非 macOS 应短路 path_exists=false");
        assert!(r.path.is_none(), "非 macOS 不应构造 path");
        assert!(r.detected_fields.is_empty(), "非 macOS detected_fields 应空");
        assert!(r.errors.is_empty(), "非 macOS errors 应空");
        assert_eq!(r.source, ImportSource::ITerm2);
    }

    #[cfg(target_os = "macos")]
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

    #[cfg(target_os = "macos")]
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

    /// Issue #206 Important 2 修复：现实中 iTerm2 plist 几乎都是 binary 格式（bplist00）·
    /// 之前所有测试都用 XML text plist · binary 解析路径未被直接覆盖。本测试构造一份
    /// binary plist fixture · 验证 plist crate 自动 dispatch + extract fields 一致。
    #[cfg(target_os = "macos")]
    #[test]
    fn scan_single_profile_binary_plist_extracts_font_shell() {
        use plist::{Dictionary, Value};

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        let mut profile = Dictionary::new();
        profile.insert("Guid".into(), Value::String("profile-bin".into()));
        profile.insert(
            "Normal Font".into(),
            Value::String("JetBrains Mono 14".into()),
        );
        profile.insert("Command".into(), Value::String("/bin/zsh".into()));

        let mut root = Dictionary::new();
        root.insert(
            "New Bookmarks".into(),
            Value::Array(vec![Value::Dictionary(profile)]),
        );

        let mut buf: Vec<u8> = Vec::new();
        Value::Dictionary(root)
            .to_writer_binary(&mut buf)
            .expect("write binary plist");
        std::fs::write(&path, &buf).unwrap();

        // 验证写入的真是 binary plist（魔数 bplist00）· 防 plist crate 默认走 XML
        assert_eq!(
            &buf[..8],
            b"bplist00",
            "fixture 必须是 binary plist · 实际 magic={:?}",
            &buf[..8]
        );

        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(
            r.detected_fields.len(),
            3,
            "binary plist 应 extract 3 fields (font_family + font_size + shell) · 实际={:?}",
            r.detected_fields
        );
        assert!(r.errors.is_empty(), "errors={:?}", r.errors);
    }

    /// Issue #206 Advisory #4 修复：ANSI N Color 缺 R/G/B 分量必须 skip 整个 color
    /// 并推 warning · 不能 silent default 到 0.0（会让缺失分量的颜色变 #000000）
    #[cfg(target_os = "macos")]
    #[test]
    fn scan_ansi_color_missing_components_skipped_with_warning() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp
            .path()
            .join("Library/Preferences/com.googlecode.iterm2.plist");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        // Ansi 0 Color 缺 Green Component · 应 skip + warning
        // Ansi 1 Color 完整 · 应 detect
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>New Bookmarks</key>
    <array>
        <dict>
            <key>Guid</key>
            <string>p</string>
            <key>Ansi 0 Color</key>
            <dict>
                <key>Red Component</key><real>0.5</real>
                <key>Blue Component</key><real>0.5</real>
            </dict>
            <key>Ansi 1 Color</key>
            <dict>
                <key>Red Component</key><real>1.0</real>
                <key>Green Component</key><real>0.0</real>
                <key>Blue Component</key><real>0.0</real>
            </dict>
        </dict>
    </array>
</dict>
</plist>"#;
        std::fs::write(&path, xml).unwrap();
        let r = scan(tmp.path());
        // 仅 Ansi 1 进 detected · Ansi 0 被 skip
        let ansi_count = r
            .detected_fields
            .iter()
            .filter(|f| matches!(f, ImportedField::AnsiColor { .. }))
            .count();
        assert_eq!(
            ansi_count, 1,
            "仅 Ansi 1 应 detect · 实际 ansi 数={} · fields={:?}",
            ansi_count, r.detected_fields
        );
        // warnings 应含 Ansi 0 缺失 Green 的说明
        assert!(
            r.errors
                .iter()
                .any(|e| e.contains("Ansi 0") && e.contains("Green")),
            "errors 应含 'Ansi 0 ... Green missing' · 实际={:?}",
            r.errors
        );
    }

    #[cfg(target_os = "macos")]
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

    #[cfg(target_os = "macos")]
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

    #[cfg(target_os = "macos")]
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

    #[cfg(target_os = "macos")]
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
    #[cfg(target_os = "macos")]
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
    #[cfg(target_os = "macos")]
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
    #[cfg(target_os = "macos")]
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
