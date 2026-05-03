---
id: MVP-21
type: mvp
title: Git Push / Pull / Fetch（远端同步）
status: ready
owner:
phase: v0.2
depends_on: ["MVP-09", "MVP-13"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1（MVP B 折中砍到 v0.2）· §6.2 git_push/pull/fetch IPC · §6.3 git:push/fetch-progress event · §11 W14 路线图 · §11.3 R23 push/pull 错误恢复
risk_ref: R23（auto-updater 网络错误恢复 · 类比 push/pull 网络可恢复性）
reviewer: Claude Code
---

<!--
  历史说明（2026-05-03 rename · 单人项目 v2-D.1 self-review + Arbiter approval）

  本 spec 原 id 与 v0.1 已 done 的同号 spec "Native Feel Quality"
  （PR #119/#125 · 5/5 全 done · 同 id 文件名 `<old-id>-native-feel-quality.md` · old-id 见 git mv history）
  frontmatter id 冲突 · 文件名不同但 id 同号 · 容易污染未来 PR / 引用 / search。

  按详化阶段（vibe sprint 2026-05-01 Worker B）建议方案 [A] · rename 为本 id：
  - v0.2 实施时是新 task · 未来 reference 都用新号 · 不污染 v0.1 同号 spec
  - 本 id（MVP-21）在仓库中无占用 · v0.3+ 占位也无相关条目
  - 文件 rename: 旧同号文件 → MVP-21-git-push-pull-fetch.md
  - 索引 docs/tasks/README.md 一并更新
  - 同 PR 翻 status: draft → ready（Arbiter approval · 2026-05-03）
-->

# MVP-21: Git Push / Pull / Fetch（远端同步）

> **状态**：`ready`（v0.2 候选 · 详化完成度 100% · 2026-05-03 Arbiter approve 翻 ready · 等待认领）
> **依赖**：MVP-09（git2 写路径已通 · commit 基础设施）+ MVP-13（branch CRUD · push/pull 需要在不同 branch 间切换）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md) · v0.2 W14 Push/Pull/Fetch
> **详化时间**：2026-05-01 sprint vibe · Worker B（Claude Code）· 2026-05-03 rename + draft → ready

---

## 🎯 目标（Goal）

在 MVP-09 commit 基础上 + MVP-13 branch CRUD 之后 · 补远端同步三件套（`git push` / `git pull` / `git fetch`）+ SSH/HTTPS auth + merge conflict graceful 处理 · 让用户不必回终端做远端操作。

## 📖 背景（Context）

- **战略地位**：`implementation-plan.md §10.1` MVP B 折中方案明确把 Push/Pull/Fetch 砍到 v0.2 · v0.1 只能本地 commit · v0.2 补"和远端同步"
- **路线图位置**：`§11 W14` 第二个 v0.2 增量 task（在 W13 branch CRUD 之后 · W15 Pane 扩展之前）· 因为 push 通常是在创建 branch + commit 后的下一个工作流环节
- **CLAUDE.md 锁定**：#13 永久锁定（A 栏）· Git 栈 = **写 git2 0.20**（push/pull/fetch backend 全用 git2 · gix 0.70 网络 API 不成熟）
- **历史尝试**：v0.1 阶段 SPIKE-04 §C 已验证 git2 0.20 的本地写路径稳定 · 但**网络层（push/pull/fetch + auth）未在 v0.1 跑过 · v0.2 是首次接触**
- **核心难度**：不在算法 · 在**跨平台 auth + 错误处理**（SSH agent 行为 / HTTPS keychain 集成 / merge conflict graceful / 网络中断恢复）

---

## 🎨 功能范围（Scope）

**Do**：

- **Push**：
  - 推到 origin（默认）或用户指定 remote（dropdown 选 · 多 remote 场景）
  - 支持 push 单分支 · push --all / push --tags 走 v0.3
  - 推送进度 streaming（git2 callback）：UI 显示 X/Y 对象 + 网络速度
  - **Force push 二次确认**（默认禁用 · UI 红色按钮 + 复述 commit SHA + 文案 `"将覆盖 origin/{branch} 的 N 个 commit"` · 二级确认才执行）
- **Pull**：
  - Fetch + Merge / Rebase 两种策略 · 用户在设置面板（MVP-10）选默认 · 也可单次操作选
  - Clean working tree 要求：dirty tree 阻止 pull · 提示用户先 commit / stash / discard
  - Conflict 中止：merge conflict 时 graceful abort（`git merge --abort` 等价）· **不破坏 worktree**（重要）
  - Conflict 处理：v0.2 不内置 mergetool · UI 提示 `"请用终端 git mergetool 解决冲突 · 或 git merge --abort"` · v0.3+ 评估 GUI mergetool
- **Fetch**：
  - 仅刷新远端 refs · 不 merge / rebase
  - 支持 prune（删除本地已不存在于 remote 的 tracking refs）· 复选框
  - 进度 streaming · 同 push
- **Auth**：
  - SSH：默认走系统 ssh-agent（环境变量 `SSH_AUTH_SOCK`）· fallback 到 `~/.ssh/id_rsa` / `id_ed25519` · 用户密钥需密码时弹密码 modal
  - HTTPS：走 git credential helper（macOS keychain / linux libsecret） · 凭证缺失时弹用户名/密码 modal · 凭证错误 retry 1 次后报错
  - **不自己实现 keyring**（用 OS 系统 helper · 由 git2 + libgit2 调用）
- **状态栏 ahead/behind**：fetch 后 status bar 显示 `↑N ↓M`（push 后清零 ↑ · pull 后清零 ↓）· 复用 MVP-07 已有 BranchInfo.ahead / behind 字段

**Don't**（明确不做 · 推后版本）：

- **Force push to main / master / trunk**（保护分支 hardcode 黑名单 · v0.3 移设置）
- **Push tags / Push --all**（v0.3 单独 spec · v0.2 仅 push current branch）
- **Submodule push/pull 级联**（保持 v0.2 不崩即可 · v0.3+ 单独 issue）
- **GUI Mergetool**（v0.3 评估 · 涉及 diff three-way + 文件级别合并 UI · v0.2 工期不够）
- **Cherry-pick / Rebase --interactive**（v0.3 MVP-16 范围）
- **Remote add / remove**（用户用终端 `git remote` · v0.3 设置面板补）
- **Stash UI**（用户用终端 · 同 MVP-13 §H.3 决策一致 · v0.3 评估）
- **GPG 签名 push**（涉及 GPG keychain · v0.3+）
- **Windows 平台**（v0.3+ · v0.2 macOS + Linux only · 因为 Windows 的 SSH 走 win32-openssh 需独立 spike）

## 🛠 实施进度

MVP-21 估时 **5d** · 拆 4 Phase 串行实施：

| Phase | 范围 | 状态 | PR |
|-------|------|------|----|
| Phase A · git2 网络层后端 + Auth + IPC | git_push / git_pull / git_fetch / remote_list 后端封装 + SSH/HTTPS auth callback + ts-rs bindings + 单元测试（fixture: 本地 bare repo 做 remote）| ⏳ todo | — |
| Phase B · UI 集成（push/pull/fetch 按钮 + 进度条 + 错误流） | Git Log 工具栏 Push/Pull 按钮（design line 1046-1051）+ progress modal + force push 二次确认 + auth modal + conflict graceful 提示 | ⏳ todo | — |
| Phase C · Conflict 处理 + 状态栏 ahead/behind | merge --abort 流程 + status bar 显示 `↑N ↓M` + post-fetch refresh | ⏳ todo | — |
| Phase D · runtime 证据 + 性能量化 + 跨平台验证 | 截图 + 录屏（push 进度 / pull conflict abort / fetch prune）+ 性能 P99（push 1MB / 100 commits） + macOS + Linux 双平台跑 + 放 `docs/runtime-evidence/mvp-21/` | ⏳ todo | — |

**Phase A 实施起点 checklist**（让 agent 接 spec 后 5 min 内启动 · 复用 MVP-09 模式）：

- [ ] `crates/core/Cargo.toml` 已含 `git2`（继承 MVP-07/09）· **不需要新增 crate**
- [ ] 验证 git2 编译时启用 `vendored-libgit2` + `vendored-openssl` features（macOS 不依赖系统 OpenSSL · 见 §H.7 跨平台依赖）
- [ ] 新建 `crates/core/src/git_sync.rs`（不和 `git_ops.rs` / `branch_ops.rs` 混 · push/pull/fetch 网络层独立模块）
- [ ] git2 API 调用链 ready-to-use（参考 §H.4 表）：
  - Push：`Repository::find_remote("origin")` → `Remote::push(&[refspec], Some(&push_options))` · `push_options` 含 `RemoteCallbacks` 配 auth + progress
  - Pull：`Remote::fetch(&[refspec], opts, None)` + `Repository::merge_analysis(...)` + `Repository::merge(...)` 或 fast-forward · rebase 策略走 `Repository::rebase(...)`
  - Fetch：`Remote::fetch(refspecs, opts, msg)` · prune 走 `FetchOptions::prune(FetchPrune::On)`
  - Auth callback：`RemoteCallbacks::credentials(|url, username_from_url, allowed_types|)` 分诊 SSH / HTTPS / cred helper
  - Progress callback：`RemoteCallbacks::transfer_progress(|stats| { ... emit event ... })`
