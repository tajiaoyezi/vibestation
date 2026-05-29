//! 配置导入 IPC 类型层 · spec §G.1 + §G.2 derive 模板
//!
//! - 所有类型 `#[derive(TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
//! - `ImportFieldType` 因含 payload · 必须 tagged union（`#[serde(tag = "kind")]`）
//! - `ImportSource` 在 super 模块定义（[`super::ImportSource`]）· 已 ts-rs derive
//! - `f32` 字段加 `#[ts(type = "number")]`（前端默认生成 bigint · 强制 number）
//! - bindings 由 `crates/app/build.rs` 生成 · 前端禁手写
//!
//! 业务编排：[`scan_all_sources_ipc`] · [`build_preview`] · [`apply`] · [`detect_conflicts_ipc`]

use crate::app_settings::SettingsUpdateRequest;
use crate::config_import::keybinding::{detect_conflicts, ConflictHit};
use crate::config_import::{ImportSource, ImportedField, RawScanResult};
use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;

// ─── 类型 0 · ThemeMode enum（Issue #206 Important 1 修复） ────────────

/// vibestation 唯一支持的主题模式 · 类型层防御 + ts-rs 自动生成 union literal
///
/// 之前是 `String`（runtime validation 在 apply round 1 修）· 但类型层仍宽
/// · parser 可能 detect "tokyo-night" 这种 ghostty palette name · 到 apply
/// 才 reject。改 enum 让 parser 层就 reject · 编译期保证只有 light/dark/auto
/// 能流到下游。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

impl ThemeMode {
    /// 持久化用字符串 · 与 app_settings.theme DB column 值一致
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Auto => "auto",
        }
    }

    /// 用户输入字符串 → ThemeMode · trim + lowercase 后匹配 ·
    /// 失败返回被拒绝的原值（让 caller 拼 error 文案）
    pub fn parse_normalized(input: &str) -> Result<Self, String> {
        match input.trim().to_lowercase().as_str() {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "auto" => Ok(Self::Auto),
            _ => Err(input.trim().to_string()),
        }
    }
}

// ─── 类型 1 · ImportFieldType（tagged union） ──────────────────────────

/// 导入字段类型 · spec §G.2 tagged union
///
/// JSON serialization 形如：`{"kind":"fontFamily","value":"JetBrains Mono"}`
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ImportFieldType {
    FontFamily {
        value: String,
    },
    FontSize {
        #[ts(type = "number")]
        value: f32,
    },
    Theme {
        value: ThemeMode,
    },
    Shell {
        value: String,
    },
    KeyBinding {
        key: String,
        action: String,
    },
    AnsiColor {
        #[ts(type = "number")]
        index: u8,
        hex: String,
    },
}

impl From<ImportedField> for ImportFieldType {
    fn from(f: ImportedField) -> Self {
        match f {
            ImportedField::FontFamily(v) => Self::FontFamily { value: v },
            ImportedField::FontSize(v) => Self::FontSize { value: v },
            ImportedField::Theme(v) => Self::Theme { value: v },
            ImportedField::Shell(v) => Self::Shell { value: v },
            ImportedField::KeyBinding { key, action } => Self::KeyBinding { key, action },
            ImportedField::AnsiColor { index, hex } => Self::AnsiColor { index, hex },
        }
    }
}

// ─── 类型 2 · ImportScanResult ─────────────────────────────────────────

/// 单源扫描结果 · spec §G.1
///
/// 不暴露内部 `path: PathBuf`（仅 [`super::RawScanResult`] 有）·
/// 改用 `path_display` 字符串方便 UI 显示（仅展示 · 不参与逻辑）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ImportScanResult {
    pub source: ImportSource,
    pub path_exists: bool,
    /// 找到的实际配置文件路径（用于 UI 显示 · `~/...` 形式 · None 若未找到）
    pub path_display: Option<String>,
    pub detected_fields: Vec<ImportFieldType>,
    pub errors: Vec<String>,
}

impl From<&RawScanResult> for ImportScanResult {
    fn from(raw: &RawScanResult) -> Self {
        Self {
            source: raw.source,
            path_exists: raw.path_exists,
            path_display: raw.path.as_ref().map(|p| prettify_home_path(p)),
            detected_fields: raw
                .detected_fields
                .iter()
                .cloned()
                .map(ImportFieldType::from)
                .collect(),
            errors: raw.errors.clone(),
        }
    }
}

/// Issue #206 Advisory #3 修复：后端用家目录环境变量准确替换家目录为 `~`
/// · 前端不再需要 hard-coded `/Users/[^/]+` / `/home/[^/]+` 正则
/// · 不命中（如 `/opt/homebrew/` / `/var/folders/`）时原样返回
///
/// task-3.2（AC4）：平台感知家目录环境变量 ——
/// - Windows：`USERPROFILE`（`HOME` 在 Windows 常缺失）
/// - 非 Windows：`HOME`
///
/// 读环境变量后委托 [`prettify_home_path_with`]（可注入 home · 测试不依赖真实 env）。
fn prettify_home_path(p: &Path) -> String {
    #[cfg(windows)]
    let home = std::env::var("USERPROFILE").ok();
    #[cfg(not(windows))]
    let home = std::env::var("HOME").ok();
    prettify_home_path_with(p, home)
}

