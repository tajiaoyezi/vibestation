# Maps to: docs/specs/tasks/task-5.1-windows-bundle.md
# Maps to: docs/specs/tasks/task-5.2-windows-ci-matrix.md
# Maps to: docs/specs/tasks/task-5.3-prepare-titlebar.md
# Maps to: docs/specs/tasks/task-6.1-windows-test-gating.md
# Maps to: docs/specs/tasks/task-6.2-windows-smoke-matrix.md
#
# 轻量 BDD（s2v §9.2）：本 .feature 作业务可读场景文档，Scenario ID 映射到 Rust/CI/vitest 断言测试。
# 关联 ADR：adr-004（Windows 安装包 NSIS 主 + MSI 辅，unsigned MVP）/ adr-005（Windows 测试门控策略）。
# 主题：tauri.conf.json 增 msi/nsis bundle + 窗口装饰条件化 / ci.yml 加 windows-latest 矩阵
#       / package.json prepare 跨平台 / Windows-gated 测试集 / 端到端 smoke 三平台零回归。

Feature: build-and-integration
  In order to 产出可安装的 Windows .exe/.msi，并让 windows-latest CI 在合入前抓住 Windows 编译/测试回归
  As a 项目 Windows 贡献者 / windows-latest CI / 维护者
  I want Windows bundle 配置、CI 矩阵、跨平台 prepare、测试门控与端到端 smoke 全部就位且三平台零回归

  Background:
    Given 开发机为 Windows 11 x64（MSVC toolchain + WebView2）
    And 当前 tauri.conf.json bundle targets 仅 dmg/appimage/deb，无 msi/nsis
    And 当前 .github/workflows/ci.yml 所有 job runs-on: ubuntu-latest

  # ---- task 5.1 · Windows bundle ----

  Scenario: SCEN-5.1.1 — tauri.conf.json 声明 Windows bundle targets
    Given bundle.targets 增加 "nsis"（主）+ "msi"（辅）
    When 解析 crates/app/tauri.conf.json 的 bundle.targets 数组
    Then 数组含 "nsis" 与 "msi" 两个 Windows target
    And 仍保留 dmg/appimage/deb，mac/Linux 打包零回归

  Scenario: SCEN-5.1.2 — Windows 上产出可安装 .exe
    Given Windows runner 具备 MSVC + WebView2 + NSIS
    When 跑 pnpm tauri:build --bundles nsis
    Then 在 bundle 目录产出可安装的 .exe（unsigned MVP）

  # ---- task 5.2 · windows-latest CI 矩阵 ----

  Scenario: SCEN-5.2.1 — CI 增加 windows-latest 矩阵跑 build+test
    Given rust-build job 改为 matrix runs-on: [ubuntu-latest, windows-latest]
    When 解析 .github/workflows/ci.yml
    Then 矩阵含 windows-latest，且 Windows job 跑 cargo build --workspace + cargo test --workspace
    And Linux-only 步骤（如 Xvfb）在 Windows 条件化跳过

  # ---- task 5.3 · prepare 跨平台 + 窗口装饰条件化 ----

  Scenario: SCEN-5.3.1 — package.json prepare 在 Windows 不静默失败
    Given prepare 脚本不再用 bash 专属 2>/dev/null 重定向，改用跨平台 Node 一行脚本
    When 在 Windows 上跑 pnpm install
    Then .git/config 的 core.hooksPath 被正确设为 .githooks（pre-push hook 生效）
    And macOS/Linux 的 hook 安装行为不变

  Scenario: SCEN-5.3.2 — 窗口装饰在 Windows 条件化不留 macOS 样式残影
    Given configure_title_bar 增加 #[cfg(target_os = "windows")] 分支（或安全 stub）
    When 在 Windows 上构建并启动窗口
    Then 窗口正常渲染，无 macOS 专属 trafficLight/hudWindow 样式残影
    And macOS titleBar overlay 行为零回归

  # ---- task 6.1 · Windows 测试门控 ----

  Scenario: SCEN-6.1.1 — Unix-only 行为测试在 Windows 被 skip 而非 panic
    Given shell_compat / git_ops_integration / pty_*_integration / env_filter 用 #[cfg(unix)] 或 #[cfg_attr(windows, ignore="...")] 门控
    When 在 Windows 上跑 cargo test --workspace
    Then Unix-only 用例显示 ignored 而非 panicked（如 /etc/shells、chmod 0o755、#!/bin/sh hook）
    And 补充的 Windows 专属测试正常执行并通过

  # ---- task 6.2 · 端到端 smoke + 三平台矩阵 ----

  Scenario: SCEN-6.2.1 — Windows 端到端 smoke 通过
    Given Windows 适配全部 phase 已合入
    When 在 Windows 11 本机跑 pnpm tauri:dev 并执行 critical UX path（新建 Tab → pwsh prompt → git --version 回显 → 改文件看 status 刷新）
    Then 各步骤行为正确，无 panic
    And 冷启动 < 500ms

  Scenario: SCEN-6.2.2 — 三平台测试矩阵零回归
    Given CI 跑 windows-latest + ubuntu-latest（+ macOS 由 reviewer/CI 保证）
    When 在三平台跑 cargo test --workspace + vitest
    Then macOS + Ubuntu 现有测试仍 100% 绿（反指标硬约束）
    And Windows-gated 测试集全绿，所有改动均经 #[cfg(target_os)] 分支