- [ ] IPC commands 注册顺序（`crates/app/src/lib.rs` `invoke_handler!`）：
  - `git_push` / `git_pull` / `git_fetch` / `git_remote_list` / `git_auth_provide` / `git_merge_abort`
  - 总 **6 个新 IPC commands**
- [ ] permission toml：`crates/app/permissions/git_sync.toml` 新建 · 含 6 个 `allow-{name}`
- [ ] capability `default.json` 引用上述 permission
- [ ] ts-rs binding 自动生成到 `web/src/bindings/`（`build.rs` 触发 · 12 个新 binding 见 §G.6）
- [ ] fixture：`git_sync.rs` 内嵌单元测试用 `tempfile` crate 在测试 dir 创建 bare repo + working repo · 不依赖外部 GitHub repo（仿 SPIKE-04 §C 模式）
- [ ] Tauri event：`git:push-progress` / `git:fetch-progress` / `git:operation-done` 三个 event 注册（payload 见 §G.7）
- [ ] 复用 MVP-09 `CommitError::IdentityMissing` 模式 → 新 `NetworkOpError` enum（含 `AuthFailed / NetworkUnreachable / RemoteNotFound / NonFastForward / MergeConflict / Aborted / DirtyWorkingTree / RejectedByRemote / Git2Error` 9 个 variant）

**下次 agent 起点**（spec 详化完后）：等 Arbiter approve PR · 翻 `ready`（同时按文件顶部 ID 冲突警告 rename）· 派 Phase A 实施 agent（首选 Codex / OpenCode 因 Phase A 涉及 git2 网络层细节 · Kimi 需要远程 prompt 附 spec 原文）。

**依赖关系说明**：MVP-21 依赖 MVP-09 done（git2 写路径基础）+ MVP-13 done（branch CRUD · push 需要切到目标 branch）· 文件域与 MVP-13 **完全隔离**（MVP-21 只动 `crates/core/src/git_sync.rs` + IPC 注册 + `web/src/dialogs/AuthDialog/` + `web/src/dialogs/PushProgress/` + `web/src/dialogs/PullConflict/`）· 但**实施顺序应 W13 → W14**（用户在测试 push 时需要先创建 branch）· v0.2 sprint 启动顺序 MVP-13 先 · MVP-21 后。

## 🖼 UI 引用

- **Git Log 工具栏 Pull/Push 按钮**：`design/directions/1-calm-studio.html` line 1046-1051
  - `<button title="Pull"><svg ... /></button>`（向下箭头）
  - `<button title="Push"><svg ... /></button>`（向上箭头）
  - 位于 Secondary Sidebar Git Log panel-head 右侧 actions
- **Status bar ahead/behind**：复用 MVP-07 + MVP-13 已有结构
  - `<span class="status-item"><span class="key">remote</span><span class="val">↑2 ↓3</span></span>`
  - `0 ↓0` 时不显示（避免噪音）· 仅有 ahead 或 behind 才显示
- **Progress modal**：参考 GitHub Desktop / Sourcetree 的 push/pull 进度
  - 顶部：操作类型（"Push to origin" / "Pull from origin/main"）+ branch chip
  - 中部：进度条（0-100%） + `X / Y objects` + `Z KB/s` 网速
  - 底部：`Cancel` 按钮（中止 · 不破坏 repo · 安全）
  - 完成时进度条变绿 + toast `"已推送 N 个 commit 到 origin/{branch}"` · modal 自动关闭（1s）
- **Auth modal**（凭证缺失时弹）：
  - 标题：`"需要凭证：{remote_url}"`
  - SSH 路径：`"密钥 {key_path} 受密码保护 · 输入密码"` + password input
  - HTTPS 路径：`"未找到 {host} 的凭证"` + Username + Password 双 input + `"保存到系统 keychain"` 复选框（默认勾选）
  - 按钮：`Cancel` / `Confirm`
- **Force Push 二次确认 modal**：
  - 标题：`"强制推送 {branch} 到 origin"`
  - body：`"将覆盖 origin/{branch} 的 N 个 commit · 这些 commit 在 origin 上**会丢失** · 除非有其他人 fetch 过"`
  - 列出 N 个被覆盖的 commit short SHA + message 第一行（最多 5 个）
  - 输入框：`"输入分支名 '{branch}' 确认"`（防误操作）
  - 红色 `"Force push (destructive)"` 按钮 + `Cancel` 默认
- **Pull Conflict 提示 modal**：
  - 标题：`"合并冲突 · 已自动中止"`
  - body：`"以下 N 个文件含冲突 · 工作区已恢复到 pull 前状态：\n - <file 1>\n - <file 2>\n..."`
  - 提示文案：`"v0.2 不支持 GUI 解决 · 请在终端运行 git pull 后用 git mergetool / 编辑器手动解决"`
  - 按钮：`"复制 git pull 命令"` / `OK`
- **截图归档**：详化时实施 PR 补到 `docs/runtime-evidence/mvp-21/`（按 `.claude/rules/runtime-evidence-location.md` R1 命名）

## ✅ Acceptance

### A. Push

- [ ] A.1 Git Log 工具栏 Push 按钮（design line 1049）点击 → 检测 remote · 单 remote 直接推 · 多 remote 弹 dropdown 选 origin / 其他
- [ ] A.2 Push 进度 modal 显示：操作类型 + branch chip + 进度条（0-100%）+ `X / Y objects` 实时数字 + 网速
- [ ] A.3 Push 成功 → toast `"已推送 N 个 commit 到 origin/{branch}"`（持续 3s）· progress modal 自动关闭 · Git Log 刷新（remote tracking ref 更新）· status bar `↑0`
- [ ] A.4 Push 失败 graceful：
  - **Non-fast-forward**（远端 ahead）→ toast error `"origin/{branch} 已有更新 · 请先 pull 或 force push"` + 红色 `Force Push` 按钮（点击进 §A.7 流程）
  - **Network unreachable** → toast error `"网络不通 · 请检查代理 / DNS"` + retry 按钮（30s 内 1 次手动 retry）
  - **Auth failed** → 弹 §UI 引用"Auth modal"· 用户重新输入凭证 · retry 1 次后还失败 → toast error 终结
- [ ] A.5 Force push 流程：必经 §UI 引用"Force Push 二次确认 modal"
  - main / master / trunk 三个保护名 hardcode 阻止 force push（toast `"受保护分支 · 不允许 force push · 请改名其他 branch"`）
  - 输入分支名匹配后才能点击 `"Force push (destructive)"`
- [ ] A.6 Cancel 中止：progress modal 的 Cancel 按钮调用 git2 内部中止 · **不破坏 repo**（已推送的 ref 保留 · 未推送的 abort）· toast `"已取消推送"` · 30s 后允许重新推
- [ ] A.7 性能：push 1MB / 100 commits（fixture: bare repo）< 5s（P99 · 测 3 次 · 本地 loopback · 不含真实网络）

### B. Pull

- [ ] B.1 Git Log 工具栏 Pull 按钮（design line 1046）点击 → 进 pull 流程
- [ ] B.2 Clean working tree 要求：dirty tree → toast warn `"工作区有未提交修改 · 请先 commit / stash / discard"` + 跳转 Status 面板 · 阻止 pull
- [ ] B.3 默认策略 = merge（设置面板 MVP-10 用户可改 rebase）· 单次操作可在 progress modal 顶部 toggle 选 rebase / merge
- [ ] B.4 Pull 进度 modal：fetch 阶段 + merge/rebase 阶段两段进度（显式标 stage）
- [ ] B.5 Pull 成功 fast-forward → toast `"已 fast-forward 到 origin/{branch} · {N} commits"` · Git Log 刷新 · status bar `↓0`
- [ ] B.6 Pull 成功 merge commit（非 ff）→ toast `"已合并 origin/{branch} · 创建合并 commit {short_sha}"` · Git Log 刷新
- [ ] B.7 Pull 成功 rebase → toast `"已 rebase {N} 个 commit 到 origin/{branch}"`（含 conflict-free rebase）
- [ ] B.8 Pull conflict graceful：
  - **Merge 策略下 conflict** → 自动 `git merge --abort` · 工作区恢复 pull 前状态 · 弹 §UI 引用"Pull Conflict 提示 modal"
  - **Rebase 策略下 conflict** → 自动 `git rebase --abort` · 同上
  - 工作区恢复必须 byte-level（用 `Repository::merge_analysis` + `Repository::cleanup_state` + 回滚 HEAD · 测试用例覆盖前后 hash 对比）
- [ ] B.9 性能：pull 1MB / 100 commits 含 ff merge < 5s P99 · pull 含 conflict abort < 3s P99（测 3 次取 P99）

### C. Fetch

