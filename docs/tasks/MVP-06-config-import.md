---
id: MVP-06
type: mvp
title: 配置导入（Ghostty + iTerm2 + Alacritty）
status: ready
owner:
phase: W7-W8
depends_on: ["MVP-04"]
blocks: []
blocked_by: []
blocked_note:
estimate: 3d
plan_ref: implementation-plan.md §10.1
risk_ref:
reviewer: Kimi
---

# MVP-06: 配置导入

> **状态**：`ready`
> **依赖**：MVP-04（终端存在才能应用配置）

---

## 🎯 目标（Goal）

从用户已有的终端配置（Ghostty / iTerm2 / Alacritty）自动导入字体、主题、快捷键、shell 偏好到 Vibestation，降低切换成本。

## 📖 背景（Context）

- `implementation-plan.md §10.1` MVP B 折中方案必做项
- `§10.5` 降级树：≤ 15h 时可砍 iTerm2/Alacritty（只留 Ghostty 覆盖 Persona C）
- 三者配置格式：Ghostty TOML / iTerm2 plist / Alacritty TOML（0.14+）+ YAML fallback（0.13-）

---

## 🎨 功能范围（Scope）

**Do**：
- 首次启动（欢迎页或设置页）可触发"导入现有配置"
- 自动扫描以下默认路径：
  - Ghostty (mac/linux): `~/.config/ghostty/config` 或 `~/Library/Application Support/com.mitchellh.ghostty/config`
  - iTerm2 (mac): `~/Library/Preferences/com.googlecode.iterm2.plist`
  - Alacritty (linux): `~/.config/alacritty/alacritty.toml`（0.14+）或 `~/.config/alacritty/alacritty.yml`（0.13- fallback）
- 识别到的配置 → 显示列表让用户勾选要导入的项：
  - 字体 family / size
  - 主题 / 颜色方案
  - Shell 选择
  - 常用快捷键（仅非冲突的）
- 不覆盖 Vibestation 已有快捷键（冲突时保留 Vibestation 原值 + 提示）
- 导入结果写入 rusqlite `app_settings`

**Don't**：
- Windows Terminal / Warp / Kitty 导入（v0.2+）
- 双向同步（Vibestation 改动回写到原终端）（v0.2+）
- 导入 profiles（多 profile）（v0.2+）

## 🖼 UI 引用

- 导入对话框：Calm Studio 风格模态，分 3 个 step：
  1. 选择源（Ghostty / iTerm2 / Alacritty / 手动跳过）
  2. 预览导入项（checkbox 列表）
  3. 确认 + 应用

## ✅ Acceptance

### A. Ghostty 导入（mac + linux）

- [ ] 自动扫描路径，文件存在 → 显示"检测到 Ghostty"；路径优先级：`~/.config/ghostty/config` > `~/Library/Application Support/com.mitchellh.ghostty/config`（macOS），两路径都存在时优先前者
- [ ] 解析 TOML，提取 font / theme / shell / keybindings；使用 `toml` crate 0.8+，schema version 检测，未知字段 `#[serde(default)]` 跳过 + `tracing::warn!` 记录
- [ ] 预览列表显示每项值，用户可勾选；字段顺序：font_family → font_size → theme → shell → keybindings，每项显示"当前值 → 导入值"对比
- [ ] 应用后 Vibestation 字体 / 主题 / shell 变更；字体切换测法：调 `document.fonts.load()` 确认加载成功再切，主题切换走 CSS var 替换，shell 切换只影响新建 Tab

### B. iTerm2 导入（mac only）

- [ ] plist 解析（`plist` crate 1.6+）；必须支持 binary plist（iTerm2 默认 binary），检测魔数 `bplist00`，fallback text plist
- [ ] 提取 default profile 的字体 / ANSI colors / shell；读 `Default Bookmark Guid` 字段，若无取第一个 profile，若 profile 为空数组则 error toast
- [ ] ANSI colors 映射到 Vibestation CSS 变量；16 色映射表（iTerm2 `Ansi X Color` key 0-15 → Vibestation `--ansi-N` CSS var，详细映射表见实施）
- [ ] font smoothing：v0.1 不导入（macOS 13+ 系统级控制，应用层设置被覆盖），MVP-06 实施时仅在 UI 标注"跳过"

### C. Alacritty 导入（linux only）

- [ ] 格式扫描优先级：TOML（`alacritty.toml`）> YAML（`alacritty.yml`）；Alacritty 0.14+ 已从 YAML 切 TOML，MVP-06 优先扫 TOML，YAML fallback（deprecated 但仍有 0.13- 用户）
- [ ] TOML 用 `toml` crate 复用，YAML 用 `serde_yaml` 0.9；提取 font / colors / key_bindings
- [ ] key_bindings 格式转换：Alacritty action enum 到 Vibestation command string 映射表，如 `SpawnNewInstance` → 无映射（Vibestation 为单窗口，跳过 + warn），`Paste` → `clipboard.paste`
- [ ] **v0.1 macOS-first 场景**：Alacritty linux-only，若 v0.1 仅 macOS 发布，Alacritty 延到 v0.2；MVP-06 spec 保留但其 IPC struct 预留不 wire，§10.5 降级树明示此场景

