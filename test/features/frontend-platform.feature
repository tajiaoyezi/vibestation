# Maps to: docs/specs/tasks/task-4.1-platform-windows-class.md
# Maps to: docs/specs/tasks/task-4.2-shortcut-display.md
#
# 轻量 BDD（s2v §9.2）：本 .feature 作业务可读场景文档，Scenario ID 映射到 vitest 测试。
# 关联：纯前端（web/src），无 Rust 依赖；键盘事件处理已正确（survey already_windows_ok），本组只修“显示”。
# 主题：web/src/index.tsx 补 platform-windows class + data-platform 属性
#       + components/{TopBar,ActivityStrip} / panels/{Terminal,CommitBar} / styles.css 的平台感知快捷键显示助手
#       （Windows 显示 Ctrl+B 而非 ⌘B）。

Feature: frontend-platform
  In order to 让 Windows 用户看到与实际按键一致的快捷键提示（Ctrl 而非 ⌘）
  As a Windows 11 上用 Claude/Codex CLI 的开发者
  I want 前端在 Windows 发 platform-windows class，且所有快捷键提示按平台显示 Ctrl/⌘

  Background:
    Given 浏览器环境 navigator.platform 返回 "Win32"
    And 当前 index.tsx 只发 platform-macos / platform-linux class（Windows 无 class）
    And 键盘事件处理（matchesMod / isMacPlatform）已正确区分 Windows（无需改动）

  # ---- task 4.1 · platform-windows class ----

  Scenario: SCEN-4.1.1 — Windows 上 html 元素带 platform-windows class
    Given index.tsx 增加 navigator.platform 含 "win" 的检测分支
    When 在 Windows 浏览器/WebView2 环境加载应用
    Then document.documentElement.classList 含 "platform-windows"
    And macOS 仍发 platform-macos、Linux 仍发 platform-linux，零回归

  Scenario: SCEN-4.1.2 — root 设置 data-platform 供 CSS-only 提示切换
    Given styles.css 依赖 :root[data-platform="windows"] 切换 maximized chip 文案
    When 在 Windows 上初始化平台检测
    Then root 元素 data-platform 属性值为 "windows"
    And maximized chip ::before 文案显示 "maximized · Ctrl+Enter or Esc"（非 ⌘Enter）

  # ---- task 4.2 · 平台感知快捷键显示 ----

  Scenario: SCEN-4.2.1 — 快捷键显示助手在 Windows 返回 Ctrl 文案
    Given 引入 platformShortcut(mac, other) 助手（isMac ? mac : other）
    When 在 Windows 上格式化 Toggle Primary Sidebar 的快捷键提示
    Then 返回 "Ctrl+B"（而非 "⌘B"）
    And 在 macOS 上返回 "⌘B"，零回归

  Scenario: SCEN-4.2.2 — Pane 右键菜单与按钮提示在 Windows 显示 Ctrl
    Given PaneTerminal 的 split/close/context-menu title 用平台感知助手
    When 在 Windows 上 hover/右键 Pane 操作按钮
    Then 提示显示 "Ctrl+Shift+O" / "Ctrl+\" 等（非 ⌘⇧O / ⌘\）
    And ActivityStrip（⌘2/⌘J）、CommitBar（⌘↵）、Settings（⌘,）提示同步按平台显示