- [ ] C.1 Git Log 工具栏 dropdown / context menu 含 `"Fetch"`（不在 Pull 按钮直接 · 因为 fetch 不修改 working tree · 走 secondary action）
- [ ] C.2 Fetch progress modal：仅显示 fetch 进度 · 无 merge/rebase 阶段
- [ ] C.3 Prune 选项：modal 顶部复选框 `"Prune deleted refs"` · 默认 unchecked · 勾选后传 `FetchPrune::On` · 完成后 toast 列出 N 个 pruned refs（如 `"已删除 origin/feat/old-x（remote 已不存在）"`）
- [ ] C.4 Fetch 成功 → toast `"已 fetch · 远端 {N} commits 到 origin/{branch}"` · status bar `↓N` 更新（如有 behind）· Git Log 不强制刷新（fetch 不改 HEAD · 用户可主动看 tracking refs）
- [ ] C.5 性能：fetch（10 远端 refs · 100 commit 增量）< 3s P99（测 3 次取 P99 · 本地 bare repo）

### D. Auth（SSH + HTTPS）

- [ ] D.1 SSH 默认走系统 ssh-agent：
  - macOS：`SSH_AUTH_SOCK` 环境变量（系统启动时设 · `~/.ssh/config` 触发）· git2 `Cred::ssh_key_from_agent("git")` 调用
  - Linux：同 macOS · 也支持 `gnome-keyring-daemon` 集成（用户可能没装 · git2 自动 fallback）
- [ ] D.2 SSH agent 不可用 → fallback 到 `~/.ssh/id_ed25519` / `id_rsa`（按优先级）· 密钥受密码保护时弹 §UI 引用"Auth modal" SSH 路径
- [ ] D.3 HTTPS 默认走 git credential helper：
  - macOS：`osxkeychain`（系统集成 · git config `credential.helper=osxkeychain` 默认）
  - Linux：`libsecret`（GNOME Keyring · 用户需装 · 否则 fallback `cache --timeout=3600` 或弹 modal）
- [ ] D.4 凭证缺失 → 弹 §UI 引用"Auth modal" HTTPS 路径 · `"保存到系统 keychain"` 复选框默认勾选
- [ ] D.5 凭证错误（401 / 403）→ 自动 retry 1 次（重新调用 credential helper · 可能用户在 modal 改了密码）· 第二次失败 → toast error `"凭证错误 · 请检查 username / password / token"`
- [ ] D.6 Auth modal 取消 → 终止当前操作 · toast `"已取消 · 凭证未提供"`
- [ ] D.7 安全：keychain 密钥 / SSH 密码**绝不**写入 rusqlite 或 logs · 仅在内存中传递（git2 callback 内）

### E. Conflict 处理 + 状态栏

- [ ] E.1 Pull merge conflict → §B.8 graceful abort · 弹 §UI 引用"Pull Conflict 提示 modal"
- [ ] E.2 modal 含 `"复制 git pull 命令"` 按钮 → 复制 `cd {repo_path} && git pull origin {branch}` 到剪贴板 · toast `"已复制 · 在终端粘贴执行"`
- [ ] E.3 Status bar ahead/behind 实时更新：
  - Push 后 `↑0`
  - Pull 后 `↓0`
  - Fetch 后更新 `↓N`（push 不变 ↑）
  - branch checkout（MVP-13）后 `↑N ↓M` 重算（基于新 branch 的 upstream）
- [ ] E.4 Status bar 数字点击 → 跳转 Git Log 高亮 ahead 或 behind commits（与 origin 比较）

### F. 错误处理 + 边界

- [ ] F.1 Remote 不存在（`git remote -v` 为空）→ toast `"未配置 remote · 请用终端 git remote add origin <url>"` + 跳转设置面板（v0.3 补 GUI）
- [ ] F.2 Remote URL 不可达（DNS 解析失败 / 端口 unreachable）→ toast error 含具体错误 · retry 按钮
- [ ] F.3 SSL 证书错误（自签名证书 / 过期）→ toast error `"SSL 证书无效 · 请检查 url 或在终端用 git config http.sslVerify false 临时跳过"`（v0.2 不提供 GUI 跳过 · 安全）
- [ ] F.4 Submodule 含 push/pull 状态 → v0.2 不级联 · 仅同步主 repo · 提示 toast `"submodule 状态请用终端管理"` · `git submodule update` 留 v0.3
- [ ] F.5 大 push（> 100MB）→ progress modal 显示 `"大文件传输中 ({size_mb}MB)"` · 不阻塞 UI · cancel 安全
- [ ] F.6 Network 中断（操作中途断网）→ git2 callback 检测 · 优雅 abort · toast `"网络断开 · 已取消 {operation} · 工作区未变更"`（重要：必须 byte-level 工作区不变）

### G. 性能 + 跨平台

- [ ] G.1 macOS（Apple Silicon · M1/M2/M3）：push/pull/fetch 全测 · 性能基线见 §A.7 / §B.9 / §C.5
- [ ] G.2 Linux（Ubuntu 24 X11 + Wayland）：双 desktop session 全测 · auth 路径走 libsecret + ssh-agent
- [ ] G.3 keychain 集成测试：macOS osxkeychain + Linux libsecret 双平台 · 用户输入 → 保存 → 重启应用 → 凭证自动加载（无需重输）
- [ ] G.4 性能（P99 · 测 3 次取最差值）：
  - Push 1MB / 100 commits < 5s
  - Pull 1MB / 100 commits（ff merge）< 5s
  - Pull conflict abort < 3s
  - Fetch 10 refs / 100 commits < 3s
  - Auth modal 响应（凭证输入到 push 重启）< 500ms

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元（core） | `git_sync.rs` 所有函数（push / pull / fetch / merge_abort / rebase_abort）+ `NetworkOpError` enum 全 variant + auth callback 模拟（mock SSH agent / mock credential helper） | `cargo test --package vibestation-core git_sync::` · fixture: tempfile + 本地 bare repo + working repo |
| 集成 | 完整链路：本地 bare repo 做 origin · push → fetch → pull → conflict abort 全流程 · 含 SSH（用 sshd local + ssh-agent fixture）+ HTTPS（用 nginx local serve git smart http） | `cargo test --features integration` |
| Criterion bench | push 1MB / 100 commits · pull ff · fetch 10 refs · 三个核心 P99 数字 | `crates/core/benches/git_sync_bench.rs` |
| E2E（Playwright） | golden path：commit → push → 模拟另一 clone fetch → pull · 跨界面流转完整 | `web/tests/e2e/git_sync.spec.ts` |
| 视觉回归 | Push progress modal · Pull conflict modal · Auth modal · Force Push 二次确认 | Playwright screenshot diff |
| 手动 QA | 真实 GitHub repo（小 fixture repo 仅用于 manual QA）· macOS + Linux 双平台 SSH/HTTPS 真实 keychain 流程 · 含密码保护的 SSH 密钥 | 手动 capture |

### C.1 · fixture 准备脚本（本地 bare repo + working repo · 不依赖 GitHub）

仿 SPIKE-04 §C 模式 · 所有 fixture 用 `tempfile::TempDir` + `git2::Repository::init_bare()` + `Repository::clone_local()` 在测试运行时生成 · **不依赖外部网络**：

```rust
// crates/core/tests/fixtures/mvp_21_helpers.rs（新建）
use git2::{Repository, Signature, RepositoryInitOptions};
use tempfile::TempDir;
use std::path::PathBuf;

struct GitSyncFixture {
    pub bare_dir: TempDir,        // origin（bare repo）
    pub working_dir: TempDir,     // local clone
    pub bare_path: PathBuf,
    pub working_path: PathBuf,
}

fn create_fixture_clean() -> GitSyncFixture {
    let bare_dir = tempfile::tempdir().unwrap();
    let mut opts = RepositoryInitOptions::new();
    opts.bare(true);
    Repository::init_opts(bare_dir.path(), &opts).unwrap();

    let working_dir = tempfile::tempdir().unwrap();
    let _repo = Repository::clone(
        bare_dir.path().to_str().unwrap(),
        working_dir.path(),
    ).unwrap();
    // 写 hello.txt + initial commit + push to origin
    GitSyncFixture {
        bare_path: bare_dir.path().to_path_buf(),
        working_path: working_dir.path().to_path_buf(),
        bare_dir, working_dir,
    }
}

fn create_fixture_with_remote_ahead() -> GitSyncFixture { /* origin 有 5 commits 比 local 多 */ }
fn create_fixture_with_local_ahead() -> GitSyncFixture { /* local 有 3 commits 比 origin 多（push 测试） */ }
fn create_fixture_with_diverged() -> GitSyncFixture { /* local 和 origin 都有 ≠ commit · pull 触发 merge 或 conflict */ }
fn create_fixture_with_conflict() -> GitSyncFixture { /* 同文件不同行修改 · pull 触发 conflict */ }
fn create_fixture_1mb_100_commits() -> GitSyncFixture { /* 性能 fixture · 100 commits 含 1MB 总变更 */ }
fn create_fixture_with_ssh_remote() -> GitSyncFixture { /* 起 local sshd 配 ssh-agent · 模拟 SSH push/pull · macOS / Linux only */ }
fn create_fixture_with_https_credential_helper() -> GitSyncFixture { /* mock credential helper 返回固定 token */ }
```

每个 helper 返回 `GitSyncFixture` · 测试用 `let fixture = create_fixture_clean();` · `bare_dir` / `working_dir` drop 自动清理。

