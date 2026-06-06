---
id: FEAT-02
type: feat
title: 语言设置与简体中文界面
status: in-progress
owner:
phase: 当前 active scope
depends_on: []
blocks: []
estimate: 3d
plan_ref: implementation-plan.md §3.1 / §10.1
risk_ref:
reviewer:
---

# FEAT-02: 语言设置与简体中文界面

> **状态**：`draft` → `ready` → `in-progress` → `done`
> **S2V 产物**：BDD feature [`test/features/language-settings.feature`](../../test/features/language-settings.feature) · ADR proposed [`ADR-025`](../adr/ADR-025-frontend-i18n-dictionary.md)

---

## 🎯 目标（Goal）

为 Vibestation 增加可持久化的应用语言设置：默认显示 English，用户可在 Preferences 中切换到简体中文，并让已接入的应用 UI 立即刷新。

## 📖 背景（Context）

- 当前设置链路已存在：`AppSettings` / `SettingsUpdateRequest` 经 `ts-rs` 生成前端 binding，前端 `stores/settings.ts` 通过 `settings_get` / `settings_update` / `settings_changed` 同步全局设置。
- 当前 UI 文案主要是直接写在 SolidJS 组件里的 English 字符串；新增语言设置若继续内联字符串，会导致后续翻译难以验证。
- 用户明确要求默认 English，且可修改为简体中文。因此第一版不做 OS locale 自动跟随，避免 Windows 中文系统上首次启动违背默认 English。

---

## 🎨 功能范围（Scope）

**Do**：

- 在 `app_settings` KV 中新增 `language` 设置，允许值仅为 `en` / `zh-Hans`，缺失或损坏值回退 `en`。
- 在 Preferences → Appearance 中新增 `Language` 下拉，选项为 `English` 与 `简体中文`。
- 新增前端 typed i18n dictionary，所有 locale 必须拥有完全一致的 key 集合，测试阻止漏翻译。
- 复用现有 `settings_get` / `settings_update` / `settings_changed` 链路，让切换语言后 UI 实时刷新，无需重启。
- 切换语言时通过 settings store 的集中 helper 同步 `document.documentElement.lang`，`en` 对应 `en`，`zh-Hans` 对应 `zh-Hans`；初始化加载、显式 reload、`settings_changed` 和 `settings_update` 返回路径都必须经过同一副作用入口，UI `onChange` 不直接写 `document.lang`。
- 第一批迁移范围覆盖应用 chrome：Settings 面板、Primary Sidebar / Activity Strip / TopBar / Bottom status、常见按钮 / tooltip / empty state，以及 FEAT-02.4 checklist 明确列出的前端自写常见错误提示。

**Don't**（显式排除，避免 scope creep）：

- 不翻译终端输出、shell/Git/系统命令原始错误、仓库文件内容、commit message、branch name。
- 不翻译 README、官网、docs、release notes 等文档/营销内容。
- 不在第一版引入 OS locale 自动检测；默认必须稳定为 English。
- 不引入远程翻译服务、运行期网络依赖或大体量第三方 i18n 框架。
- 不把 Tauri 原生菜单 label 纳入第一版；若需要随语言动态重建 native menu，后续另开 task。

## 🖼 UI 引用（UI Reference）

- 设置入口：`web/src/panels/Settings/SettingsPanel.tsx`
- 外观设置组：`web/src/panels/Settings/AppearanceGroup.tsx`
- 当前设置 store：`web/src/stores/settings.ts`
- 视觉基线：`design/directions/1-calm-studio.html` 的 Settings / drawer 类控制密度；新增控件应复用现有 `vs-settings-field` / `vs-settings-select` 风格。

## ✅ Acceptance

