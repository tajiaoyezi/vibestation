//! iTerm2 plist 配置 parser（macOS only · linux/windows 直接返回 not found）
//!
//! 路径：~/Library/Preferences/com.googlecode.iterm2.plist
//! 格式：binary plist（默认 · 魔数 bplist00）· fallback text plist
//! 字段：Default Bookmark Guid → 找到 default profile → 提取 Normal Font / Non Ascii Font / ANSI Color N / Shell / Command

use super::{ConfigImportError, ImportScanResult, ImportSource, ImportedField};
use std::path::Path;

pub fn scan(home: &Path) -> ImportScanResult {
    let path = home.join("Library/Preferences/com.googlecode.iterm2.plist");
    if !path.exists() {
        return ImportScanResult {
            source: ImportSource::ITerm2,
            path: None,
            path_exists: false,
            detected_fields: Vec::new(),
            errors: Vec::new(),
        };
    }
    match parse_file(&path) {
        Ok(fields) => ImportScanResult {
            source: ImportSource::ITerm2,
            path: Some(path),
            path_exists: true,
            detected_fields: fields,
            errors: Vec::new(),
        },
        Err(e) => ImportScanResult {
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
    // ANSI color / keybinding 留 Phase B
    Ok(fields)
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
}
