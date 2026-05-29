# Maps to: docs/specs/tasks/task-3.2-config-import-paths.md
# Maps to: docs/specs/tasks/task-3.3-keybinding-platform.md
#
# 轻量 BDD（s2v §9.2）：本 .feature 作业务可读场景文档，Scenario ID 映射到 Rust 测试。
# 关联 ADR：adr-002（跨平台家目录用 dirs crate）。
# 主题：crates/core/src/config_import/{ghostty,alacritty,iterm2,ipc}.rs 走 %APPDATA% 路径
#       + keybinding.rs 的 Cmd↔Ctrl 平台映射（Windows 上 Meta/Super/Win → Ctrl），mac/Linux 零回归。

Feature: config-import
  In order to 让 Windows 用户从 Alacritty/Ghostty（%APPDATA% 配置）导入字体/快捷键且 keybinding Ctrl 正确识别
  As a Windows 11 上用 Claude/Codex CLI 的开发者
  I want config import 探测 Windows 配置路径，keybinding 规范化按平台映射 Cmd/Ctrl

  Background:
    Given 开发机为 Windows 11 x64
    And 当前 alacritty/ghostty scan 只探测 ~/.config 与 macOS Library 路径
    And 当前 keybinding 把 cmd/command/meta/super/win 全映射到 canonical "Cmd"

  # ---- task 3.2 · %APPDATA% 配置导入路径 ----

  Scenario: SCEN-3.2.1 — Windows 上 Alacritty 配置经 %APPDATA% 被发现
    Given Windows 上 %APPDATA%\alacritty\alacritty.toml 存在且含 [font] section
    When 调用 alacritty scan()
    Then path_exists=true 且 font 字段被 detected
    And 同时兼容尝试 ~/.config/alacritty/alacritty.toml（WSL 兼容）

  Scenario: SCEN-3.2.2 — Windows 上 Ghostty 配置经 %APPDATA% 被发现
    Given Windows 上 %APPDATA%\ghostty\config 存在
    When 调用 ghostty scan()
    Then path_exists=true 且字段被 detected
    And macOS/Linux 原有路径探测零回归

  Scenario: SCEN-3.2.3 — iTerm2 在 Windows 安全短路
    Given iTerm2 仅存在于 macOS，Windows 无对应产品
    When 在 Windows 上运行 scan_all_sources()
    Then iTerm2 结果 path_exists=false 且 detected_fields 为空
    And 不构造虚假的 Library/Preferences 路径

  Scenario: SCEN-3.2.4 — prettify_home_path 在 Windows 返回 ~ 前缀
    Given prettify_home_path 在 Windows 用 USERPROFILE / dirs::home_dir()（不只读 HOME）
    When 对 C:\Users\alice\AppData\Roaming\alacritty\config 调用 prettify_home_path()
    Then 返回 ~/AppData/Roaming/... 形式而非整段绝对路径
    And HOME 已设置时仍正常工作

  # ---- task 3.3 · keybinding 平台映射 ----

  Scenario: SCEN-3.3.1 — Windows 上 win+t 规范化为 Ctrl+T
    Given keybinding canonicalize 接受平台参数，在 Windows 把 Meta/Super/Win 映射到 Ctrl
    When 在 Windows 上调用 canonicalize_keybinding("win+t")
    Then 返回 "Ctrl+T"
    And 在 macOS 上同输入仍返回 "Cmd+T"，零回归

  Scenario: SCEN-3.3.2 — Windows 内置快捷键用 Ctrl 使冲突检测生效
    Given vibestation_builtins() 在 Windows 返回 Ctrl+X 形式（而非 Cmd+X）
    When 导入一个 Ctrl+T 快捷键并调用 detect_conflicts()
    Then 正确识别与内置 Ctrl+T 冲突
    And macOS 内置仍为 Cmd+X，冲突检测行为不变