### D. 快捷键冲突处理

- [ ] 检测导入的快捷键是否与 Vibestation 内置冲突（`⌘T` 等）；冲突检测算法：key chord canonical form，Vibestation 用 `⌘+T`/`Ctrl+T` 统一为 `Cmd+T`/`Ctrl+T`（Modifier 按 Cmd > Ctrl > Alt > Shift 排序），比较基于 canonical form，不区分 `⌘` 和 `Cmd`
- [ ] 冲突 → 不导入 + 在预览列表用黄色标记"冲突，保留 Vibestation 原值"；UI：预览列表冲突行背景 `oklch(85% 0.15 95)`（Calm Studio warning token，待确认），icon ⚠，tooltip 显示 "Vibestation 内置 ⌘T 新建 Tab · 保留原值"
- [ ] 用户可强制覆盖（"替换 Vibestation 快捷键"勾选）；勾选后弹二次确认（"⌘T 将改为执行 XXX · 确定？"），默认取消
- [ ] **v0.1 不做**：自动 remap（冲突时建议 `⌘⇧T` 等替代键），推到 v0.2+

### E. 边界情况

- [ ] 配置文件不存在 → "未检测到 X 配置，可手动选择文件"；分档提示：完全未检测 / 检测到但解析失败 / 部分字段缺失，每档独立文案，用户可手动 "Browse..." 选文件
- [ ] 配置格式损坏 → 明确错误提示 + 保留当前设置不变；graceful fallback：能解析的字段应用，失败字段列出（"font_family 解析失败 · 跳过"），整体不崩
- [ ] 字体文件在 Vibestation 侧不可用 → fallback 到 JetBrains Mono + 提示；字体存在性检测：`document.fonts.check('12px "Ghostty Import Font"')`，失败 fallback JetBrains Mono，toast 提示 "字体 X 未安装 · 用 JetBrains Mono"

### F. 跨平台

- [ ] mac：Ghostty + iTerm2 都支持
- [ ] Linux：Ghostty + Alacritty 都支持
- [ ] 未列出的 OS 组合（如 Ghostty on Windows）→ 错误提示（MVP 不支持 Windows）

**v0.1 macOS-first 降级表**：

| 平台 | Ghostty | iTerm2 | Alacritty |
|---|---|---|---|
| macOS | ✅ v0.1 | ✅ v0.1 | N/A（linux only）|
| Linux（Ubuntu · v0.1.x 低优先 或 v0.2）| ✅ | N/A（macOS only）| ✅ |
| Windows | ❌ 不支持 · toast 提示 | — | — |

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元（core）| 三种格式解析器 + 字段映射 |
| 集成 | 真实配置文件 fixture 导入端到端 |
| 手动 QA | 三个平台准备真实用户配置，对比导入前后 |

## 💾 数据模型变更

无新 table。导入结果写入 `app_settings`：
- `font_family` / `font_size` / `theme` / `shell` / `keybindings`

## ⚠️ 已知风险

- **iTerm2 plist binary format**：需要 `plist` crate 支持 binary plist（文本 plist 罕见）
- **Ghostty 配置演进**：Ghostty 还在活跃开发，TOML schema 可能变 → 解析器加 version 检测，不识别的字段跳过（warn）
- **Alacritty 格式迁移**：0.14+ 切 TOML，老用户仍用 YAML → 需双格式支持，增加解析器维护面

## 📝 Notes

- MVP-06 不做"导出 Vibestation 配置回到原终端"——单向导入即可
- `§10.5` 降级：若投入 ≤ 15h，仅做 Ghostty（覆盖主 persona）

## 🔗 相关

- `implementation-plan.md` §10.1 · §10.5 降级树
- 上游：MVP-04
- 下游：无

---

## §G. IPC Contract（ts-rs）

### G.1 预期 IPC struct 清单

| Rust struct | 用途 | 前端 import |
|---|---|---|
| `ImportSource` | 导入源枚举 · string union | `./bindings/ImportSource` |
| `ImportScanResult` | 扫描单个源的结果（path_exists · parsed_fields · errors）| `./bindings/ImportScanResult` |
| `ImportPreview` | 预览数据（跨 source 合并 · 用户可勾选字段）| `./bindings/ImportPreview` |
| `ImportFieldType` | 字段类型（font/theme/shell/keybindings · tagged union 含 payload）| `./bindings/ImportFieldType` |
| `ImportApplyRequest` | 应用导入（source + 勾选字段集合）| `./bindings/ImportApplyRequest` |
| `ImportApplyResult` | 应用结果（applied · conflicts · errors）| `./bindings/ImportApplyResult` |
| `KeyBindingConflict` | 冲突描述（vibe_key · source_key · resolution · 用户决策）| `./bindings/KeyBindingConflict` |