/// 可注入实现：`home` 显式传入 · 命中前缀折叠为 `~` · 否则原样返回。
fn prettify_home_path_with(p: &Path, home: Option<String>) -> String {
    let s = p.to_string_lossy();
    if let Some(home) = home.filter(|h| !h.is_empty()) {
        if let Some(stripped) = s.strip_prefix(&home) {
            return format!("~{stripped}");
        }
    }
    s.into_owned()
}

// ─── 类型 3 · ImportPreview ────────────────────────────────────────────

/// 跨源合并的预览（spec §G.1 · UI Step 2 数据）
///
/// 同一 field type 多源命中时 · 用户最终勾选生效
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    /// 所有扫描的源（含未检测到 · 但保留以便 UI 全展示）
    pub sources: Vec<ImportScanResult>,
    /// 内置冲突（已 canonical 化）
    pub conflicts: Vec<KeyBindingConflict>,
}

// ─── 类型 4 · ImportApplyRequest ───────────────────────────────────────

/// 应用导入的请求（spec §G.1 · UI Step 3 提交）
///
/// 用户从预览列表勾选要写入 app_settings 的字段
///
/// Issue #206 Advisory #2 修复：删除 source 字段 · apply 函数从未引用 · 字段冗余
/// · 字段实际数据全在 fields 里 · UI 端 source 用于 step 状态 ≠ apply IPC 负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ImportApplyRequest {
    /// 勾选生效的字段集合
    pub fields: Vec<ImportFieldType>,
    /// 用户对每个冲突的决策（user_choice 已填）
    pub conflict_resolutions: Vec<KeyBindingConflict>,
}

// ─── 类型 4.5 · AppliedField enum（Issue #206 Advisory #7 修复） ──────

/// 实际写入 app_settings 的字段类型 · 替代原 `Vec<String>` 中硬编码字段名
///
/// 改前：`applied = vec!["font_family", "default_shell"]` · 前端 switch string · 易拼错
/// 改后：`applied = vec![AppliedField::FontFamily, AppliedField::DefaultShell]`
/// · ts-rs 自动生成 union literal · 前端 exhaustive match · 编译期保证
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum AppliedField {
    FontFamily,
    FontSize,
    Theme,
    DefaultShell,
    ImportedKeybindings,
}

// ─── 类型 5 · ImportApplyResult ────────────────────────────────────────

/// 应用结果（spec §G.1 · 写入 app_settings 的反馈）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ImportApplyResult {
    /// 实际写入 app_settings 的字段（Issue #206 Advisory #7 · 由 `Vec<String>` 改为 enum vec）
    pub applied: Vec<AppliedField>,
    /// 跳过的冲突（user_choice == KeepVibe 的）
    pub skipped_conflicts: Vec<KeyBindingConflict>,
    /// 字段级错误（不阻止整体 · graceful）
    pub errors: Vec<String>,
}

