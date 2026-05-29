# Maps to: docs/specs/tasks/task-3.1-external-term-windows.md
#
# 轻量 BDD（s2v §9.2）：本 .feature 作业务可读场景文档，Scenario ID 映射到 Rust 测试。
# 关联 ADR：adr-005（Windows 测试门控策略）。
# 主题：crates/core/src/external_term/{detect,launch,env_filter}.rs 增加 Windows 平台变体与启动配方
#       （DetectionPlatform::Windows / Platform::Windows / where.exe 探测 / wt.exe 启动），mac/Linux 零回归。

Feature: external-term
  In order to 让 Windows 用户能把当前 Pane 弹出到外部终端（Windows Terminal / conhost / pwsh）
  As a Windows 11 上用 Claude/Codex CLI 的开发者
  I want 外部终端检测返回非空列表，且启动命令能在 Windows 正确拉起目标终端

  Background:
    Given 开发机为 Windows 11 x64
    And 当前 detect.rs 的 DetectionPlatform 枚举无 Windows 变体，Windows 上返回空列表
    And 当前 launch.rs 的 current_platform() 把 Windows 当 Platform::Linux 回落

  Scenario: SCEN-3.1.1 — Windows 外部终端检测返回非空列表
    Given DetectionPlatform 增加 Windows 变体，TERMINALS 增加 Windows 条目（windows-terminal / conhost / pwsh）
    And 系统装有 Windows Terminal
    When 调用 detect_terminals()（current_detection_platform 返回 Windows）
    Then 列表非空，至少含 Windows Terminal / Conhost / PowerShell 之一
    And macOS / Linux 检测结果不受影响

  Scenario: SCEN-3.1.2 — Windows 终端探测用 where 而非 which
    Given command_exists 在 Unix 用 which，在 Windows 应用 where.exe（或 which crate）
    When 在 Windows 上调用 command_exists("powershell")
    Then 返回 true（经 where.exe 在 PATH 中找到）
    And Unix 分支仍调用 which，行为不变

  Scenario: SCEN-3.1.3 — Windows Terminal 启动命令配方正确
    Given launch.rs 增加 Platform::Windows 与对应启动配方
    When 调用 build_launch_command("windows-terminal", cwd, "cmd.exe", Platform::Windows)
    Then 返回基于 wt.exe -d <cwd> 的 LaunchCommand（Windows 兼容路径参数）
    And 对 macOS-only 终端（iterm2 / terminal-app）在 Windows 返回 UnsupportedCombination 错误，不误回落

  Scenario: SCEN-3.1.4 — env_filter 在 Windows 展示 ComSpec/PATHEXT
    Given env_filter WHITELIST 在 Windows 增加 COMSPEC/PATHEXT/USERPROFILE/HOMEDRIVE/HOMEPATH
    When 在 Windows 上过滤进程环境变量
    Then COMSPEC 等 Windows 变量被纳入展示（若已设置）
    And Unix 分支仍保留 SHELL，零回归