- [ ] **AC1 默认 English**：全新安装或 DB 中没有 `language` key 时，`settings_get` 返回 `language = "en"`，UI 首屏文案为 English，`document.documentElement.lang = "en"`。
- [ ] **AC2 用户可切换简体中文**：Preferences → Appearance 中显示 `Language` 下拉；选择 `简体中文` 后调用 `settings_update({ language: "zh-Hans" })` 并持久化。
- [ ] **AC3 即时刷新**：语言切换成功后，已接入 i18n 的 UI 文案在当前窗口立即刷新，无需重启或重新打开 workspace。
- [ ] **AC4 跨重启保持**：关闭并重新启动应用后，上一轮选择的 `zh-Hans` 仍然生效。
- [ ] **AC5 损坏值安全回退**：若 DB 中 `language` 为非法值（例如 `fr` / 空字符串 / `zh`），后端返回 `en`，前端不 crash，Settings 下拉显示 English；读取路径不回写 DB，后续 `settings_update` 会写入规范化后的合法值。
- [ ] **AC6 字典 key 完整性**：`en` 与 `zh-Hans` 的 dictionary key 完全一致；任一 locale 缺 key 时 vitest 失败。
- [ ] **AC7 第一批迁移范围达标**：FEAT-02.4 checklist 列出的 Settings 面板、Primary Sidebar / Activity Strip / TopBar / Bottom status、常见按钮 / tooltip / empty state 与前端自写固定错误随语言变化；终端输出与 Git 内容保持原文。
- [ ] **AC8 无新运行期外部依赖**：实现不新增网络请求，不依赖远程翻译服务；若引入 npm i18n 包，必须先更新 ADR-025 并经评审。

## 🧪 测试策略

| 层次               | 范围                              | 覆盖路径                                                                                          |
| ------------------ | --------------------------------- | ------------------------------------------------------------------------------------------------- |
| Rust unit          | `crates/core/src/app_settings.rs` | 默认 `language = en` · update 持久化 · 非法值 fallback `en`                                       |
| Contract           | `ts-rs` bindings                  | `AppSettings.language` / `SettingsUpdateRequest.language` 生成并由 `pnpm typecheck` 验漂移        |
| Frontend unit      | `web/src/i18n`                    | dictionary key parity · `t()` 查找 · unknown key/fallback 行为                                    |
| Frontend component | Settings / app chrome             | mock Tauri `invoke` + `reloadSettings()` 保留 store 响应性；验证语言下拉 payload 与已迁移文案刷新 |
| Runtime smoke      | `pnpm tauri:dev`                  | English 默认首屏 · 切换简体中文 · 重启后保持                                                      |

## 💾 数据模型变更（如有）

- 表：沿用 `app_settings` KV 表，不新增 rusqlite table。
- Key：`language`
- Value：`en` / `zh-Hans`
- 迁移策略：无 schema migration；旧 DB 缺 key 时由 `AppSettings::default()` / `get_all` fallback 到 `en`。
- 损坏值策略：后端读取时 normalize + validate；非法值不向前端传播，返回 `en`。`settings_get` / `get_all` 保持无副作用，不在读取时回写 DB；`settings_update` 收到非法 language 时写入规范化后的 `en`。

## §7 追踪表（AC ↔ SCEN ↔ TEST）

| AC  | BDD Scenario   | 预期测试                                                                                                                                  | Status      |
| --- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| AC1 | SCEN-FEAT-02.1 | `app_settings_language_default_is_en` · `app_settings_language_get_all_defaults_to_en_when_empty` · `language_default_sets_document_lang` | Done        |
| AC2 | SCEN-FEAT-02.2 | `settings_language_selector_updates_language`                                                                                             | Done        |
| AC3 | SCEN-FEAT-02.2 | `language_change_updates_translated_chrome_without_restart`                                                                               | In Progress |
| AC4 | SCEN-FEAT-02.3 | `app_settings_language_persists_across_get_all` · `app_settings_language_persists_across_pool_reopen` · runtime smoke                     | In Progress |
| AC5 | SCEN-FEAT-02.4 | `app_settings_language_invalid_value_falls_back_to_en`                                                                                    | Done        |
| AC6 | SCEN-FEAT-02.5 | `all_locale_dictionaries_have_identical_keys`                                                                                             | Done        |
| AC7 | SCEN-FEAT-02.6 | component tests for Settings / Sidebar / status labels                                                                                    | In Progress |
| AC8 | SCEN-FEAT-02.7 | package diff / ADR review gate                                                                                                            | In Progress |