### C.2 · Criterion bench 模板

新建 `crates/core/benches/git_sync_bench.rs`：

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_push_1mb_100_commits(c: &mut Criterion) {
    c.bench_function("push_1mb_100_commits", |b| {
        b.iter(|| {
            let fixture = create_fixture_1mb_100_commits();
            // call vibestation_core::git_sync::push(&fixture, "origin", "main")
        });
    });
}

fn bench_pull_ff(c: &mut Criterion) { /* fast-forward pull · 100 commits */ }
fn bench_fetch_10_refs(c: &mut Criterion) { /* fetch 10 远端 refs */ }
fn bench_pull_conflict_abort(c: &mut Criterion) { /* conflict 触发 + abort 时间 */ }

criterion_group!(
    benches,
    bench_push_1mb_100_commits, bench_pull_ff, bench_fetch_10_refs, bench_pull_conflict_abort
);
criterion_main!(benches);
```

跑 `cargo bench --bench git_sync_bench` · P99 数字写入 PR description。

## 💾 数据模型变更

新增 2 个 `app_settings` 表 key（不新建表 · 复用 MVP-03 已建 schema）：

```rust
// app_settings 表 key 示例：
// "pull_strategy_default" → "merge" | "rebase" · 默认 merge · 设置面板 MVP-10 控制
// "fetch_prune_default" → "true" | "false" · 默认 false
```

**禁止**：不缓存 remote refs（每次实时查 git2）· 不缓存 push/pull 历史（git reflog 即真相 · 用户用终端 `git reflog` 看）。

**禁止**：**绝不**持久化 SSH 密钥密码 / HTTPS 凭证（D.7 安全要求）· 仅由系统 keychain / ssh-agent 管理。

**rusqlite schema 不动**：仅复用 MVP-03 `app_settings` key-value 表。

## ⚠️ 已知风险

- **R1 · git2 网络层 callback 错误恢复复杂** · git2 0.20 的 `RemoteCallbacks` 在网络中途断开时可能 panic 或留 partial state · 缓解：每次 push/pull/fetch 必须包 `panic::catch_unwind` 或显式 `Result` 处理 · 单元测试覆盖网络中断场景（mock callback emit error）· P0 优先级
- **R2 · SSH agent 跨平台行为差异** · macOS launchd 启动 ssh-agent vs Linux systemd-user 启动 · 部分 Linux 发行版默认无 ssh-agent · 缓解：fallback 到 `~/.ssh/id_*` 文件读取 · 实施时双平台测试矩阵（macOS / Ubuntu X11 / Ubuntu Wayland）· 失败时 toast error 引导用户启动 ssh-agent
- **R3 · HTTPS keychain 集成的 Linux 兼容性** · libsecret 需要 GNOME Keyring（KDE / 其他 DE 用户可能没装）· 缓解：fallback `cache --timeout=3600` 或弹 modal 每次问 · 文案明确告知（v0.2 文档章节 + 设置面板提示 v0.3 补）
- **R4 · 大 push 体验** · 100MB+ push 在弱网下可能数分钟 · 缓解：progress modal cancel 安全 · UI 文案 `"大文件传输中"` 安抚 · v0.3+ 评估增量 push（`--no-thin` opt-in）
- **R5 · Conflict 处理工作区恢复 byte-level 一致性** · git2 `merge --abort` 在某些 corner case 可能不完全恢复（如 unmerged paths 留下 .orig 文件）· 缓解：abort 后用 `Repository::cleanup_state` + `git reset --hard ORIG_HEAD` 双保险 · 单元测试用 fixture 验证 abort 前后 working tree byte-level diff 为零
- **R6 · Force push 误操作** · 用户在脏 branch 上 force push 主 branch 是灾难性操作 · 缓解：保护名单 hardcode + 二次确认要求输入 branch 名 + 列出被覆盖 commit · 同时记录到 status bar 历史（v0.3 补 audit log）

## 📝 Notes

- MVP-21 是 v0.2 第二个 git 写路径扩展 · 模式（git2 backend + ts-rs binding + Tauri permission + cmd 注册）和 MVP-09 / MVP-13 完全一致 · 实施 agent 直接复用
- **GUI Mergetool**（v0.3 评估）：等 v0.3 评估 three-way diff UI + 文件级别合并 UX · v0.2 用户必须切终端
- **Push tags / Push --all**（v0.3 单独 spec）：v0.2 仅 push current branch + tracking · 因为多 ref push 错误处理复杂度爆炸
- **Windows v0.3+**：win32-openssh 集成 + path separator + credential helper 差异 · v0.2 不投入
- **设置面板集成**：默认 pull 策略 / fetch prune 默认值 / 设置 SSH key path 优先级 / HTTPS 凭证清除按钮 · 都依赖 MVP-10 设置面板（已 done）扩展 · v0.2 sprint 内一并落地
- **测试架构**：本地 bare repo + sshd / nginx fixture 比真实 GitHub repo 更稳定 · CI 不依赖外部服务 · GitHub real-repo 测试仅在手动 QA 阶段做
- **id 冲突历史**（已解决 · 2026-05-03）：本 spec 详化时与 v0.1 已 done "Native Feel Quality" 同号冲突 · 已 rename 为 MVP-21（详化阶段建议方案 [A]）· 详见文件顶部历史说明 + git mv history

## 🔗 相关

- `CLAUDE.md` #13 Git 栈混用决策（写 git2 0.20 · 网络层全用 git2）
- ADR-007 Git 栈混用决策
- `implementation-plan.md` §10.1 v0.2 砍到 push/pull/fetch · §6.2 git_push/pull/fetch IPC · §6.3 git:push/fetch-progress event · §11 W14 路线图 · §11.3 R23 类比（push/pull 网络可恢复性）
- 上游：MVP-09（git2 写路径基础）· MVP-13（branch CRUD · 用户在切完 branch 后 push）· SPIKE-04（git2 写 smoke test）
- 下游：v0.3 MVP-16 rebase/merge/cherry-pick · v0.3 GUI Mergetool · v0.3 Windows 支持

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)（H2 根因消除 · SPIKE-08 §A PASS · 规范源头）。所有 IPC struct 必须遵循 ADR-014 §规范 5 条 + H2 regression proof 6 步。

本 MVP 所有 IPC struct 必须单点维护——**Rust struct 为 source of truth**，禁止前端手写对偶 TypeScript interface。

### G.1 本 MVP 涉及的 IPC struct 清单（预期）

| Rust struct | 用途 | 前端 import 路径 |
|-------------|------|-----------------|
| `RemoteInfo` | 单个 remote 元数据 | 新增 |
| `PushRequest` | push 输入 · `{ workspace_id, remote, branch, force }` | 新增 |
| `PushResult` | push 输出 · `{ pushed_commits, new_remote_head }` | 新增 |
| `PullRequest` | pull 输入 · `{ workspace_id, remote, branch, strategy: "merge" \| "rebase", frontend_status_snapshot: GitStatusResponse, frontend_status_taken_at: i64 }` · snapshot 字段是 race guard 必需（§H.4.1 · backend 重检 dirty drift 用） | 新增 |
| `PullResult` | pull 输出 · `{ stage: "ff" \| "merge" \| "rebase", new_head, merged_commits }` | 新增 |
| `FetchRequest` | fetch 输入 · `{ workspace_id, remote, prune }` | 新增 |
| `FetchResult` | fetch 输出 · `{ fetched_refs, pruned_refs }` | 新增 |
| `AuthMethod` | enum · `SshAgent` / `SshKeyFile { path, passphrase: Option<String> }` / `HttpsHelper` / `HttpsManual { username, password }` | 新增 |
| `AuthRequest` | 输入侧（auth modal 提交后回调）· `{ workspace_id, auth_challenge_id, task_id, remote_url, allowed_methods, method }` · **必须** challenge-bound（防多远端并发凭证错绑） | 新增 |
| `AuthChallenge` | 输出侧（后端发起 auth 提示前广播）· `{ workspace_id, auth_challenge_id, task_id, remote_url, host_fingerprint, allowed_methods, expires_at }` | 新增 |
| `NetworkOpError` | 错误枚举 · 含 payload | 新增 |
| `MergeConflictInfo` | conflict 详情 · `{ files: ConflictFile[], aborted: bool }` | 新增 |
| `ConflictFile` | 单个冲突文件 · `{ path, ours_oid, theirs_oid }` | 新增 |
| `RemoteListRequest` / `RemoteListResponse` | 列出 remote · `{ workspace_id }` → `{ remotes: RemoteInfo[] }` | 新增 |

> 实际 struct 名和字段以实施 PR 为准 · 但**必须**全部走 ts-rs 自动生成。

### G.2 derive 模板（以 `PushRequest` + `NetworkOpError` 为例）

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PushRequest {
    pub workspace_id: String,
    pub remote: String,         // "origin" / 用户指定
    pub branch: String,
    pub force: bool,            // force push · 后端额外校验保护名单
    /// force-with-lease: 期望的远端 head OID（hex）
    /// - force=false 时忽略（None）
    /// - force=true 时**必填** · 后端 push 前 verify `origin/<branch>` 当前 OID == expected_remote_oid
    ///   - 不一致 → 拒绝 push 返回 `NetworkOpError::StaleLease { expected, actual }` · UI 重新 fetch + 弹新 confirmation
    ///   - 防止 confirmation modal 弹出后远端 advance · 用户在不知情下覆盖新 commits
    /// - 实施时：UI 在 confirmation 前 fetch 一次 · 把 `origin/<branch>` 当前 OID 存到 PushRequest.expected_remote_oid
    /// - 见 §H.X force-with-lease 语义说明
    pub expected_remote_oid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub workspace_id: String,
    pub remote: String,
    pub branch: String,
    pub strategy: PullStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PullStrategy {
    Merge,
    Rebase,
}

// AuthMethod 含敏感凭证字段 · **禁止 derive Debug**（会通过 log/panic/tracing 泄漏 password）
// 实施时必须 manual impl Debug · passphrase / password 字段渲染为 "***REDACTED***"
// 参考下方 impl 模板（v0.2 实施时按此模板写 · 不要省略）
#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AuthMethod {
    SshAgent,
    SshKeyFile { path: String, passphrase: Option<String> },
    HttpsHelper,                                  // 走 git credential helper
    HttpsManual { username: String, password: String },
}

// 必须的 manual Debug · 永远不暴露 passphrase / password 明文（v0.2 实施时强制实现）
impl std::fmt::Debug for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SshAgent => write!(f, "AuthMethod::SshAgent"),
            Self::SshKeyFile { path, passphrase } => f
                .debug_struct("AuthMethod::SshKeyFile")
                .field("path", path)
                .field(
                    "passphrase",
                    &passphrase.as_ref().map(|_| "***REDACTED***"),
                )
                .finish(),
            Self::HttpsHelper => write!(f, "AuthMethod::HttpsHelper"),
            Self::HttpsManual { username, .. } => f
                .debug_struct("AuthMethod::HttpsManual")
                .field("username", username)
                .field("password", &"***REDACTED***")
                .finish(),
        }
    }
}

// 进阶（可选 v0.3+ · 不强制 v0.2）：用 `secrecy::SecretString` 或 `zeroize::Zeroizing<String>`
// 包 password / passphrase · drop 时自动清零内存 · 防 process dump 泄漏

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NetworkOpError {
    AuthFailed { detail: String },                              // 401 / 403 / 密钥错误
    NetworkUnreachable { detail: String },                      // DNS 失败 / 端口 unreachable
    RemoteNotFound { remote: String },                          // remote 不存在
    NonFastForward { remote_branch: String, local_ahead: u32, remote_ahead: u32 },
    MergeConflict { files: Vec<ConflictFile>, aborted: bool },  // pull conflict · aborted=true 表已恢复
    Aborted { reason: String },                                 // 用户 cancel
    DirtyWorkingTree { modified: Vec<String>, staged: Vec<String>, untracked: Vec<String> },
    RejectedByRemote { detail: String },                        // 远端 hook 拒绝（pre-receive）
    StaleLease { expected: String, actual: String },            // force-with-lease 失败 · 远端 advance · UI 需重新 fetch + 弹新 confirmation（防覆盖未见 commits）
    SslError { detail: String },                                // SSL 证书无效
    Git2Error { class: String, code: i32, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConflictFile {
    pub path: String,
    pub ours_oid: String,    // local HEAD 该文件 OID
    pub theirs_oid: String,  // remote HEAD 该文件 OID
}
```

