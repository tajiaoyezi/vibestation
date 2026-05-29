# Maps to: docs/specs/tasks/task-3.4-fs-watch-windows.md
#
# 轻量 BDD（s2v §9.2）：本 .feature 作业务可读场景文档，Scenario ID 映射到 Rust 测试。
# 关联 ADR：adr-006（fs_watch Windows 启用 notify backend）。
# 主题：crates/core/src/fs_watch.rs 移除 Windows UnsupportedPlatform 短路，启用 notify 的 Windows
#       backend（ReadDirectoryChangesW），保持 200ms debounce + .git/index.lock 排除契约不变。

Feature: fs-watch
  In order to 让 Windows 用户在 pwsh Tab 改文件后 Git status 徽章实时刷新
  As a Windows 11 上用 Claude/Codex CLI 的开发者
  I want GitStatusWatcher 在 Windows 不再短路，复用 notify 跨平台 backend，契约（debounce / 排除）不变

  Background:
    Given 开发机为 Windows 11 x64
    And 当前 fs_watch.rs 在 Windows 直接返回 GitStatusWatchError::UnsupportedPlatform
    And GIT_STATUS_WATCH_DEBOUNCE = 200ms 为跨平台契约常量

  Scenario: SCEN-3.4.1 — Windows 上 GitStatusWatcher 成功 spawn
    Given Windows 的 UnsupportedPlatform 短路已移除，启用 notify Windows backend
    When 在 Windows 的 git 仓库上调用 GitStatusWatcher::spawn()
    Then 返回 Ok（watcher 正常建立），不再返回 UnsupportedPlatform
    And macOS/Linux 的 spawn 路径行为不变

  Scenario: SCEN-3.4.2 — Windows workspace 改文件触发 status 刷新
    Given Windows 的 git workspace 已被 watch
    When 在该 workspace 内修改一个被 git 跟踪的文件
    Then Git status 在 200ms（GIT_STATUS_WATCH_DEBOUNCE）debounce 窗口内刷新一次
    And 刷新事件经 ReadDirectoryChangesW 经 notify 上抛

  Scenario: SCEN-3.4.3 — .git/index.lock 仍被排除（契约不变）
    Given watcher 对 .git/index.lock 的写入应被忽略（避免 git 操作自触发风暴）
    When 在 Windows 上 git 操作产生 .git/index.lock 抖动
    Then 该路径变更不触发 status 刷新
    And 排除契约与 Unix 平台一致