## §8 任务拆分（S2V RED → GREEN）

> 本拆分是 FEAT-02 的实施拓扑。每个子任务都必须先提交 RED 测试，再提交 GREEN 实现；执行细节见实施计划 [`docs/superpowers/plans/2026-06-05-language-settings-implementation-plan.md`](../superpowers/plans/2026-06-05-language-settings-implementation-plan.md)。

| 子任务    | 模块                 | 范围                                                                                                                                                                                                                                                                                             | 覆盖 AC               | 依赖                                  | 完成信号                                                                                     |
| --------- | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------- | ------------------------------------- | -------------------------------------------------------------------------------------------- |
| FEAT-02.1 | settings-contract    | Rust `AppSettings` / `SettingsUpdateRequest` 新增 `language`，后端 validate + fallback，ts-rs binding 刷新                                                                                                                                                                                       | AC1 / AC4 / AC5       | 无                                    | `cargo test --workspace` 中 app_settings language 测试通过，`web/src/bindings` 含 `language` |
| FEAT-02.2 | i18n-core            | 新增 `web/src/i18n` typed dictionary、`t()`/locale helper、key parity 测试、`document.lang` 同步 helper；settings store 在 init/reload/event/update 路径集中调用该 helper                                                                                                                        | AC1 / AC3 / AC6 / AC8 | FEAT-02.1 的 `settings.language` 类型 | `web/tests/i18n/*.test.ts` 通过，`en` / `zh-Hans` key 集一致                                 |
| FEAT-02.3 | language-selector    | Codex 本地实施 RED/GREEN，不再等待外部 UI-test agent；Preferences → Appearance 新增 Language 下拉，选择后走 `updateSettings({ language })` 并即时刷新；仅新 selector 的 label/options 接入 `t()`，不得迁移 `Theme`、其他 Settings 组或 app chrome 文案；OpenCode 可在 GREEN 后做只读 scope audit | AC2 / AC3 / AC4       | FEAT-02.1 / FEAT-02.2                 | Settings 组件测试证明 English/简体中文切换 payload 与 UI 状态正确                            |
| FEAT-02.4 | app-chrome-migration | 第一批 app chrome 固定文案迁移到 `t()`：Settings、Primary Sidebar、Activity Strip、TopBar、Bottom status、常见按钮/tooltip/empty state、checklist 明确列出的前端自写常见错误提示                                                                                                                 | AC3 / AC7             | FEAT-02.2 / FEAT-02.3                 | 组件/文本测试证明第一批范围随语言变化；终端/Git 内容不纳入                                   |
| FEAT-02.5 | verification-docs    | Runtime smoke、§10 Completion Notes、FEAT-02 status 翻转准备、ADR-025 accepted 准备                                                                                                                                                                                                              | AC1-AC8               | FEAT-02.1-02.4                        | §9 命令与 runtime smoke 记录完整，spec 可进入 done/ready gate                                |

### FEAT-02.4 首批迁移 checklist

FEAT-02.4 实施前必须以 OpenCode app chrome 盘点结果为边界。未列入本 checklist 的固定文案默认保留 English，不计入 AC7 回归；已存在于下列区域的硬编码中文也必须迁移到 dictionary，避免默认 English 被中文固定文案破坏。