> `AuthMethod` / `NetworkOpError` 因含 payload（passphrase / detail / 文件列表）必须用 tagged union（`#[serde(tag = "kind")]`）· 前端 TS 生成 discriminated union。

### G.3 强制规范

- [ ] 所有 IPC struct + enum **默认** `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
- [ ] 简单无 payload enum（`PullStrategy`）用 string union（`rename_all` + 无 tag）· 含 payload enum 用 tagged union
- [ ] **敏感类型例外**（**覆盖** "默认 derive Debug" 规则 · 适用 `AuthMethod` 含 `passphrase` / `password` 字段的类型）：
  - **禁止** `#[derive(Debug)]` · 必须 manual `impl Debug` 把 sensitive 字段渲染成 `"***REDACTED***"`
  - `Serialize` / `Deserialize` 保留（前端→backend 单向需要）· 但 `Display` impl 也禁含 sensitive 字段
  - 实施 PR 必须含 regression test：构造 AuthMethod 实例 · 走 `format!("{:?}", auth)` / `tracing::debug!` / `panic!` 任一路径 · assert 输出**不含** passphrase / password 明文
  - 完整 manual Debug 模板见 §G.2 AuthMethod 段（v0.2 实施 copy-paste）
- [ ] bindings 由 `crates/app/build.rs` 在 `cargo build` 时自动生成到 `web/src/bindings/`
- [ ] 前端**禁止**手写 `interface PushRequest { ... }`——所有类型必须从 `./bindings/*` import
- [ ] `.prettierignore` 已排除 `web/src/bindings/`（防止 prettier 与生成格式冲突）

### G.4 H2 类 regression proof

复用 MVP-04 §G.3 / MVP-09 §G.4 / MVP-13 §G.4 模式 · 流程：

1. 临时在任一 IPC struct（如 `PushRequest`）的某个字段上加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'PushRequest'`——FAIL 证明防御生效
5. **回滚**：撤销 `#[ts(rename = ...)]` · 确认 `pnpm typecheck` 恢复 PASS

> 本 proof 只需做一次 · 结果写入 PR description 或 `docs/runtime-evidence/mvp-21/`。

### G.5 · 与上游已落地 binding 的复用决策

MVP-21 实施前必须明确复用 / 新增边界：

| 已有 binding | MVP-21 §G.1 涉及 | 决策 | 理由 |
|---|---|---|---|
| `BranchInfo`（MVP-07 已生成 · 含 `name / upstream / ahead / behind`）| status bar `↑N ↓M` 数字来源 | ✅ **复用** · 不重新查 | MVP-07 + MVP-13 已通过 BranchInfo 提供 ahead/behind · MVP-21 fetch 后只需触发 BranchInfo 重算事件 |
| `BranchKind` / `CommitAuthor` | 不直接涉及 | ⛔ 不复用 | MVP-21 不操作 commit metadata · 不引入 |
| `FileChange`（MVP-07/08）| `MergeConflictInfo.files` 含路径 | ⛔ 不复用 · 用独立 `ConflictFile` | conflict 文件需要 ours_oid / theirs_oid 元数据 · `FileChange` 没有 · 强行复用会让 union 膨胀 |
| `GitStatusResponse`（MVP-08）| pull 前 dirty tree 检测 | ✅ 前端**复用** · 不新建 | MVP-21 前端调 MVP-08 已有 `git_status` IPC 检测 dirty · 不重复实现 |
| `CommitError`（MVP-09）| 不直接涉及 | ⛔ 不复用 · 新建独立 `NetworkOpError` | 错误语义完全不同（push/pull 是网络层 · commit 是本地写）· 强行复用让 union 膨胀 |
| `BranchError`（MVP-13）| 不直接涉及 | ⛔ 不复用 · 同上 | branch 操作错误 ≠ 网络操作错误 |

### G.6 · MVP-21 新增 binding 清单（明确数量）

以下 **12 个 binding** 为 MVP-21 **新增** · 实施时 `web/src/bindings/` 应新增 12 个 `.ts` 文件：

| Rust struct / enum | 用途 | 前端 import 路径 |
|---|---|---|
| `RemoteInfo` | remote 元数据 · `{ name, url, fetch_url }` | `import type { RemoteInfo } from "../bindings/RemoteInfo"` |
| `RemoteListRequest` / `RemoteListResponse` | list remote · 2 binding | `import type { RemoteListRequest, RemoteListResponse } from "../bindings/..."` |
| `PushRequest` | push 输入 | `import type { PushRequest } from "../bindings/PushRequest"` |
| `PushResult` | push 输出 · `{ pushedCommits, newRemoteHead }` | `import type { PushResult } from "../bindings/PushResult"` |
| `PullRequest` / `PullStrategy` | pull 输入 · 2 binding | `import type { PullRequest, PullStrategy } from "../bindings/..."` |
| `PullResult` | pull 输出 · `{ stage, newHead, mergedCommits }` | `import type { PullResult } from "../bindings/PullResult"` |
| `FetchRequest` | fetch 输入 | `import type { FetchRequest } from "../bindings/FetchRequest"` |
| `FetchResult` | fetch 输出 · `{ fetchedRefs, prunedRefs }` | `import type { FetchResult } from "../bindings/FetchResult"` |
| `AuthMethod` | auth 枚举 · 含 payload tagged union | `import type { AuthMethod } from "../bindings/AuthMethod"` |
| `AuthRequest` | auth modal 提交回调 · challenge-bound | `import type { AuthRequest } from "../bindings/AuthRequest"` |
| `AuthChallenge` | 后端发起 auth 提示前广播（§G.1 · spec round 2 fix 加） | `import type { AuthChallenge } from "../bindings/AuthChallenge"` |
| `NetworkOpError` | 错误枚举 · 含 payload tagged union（10 variant · 含 StaleLease + DirtyWorkingTree）| `import type { NetworkOpError } from "../bindings/NetworkOpError"` |
| `MergeConflictInfo` / `ConflictFile` | conflict 详情 · 2 binding | `import type { MergeConflictInfo, ConflictFile } from "../bindings/..."` |

