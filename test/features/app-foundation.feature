# Maps to: docs/specs/tasks/task-1.2-home-dir-helper.md
# Maps to: docs/specs/tasks/task-1.3-shell-default-setting.md
#
# 轻量 BDD（s2v §9.2）：本 .feature 作业务可读场景文档，Scenario ID 映射到 Rust 测试。
# 关联 ADR：adr-002（跨平台家目录用 dirs crate）/ adr-003（Windows 默认 shell 探测链）。
# 主题：crates/app/src/lib.rs 的跨平台 home_dir() 助手（替换 HOME 硬编码 + "/" 兜底）
#       + crates/core/src/app_settings.rs 的 Windows 默认 shell 分支（替换 /bin/zsh、/bin/bash）。

Feature: app-foundation
  In order to 让 workspace 初始化 / config import / PTY pool refill 在 Windows 拿到正确的家目录与默认 shell
  As a Windows 11 上用 Claude/Codex CLI 的开发者
  I want home_dir() 与 default_shell 默认值在 Windows 解析正确，mac/Linux 零回归

  Background:
    Given 开发机为 Windows 11 x64
    And 当前 crates/app/src/lib.rs 用 std::env::var("HOME") 缺失时回落 PathBuf::from("/")
    And 当前 crates/core/src/app_settings.rs 默认 shell 写死 /bin/zsh（macOS）/ /bin/bash（其他）

  # ---- task 1.2 · 跨平台 home_dir() 助手 ----

  Scenario: SCEN-1.2.1 — Windows 解析到用户家目录而非根目录
    Given home_dir() 助手在 Windows 用 USERPROFILE / dirs::home_dir()
    When 在 Windows 上调用 home_dir()
    Then 返回 C:\Users\<user> 形式的真实家目录
    And 不返回 "/"（Unix 根），workspace_init / config_import_scan / pty_pool::refill 拿到正确根

  Scenario: SCEN-1.2.2 — HOME 未设时用 USERPROFILE 兜底
    Given Windows 上 HOME 环境变量未设置（常见）
    When 调用 home_dir()
    Then 经 USERPROFILE（或 dirs crate）兜底解析出正确家目录
    And config import 不会去扫 /Library/Preferences/... 等虚假 Unix 路径

  Scenario: SCEN-1.2.3 — Unix 家目录解析零回归
    Given 在 macOS / Linux 上 HOME 正常设置
    When 调用 home_dir()
    Then 返回 $HOME 指向的家目录，与适配前行为一致
    And dirs crate 引入不改变 Unix 路径解析结果

  # ---- task 1.3 · Windows 默认 shell 设置 ----

  Scenario: SCEN-1.3.1 — Windows 默认 shell 设置为可用 Windows shell
    Given app_settings 默认 shell 增加 #[cfg(target_os = "windows")] 分支
    When 在 Windows 上创建 workspace 并读取 default_shell 设置
    Then default_shell 为 cmd.exe 或可用的 pwsh/powershell 路径
    And 不为 /bin/bash（在 Windows 不存在，会令 PTY spawn 立即失败）

  Scenario: SCEN-1.3.2 — get_all fallback 在 Windows 不回落 Unix 路径
    Given app_settings get_all 的 fallback 同样按平台分支
    When 在 Windows 上无显式 default_shell 配置而走 fallback
    Then 返回 Windows shell 默认值（cmd.exe / pwsh）
    And macOS 仍 fallback /bin/zsh、Linux 仍 /bin/bash，零回归
