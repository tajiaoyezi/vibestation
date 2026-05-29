# Maps to: docs/specs/tasks/task-1.1-pty-platform-split.md
# Maps to: docs/specs/tasks/task-2.1-windows-shell-detection.md
# Maps to: docs/specs/tasks/task-2.2-conpty-spawn-io.md
#
# 轻量 BDD（s2v §9.2）：本 .feature 作业务可读场景文档，Scenario ID 映射到 Rust 测试。
# 关联 ADR：adr-001（PTY cfg 分离 + portable-pty ConPTY）/ adr-003（shell 探测链）/ adr-005（测试门控）。
# 主题：crates/core/src/pty.rs 的 Unix-only 内核（mio::unix / libc 信号 / fcntl / PermissionsExt）
#       按 #[cfg(target_os)] 分离，Windows 走 portable-pty 的 ConPTY 阻塞读路径，mac/Linux 零回归。

Feature: pty
  In order to 让 Windows 11 上的 AI-agent 开发者能在多 Tab 终端经 ConPTY 拉起 pwsh/cmd 并正常读写
  As a Windows 11 上用 Claude/Codex CLI 的开发者
  I want crates/core 的 PTY 子系统在 Windows 上可编译、能探测 shell、能拉起进程并检测退出

  Background:
    Given 开发机为 Windows 11 x64（MSVC toolchain）
    And crates/core 当前 PTY 内核为 Unix-only（mio::unix::SourceFd + libc::kill/tcgetpgrp/fcntl + PermissionsExt）

  # ---- task 1.1 · PTY 平台分离（解锁编译）----

  Scenario: SCEN-1.1.1 — crates/core 在 Windows 编译通过且 Unix 路径零回归
    Given pty.rs 的 mio::unix / libc 信号 / fcntl / PermissionsExt 已按 #[cfg(unix)] / #[cfg(windows)] 分离
    When 在 Windows 上跑 cargo build --workspace
    Then 编译零错误
    And 在 macOS / Linux 上 cargo build --workspace 仍 0 错误，Unix reader loop / 信号 / fd 行为不变

  Scenario: SCEN-1.1.2 — Windows 文件可执行性判定不走 Unix mode bits
    Given is_executable_file 在 Unix 用 (mode & 0o111) != 0，在 Windows 应改判文件存在性
    When 在 Windows 上对 pwsh.exe / cmd.exe 路径调用 is_executable_file
    Then 返回 true（按 .exe/.bat 扩展名与文件存在判定，不触碰 PermissionsExt）
    And Unix 分支仍按 0o111 mode 位判定，行为不变

  Scenario: SCEN-1.1.3 — Windows 信号路由退化为单进程终止
    Given Unix 用 libc::kill + tcgetpgrp 做前台进程组信号，Windows 无进程组概念
    When 在 Windows 上对一个 PtySession 调用 .signal("SIGTERM")
    Then 走 child.kill() / TerminateProcess 等价路径，不引用 libc 常量
    And Unix 的 SIGINT/SIGTERM/SIGTSTP 经进程组传递行为零回归

  # ---- task 2.1 · Windows shell 探测 ----

  Scenario: SCEN-2.1.1 — Windows 默认 shell 走 pwsh→powershell→cmd 探测链
    Given Windows 无 /etc/shells，且未必装 PowerShell 7
    When 调用 resolve_default_shell(None)
    Then 返回探测链 pwsh.exe → powershell.exe → cmd.exe 中第一个可用者的完整路径
    And 绝不返回 /bin/bash 或任何 Unix 路径

  Scenario: SCEN-2.1.2 — Windows shell 枚举走 PATH/where 而非 /etc/shells
    Given Windows 上 list_available_shells 不能读 /etc/shells
    When 调用 list_available_shells()
    Then 返回经 PATH/where 探测到的 Windows shell（含 cmd.exe 保底），列表非空
    And Unix 分支仍读 /etc/shells + 过滤 PRIMARY_SHELL_BASENAMES，行为不变

  Scenario: SCEN-2.1.3 — 未装 pwsh 时回落不崩溃
    Given 系统未安装 PowerShell 7（无 pwsh.exe）
    When 解析默认 shell
    Then 回落 powershell.exe（5.1 内置），再回落 cmd.exe
    And 不因找不到 pwsh 而 panic 或拉起不存在的 shell

  # ---- task 2.2 · ConPTY spawn / IO / 退出检测 ----

  Scenario: SCEN-2.2.1 — Windows 经 ConPTY 拉起 cmd.exe 并收到回显
    Given pty.rs Windows reader 路径用 portable-pty 的 ConPTY backend
    When spawn 一个 cmd.exe 会话并写入 "echo hello\r\n"
    Then PtySession 收到 stdout 事件，输出含 "hello"
    And reader 循环不 hang、不丢尾部输出

  Scenario: SCEN-2.2.2 — Windows 进程退出被正确检测
    Given ConPTY 无 Unix kqueue/epoll close event，退出靠 child.try_wait()
    When 被拉起的进程自行退出（如 cmd.exe 跑 "exit"）
    Then PtySession 在阻塞读循环中经 child.try_wait() 检测到退出并发出 exit 事件
    And 不发生读循环挂死

  Scenario: SCEN-2.2.3 — Windows cwd 用 spawn-time 缓存兜底
    Given detect_process_cwd 在 Windows 无精确实现（Out of Scope）
    When 查询某 tab 的 working_directory
    Then 返回 spawn 时缓存的 initial_cwd（MVP 兜底），不返回 None 导致功能缺失
    And 不 panic