> 复用上游：`BranchInfo`（MVP-13）· `GitStatusResponse`（MVP-08 · 用于 PullRequest.frontend_status_snapshot · §H.4.1 race guard）· 实施时 bindings 目录新增约 **15 个** `.ts` 文件（ts-rs 每个 `#[derive(TS)]` 生成独立文件 · 实际数以 `cargo build` 后 `web/src/bindings/` 实际产物为准 · 不假设小 enum 被内联）。

### G.7 · Tauri Event Payload（progress streaming）

MVP-21 用 Tauri event 推送 push/pull/fetch 进度 · 不走 IPC return（异步 streaming · 类似 MVP-04 PTY stdout pattern）：

```rust
// crates/core/src/git_sync.rs

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PushProgressEvent {
    pub workspace_id: String,
    pub task_id: String,                  // 关联 IPC 调用的任务 ID（用户可 cancel）
    pub stage: String,                    // "counting" | "compressing" | "writing"
    #[ts(type = "number")]
    pub objects_total: u32,
    #[ts(type = "number")]
    pub objects_done: u32,
    #[ts(type = "number")]
    pub bytes_total: u64,
    #[ts(type = "number")]
    pub bytes_done: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FetchProgressEvent {
    pub workspace_id: String,
    pub task_id: String,
    pub stage: String,                    // "fetching" | "indexing" | "resolving"
    #[ts(type = "number")]
    pub received_objects: u32,
    #[ts(type = "number")]
    pub total_objects: u32,
    #[ts(type = "number")]
    pub received_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OperationDoneEvent {
    pub workspace_id: String,
    pub task_id: String,
    pub outcome: String,                  // "success" | "cancelled" | "error"
    pub error: Option<NetworkOpError>,
}
```

事件名（全局规范）：
- `git:push-progress` (payload: `PushProgressEvent`)
- `git:fetch-progress` (payload: `FetchProgressEvent`)
- `git:operation-done` (payload: `OperationDoneEvent`)

前端 listen 模式（仿 MVP-04 `pty:stdout`）：
```typescript
// web/src/utils/gitSyncEvents.ts
import { listen } from "@tauri-apps/api/event";
import type { PushProgressEvent } from "../bindings/PushProgressEvent";

const unlisten = await listen<PushProgressEvent>("git:push-progress", (e) => {
  // 更新 progress modal 状态
});
```

## §H. Git 栈约束 + 决策锁定（MVP-21 专有 · 防 v0.2 实施期反复讨论）

MVP-21 是**纯写路径 + 网络** · 对齐 CLAUDE.md 决策表 #13（2026-04-19 accepted · [ADR-007](../adr/ADR-007-git-stack.md)）· 必须明确：

### H.1 本 MVP Git 栈（写 + 网络全用 git2）

- **库选型**：`git2 0.20`
- **场景**：push / pull（fetch + merge / rebase）/ fetch / auth callback
- **依据**：
  - SPIKE-04 §C 已验证 git2 0.20 写路径 smoke test 通过
  - **gix 0.70 网络层 API 不成熟**（push/fetch 异步 API + auth callback 不齐全 · v0.2 不试水）
  - libgit2（C 库 · git2 binding）成熟度高 · 错误恢复行为可预测

### H.2 Auth 方法决策锁定（防 v0.2 实施反复讨论）

| Auth 类型 | 默认路径 | 缺失时 fallback | v0.2 实现优先级 |
|----------|---------|----------------|----------------|
| SSH | 系统 ssh-agent（`SSH_AUTH_SOCK`）| `~/.ssh/id_ed25519` → `~/.ssh/id_rsa` 顺序读 · 密码弹 modal | P0 必须 |
| HTTPS | git credential helper（osxkeychain / libsecret）| 弹 modal · `"保存到 keychain"` 复选框 | P0 必须 |
| GPG signing | ❌ 不支持 | — | v0.3+ |
| OAuth token（GitHub Personal Access Token）| 走 HTTPS 同路径（user 输入 token 当 password）| 同 HTTPS | P0（共享 HTTPS 路径） |
| Kerberos / NTLM | ❌ 不支持 | — | v1.0+ 企业版 |

**关键决策**：**不自己实现 keyring**（OS keychain 集成由 git credential helper 负责 · 经过 git2 + libgit2 调用）· 减少 attack surface + 维护成本。

### H.3 Merge vs Rebase 默认决策锁定

**v0.2 默认 = merge**（保守 · 不改写历史）· 用户在 MVP-10 设置面板可改 rebase。

理由：

| 选项 | 优点 | 缺点 | v0.2 评估 |
|------|------|------|-----------|
| (a) **merge 默认**（**v0.2 选定**）| 不改写本地历史 · 用户预期符合 GitHub Desktop 默认 · 不易出 force-push 灾难 | 历史变 messy | ✅ MVP 安全优先 |
| (b) rebase 默认 | 历史干净 | 改写本地历史 · 用户在 push 时遇 non-fast-forward 困惑 · 容易导致 force push | ❌ MVP 不可接受 |
| (c) 弹 modal 每次问 | 灵活 | UX 太烦 | ❌ |

**v0.3 升级触发条件**：用户研究证明 70%+ 用户偏好 rebase（实施 telemetry 收集 v0.2 的实际使用数据 · MVP-10 anonymized）。

### H.4 git2 0.20 网络 API 使用要点（实施参考）

| 操作 | git2 API 调用链 |
|------|----------------|
| Remote list | `Repository::remotes()` → `Vec<&str>` (远端名)→ `Repository::find_remote(name)` |
| Push | `Repository::find_remote("origin")` → `Remote::push(refspecs, Some(&PushOptions))` · `PushOptions` 含 `RemoteCallbacks` |
| Pull (fetch + ff) | **必须先 backend snapshot dirty check**（见下方 race guard）→ `Remote::fetch(...)` + `Repository::merge_analysis(annot)` 检测 ff → `Repository::reset(target_commit, Hard, ...)` 直接 ff |
| Pull (merge) | **必须先 backend snapshot dirty check** → `Remote::fetch(...)` + `Repository::merge(...)` + 检 conflict + `Repository::commit(...)` 创建 merge commit |
| Pull (rebase) | **必须先 backend snapshot dirty check** → `Remote::fetch(...)` + `Repository::rebase(Some(branch), Some(upstream), None, RebaseOptions)` + 循环 `next` + `commit` |
| Fetch | `Remote::fetch(refspecs, opts, msg)` · `FetchOptions::prune(FetchPrune::On)` for prune |
| Auth callback | `RemoteCallbacks::credentials(\|url, username, allowed\| { match allowed { ... } })` |
| Progress callback | `RemoteCallbacks::transfer_progress(\|stats\| { emit_event(...); true })` · 返回 false 表 cancel |
| Merge abort | `Repository::cleanup_state()` + `Repository::reset(orig_head, Hard, ...)` 双保险（§R5） |
| 错误分诊 | `git2::Error::class()` / `git2::Error::code()` → 映射到 `NetworkOpError` enum |

### H.4.1 Pull / Rebase race guard（**强制** backend-side dirty check）

**问题**：spec §A 要求 dirty tree pull 弹"先 stash / discard / cancel" · 但该检查由 frontend 调 `git_status` 完成。frontend 检查 → 用户点 OK → backend 跑 pull 之间 · 其他进程（IDE 自动保存 / git CLI / formatter）可能写入 worktree。如果 backend 不再做检查直接 `reset(Hard)` 或 `merge` · 会**字节级丢失**用户编辑（违反 byte-level no-worktree-loss 保证）。

**解法**：backend 必须在 fetch 之后、reset/merge/rebase 之前**自己再做一次 status check** · 与 PullRequest 中携带的 frontend snapshot 比较 · 不一致 → 抛 `NetworkOpError::DirtyWorkingTree { ... }`。

```rust
// PullRequest 增加字段（已在 §G.1 表更新 · 复用 MVP-08 GitStatusResponse · 不新建 struct）：
//   pub frontend_status_snapshot: GitStatusResponse   // frontend 上次 git_status 的快照（复用 MVP-08 type）
//   pub frontend_status_taken_at: i64                  // unix epoch · debug 用

// backend pull 流程：
fn pull(repo: &Repository, req: &PullRequest) -> Result<PullResult, NetworkOpError> {
    // 1. fetch（不动 worktree · 安全）
    let mut remote = repo.find_remote(&req.remote)?;
    remote.fetch(...)?;

    // 2. 关键 · backend 自己再做一次 status snapshot
    let backend_snapshot = compute_status_snapshot(repo)?;
    if backend_snapshot != req.frontend_status_snapshot {
        // 出现漂移 · 拒绝 pull · 让 UI 重新检查
        return Err(NetworkOpError::DirtyWorkingTree {
            modified: backend_snapshot.modified,
            staged: backend_snapshot.staged,
            untracked: backend_snapshot.untracked,
        });
    }

    // 3. 现在才能安全 reset/merge/rebase
    match req.strategy {
        PullStrategy::Merge => repo.merge(...)?,
        PullStrategy::Rebase => repo.rebase(...)?,
    }
    Ok(PullResult { ... })
}

// abort 路径同样必须先 snapshot check · 不能盲 reset
fn abort_pull(repo: &Repository, orig_head: Oid) -> Result<(), NetworkOpError> {
    let snapshot = compute_status_snapshot(repo)?;
    if snapshot.has_uncommitted_changes() {
        return Err(NetworkOpError::DirtyWorkingTree { ... });
    }
    repo.cleanup_state()?;
    repo.reset(orig_head, Hard, ...)?;
    Ok(())
}
```