- [x] Settings shell：`Preferences` 标题、`Import...` 按钮、关闭按钮 aria-label、`Appearance` / `Terminal` / `External Terminal` / `Git` / `Privacy` 分组标题。
- [x] Settings Appearance group：`Language` / `Theme` / `Auto` / `Light` / `Dark` / `Font family` / `Font size` / `Background opacity` / `Background blur` / `Window padding` / `Cursor style` / `Block` / `Bar` / `Underline` / `Cursor blink`。
- [x] Settings Terminal / External Terminal / Git / Privacy first-level controls：`Default shell` / `Paste protection` / `PTY warm pool` / `Don't ask again` / `Ask every time` / `Telemetry` / `Collection endpoint` / `View what we collect` 等固定 label、help text、aria-label；长隐私/遥测说明段落见延期项。
- [x] Primary Sidebar：`Workspaces`、`No workspaces yet.`、`Import settings from another terminal`、`Create workspace` aria-label。
- [x] Activity Strip / Bottom Panel：`Git Log`、`Git Status`、`Output`、`Diff`、panel toggle tooltip / aria-label；Git 内容、branch/commit/message 原文排除。
- [x] TopBar / StatusBar / App chrome：window control label（`Minimize` / `Maximize` / `Close`）、`Toggle primary sidebar`、`remote`、`Merge`、settings gear label、`vX · alpha` status pattern、`ipc: connecting...`、`ipc error`、`Dismiss error`。
- [x] Main / Secondary chrome：`Select or create a workspace to get started`、`Back to Terminal`、resize/sidebar aria-label。
- [x] Dialogs phase 1（仅通用 chrome）：Telemetry opt-in 的 `Help improve Vibestation` / `Decline` / `Accept`，Pop to External 的 `Open in External Terminal` / `Don't ask again` / `Cancel` / `Retry` / `No external terminals detected.`，Branch Switcher 的 `No branch matched` / `Loading branches...` / `Switch branch`。
- [x] Dialogs phase 2（仅通用 chrome）：Create Branch、Merge、Cherry-pick、Remote Selector 的标题、`Cancel` / `Confirm` / `Continue`、close aria-label、固定 loading/search/dirty-warning 文案；branch name、commit message、remote 名称等运行期数据保持原文。
- [ ] Dialogs phase 3 remaining（仅通用 chrome）：Config Import、Dirty tree、Force push / delete、auth / sync 等剩余 Git operation dialogs 的 `Cancel`、`Retry`、`Close <name> dialog` aria-label 与 checklist 内固定 empty/loading 文案。
- [ ] Existing non-English hardcoded copy：OpenCode 已发现的 App remote aria 中文句、CreateBranch `取消` / `确认`、删除模态说明、部分 loading 错误；若位于上述 chrome/dialog/common error 范围内，统一迁移为 `en` canonical + `zh-Hans` 对照。
- [ ] Frontend self-written common errors：只迁移 `ipc: connecting...` / `ipc error`、shell 未安装、`No ... detected` 这类第一屏 chrome 或通用 dialog 错误；复杂业务流程错误另开 follow-up。

FEAT-02.4 明确排除：class / role / type / 内部 data 值、字体名、env 列表、动态 backend label、后端原始错误、Tauri native menu。`We collect` / `We do NOT collect` 及其长 bullet 文案作为 `content.privacy.*` follow-up，不阻塞 AC7。

## §9 Verification

