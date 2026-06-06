# ADR-025: 前端 i18n 采用本地 typed dictionary

**状态**：accepted
**日期**：2026-06-05 proposed · 2026-06-06 accepted
**决策者**：Codex CLI（实施）· multi-agent review（Grok / Kimi / OpenCode）· tajiaoyezi（S2V Arbiter flow）
**关联**：FEAT-02 语言设置与简体中文界面

---

## 背景与问题（Context and Problem Statement）

FEAT-02 要求应用默认 English，并允许用户切换到简体中文。当前 Vibestation 前端固定文案直接写在 SolidJS 组件中，设置数据已通过 `AppSettings` / `SettingsUpdateRequest` / `settings_changed` 链路持久化并广播。

如果只在组件里临时写 `language === "zh-Hans" ? "..." : "..."`，短期能工作，但会快速产生三个问题：

1. 字典 key 无法统一验证，容易出现半中文半英文。
2. 后续新增语言时无法知道哪些文案漏翻。
3. 组件逻辑与文案分支耦合，设置面板和 app chrome 会变得难以维护。

本 ADR 决定 FEAT-02 第一版 i18n 的架构边界。

## 决策驱动因素（Decision Drivers）

- **D1 · 默认 English 必须稳定**：用户明确要求默认 English，不能在中文 Windows 系统上自动变中文。
- **D2 · 小范围先行**：第一版只覆盖 app chrome 和 Settings，不应引入大框架或运行期服务。
- **D3 · 可测试**：需要测试能机械证明 `en` 与 `zh-Hans` key 集一致。
- **D4 · 现有设置链路可复用**：语言只是新的 app setting，不需要独立状态通道。

## 考虑的选项（Considered Options）

### 选项 (a) · 本地 typed dictionary（chosen / accepted）

新增 `web/src/i18n/`，用 TypeScript 对 dictionary key 建模；`en` 与 `zh-Hans` 是本地静态对象；测试递归比较 key 集。前端通过 `t(key)` 或等价 helper 取文案，语言值来自 `settings.language`。

### 选项 (b) · 引入 i18next / FormatJS 等第三方框架

功能完整，适合复数、插值、懒加载、多语言规模化。但 FEAT-02 只有 `en` / `zh-Hans` 两种语言，第一批覆盖 app chrome；引入框架会增加依赖、配置和测试面。暂不选。

### 选项 (c) · 组件内联条件分支

改动最少，但不可扩展，无法做 key parity 测试；后续每个组件会复制语言判断。拒绝。

### 选项 (d) · 默认跟随 OS locale

对本地化应用常见，但与本需求“默认 English”冲突。后续可单独新增 `auto` 语言模式；FEAT-02 不做。

## 决策（Decision Outcome）

**选择（accepted）**：选项 (a)。

约束：

1. `language` 持久化值仅允许 `en` / `zh-Hans`。
2. 缺失或非法持久化值回退 `en`。
3. 不做 OS locale 自动跟随；默认值固定 `en`。
4. `en` / `zh-Hans` dictionary 必须通过 key parity 测试。
5. 第一版不引入运行期网络翻译服务；若后续引入第三方 i18n 包，需更新本 ADR。

## 实施约定（Implementation Conventions）