// ─── 类型 6 · KeyBindingConflict + 类型 7 · KeyBindingResolution ───────

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum KeyBindingResolution {
    /// 保留 Vibestation 内置（默认 · 黄色 warning · 不导入此 keybinding）
    KeepVibe,
    /// 用户强制覆盖（导入 · 写入 imported_keybindings · v0.2+ 才会激活到菜单）
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct KeyBindingConflict {
    /// canonical form · 例 "Cmd+T"
    pub vibe_key: String,
    /// canonical form · 同 vibe_key
    pub source_key: String,
    /// 例 "tabs.create"（Vibestation 内置 action）
    pub vibe_action: String,
    /// 例 "new_tab"（原终端 action 名）
    pub source_action: String,
    /// 用户决策（默认 KeepVibe · UI 可改 Override）
    pub user_choice: KeyBindingResolution,
}

impl From<&ConflictHit> for KeyBindingConflict {
    fn from(hit: &ConflictHit) -> Self {
        Self {
            vibe_key: hit.vibe_key.clone(),
            source_key: hit.source_key.clone(),
            vibe_action: hit.vibe_action.clone(),
            source_action: hit.source_action.clone(),
            user_choice: KeyBindingResolution::KeepVibe,
        }
    }
}

// ─── 业务编排 · scan / preview / apply / detect ────────────────────────

/// IPC 入口：扫描默认路径 + 转换为 IPC 形态
///
/// 不读 DB · 不写 DB · 纯解析 + 转换
#[must_use]
pub fn scan_all_sources_ipc(home: &Path) -> Vec<ImportScanResult> {
    super::scan_all_sources(home)
        .iter()
        .map(ImportScanResult::from)
        .collect()
}

/// 构建预览：跨源合并扫描 + 全局冲突检测
///
/// `selected_sources` 限定参与冲突检测的源（UI Step 1 选择）·
/// 空列表 → 所有扫描到的源都参与
#[must_use]
pub fn build_preview(home: &Path, selected_sources: &[ImportSource]) -> ImportPreview {
    let scan_results = scan_all_sources_ipc(home);

    // 收集需要参与冲突检测的源（path_exists 且在 selected · 或 selected 为空时全要）
    let mut imported_keybindings: Vec<(String, String)> = Vec::new();
    for result in &scan_results {
        if !result.path_exists {
            continue;
        }
        if !selected_sources.is_empty() && !selected_sources.contains(&result.source) {
            continue;
        }
        for f in &result.detected_fields {
            if let ImportFieldType::KeyBinding { key, action } = f {
                imported_keybindings.push((key.clone(), action.clone()));
            }
        }
    }

    let hits = detect_conflicts(&imported_keybindings);
    let conflicts: Vec<KeyBindingConflict> = hits.iter().map(KeyBindingConflict::from).collect();

    ImportPreview {
        sources: scan_results,
        conflicts,
    }
}

/// 仅做冲突检测（UI 切换源 / 勾选字段时复用）
#[must_use]
pub fn detect_conflicts_ipc(fields: &[ImportFieldType]) -> Vec<KeyBindingConflict> {
    let imported: Vec<(String, String)> = fields
        .iter()
        .filter_map(|f| match f {
            ImportFieldType::KeyBinding { key, action } => Some((key.clone(), action.clone())),
            _ => None,
        })
        .collect();

    detect_conflicts(&imported)
        .iter()
        .map(KeyBindingConflict::from)
        .collect()
}

/// 应用导入：写 app_settings · 返回结果
///
/// - 字段顺序：font_family / font_size / theme / shell / keybindings（spec §A）
/// - keybindings：非冲突 + override 决策的 → JSON 字符串写 `imported_keybindings`
///   （v0.1 不激活到菜单 · 留 v0.2+ 接入 menu accelerator）
/// - graceful：单个字段写失败不阻止其他（errors 字段记录）
pub fn apply(pool: &DbPool, req: &ImportApplyRequest) -> ImportApplyResult {
    use crate::config_import::keybinding::{canonicalize_keybinding, detect_conflicts};

    // Issue #206 Advisory #7 修复：applied 由 Vec<String> 改 Vec<AppliedField> enum
    let mut applied: Vec<AppliedField> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut skipped_conflicts: Vec<KeyBindingConflict> = Vec::new();

    // ─── §1 · 服务端独立冲突检测（Codex review 2026-05-01 finding 2 修复）─────
    // 不 trust client 传的 conflict_resolutions · server 必须基于 req.fields 重检：
    // - 默认 KeepVibe（保护 vibe 内置 · 即使 client 没传 / 传错）
    // - 仅当 client 对该 canonical key explicit Override 才允许覆盖
    let imported_keybindings_input: Vec<(String, String)> = req
        .fields
        .iter()
        .filter_map(|f| match f {
            ImportFieldType::KeyBinding { key, action } => Some((key.clone(), action.clone())),
            _ => None,
        })
        .collect();
    let server_conflicts = detect_conflicts(&imported_keybindings_input);
    let server_conflict_keys: std::collections::HashSet<String> = server_conflicts
        .iter()
        .map(|h| h.source_key.clone())
        .collect();
    let client_overrides: std::collections::HashSet<String> = req
        .conflict_resolutions
        .iter()
        .filter(|c| matches!(c.user_choice, KeyBindingResolution::Override))
        .map(|c| c.source_key.clone())
        .collect();

    // ─── §2 · 收集字段意图（with theme validation）──────────────────────────
    let mut update_req = SettingsUpdateRequest::default();
    let mut keybindings_to_persist: Vec<(String, String)> = Vec::new();
    // Issue #206 Advisory #7 修复：field_intent 由 &'static str 改 AppliedField enum
    let mut field_intent: Vec<AppliedField> = Vec::new();

    for field in &req.fields {
        match field {
            ImportFieldType::FontFamily { value } => {
                update_req.font_family = Some(value.clone());
                field_intent.push(AppliedField::FontFamily);
            }
            ImportFieldType::FontSize { value } => {
                let rounded = value.round().clamp(8.0, 72.0) as u32;
                update_req.font_size = Some(rounded);
                field_intent.push(AppliedField::FontSize);
            }
            ImportFieldType::Theme { value } => {
                // Issue #206 Important 1 修复：value 现为 ThemeMode enum（type-level
                // 保证仅 light/dark/auto）· 不需要 runtime normalize + match。
                // 旧 runtime validation 已 supersede · parser 层 reject 任意 palette name。
                update_req.theme = Some(value.as_db_str().to_string());
                field_intent.push(AppliedField::Theme);
            }
            ImportFieldType::Shell { value } => {
                // Issue #205 Finding 1 修复：导入前先验证 shell 存在 · 防 default_shell_set
                // 的 check_shell_exists 旁路 · 否则用户导入 Ghostty 配的 /bin/zsh 但本机没装
                // zsh · UI 显示 default_shell applied 但实际 PTY spawn 失败。
                if let Err(e) = crate::pty::check_shell_exists(value) {
                    errors.push(format!("shell '{value}' not found: {e}"));
                } else {
                    update_req.default_shell = Some(value.clone());
                    field_intent.push(AppliedField::DefaultShell);
                }
            }
            ImportFieldType::KeyBinding { key, action } => {
                let canonical = canonicalize_keybinding(key);
                if canonical.is_empty() {
                    continue;
                }
                if server_conflict_keys.contains(&canonical) {
                    if client_overrides.contains(&canonical) {
                        // explicit Override · 允许覆盖 vibe 内置
                        keybindings_to_persist.push((canonical, action.clone()));
                    } else {
                        // 默认 KeepVibe（保护 vibe 内置 · 即使 client 没传或传 KeepVibe）
                        if let Some(hit) =
                            server_conflicts.iter().find(|h| h.source_key == canonical)
                        {
                            skipped_conflicts.push(KeyBindingConflict {
                                vibe_key: hit.vibe_key.clone(),
                                source_key: hit.source_key.clone(),
                                vibe_action: hit.vibe_action.clone(),
                                source_action: hit.source_action.clone(),
                                user_choice: KeyBindingResolution::KeepVibe,
                            });
                        }
                    }
                } else {
                    keybindings_to_persist.push((canonical, action.clone()));
                }
            }
            ImportFieldType::AnsiColor { .. } => {
                // ANSI color 由前端处理 · 直接转 CSS var · v0.1 不进 app_settings
            }
        }
    }

    // ─── §3 · 原子写入（Codex review finding 3 修复）──────────────────────
    // 原实现：N 次独立 `AppSettingsStore::set` · 中途 fail 留半状态 · errors 只报
    // generic · applied 不准。改成 unchecked_transaction 包整个写入：任一 fail
    // → rollback · errors 报具体字段 · applied 全空不 misreport。全成功 → commit。
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("pool get failed: {e}"));
            return ImportApplyResult {
                applied,
                skipped_conflicts,
                errors,
            };
        }
    };
    let tx = match conn.unchecked_transaction() {
        Ok(t) => t,
        Err(e) => {
            errors.push(format!("transaction begin failed: {e}"));
            return ImportApplyResult {
                applied,
                skipped_conflicts,
                errors,
            };
        }
    };

    let upsert_sql = "INSERT INTO app_settings (key, value) VALUES (?1, ?2) \
                      ON CONFLICT(key) DO UPDATE SET value = excluded.value";

    let tx_result: Result<(), String> = (|| {
        if let Some(ref v) = update_req.font_family {
            tx.execute(upsert_sql, rusqlite::params!["font_family", v])
                .map_err(|e| format!("font_family: {e}"))?;
        }
        if let Some(v) = update_req.font_size {
            tx.execute(upsert_sql, rusqlite::params!["font_size", v.to_string()])
                .map_err(|e| format!("font_size: {e}"))?;
        }
        if let Some(ref v) = update_req.theme {
            tx.execute(upsert_sql, rusqlite::params!["theme", v])
                .map_err(|e| format!("theme: {e}"))?;
        }
        if let Some(ref v) = update_req.default_shell {
            tx.execute(upsert_sql, rusqlite::params!["default_shell", v])
                .map_err(|e| format!("default_shell: {e}"))?;
        }
        if !keybindings_to_persist.is_empty() {
            // review round 5 fix: serialize_keybindings 返回 Result · 失败显式向上传播
            // （改前 unwrap_or "[]" 静默吞 serde_json error · 写空数组进 DB · applied 仍含
            // imported_keybindings · 用户看到成功但数据丢失）
            let json = serialize_keybindings(&keybindings_to_persist)
                .map_err(|e| format!("imported_keybindings serialize: {e}"))?;
            tx.execute(upsert_sql, rusqlite::params!["imported_keybindings", json])
                .map_err(|e| format!("imported_keybindings: {e}"))?;
        }
        Ok(())
    })();

    match tx_result {
        Ok(()) => match tx.commit() {
            Ok(()) => {
                applied.extend(field_intent.iter().copied());
                if !keybindings_to_persist.is_empty() {
                    applied.push(AppliedField::ImportedKeybindings);
                }
            }
            Err(e) => {
                errors.push(format!("transaction commit failed: {e}"));
            }
        },
        Err(e) => {
            errors.push(format!(
                "settings write failed: {e} (transaction rolled back · no fields written)"
            ));
            // tx drop 自动 rollback · 不需显式调用
        }
    }

    ImportApplyResult {
        applied,
        skipped_conflicts,
        errors,
    }
}