### G.2 derive 模板

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ImportSource { Ghostty, ITerm2, Alacritty }

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ImportFieldType {
    FontFamily { value: String },
    FontSize {
        #[ts(type = "number")]
        value: f32
    },
    Theme { value: String },
    Shell { value: String },
    KeyBinding { key: String, action: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ImportScanResult {
    pub source: ImportSource,
    pub path_exists: bool,
    pub detected_fields: Vec<ImportFieldType>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct KeyBindingConflict {
    pub vibe_key: String,       // canonical form · e.g. "Cmd+T"
    pub source_key: String,     // canonical form · e.g. "Cmd+T"
    pub vibe_action: String,    // e.g. "tabs.create"
    pub source_action: String,  // e.g. "new_tab"（原终端 action 名）
    pub user_choice: KeyBindingResolution, // keep_vibe / override
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum KeyBindingResolution { KeepVibe, Override }
```

### G.3 强制规范

- 所有 IPC struct `#[derive(TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
- `ImportFieldType` 因含 payload，必须 tagged union（`#[serde(tag = "kind")]`），不能 string union
- `ImportSource` / `KeyBindingResolution` 简单 enum，string union（仅 `rename_all`，无 tag）
- `ImportFieldType::FontSize.value: f32` 加 `#[ts(type = "number")]`（前端 TS 默认生成 `bigint`，强制 `number`）
- bindings 由 `build.rs` 生成，前端禁手写 interface，H2 regression proof 引用 MVP-04 §G.3

---

## §H. 配置导入决策锁定

### H.1 · 解析库选型

- Ghostty TOML · **`toml` crate 0.8+**（Rust ecosystem 标准 · 已在 workspace 依赖）
- iTerm2 plist · **`plist` crate 1.6+**（支持 binary plist · 必选 binary 能力）
- Alacritty TOML + YAML fallback · **`toml` crate 复用** + **`serde_yaml` 0.9**（YAML fallback · deprecated 但仍有 Alacritty 0.13- 用户）
- **禁止**：
  - 引入第 2 个 TOML parser（如 `toml_edit` · 除非需要往回写 · MVP-06 只读所以不需要）
  - 手写 plist binary 解析（`plist` crate 已成熟 · 自己写是 YAGNI）
  - 依赖 Python / Node.js / GUI 工具（Vibestation 是 Rust 单体）
  - 第 4 个解析库（Windows Terminal JSON 等 · 推到 v0.2+）

### H.2 · 三家导入优先级 + 降级树

| 投入 | 覆盖 |
|---|---|
| ≤ 15h | 仅 Ghostty（macOS + Linux · 覆盖主 persona C） |
| 15-24h | Ghostty + iTerm2（macOS 完整覆盖） |
| 24h+（本 MVP 估 3d ≈ 24h）| 三家全做 |
| **v0.1 macOS-first 场景** | Alacritty 延到 v0.2 · MVP-06 实施时不做 Alacritty · 其 IPC struct 预留但不 wire · Spec 正文明示此场景 |

### H.3 · 快捷键冲突用户决策

- 默认行为：冲突时**保留 Vibestation 原值** + 黄色标记提示
- 用户决策：复选框"替换 Vibestation 快捷键" + 二次确认
- **v0.1 不做自动 remap**（如 `⌘T` 冲突时建议 `⌘⇧T`）· 推到 v0.2+
- **锁定 canonical form 算法**：`Modifier 按 Cmd > Ctrl > Alt > Shift 排序 + 大写 Key`（例 `⌘T` = `Cmd+T` = `Meta+t` 统一为 `Cmd+T`）

### H.4 · 字体 fallback 链

- 导入的 font_family 在本机不可用 → fallback `JetBrains Mono`（原型定义）+ toast 提示
- **v0.1 不做**：自动下载字体（涉及网络 · 隐私合规 · telemetry 默认关冲突）
- **锁定字体检测 API**：`document.fonts.check('12px "Target Font"')`（Web API · 前端检测 · 不需 Tauri plugin）

---

## 📋 上游依赖状态检查

| 依赖 | 状态 | 评估 |
|---|---|---|
| MVP-04 | ready · Phase A storage done（PR #72 · `app_settings` 表已存在）| 解阻塞 · MVP-06 直接 write app_settings |

---

**自审四问**（2026-04-21）：

1. **递归完备性**：Acceptance A-F + §G IPC + §H 决策锁定 + 边界（v0.1 macOS-first / 字体 fallback / 格式损坏）全覆盖 ✅
2. **反向场景**：配置损坏 graceful · 冲突二次确认 · 字体不可用 fallback · Alacritty 缺失降级 ✅
3. **边界适用性**：macOS / Linux 分测 · Windows 错误提示（MVP 不支持）· v0.1 macOS-first 有显式降级表 ✅
4. **YAGNI**：Windows Terminal / Warp / Kitty / 双向同步 / 多 profile / 自动 remap 都推后 ✅