**Acceptance**（实施 PR 必须含）：

- [ ] PullRequest 含 `frontend_status_snapshot` 字段（spec §G.1 表必须更新）
- [ ] race 测试：frontend 拿 status A → 用户点 pull → mock 其他进程写文件 → backend pull 时 snapshot B ≠ A → 必须返回 DirtyWorkingTree · worktree 不变
- [ ] abort race 测试：rebase 中途用户改文件 → abort_pull 时 snapshot 检查 → 拒绝 reset · 提示用户先 stash

### H.5 AuthMethod 安全实现（防泄露）

`AuthMethod` 含 `passphrase` / `password` 字段 · 必须满足：

```rust
// 1. struct derive 时手动实现 Debug · 把敏感字段打 redacted
impl std::fmt::Debug for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SshAgent => write!(f, "SshAgent"),
            Self::SshKeyFile { path, .. } => write!(f, "SshKeyFile {{ path: {:?}, passphrase: <redacted> }}", path),
            Self::HttpsHelper => write!(f, "HttpsHelper"),
            Self::HttpsManual { username, .. } => write!(f, "HttpsManual {{ username: {:?}, password: <redacted> }}", username),
        }
    }
}

// 2. 不允许 `format!("{:?}", method)` 输出到 log · 用 tracing! 时显式不带敏感字段
// 3. 凭证用完立即 drop · 不缓存到 backend store
// 4. drop 时显式 zeroize（passphrase / password 字段类型考虑 secrecy::SecretString · v0.2 暂用 String + 显式 drop）

// 5. 实施时单元测试覆盖：
//    - println!("{:?}", method) 输出不含 password 子字符串
//    - tracing log 输出不含 password
//    - panic backtrace 不含 password
```

### H.5.1 SSH host key verification（**强制**）

**问题**：libgit2 默认 `RemoteCallbacks::credentials` 走 ssh-agent / SSH key file · 但**不验证远端 host key**。如果不显式配 `certificate_check` callback · 等于 disable host verification · 用户连第一次见的远端 host 就 push 凭证 · MITM 攻击窗口 / DNS 劫持下凭证可被截获。

**强制**：所有 SSH push/pull/fetch 必须配 `RemoteCallbacks::certificate_check` callback · 走 known_hosts + first-seen TOFU（trust on first use）模式：

```rust
use git2::{CertificateCheckStatus, RemoteCallbacks};
use std::path::PathBuf;

fn ssh_known_hosts_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".ssh/known_hosts")
}

fn certificate_check_callback(
    callbacks: &mut RemoteCallbacks,
    workspace_id: String,
) {
    callbacks.certificate_check(move |cert, host| {
        // 1. 取远端 host key fingerprint（cert.as_hostkey() 或 cert.as_x509()）
        let fingerprint = compute_host_fingerprint(cert)?;

        // 2. 查 known_hosts
        match lookup_known_hosts(&ssh_known_hosts_path(), host, &fingerprint)? {
            HostKeyMatch::Known => Ok(CertificateCheckStatus::CertificateOk),

            HostKeyMatch::Changed { stored } => {
                // ⚠ MITM 警告 · 块连接 · 用户必须显式手动恢复
                emit_event(
                    "git:host-key-changed",
                    HostKeyChange {
                        workspace_id: workspace_id.clone(),
                        host: host.to_string(),
                        stored_fingerprint: stored,
                        new_fingerprint: fingerprint,
                    },
                );
                Err(git2::Error::from_str(
                    "SSH host key changed · refusing connection (potential MITM) · review ~/.ssh/known_hosts",
                ))
            }

            HostKeyMatch::Unknown => {
                // 首次见此 host · 弹 UI confirmation · 用户 explicit trust 后写入 known_hosts
                let trusted = prompt_user_first_seen_trust(workspace_id.clone(), host, &fingerprint)?;
                if trusted {
                    append_to_known_hosts(host, &fingerprint)?;
                    Ok(CertificateCheckStatus::CertificateOk)
                } else {
                    Err(git2::Error::from_str("SSH host not trusted by user"))
                }
            }
        }
    });
}
```

**Acceptance**（实施 PR 必须含）：

- [ ] 所有 push/pull/fetch RemoteCallbacks **必须** 配 certificate_check（无配的 PR 拒收）
- [ ] **测试 unknown host**：mock 一个新 host · 调 push → UI 弹 first-seen trust 确认 modal · 用户拒绝 → push 失败 · 用户接受 → known_hosts 追加 + push 成功
- [ ] **测试 changed host key**：mock 已 known host 但 fingerprint 改变 · 调 push → 必须 fail with `SSH host key changed` · UI 弹"远端可能被劫持"警告 · 用户必须手动 vi ~/.ssh/known_hosts 才能恢复（不提供一键覆盖按钮 · 防误点）
- [ ] **测试 known good host**：调 push → certificate_check 返回 Ok · 不弹 UI

**v0.3+ 升级**：known_hosts 路径 / 信任策略移到设置面板（用户可选 strict / TOFU / disabled · disabled 必须显式 opt-in 警告）

### H.6 Force push 保护策略 + force-with-lease（**强制**）

force push 必须同时满足 **2 道防线**：

**防线 1 · 保护分支名单**（黑名单）：

```rust
// crates/core/src/git_sync.rs · push 函数前置检查

const PROTECTED_BRANCHES: &[&str] = &["main", "master", "trunk"];

fn check_protected_branch(req: &PushRequest) -> Result<(), NetworkOpError> {
    if req.force && PROTECTED_BRANCHES.contains(&req.branch.as_str()) {
        return Err(NetworkOpError::RejectedByRemote {
            detail: format!(
                "Branch '{}' is protected · force push is not allowed · rename branch or remove protection",
                req.branch
            ),
        });
    }
    Ok(())
}
```

**防线 2 · 真 force-with-lease**（绑在 push operation · 不是 preflight）：

⚠️ **关键认识**：preflight check（push 前 fetch + 比较 OID）**不是真 lease** · 是 TOCTOU 漏洞 · check 后 push 前远端再 advance 仍会被覆盖。**真 lease 必须绑在 push operation** · 通过 git2 `RemoteCallbacks::push_negotiation` callback 实现（git2 0.20 已支持 · 已 verify）。

```rust
use git2::{PushOptions, Remote, RemoteCallbacks, Repository};

fn force_push_with_lease(
    repo: &Repository,
    req: &PushRequest,
) -> Result<(), NetworkOpError> {
    // §防线 1 仍然在最前 · 拒绝 protected branch
    check_protected_branch(req)?;

    if req.force && req.expected_remote_oid.is_none() {
        return Err(NetworkOpError::Aborted {
            reason: "force=true requires expected_remote_oid (force-with-lease)".into(),
        });
    }

    let expected_oid = req.expected_remote_oid.clone();
    let force = req.force;

    // ⭐ 真 lease：push_negotiation callback 在 push operation 内被 libgit2 调用
    // server 已经 advertised 了 ref 当前状态 · libgit2 把 (src_oid, dst_oid_remote, refname)
    // 三元组传进来 · 我们在这一刻 verify `dst_oid_remote == expected_oid`
    // 如果不匹配 · 返回 Err 让 push 中止（不发 pack data）
    let mut callbacks = RemoteCallbacks::new();
    callbacks.push_negotiation(move |updates| {
        if !force {
            return Ok(()); // 非 force push · 跳过 lease 检查
        }
        let expected = expected_oid.as_ref().ok_or_else(|| {
            git2::Error::from_str("force-with-lease: expected_remote_oid missing")
        })?;
        for update in updates {
            // update.dst() 是远端**当前**该 ref 的 OID（server 实时 advertise 的）· 不是缓存
            let remote_oid = update.dst().to_string();
            if &remote_oid != expected {
                return Err(git2::Error::from_str(&format!(
                    "force-with-lease: stale (expected {expected} · actual {remote_oid})"
                )));
            }
        }
        Ok(())
    });
    // 同时配 credentials / certificate_check / push_update_reference 等其他 callback（略）

    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    let mut remote = repo
        .find_remote(&req.remote)
        .map_err(network_op_error_from_git2)?;
    let refspec = format!("refs/heads/{0}:refs/heads/{0}", req.branch);
    let force_refspec = if req.force {
        format!("+{refspec}") // git refspec 语法 · `+` 前缀 = force
    } else {
        refspec
    };

    // push_negotiation callback 错误会以 git2::Error 形式从 push() 抛出 · 在外层捕获并映射 StaleLease
    remote.push(&[&force_refspec], Some(&mut push_opts)).map_err(|e| {
        if e.message().contains("force-with-lease: stale") {
            // 解析 expected/actual · 返回 StaleLease
            NetworkOpError::StaleLease {
                expected: expected_oid.clone().unwrap_or_default(),
                actual: e.message().to_string(),
            }
        } else {
            network_op_error_from_git2(e)
        }
    })
}
```