/// 把 (key, action) 序列化为 JSON 字符串 · 写入 app_settings.imported_keybindings
///
/// 返回 Result · 失败让 caller 显式处理（review round 5 fix · 改前 unwrap_or "[]"
/// 静默吞 serde error · 写空数组进 DB · applied 仍含 imported_keybindings ·
/// 用户看到成功但数据丢失）
fn serialize_keybindings(pairs: &[(String, String)]) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct Entry<'a> {
        key: &'a str,
        action: &'a str,
    }
    let entries: Vec<Entry<'_>> = pairs
        .iter()
        .map(|(k, a)| Entry { key: k, action: a })
        .collect();
    serde_json::to_string(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_settings::AppSettingsStore;
    use crate::db;
    use tempfile::TempDir;

    fn setup_pool() -> (TempDir, DbPool) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test_config_import.db");
        let pool = db::open_pool(&path).unwrap();
        (dir, pool)
    }

    #[test]
    fn imported_field_to_ipc_roundtrip() {
        let f = ImportedField::FontFamily("JetBrains Mono".to_string());
        let ipc = ImportFieldType::from(f);
        match ipc {
            ImportFieldType::FontFamily { value } => assert_eq!(value, "JetBrains Mono"),
            _ => panic!("wrong variant"),
        }

        let f = ImportedField::AnsiColor {
            index: 7,
            hex: "#cccccc".to_string(),
        };
        let ipc = ImportFieldType::from(f);
        match ipc {
            ImportFieldType::AnsiColor { index, hex } => {
                assert_eq!(index, 7);
                assert_eq!(hex, "#cccccc");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn scan_ipc_all_three_sources_empty_home() {
        let dir = TempDir::new().unwrap();
        let results = scan_all_sources_ipc(dir.path());
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| !r.path_exists));
    }

    #[test]
    fn build_preview_no_conflicts_empty_home() {
        let dir = TempDir::new().unwrap();
        let preview = build_preview(dir.path(), &[]);
        assert_eq!(preview.sources.len(), 3);
        assert!(preview.conflicts.is_empty());
    }

    #[test]
    fn detect_conflicts_ipc_finds_cmd_t() {
        let fields = vec![
            ImportFieldType::KeyBinding {
                key: "cmd+t".to_string(),
                action: "new_tab".to_string(),
            },
            ImportFieldType::FontFamily {
                value: "Mono".to_string(),
            },
        ];
        let conflicts = detect_conflicts_ipc(&fields);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].vibe_key, "Cmd+T");
        assert_eq!(conflicts[0].source_action, "new_tab");
        // user_choice 默认 KeepVibe
        assert_eq!(conflicts[0].user_choice, KeyBindingResolution::KeepVibe);
    }

    #[test]
    fn apply_writes_font_family_and_theme() {
        // round 2: theme value 改为 supported（worker A 原写 "tokyo-night" 反映 finding 1
        // 的 bug 契约 · 现在 theme 验证要 light/dark/auto · 测 supported 值的正常路径）
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![
                ImportFieldType::FontFamily {
                    value: "JetBrains Mono".to_string(),
                },
                ImportFieldType::FontSize { value: 16.0 },
                ImportFieldType::Theme {
                    value: ThemeMode::Dark,
                },
            ],
            conflict_resolutions: vec![],
        };
        let result = apply(&pool, &req);
        assert!(result.errors.is_empty());
        assert!(result.applied.contains(&AppliedField::FontFamily));
        assert!(result.applied.contains(&AppliedField::FontSize));
        assert!(result.applied.contains(&AppliedField::Theme));

        let s = AppSettingsStore::get_all(&pool);
        assert_eq!(s.font_family, "JetBrains Mono");
        assert_eq!(s.font_size, 16);
        assert_eq!(s.theme, "dark");
    }

    #[test]
    fn apply_clamps_font_size_lower_bound() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::FontSize { value: 4.5 }],
            conflict_resolutions: vec![],
        };
        let _ = apply(&pool, &req);
        let s = AppSettingsStore::get_all(&pool);
        assert_eq!(s.font_size, 8); // clamp 下限 8
    }

    /// /review-pr round 5 regression: font_size 上界 72 boundary symmetry
    /// 原 spec 只测下界 · 上界没测 · 现补
    #[test]
    fn apply_clamps_font_size_upper_bound() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::FontSize { value: 144.0 }], // 远超 72 上限
            conflict_resolutions: vec![],
        };
        let _ = apply(&pool, &req);
        let s = AppSettingsStore::get_all(&pool);
        assert_eq!(s.font_size, 72); // clamp 上限 72
    }

    /// /review-pr round 5 regression: 部分字段 reject + 其他字段成功的 graceful 测试
    ///
    /// Issue #206 Important 1 重构后：theme 已是 ThemeMode enum · 不可能传 unsupported 值
    /// 到 apply（parser 层 reject）。改测 shell 不存在的 reject 路径（Issue #205 Finding 1
    /// 新增）· 验证 graceful 部分 reject 行为仍正确：reject 字段不进 applied + 其他字段照写入
    #[test]
    fn apply_graceful_reject_shell_writes_other_fields() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![
                ImportFieldType::FontFamily {
                    value: "JetBrains Mono".to_string(),
                },
                ImportFieldType::Shell {
                    value: "/nonexistent/path/fake-shell-xyz".to_string(),
                },
                ImportFieldType::FontSize { value: 16.0 },
            ],
            conflict_resolutions: vec![],
        };
        let result = apply(&pool, &req);
        // shell reject · 不进 applied · errors 含说明
        assert!(!result.applied.contains(&AppliedField::DefaultShell));
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("shell") && e.contains("not found")));
        // 但其他字段（font_family + font_size）必须仍写入（graceful · 部分 reject 不阻塞 transaction）
        assert!(result.applied.contains(&AppliedField::FontFamily));
        assert!(result.applied.contains(&AppliedField::FontSize));
        let s = AppSettingsStore::get_all(&pool);
        assert_eq!(s.font_family, "JetBrains Mono");
        assert_eq!(s.font_size, 16);
        // shell 保留 default（不被无效路径覆盖）
        assert_ne!(s.default_shell, "/nonexistent/path/fake-shell-xyz");
    }

    #[test]
    fn apply_keybinding_conflict_kept_vibe_default() {
        let (_dir, pool) = setup_pool();
        // 导入 Cmd+T · 默认 conflict_resolutions 为空 → conflict_map 为空 →
        // map.get(canonical) is None → fall through 写 imported_keybindings
        // 但 spec D.2 默认行为是 KeepVibe · 即默认应该 SKIP
        // 修正：apply 逻辑里 · 若没有 conflict_resolutions 但 key 命中 builtin · 也应跳过
        // 这里我们走一个完整路径 · UI 应该总是先 detect_conflicts_ipc + 把结果回填 conflict_resolutions

        // 模拟 UI 行为：先 detect 出 1 个 conflict（默认 KeepVibe）· 回填到 req
        let detected = detect_conflicts_ipc(&[ImportFieldType::KeyBinding {
            key: "Cmd+T".to_string(),
            action: "spawn_tab".to_string(),
        }]);
        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::KeyBinding {
                key: "Cmd+T".to_string(),
                action: "spawn_tab".to_string(),
            }],
            conflict_resolutions: detected,
        };
        let result = apply(&pool, &req);
        assert!(result.errors.is_empty());
        // 跳过此 keybinding · 不应写 imported_keybindings
        assert!(!result.applied.contains(&AppliedField::ImportedKeybindings));
        assert_eq!(result.skipped_conflicts.len(), 1);
    }

    #[test]
    fn apply_keybinding_override_writes_imported_keybindings() {
        let (_dir, pool) = setup_pool();
        let mut detected = detect_conflicts_ipc(&[ImportFieldType::KeyBinding {
            key: "Cmd+T".to_string(),
            action: "spawn_tab".to_string(),
        }]);
        // 用户 override
        detected[0].user_choice = KeyBindingResolution::Override;

        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::KeyBinding {
                key: "Cmd+T".to_string(),
                action: "spawn_tab".to_string(),
            }],
            conflict_resolutions: detected,
        };
        let result = apply(&pool, &req);
        assert!(result.applied.contains(&AppliedField::ImportedKeybindings));
        assert!(result.skipped_conflicts.is_empty());

        // 验证 DB 内容是 JSON
        let raw = AppSettingsStore::get(&pool, "imported_keybindings").unwrap();
        assert!(raw.contains("Cmd+T"));
        assert!(raw.contains("spawn_tab"));
    }

    #[test]
    fn apply_non_conflicting_keybinding_persists() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::KeyBinding {
                key: "Cmd+Shift+P".to_string(),
                action: "command_palette".to_string(),
            }],
            conflict_resolutions: vec![], // 无冲突
        };
        let result = apply(&pool, &req);
        assert!(result.errors.is_empty());
        assert!(result.applied.contains(&AppliedField::ImportedKeybindings));
        let raw = AppSettingsStore::get(&pool, "imported_keybindings").unwrap();
        assert!(raw.contains("Cmd+Shift+P"));
    }

    #[test]
    fn apply_ansi_color_skipped_silently() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::AnsiColor {
                index: 0,
                hex: "#000000".to_string(),
            }],
            conflict_resolutions: vec![],
        };
        let result = apply(&pool, &req);
        // ANSI color v0.1 不入 DB · applied 列表不应含 ansi
        assert!(result.errors.is_empty());
        assert!(result.applied.is_empty());
    }

    // ─── Codex review (2026-05-01) round 2 regression tests ───────────────
    //
    // Issue #206 Important 1 重构后：apply 层的 theme reject + normalize 逻辑已经
    // upstream 到 ghostty parser + ThemeMode::parse_normalized。原 2 个测试
    // (apply_rejects_unsupported_theme_value / apply_accepts_supported_theme_value_normalized)
    // 在新设计下不可达（ImportFieldType::Theme value 类型已是 ThemeMode enum ·
    // unsupported String 不能 construct）· 已删。覆盖迁到：
    // - ThemeMode::parse_normalized 直接单测（见 theme_mode_tests）
    // - ghostty parser theme reject 测试（见 ghostty.rs tests mod）

    /// Finding 2: client 不传 conflict_resolutions（绕过 detect_conflicts）·
    /// server 必须独立检测 · 默认保护 vibe 内置 ⌘T · 不写入 imported_keybindings
    #[test]
    fn apply_protects_vibe_builtins_when_client_omits_conflict_resolutions() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::KeyBinding {
                key: "Cmd+T".to_string(), // vibe 内置 tabs.create
                action: "spawn_tab".to_string(),
            }],
            conflict_resolutions: vec![], // ❗client 故意不传冲突决策
        };
        let result = apply(&pool, &req);
        // ⌘T 必须被保护 · 不进 imported_keybindings
        assert!(
            !result.applied.contains(&AppliedField::ImportedKeybindings),
            "vibe 内置 ⌘T 不应被覆盖 · applied={:?}",
            result.applied
        );
        // 必须出现在 skipped_conflicts（默认 KeepVibe）
        assert_eq!(result.skipped_conflicts.len(), 1);
        assert_eq!(result.skipped_conflicts[0].source_key, "Cmd+T");
        assert_eq!(
            result.skipped_conflicts[0].user_choice,
            KeyBindingResolution::KeepVibe
        );
        // DB 内 imported_keybindings 不应被写
        assert!(AppSettingsStore::get(&pool, "imported_keybindings").is_err());
    }

    /// Finding 2: client explicit Override · server 允许覆盖 vibe 内置
    #[test]
    fn apply_allows_explicit_override_of_vibe_builtin() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::KeyBinding {
                key: "Cmd+T".to_string(),
                action: "spawn_tab".to_string(),
            }],
            conflict_resolutions: vec![KeyBindingConflict {
                vibe_key: "Cmd+T".to_string(),
                source_key: "Cmd+T".to_string(),
                vibe_action: "tabs.create".to_string(),
                source_action: "spawn_tab".to_string(),
                user_choice: KeyBindingResolution::Override,
            }],
        };
        let result = apply(&pool, &req);
        assert!(result.applied.contains(&AppliedField::ImportedKeybindings));
        assert!(result.skipped_conflicts.is_empty());
        let raw = AppSettingsStore::get(&pool, "imported_keybindings").unwrap();
        assert!(raw.contains("Cmd+T"));
        assert!(raw.contains("spawn_tab"));
    }

    /// Finding 3: apply 是 transactional · 多字段同时写 · 全 commit 一致
    /// · applied 反映完整成功状态 · errors empty
    ///
    /// task-3.2 note：fixture 硬编码 Unix shell `/bin/zsh` · apply 内 shell 可执行性
    /// 校验（shell-existence 子系统 · task-2.1 territory · 非 config-path 范围）在
    /// Windows 必失败（无 `/bin/zsh`）· 故 cfg-gate 到非 Windows（Unix shell 语义专属）。
    #[cfg(not(windows))]
    #[test]
    fn apply_writes_multiple_fields_atomically() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![
                ImportFieldType::FontFamily {
                    value: "JetBrains Mono".to_string(),
                },
                ImportFieldType::FontSize { value: 14.5 },
                ImportFieldType::Theme {
                    value: ThemeMode::Dark,
                },
                ImportFieldType::Shell {
                    value: "/bin/zsh".to_string(),
                },
                ImportFieldType::KeyBinding {
                    key: "Cmd+Shift+P".to_string(),
                    action: "command_palette".to_string(),
                },
            ],
            conflict_resolutions: vec![],
        };
        let result = apply(&pool, &req);
        assert!(
            result.errors.is_empty(),
            "errors 应空 · 实际={:?}",
            result.errors
        );
        // 全部 5 个字段都进 applied（含 keybindings）
        assert!(result.applied.contains(&AppliedField::FontFamily));
        assert!(result.applied.contains(&AppliedField::FontSize));
        assert!(result.applied.contains(&AppliedField::Theme));
        assert!(result.applied.contains(&AppliedField::DefaultShell));
        assert!(result.applied.contains(&AppliedField::ImportedKeybindings));
        // DB 内字段都正确持久化
        let s = AppSettingsStore::get_all(&pool);
        assert_eq!(s.font_family, "JetBrains Mono");
        assert_eq!(s.font_size, 15); // 14.5 round → 15
        assert_eq!(s.theme, "dark");
        assert_eq!(s.default_shell, "/bin/zsh");
    }

    /// Issue #206 Important 1 修复：ThemeMode::parse_normalized 直接单测
    /// · 覆盖 light/dark/auto + 大小写 + 空格 + reject
    #[test]
    fn theme_mode_parse_normalized_accepts_lowercase() {
        assert_eq!(ThemeMode::parse_normalized("light"), Ok(ThemeMode::Light));
        assert_eq!(ThemeMode::parse_normalized("dark"), Ok(ThemeMode::Dark));
        assert_eq!(ThemeMode::parse_normalized("auto"), Ok(ThemeMode::Auto));
    }

    #[test]
    fn theme_mode_parse_normalized_accepts_mixed_case_with_whitespace() {
        assert_eq!(ThemeMode::parse_normalized("  Dark  "), Ok(ThemeMode::Dark));
        assert_eq!(ThemeMode::parse_normalized("LIGHT"), Ok(ThemeMode::Light));
        assert_eq!(ThemeMode::parse_normalized("\tAUTO\n"), Ok(ThemeMode::Auto));
    }

    #[test]
    fn theme_mode_parse_normalized_rejects_palette_names() {
        let rejected = ThemeMode::parse_normalized("tokyo-night");
        assert!(rejected.is_err());
        assert_eq!(rejected.unwrap_err(), "tokyo-night");

        let rejected2 = ThemeMode::parse_normalized("  Nord  ");
        assert!(rejected2.is_err());
        assert_eq!(rejected2.unwrap_err(), "Nord");
    }

    #[test]
    fn theme_mode_as_db_str_round_trip() {
        for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::Auto] {
            let back = ThemeMode::parse_normalized(mode.as_db_str()).unwrap();
            assert_eq!(back, mode);
        }
    }

    /// Issue #205 Finding 1 修复：apply 接收不存在的 shell 应该 reject + 推 errors
    /// · 而不是写入 default_shell · 否则用户后续 PTY spawn 会失败但 UI 显示导入成功
    #[test]
    fn apply_shell_rejects_nonexistent_shell() {
        let (_dir, pool) = setup_pool();
        let req = ImportApplyRequest {
            fields: vec![ImportFieldType::Shell {
                value: "/nonexistent/path/to/fake-shell-xyz".to_string(),
            }],
            conflict_resolutions: vec![],
        };
        let result = apply(&pool, &req);

        // shell 验证失败 · default_shell 不进 applied
        assert!(
            !result.applied.contains(&AppliedField::DefaultShell),
            "未存在的 shell 不应进 applied · 实际={:?}",
            result.applied
        );

        // errors 应含具体的 shell not found 说明
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("shell") && e.contains("not found")),
            "errors 应含 shell not found 说明 · 实际={:?}",
            result.errors
        );

        // DB 内 default_shell 不应被无效值覆盖
        let s = AppSettingsStore::get_all(&pool);
        assert_ne!(
            s.default_shell, "/nonexistent/path/to/fake-shell-xyz",
            "无效 shell 不应被持久化"
        );
    }

    // ─── task-3.2 · prettify_home_path 平台感知（可注入 home · 不依赖真实 env）──

    /// TEST-3.2.4（AC4）：Windows 上以 `USERPROFILE` 折叠家目录为 `~` 前缀。
    /// 用可注入的 `prettify_home_path_with(p, home)` 显式传 home（spec §8 R2 ·
    /// 不依赖真实环境变量 · CI 无 USERPROFILE 也稳定）。
    #[test]
    fn test_3_2_4_prettify_userprofile_tilde() {
        // Windows 风格家目录 + 反斜杠子路径
        let home = "C:\\Users\\alice";
        let p = Path::new("C:\\Users\\alice\\AppData\\Roaming\\alacritty\\config");
        let pretty = prettify_home_path_with(p, Some(home.to_string()));
        assert!(
            pretty.starts_with('~'),
            "应折叠为 ~ 前缀 · 实际={pretty}"
        );
        assert!(
            pretty.contains("alacritty"),
            "应保留子路径 · 实际={pretty}"
        );
        assert!(
            !pretty.contains("C:\\Users\\alice\\AppData"),
            "不应再含完整家目录前缀 · 实际={pretty}"
        );
    }

    /// TEST-3.2.4b（AC4 · 不命中原样返回）：path 不在 home 下时原样返回（不误折叠）。
    #[test]
    fn test_3_2_4b_prettify_non_home_unchanged() {
        let home = "C:\\Users\\alice";
        let p = Path::new("D:\\OtherDrive\\config");
        let pretty = prettify_home_path_with(p, Some(home.to_string()));
        assert_eq!(pretty, "D:\\OtherDrive\\config");
    }

    /// TEST-3.2.4c（AC4/AC6 · Unix 语义保持）：Unix 风格 home 折叠仍正确。
    #[test]
    fn test_3_2_4c_prettify_unix_home_unchanged() {
        let home = "/Users/alice";
        let p = Path::new("/Users/alice/.config/ghostty/config");
        let pretty = prettify_home_path_with(p, Some(home.to_string()));
        assert_eq!(pretty, "~/.config/ghostty/config");
    }

    /// TEST-3.2.4d（AC4 · home 缺失/空时原样返回）。
    #[test]
    fn test_3_2_4d_prettify_no_home_returns_raw() {
        let p = Path::new("C:\\Users\\alice\\AppData\\Roaming\\x");
        assert_eq!(
            prettify_home_path_with(p, None),
            "C:\\Users\\alice\\AppData\\Roaming\\x"
        );
        assert_eq!(
            prettify_home_path_with(p, Some(String::new())),
            "C:\\Users\\alice\\AppData\\Roaming\\x"
        );
    }
}