- **Key 命名**：采用 dot-notation 的分层 key，并与 dictionary 对象结构保持一致，例如 `settings.appearance.language`、`settings.groups.terminal`、`chrome.status.remote`。key 使用 lowerCamelCase segment；不采用全大写常量 key。
- **命名空间边界**：`settings.*` 只放 Settings 面板固定 label / help / aria；`chrome.*` 放第一批 app chrome，其中可细分为 `chrome.sidebars.*`、`chrome.activity.*`、`chrome.tabs.*`、`chrome.topbar.*`、`chrome.status.*`、`chrome.window.*`、`chrome.buttons.*`、`chrome.empty.*`、`chrome.dialogs.*`。长段落内容预留 `content.*`，但除 FEAT-02.4 checklist 明确列入者外，不进入本次迁移。
- **Dictionary 结构**：`en` 是 canonical dictionary；`zh-Hans` 必须与 `en` 递归 key parity。新增 key 先加 `en`，再补 `zh-Hans`。
- **运行期 fallback**：`normalizeLanguage()` 只接受 `en` / `zh-Hans`，其他输入回退 `en`。`t()` 查找失败时先尝试 `en` 同 key；若 `en` 也缺失，返回 key 字符串本身，便于测试和调试发现漏 key。
- **插值策略**：FEAT-02 第一批文案优先选择无插值字符串。若首批 chrome 必须包含简单动态值，使用 `{name}` placeholder 和 `Record<string, string | number>` 的最小插值 helper；不引入 ICU plural/date/number formatting。复杂复数、日期、数字本地化另开 ADR/task。
- **长文案策略**：隐私/遥测说明中的 `We collect` / `We do NOT collect` 及 bullet 型长说明不作为 FEAT-02.4 Ready 阻塞项；若后续迁移，应放入 `content.privacy.*` 并单独测试，不混入通用 `chrome.*`。
- **副作用入口**：`document.documentElement.lang` 只由 settings store 的集中 helper 同步；初始化加载、显式 reload、`settings_changed` 事件和 `settings_update` 成功返回都必须经过该 helper。UI selector 的 `onChange` 只提交 settings update，不直接写 DOM。
- **持久化非法值**：后端读取非法 `language` 时返回 `en`，但不在读取路径回写 DB，保持 `settings_get` 无副作用；`settings_update` 会把非法输入规范化为 `en` 后持久化。

## 后果（Consequences）

### 正面

- 依赖面小，符合当前项目已有 SolidJS + TypeScript 模式。
- 字典完整性可由 vitest 机械验证。
- 后续新增 locale 时路径清晰：新增 dictionary 文件 + key parity 测试。

### 负面

- 第一版不处理复杂 ICU plural / date / number formatting。
- key 命名和分组需要 reviewer 守住一致性，否则 dictionary 可能变成无结构的大对象。
- Tauri native menu 动态本地化不在第一版，需要后续 task 处理。

### 风险

- **R1 · 迁移面过大**：一次性替换全前端文案会造成大 PR。缓解：FEAT-02 限定第一批 app chrome，复杂面板后续分批迁移。
- **R2 · 半迁移状态**：未迁移区域仍显示 English。缓解：FEAT-02 AC7 明确第一批范围，未纳入范围不算回归。

## 实施

FEAT-02 实施时：

1. 后端新增 `language` 字段与 fallback；读取非法值不回写，更新非法值写入规范化后的 `en`。
2. 前端新增 `web/src/i18n/` dictionary 与 helper，并按上方 key/fallback/插值约定实现。
3. Settings Appearance 增加 Language selector。
4. 按 FEAT-02.4 checklist 迁移第一批 app chrome 文案；长隐私/遥测内容保留 follow-up 边界。
5. 补 Rust / vitest / runtime smoke 验证。

## 关联

- Task：[`FEAT-02-language-settings.md`](../tasks/FEAT-02-language-settings.md)
- BDD：[`test/features/language-settings.feature`](../../test/features/language-settings.feature)
- 代码入口：`crates/core/src/app_settings.rs` · `web/src/stores/settings.ts` · `web/src/panels/Settings/AppearanceGroup.tsx`

## 自审四问

1. **递归完备性**：本 ADR 已在 FEAT-02.4 第一批迁移完成后翻为 accepted；若后续要修改 CLAUDE.md 决策表，仍需单独走项目级决策流程。
2. **反向场景**：若不做 typed dictionary，会出现 key 漏翻无法测试；本 ADR 用 parity test 约束。
3. **边界适用性**：三平台一致，离线可用，旧 DB 缺 key 回退 English。
4. **YAGNI**：不引入完整 i18n 框架、不做 OS 自动语言、不做 native menu 动态重建。