**Acceptance（实施 PR 必须含）**：

- [ ] `force=true` 但 `expected_remote_oid: None` → 后端拒绝 `NetworkOpError::Aborted`
- [ ] `force=true` + lease 在 push_negotiation callback 内不匹配 → 后端返回 `NetworkOpError::StaleLease { expected, actual }` · push pack 不发 · 远端 ref 不变
- [ ] **race 测试 1**（preflight gap）：confirmation modal 弹出后 fetch 旧 OID · 远端 advance · push 时 push_negotiation 在 actual ref 上 verify · 必须返回 StaleLease
- [ ] **race 测试 2**（in-flight gap）：push_negotiation callback 后到 actual ref update 之间 · 远端再 advance · 由 libgit2 server 协议层（fetch-pack atomic ref update）处理 · 实施时验证 git2 0.20 + libgit2 1.7 行为：若不能保证 · spec 必须明确接受残余 race · 或砍 force push 推 v0.3
- [ ] `force=false`（普通 push）：push_negotiation callback 直接返回 Ok · git push 自身处理 non-fast-forward 返回 NonFastForward
- [ ] 验证 git2 0.20 `RemoteCallbacks::push_negotiation` 实际行为：`docs.rs/git2/0.20.0/git2/struct.RemoteCallbacks.html#method.push_negotiation` · 实施 spike 第一步必须 PoC · 不假设

v0.3+ 升级：`PROTECTED_BRANCHES` 移到设置面板（用户可自定义）· 支持 glob pattern（如 `release/*`）。

### H.7 跨平台依赖

| 平台 | git2 features | libssh2 / OpenSSL | keychain | v0.2 状态 |
|------|---------------|-------------------|----------|-----------|
| macOS（Apple Silicon / Intel）| `vendored-libgit2 + vendored-openssl` | 静态链接 · 不依赖系统 OpenSSL | osxkeychain（系统集成） | ✅ |
| Linux（Ubuntu 24 X11） | 同上 | 同上 | libsecret（GNOME Keyring · 用户需装） | ✅ |
| Linux（Ubuntu 24 Wayland） | 同上 | 同上 | 同上 | ✅ |
| Windows | 需 win32-openssh + path separator 适配 | TBD | wincred（Windows Credential Manager） | ❌ v0.3+ |

`crates/core/Cargo.toml` 需明确 git2 features：
```toml
git2 = { version = "0.20", default-features = false, features = ["vendored-libgit2", "vendored-openssl", "ssh"] }
```

避免依赖系统 OpenSSL 版本（macOS 默认 LibreSSL · Linux 各发行版差异）。

### H.8 与 MVP-09 / MVP-13 的边界

MVP-21 是**网络层** · 严格隔离：

| 场景 | MVP-09 责任 | MVP-13 责任 | MVP-21 责任 |
|------|------------|------------|-------------|
| 本地 commit | ✅ | ❌ | ❌ |
| Branch CRUD | ❌ | ✅ | ❌ |
| Push to remote | ❌ | ❌ | ✅ |
| Pull / Fetch | ❌ | ❌ | ✅ |
| Push --delete remote branch | ❌ | ❌ | ✅（v0.3 评估 · v0.2 不做）|
| Auth | ❌ | ❌ | ✅ |
| Conflict 处理 | ❌ | ❌ | ✅（仅 abort · GUI mergetool v0.3）|
| 状态栏 ahead/behind 计算 | ❌ | ❌（仅触发 BranchInfo 刷新）| ✅（fetch 后更新） |

实施时严格遵循 · MVP-21 PR 不应碰 `git_ops.rs` / `branch_ops.rs`。

---

**自审四问**（2026-05-01 · vibe sprint Worker B 详化）：

1. **递归完备性**：Acceptance 清单覆盖 Push / Pull / Fetch / Auth / Conflict / 状态栏 / 错误处理 / 性能 / 跨平台 / fixture / IPC contract / Git 栈 全维度 ✅
2. **反向场景**：
   - Non-fast-forward → toast + Force Push 引导 ✅
   - Network unreachable → toast + retry ✅
   - Auth failed → modal 重输 + retry 1 次 + 终结 ✅
   - Force push 误操作 → 保护名单 + 二次确认 + 输入 branch 名 ✅
   - Pull conflict → graceful abort + byte-level 工作区恢复 ✅
   - 中途网络断开 → callback abort + 工作区不变 ✅
   - SSL 错误 / Submodule / 大文件 / Remote 不存在 → §F 全覆盖 ✅
3. **边界适用性**：
   - 0 commit / 100 commit / 1MB 性能（§G.4）
   - SSH agent / SSH key file 两路径
   - HTTPS keychain / Manual modal 两路径
   - macOS / Linux X11 / Linux Wayland 三平台
   - Merge / Rebase 两策略
   - Multi-remote scenarios
4. **YAGNI**：
   - 不做：force push to protected / push tags / push --all / submodule cascade / GUI mergetool / cherry-pick / rebase --interactive / remote add/remove GUI / stash UI / GPG signing / Windows · 全在 §Don't 明示
   - 不引入：第三方 keyring crate（用 OS git credential helper）/ 第三方 git 库
5. **对齐上游 binding**（§G.5）：复用 MVP-07 `BranchInfo` + MVP-08 `GitStatusResponse` · 不造平行类型 · 新增 12 个独立 binding 清单明确
6. **§H 决策锁定全覆盖**：H.1 Git 栈 / H.2 Auth 矩阵 / H.3 Merge vs Rebase 默认 / H.4 API 调用链 / H.5 Auth 安全 / H.6 Force push 保护 / H.7 跨平台依赖 / H.8 与 MVP-09/13 边界 · 防 v0.2 实施期反复讨论
7. **runtime evidence 路径已锁定**：§Phase D 明确 `docs/runtime-evidence/mvp-21/`（按 `.claude/rules/runtime-evidence-location.md` R1）
8. **ID 冲突已解决**：2026-05-03 session 23 PR #210 完成 rename · 文件顶部历史 HTML 注释保留作 trace

---

## 详化完成度评估（Arbiter 审 PR 时参考）

| 12 段必含 | 状态 | 备注 |
|----------|------|------|
| 1. frontmatter | ✅ | id (rename 自原 MVP-11) / type / title / status:**ready** / depends_on / phase / estimate / plan_ref / risk_ref / reviewer (Claude Code) 全填 · ID 冲突已解决 |
| 2. 🎯 目标 Goal | ✅ | 一句话核心 + plan_ref link |
| 3. 📖 背景 Context | ✅ | implementation-plan + CLAUDE.md + 路线图 W14 + 历史尝试 |
| 4. 🛠 实施进度表 | ✅ | Phase A/B/C/D 拆分 + Phase A 起点 checklist（11 项） |
| 5. 🎨 功能范围 Scope | ✅ | Do 5 大类 / Don't 9 项 |
| 6. 🖼 UI 引用 | ✅ | design 原型 line 引用 + 6 类 UI 元素描述（含 force push / pull conflict / auth modal） |
| 7. ✅ Acceptance | ✅ | A-G 7 大组 / 50+ 项 checkbox · 每项含具体测法 |
| 8. 🧪 测试策略 | ✅ | 单元 / 集成 / Criterion / E2E / 视觉回归 / 手动 QA + fixture（带 sshd / nginx mock）+ bench 模板 |
| 9. 💾 数据模型变更 | ✅ | 2 个 app_settings key · 不新建表 + 反模式禁止 + **绝不持久化凭证** |
| 10. §G IPC Contract | ✅ | 14 struct + derive 模板 + G.5 复用 + G.6 新增 12 binding 清单 + G.7 progress event |
| 11. §H 决策锁定 | ✅ | H.1-H.8 8 子段 · 含 Auth 矩阵 + Merge/Rebase 表 + git2 API 表 + 跨平台依赖矩阵 + Auth 安全实现 |
| 12. ⚠️ 已知风险 + Notes + 相关 + 自审四问 | ✅ | 6 风险 + 7 Notes + 6 相关 + 8 条自审 |

**完成度**：12/12 = **100%** · status 已翻 `ready`（2026-05-03 session 23 PR #210 rename + Arbiter approve）。

**遗留问题**：
- ✅ ID 冲突已解决（PR #210 rename · 文件顶部历史 HTML 注释保留作 trace）
- ✅ 决策表已锁定 · 没有"v0.2 启动后再讨论"的悬空项
- ⏳ 等 v0.2 sprint 启动派 Phase A 实施 agent（前置 MVP-13 done · 当前 MVP-13 ready · session 23 主 agent 已派 Codex 跑 MVP-13 Phase A）