实施完成后必须实际运行：

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
pnpm typecheck
pnpm --filter @vibestation/web exec vitest run
pnpm tauri:dev
```

Runtime smoke 记录项：

- 首次启动 English 文案。
- Preferences → Appearance → Language 选择 `简体中文` 后，Settings / Sidebar / status 文案即时变中文。
- 关闭 app 后重新 `pnpm tauri:dev`，仍为简体中文。
- 终端输出与 Git 内容未被翻译。

## §10 Completion Notes

> 2026-06-06 回填：FEAT-02 仍为 `in-progress`。FEAT-02.1-02.3 已完成，FEAT-02.4a（Settings first）、FEAT-02.4b（workspace chrome）与 FEAT-02.4c phase 1（Main/Secondary + common dialogs）/ phase 2（CreateBranch / Merge / CherryPick / RemoteSelector）已完成；dialog phase 3 remaining、runtime smoke、ADR accepted 翻转与最终 done gate 仍待执行。

- RED commit：
  - `8a8c087 test(settings): 增加语言设置契约 RED 测试`
  - `5db114c test(i18n): 增加语言字典 RED 测试`
  - `c9f8ec3 test(settings): 加语言选择器 RED 测试`
  - `ebbe03f test(i18n): 加首批设置文案迁移 RED 测试`
  - `8f3c84d test(i18n): 加设置控件文案迁移 RED 测试`
  - `8cf7478 test(i18n): 加工作台 chrome 文案 RED 测试`
  - `3bcbfa4 test(i18n): 加剩余 chrome 文案 RED 测试`
  - `c1de708 test(i18n): 加 Git 对话框 chrome RED 测试`
- GREEN commit：
  - `1ffc9fa feat(settings): 持久化应用语言设置`
  - `3f806ad feat(i18n): 添加本地语言字典核心`
  - `84f252d test(web): 禁用 Vitest Solid HMR runtime`
  - `5efd8cf feat(settings): 添加语言选择器`
  - `985efc7 feat(i18n): 迁移设置面板文案`
  - `3451e48 feat(i18n): 迁移设置控件文案`
  - `73ec6ca feat(i18n): 迁移工作台 chrome 文案`
  - `decb2e3 feat(i18n): 迁移剩余主界面与常见对话框文案`
  - `2658179 feat(i18n): 迁移 Git 对话框通用文案`
- REFACTOR / review follow-up commit：
  - `a73d84b test(settings): 补语言设置持久化不变量测试`
  - `9c69a42 test(settings): 补外观文案审计跟进`
- Docs / gate record commit：
  - `34a7bed docs(spec): 回填 FEAT-02.4 设置文案迁移记录`
  - `588b4af docs(spec): 回填 FEAT-02.4 设置控件迁移记录`
  - `c9fb43c docs(spec): 记录 FEAT-02.4 Grok 审计跟进`
- 实际验证命令与结果：
  - `cargo test -p vibestation-core app_settings -- --nocapture` PASS（20 passed）。
  - `cargo check -p vibestation-app` PASS，并刷新 `ts-rs` bindings。
  - `pnpm --filter @vibestation/web exec vitest run tests/i18n/chrome-copy.test.ts tests/components/chrome-copy.test.tsx` PASS（2 files / 8 tests）。
  - `pnpm --filter @vibestation/web exec vitest run tests/i18n/language.test.ts tests/i18n/chrome-copy.test.ts tests/components/chrome-copy.test.tsx tests/panels/Settings/ExternalTerminalGroup.test.tsx tests/panels/Settings/language-selector.test.tsx tests/panels/Settings/settings-panel-copy.test.tsx` PASS（6 files / 26 tests）。
  - `pnpm --filter @vibestation/web exec vitest run tests/i18n/chrome-copy.test.ts tests/components/chrome-copy.test.tsx tests/dialogs/chrome-copy.test.tsx` PASS（3 files / 16 tests）。
  - `pnpm --filter @vibestation/web exec vitest run tests/dialogs/PopToExternal/PopToExternalDialog.test.tsx` PASS（1 file / 6 tests）。
  - `pnpm --filter @vibestation/web exec vitest run tests/i18n/language.test.ts tests/i18n/chrome-copy.test.ts tests/components/chrome-copy.test.tsx tests/dialogs/chrome-copy.test.tsx tests/panels/Settings/ExternalTerminalGroup.test.tsx tests/panels/Settings/language-selector.test.tsx tests/panels/Settings/settings-panel-copy.test.tsx` PASS（7 files / 34 tests）。
  - `pnpm --filter @vibestation/web exec vitest run tests/i18n/chrome-copy.test.ts tests/dialogs/chrome-copy.test.tsx` PASS（2 files / 16 tests）。
  - `pnpm --filter @vibestation/web exec vitest run tests/i18n/language.test.ts tests/i18n/chrome-copy.test.ts tests/components/chrome-copy.test.tsx tests/dialogs/chrome-copy.test.tsx tests/dialogs/PopToExternal/PopToExternalDialog.test.tsx tests/panels/Settings/ExternalTerminalGroup.test.tsx tests/panels/Settings/language-selector.test.tsx tests/panels/Settings/settings-panel-copy.test.tsx` PASS（8 files / 45 tests）。
  - `pnpm typecheck` PASS（`tsc --noEmit`）。
  - `git diff --check` PASS。
  - `npx prettier --check` on touched FEAT-02 source/test/spec files PASS。
  - `pnpm --filter @vibestation/web lint` / `pnpm lint` FAILS on pre-existing repository-wide Prettier warnings outside FEAT-02 touched files; latest observed count 103 files.
- 偏离 spec 的说明：
  - 按 Kimi `READY-WITH-NITS` 建议采用 `split-settings-first`，将 FEAT-02.4 拆为 02.4a Settings、02.4b app chrome、02.4c dialogs/common errors；当前声明 02.4a、02.4b 与 02.4c phase 1/phase 2 完成。
  - `We collect` / `We do NOT collect` 及隐私长 bullet 文案保持排除，后续放入 `content.privacy.*`。
  - 动态 shell label/path、external terminal displayName、endpointHost、Git 内容、terminal output、branch name、commit message 继续保持原文。
  - 当前未新增通用插值 helper；remote ahead/behind 的运行期计数只用 dictionary 中的静态前后缀拼接，未把动态数据迁入 dictionary。
- 后续 follow-up：
  - FEAT-02.4c phase 3 remaining：Config Import、Dirty tree、Force push/delete、auth/sync 等剩余 dialog/common error chrome，继续排除业务操作长文案与 backend raw errors。
  - FEAT-02.5：完整 §9 verification、runtime smoke、ADR-025 accepted 准备、FEAT-02 done gate。

## 📝 Notes / 讨论

- 推荐实现 `type Language = "en" | "zh-Hans"`，但后端仍需对持久化字符串做 validate，不能信任 DB 内容。
- 前端组件测试应复用现有 `ExternalTerminalGroup.test.tsx` 模式：mock `@tauri-apps/api/core` 的 `invoke`，再调用 `reloadSettings()` 初始化真实 settings store；避免 mock `useSettings()` 返回静态对象，否则无法覆盖语言切换后的 Solid 响应式刷新。
- 第一批迁移应以“用户启动后立即看到的 chrome + settings 自身”为边界，避免一次性翻完整个复杂工作台导致 PR 过大。
- 若后续需要更多语言，先扩展 dictionary key parity 测试，再逐个新增 locale 文件。

## 🔗 相关

- ADR：[`ADR-025`](../adr/ADR-025-frontend-i18n-dictionary.md)（proposed）
- BDD：[`test/features/language-settings.feature`](../../test/features/language-settings.feature)
- 相关代码入口：`crates/core/src/app_settings.rs` · `web/src/stores/settings.ts` · `web/src/panels/Settings/AppearanceGroup.tsx`

---

**填写完毕后自审**（CLAUDE.md "📝 写规则/清单前的自审四问"）：

1. **递归完备性**：默认值、切换、持久化、fallback、字典完整性、排除范围均有 AC 和测试映射。
2. **反向场景**：若不限制 key parity，会出现半中文半英文且测试不报错；AC6 专门防此类漂移。
3. **边界适用性**：三平台一致；不依赖 OS locale；旧 DB 缺 key 安全回退。
4. **YAGNI**：第一版只做 English / 简体中文与本地 dictionary，不做运行期翻译服务或全仓文档本地化。
