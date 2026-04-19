---
id: MVP-06
type: mvp
title: 配置导入（Ghostty + iTerm2 + Alacritty）
status: draft
owner:
phase: W7-W8
depends_on: ["MVP-04"]
blocks: []
blocked_by: []
blocked_note:
estimate: 3d
plan_ref: implementation-plan.md §10.1
risk_ref:
reviewer:
---

# MVP-06: 配置导入

> **状态**：`draft`
> **依赖**：MVP-04（终端存在才能应用配置）

---

## 🎯 目标（Goal）

从用户已有的终端配置（Ghostty / iTerm2 / Alacritty）自动导入字体、主题、快捷键、shell 偏好到 Vibestation，降低切换成本。

## 📖 背景（Context）

- `implementation-plan.md §10.1` MVP B 折中方案必做项
- `§10.5` 降级树：≤ 15h 时可砍 iTerm2/Alacritty（只留 Ghostty 覆盖 Persona C）
- 三者配置格式：Ghostty TOML / iTerm2 plist / Alacritty YAML

---

## 🎨 功能范围（Scope）

**Do**：
- 首次启动（欢迎页或设置页）可触发"导入现有配置"
- 自动扫描以下默认路径：
  - Ghostty (mac/linux): `~/.config/ghostty/config` 或 `~/Library/Application Support/com.mitchellh.ghostty/config`
  - iTerm2 (mac): `~/Library/Preferences/com.googlecode.iterm2.plist`
  - Alacritty (linux): `~/.config/alacritty/alacritty.toml`
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

- [ ] 自动扫描路径，文件存在 → 显示"检测到 Ghostty"
- [ ] 解析 TOML，提取 font / theme / shell / keybindings
- [ ] 预览列表显示每项值，用户可勾选
- [ ] 应用后 Vibestation 字体 / 主题 / shell 变更

### B. iTerm2 导入（mac only）

- [ ] plist 解析（`plist` crate）
- [ ] 提取 default profile 的字体 / ANSI colors / shell / font smoothing
- [ ] ANSI colors 映射到 Vibestation CSS 变量

### C. Alacritty 导入（linux only）

- [ ] YAML 解析（`serde_yaml`）
- [ ] 提取 font / colors / key_bindings

### D. 快捷键冲突处理

- [ ] 检测导入的快捷键是否与 Vibestation 内置冲突（`⌘T` 等）
- [ ] 冲突 → 不导入 + 在预览列表用黄色标记"冲突，保留 Vibestation 原值"
- [ ] 用户可强制覆盖（"替换 Vibestation 快捷键"勾选）

### E. 边界情况

- [ ] 配置文件不存在 → "未检测到 X 配置，可手动选择文件"
- [ ] 配置格式损坏 → 明确错误提示 + 保留当前设置不变
- [ ] 字体文件在 Vibestation 侧不可用 → fallback 到 JetBrains Mono + 提示

### F. 跨平台

- [ ] mac：Ghostty + iTerm2 都支持
- [ ] Linux：Ghostty + Alacritty 都支持
- [ ] 未列出的 OS 组合（如 Ghostty on Windows）→ 错误提示（MVP 不支持 Windows）

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

## 📝 Notes

- MVP-06 不做"导出 Vibestation 配置回到原终端"——单向导入即可
- `§10.5` 降级：若投入 ≤ 15h，仅做 Ghostty（覆盖主 persona）

## 🔗 相关

- `implementation-plan.md` §10.1 · §10.5 降级树
- 上游：MVP-04
- 下游：无

---

**自审四问**：1. 三种源 + 冲突 + 边界都覆盖 ✅ · 2. 配置损坏 graceful ✅ · 3. mac/linux 分开测 ✅ · 4. Windows / 导出 / 多 profile 都推后 ✅
