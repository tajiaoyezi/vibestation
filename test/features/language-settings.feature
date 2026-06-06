# Maps to: docs/tasks/FEAT-02-language-settings.md
#
# 轻量 BDD（S2V）：本 .feature 作业务可读场景文档，Scenario ID 映射到 Rust / vitest / runtime smoke。
# 主题：应用语言设置默认 English，可切换简体中文；持久化走 app_settings.language；前端固定 UI 文案走 typed dictionary。

Feature: language-settings
  In order to 在默认英文界面的同时支持中文用户
  As a Vibestation 用户
  I want 应用语言默认 English，并能在 Preferences 中切换为简体中文

  Background:
    Given 应用已经初始化 workspace database
    And settings_get / settings_update / settings_changed 链路可用
    And 当前没有 OS locale 自动跟随需求

  Scenario: SCEN-FEAT-02.1 — 首次启动默认 English
    Given app_settings 中不存在 language key
    When 前端加载 settings_get 的结果
    Then settings.language 为 "en"
    And document.documentElement.lang 为 "en"
    And Settings / Sidebar / status 等已接入区域显示 English 文案

  Scenario: SCEN-FEAT-02.2 — 用户在 Preferences 中切换为简体中文
    Given Preferences → Appearance 显示 Language 下拉
    When 用户选择 "简体中文"
    Then 前端调用 settings_update，payload 中 language 为 "zh-Hans"
    And settings_changed 事件返回 language 为 "zh-Hans"
    And 已接入 i18n 的 UI 文案无需重启即刷新为简体中文

  Scenario: SCEN-FEAT-02.3 — 语言设置跨重启保持
    Given 用户已将 language 设置为 "zh-Hans"
    When 应用关闭后重新启动
    Then settings_get 返回 language 为 "zh-Hans"
    And Settings / Sidebar / status 等已接入区域显示简体中文文案

  Scenario: SCEN-FEAT-02.4 — 损坏的持久化语言值回退 English
    Given app_settings.language 被外部写成非法值 "fr"
    When 后端执行 settings_get
    Then 返回给前端的 language 为 "en"
    And 前端不 crash
    And Language 下拉显示 "English"

  Scenario: SCEN-FEAT-02.5 — 两套字典 key 必须完全一致
    Given en dictionary 包含所有已迁移 UI key
    When zh-Hans dictionary 缺少任一 key
    Then vitest 字典完整性测试失败
    And 缺失 key 的 locale 不允许合入

  Scenario: SCEN-FEAT-02.6 — 第一批 app chrome 文案随语言变化
    Given Settings 面板、Primary Sidebar、Activity Strip、TopBar、Bottom status 已接入 t()
    When 用户在 English 与简体中文之间切换
    Then 这些区域的按钮、标题、tooltip、empty state 文案随语言刷新
    And 已列入 FEAT-02.4 checklist 的硬编码中文通过 en canonical dictionary 在默认语言下显示 English
    And 终端输出、Git 内容、commit message、branch name 保持原文

  Scenario: SCEN-FEAT-02.7 — 语言功能不引入运行期外部服务
    Given 应用离线启动
    When 用户切换语言
    Then UI 通过本地 dictionary 完成翻译
    And 不发出任何翻译服务网络请求
