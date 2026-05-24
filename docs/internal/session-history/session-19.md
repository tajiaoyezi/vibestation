# Session 19 · 2026-04-25

**session**: 19
**date**: 2026-04-25
**pr_range**: #117-#152（36 PR · 史上最高产 · 含 #152 ADR-015 翻转跨入 session 20 入口）
**theme**: MVP-11 Native Feel Quality 全 done · MVP-05 Pane 落地（Phase A/B/C）· ADR-006 Ubuntu validated · branch protection 机械化 · 跨 agent fix-up 模式成型

---

## 主题摘要（按 MVP / 治理维度组织 · 36 PR）

### 1 · MVP-11 Native Feel Quality 全 5/5 Phase ✅ done（11 PR）

对标 MUX0 治"web 套壳"观感 · 5 phase 一波收官 · 含跨 agent fix-up 多轮。

- **PR #119**：spec 草稿 + session 19 入口同步（5 phase 决策 + §H 锁 + R31/R-PHASE-1/E 风险）
- **PR #122**：Phase 5 Typography 字体对齐 HIG（系统字体 · 不 bundle）· Kimi 2 远程
- **PR #123**：Phase 1 macOS Vibrancy + 透明窗口 + 禁 webview 行为 · OpenCode · 🟡 待截图补
- **PR #124**：Phase 3 Native Context Menu + 快捷键 · Kimi 1 远程 · 主 agent 修 3 处编译错 + cargo fmt
- **PR #125**：spec round 3 macos-private-api 双启用（`tauri.conf.json` + `Cargo.toml features` 双开 · cargo build 实测发现）
- **PR #126**：Phase 3 round 2 修 PR #124 review 5 处 UX bug（term.reset 替代 SIGINT · Position::Logical · await 串行 · ⌘, 单源）
- **PR #127**：Phase 4 Appearance 7 字段对标 MUX0（bg_opacity / blur / padding / cursor）+ round 2 接 3 处空心字段（bg_blur backdrop-filter · cursor xterm options · settings IPC 链路）
- **PR #128**：Phase 4 D.6 重启持久化自动化测试 · sqlite pool drop 模拟 app 重启 · 替代 manual QA
- **PR #129**：Phase 2 title bar Overlay · Codex CLI 实施 · 主 agent rebase + 修 PR # 引用错
- **PR #130**：Phase 4 D.7 视觉对比 + Phase 1 A.2 截图 + theme race 修复（`#root` whole-tree opacity · ThemeProvider IPC race condition）
- **PR #131**：Phase 1 §A.4 webview 行为禁用 30s 录屏 · production binary `screencapture -V` + `osascript keystroke` 自动化捕获 · Phase 1 ✅ done

### 2 · MVP-05 Pane 分屏 Phase A/B/C 落地（9 PR）

multi-pane terminal 功能从 storage prep（session 18）→ Phase B 完整后端 → Phase C UI scaffolding + 完成版 一波。

- **PR #141**：Phase B Step 2 layout 4 pure function + 17 单元测试 + 7 micro-bench · Sub-agent D（后台 isolation worktree）· 全 sub-microsecond（48-210ns）
- **PR #142**：Phase B Step 1 pane_pty 5 IPC backend · §H.6 锁 A 独立命名空间 · 防 tab_pty collision
- **PR #143**：Phase B Step 2 IPC · 5 layout command + transactional pane_service · rusqlite Transaction · §H.3 atomicity rollback
- **PR #144**：Phase C UI scaffolding · pane_init_for_tab IPC + 3 SolidJS 组件（PaneTerminal / PaneSplitView / PaneSplitter）+ 快捷键 hook · 0 集成
- **PR #147**：Phase C Track B PaneSplitter 拖拽 60FPS + ratio commit · 3 路并发 sub-agent B
- **PR #148**：Phase C Track C SmartLayoutMenu 组件 + dry-run 预览 + 二次确认 UX · 3 路并发 sub-agent C-2
- **PR #149**：Phase C Track A Terminal.tsx 集成 PaneSplitView + 快捷键 wire + pane_focus + PaneListResponse.focusedPaneId · 主 agent
- **PR #150**：Phase C §C Smart Layouts wire · ⌘⇧P 命令面板触发 · 主 agent
- **PR #151**：Phase C §F 仪表化（4 个 inline performance.now）+ Phase D capture script + spec done 翻转 · 主 agent · runtime 实测留 Arbiter 30 min

### 3 · ADR-006 Ubuntu validated · v0.1 GA 双平台解锁（3 PR + ADR 翻转 + ADR-015 翻转）

kimi-ubuntu24 远程独立电脑实测 · 30 cold boot 0 fail · X11 108ms / Wayland 107ms · v0.1 GA macOS-first → 双平台。

- **PR #137**：SPIKE-01+02 Phase B Ubuntu 双 backend 验证 · IME fcitx5 PASS · pnpm tauri build · 截图归档 · ADR-006 caveat 解除条件达成 · kimi-ubuntu24
- **PR #138**：ADR-006 caveat removal · Ubuntu validated · v0.1 GA 双平台 · 本机 Kimi（doc 专职）· 0 代码改动
- **PR #139**：Ubuntu installer + MVP-11 Linux vibrancy/titlebar 验证 · AppImage 78MB / deb 5.5MB · round 3 fix-up（B4 真实 capture · scrot → import -window mutter-frame）· kimi-ubuntu24

### 4 · MVP-10 Phase B Telemetry Spike + ADR-015（2 PR）

锁定 telemetry 实施栈 · 解锁 SDK 编码。

- **PR #120**：Sentry Spike + ADR-015 proposed · Codex CLI · §H.1.1 4 步验证全过 · `default_integrations = false` + `before_send` 双层白名单 + cargo-bloat 实测 · round 2 ADR Step 3 number 澄清
- **PR #152**：ADR-015 proposed → accepted by Arbiter · 主 agent · session 20 入口翻转 · 解锁 MVP-10 Phase B SDK 编码（B5 任务）

### 5 · MVP-08 Phase E fix-up + R-PHASE-E 技术债（3 PR）

OpenCode hallucinate 4 数字 · 主 agent round 2 降为 partial done + v0.2 fixture 进 git。

- **PR #117**：Phase E fix-up · DevTools 量化 + Phase D PNG 重命名 + spec done · OpenCode round 1 + 主 agent round 2 commit 6d04fb8 降级 partial（A.2 / A.6 / F.3 / F.6 4 数字 metrics 文件实际未含）+ R-PHASE-E 技术债（manual capture 30-60min）
- **PR #136**：R-PHASE-E round 2 真实 burst capture 替换 round 1 估算 · 9 张 runtime 证据
- **PR #140**：R-PHASE-E v0.2 fixture generator 脚本 · `gen-10k-diff.sh` + `gen-1k-files.sh` · sub-agent C 后台 isolation worktree · 反模式记录（中文全宽圆括号 U+FF08 触发 bash unbound var）

### 6 · MVP-09 Phase B Status 面板（1 PR）

- **PR #118**：Status 面板加 Stage/Unstage/StageAll/UnstageAll + CommitBar 新建 + 错误对话框（IdentityMissing / DetachedHead / PreCommitHook）· Kimi 1 远程 · round 2 修 detachedHead 死循环（retry 引发 dialog 重开）+ StageAll/UnstageAll missing revert + console.log 清理 + IdentityDialog email 校验

### 7 · 治理：CI / Branch protect / 流程（2 PR）

- **PR #121**：CI 关闭 main push trigger · 仅保留 workflow_dispatch · GitHub Actions billing 失败应对 · 转向本地 7 gate
- **PR #145**：pre-push hook 阻止直推 main · `package.json prepare` script + `core.hooksPath = .githooks` 自动配置 · 每台机器 clone + pnpm install 即激活 · 4 场景实测全过（main 阻 / bypass 通 / feat 通 / pnpm install 自动配）

### 8 · Doc / Progress 滚动同步（4 PR）

- **PR #132**：session 19 同步 PR #129-#131 + MVP-11 Phase 1/4 ✅
- **PR #133**：MVP-11 5/5 全 done + 本地分支 cleanup（18 origin/ → 1）+ 团队收缩 3 人
- **PR #134**：session 17/18 归档至 `docs/session-history/` · M-2 滚动窗口规则首次完整执行
- **PR #146**：session 19 收尾 doc 同步 · A+B+C+E 4 项 update

### 9 · MVP-04 §I shell 兼容矩阵（1 PR）

- **PR #135**：§I 22 用例 shell 兼容矩阵 cargo test 化 + git_ops clippy fix · 完成 MVP-04 Phase D follow-up

---

## 特色（session 19 史上最高产 + 协作模式成型）

### 史上最高产 36 PR / single session

PR #117-#152 · 单 session 史上最高 · MVP-11 全 done + MVP-05 Phase A/B/C + ADR-006 Ubuntu validated + branch protect 机械化 一波收官。session 18（11 PR）的 3.3 倍。

### MVP-11 Native Feel Quality 完整收官 🎉

5/5 Phase ✅ 全 done · 对标 MUX0 治"web 套壳"观感 · **零技术债交付**（A.5 + R-PHASE-2.linux Linux 部分由 PR #139 kimi-ubuntu24 round 3 真实 capture 补完）。主 agent 自动化工具链 capture 全部 manual evidence（screencapture -V/-R + osascript AXRaise/keystroke + Read 工具视觉验证 + sqlite3 直改 KV + pkill+restart dev tree）替代"必须 Arbiter 本地补"的常规假设。

### Tauri 2 dual-enablement 实测发现

`tauri.conf.json macOSPrivateApi: true` 和 `Cargo.toml features = ["macos-private-api"]` 必须**双启用** · 缺一 cargo build error `dependency features ... does not match the allowlist defined under tauri.conf.json`。spec round 3（PR #125）固化此发现。

### Sub-agent 后台并发协作模式成型

PR #140（fixture generator）+ PR #141（layout pure functions）首次用 `isolation: worktree` + `run_in_background: true` 跑 Claude Code sub-agent · 主 agent 监听通知后 push + PR + merge · 替代 OpenCode/Codex（已离开 · 团队收缩 3 人）。

PR #147-#150 进一步推动到 **3 路并发**（Track A 主 agent · Track B / Track C sub-agent）· Phase C 完成版 4 PR 一气呵成。

### 跨 agent fix-up pattern 成型

主 agent 在外部 agent PR 后立即接 round 2/3 修编译错 / 格式化 / hallucinate / UX bug / 空心字段 / 视觉语义错 / B4 真实捕获 · session 19 实证 8 PR：

| Round 1 PR                       | Round 2/3 fix-up        | 修什么                                                          |
| -------------------------------- | ----------------------- | --------------------------------------------------------------- |
| PR #117（OpenCode）              | 主 agent commit 6d04fb8 | hallucinate 4 数字降级 partial done + R-PHASE-E                 |
| PR #118（Kimi 1）                | round 2 commit          | detachedHead 死循环 + Missing revert + console.log + email 校验 |
| PR #124（Kimi 1）                | 主 agent 修编译错       | ContextMenu trait import + popup_at owned + set_menu Result     |
| PR #126（主 agent）              | —                       | 修 PR #124 review 5 处 UX bug                                   |
| PR #127（OpenCode round 1）      | 主 agent commit 05199d1 | bg_blur CSS var 无人消费 + cursor xterm 不读 CSS + spec ✅ → 🟡 |
| PR #129（Codex CLI）             | 主 agent rebase         | 解 styles.css conflict + 修 PR # 引用错                         |
| PR #130（主 agent）              | —                       | theme race condition + #root whole-tree opacity                 |
| PR #139（kimi-ubuntu24 round 3） | —                       | B4 真实 capture（scrot → import -window mutter-frame）          |

### 主 agent macOS GUI 自动化突破

`screencapture -R/-V` + `osascript AXRaise/keystroke` + Read 工具视觉验证 + sqlite3 直改 KV + pkill+restart dev tree 工具链 · 替代"必须 Arbiter 本地补"的常规假设 · 解决 R-PHASE-1（Phase 1 A.4 webview 30s 录屏）+ R-PHASE-4（D.7 Opacity 3 档对比截图）共 5 项 manual capture。

### Branch protection 机械化（PR #145）

`.githooks/pre-push` + `package.json prepare` + `core.hooksPath` 自动配置 · 每台机器 clone + pnpm install 即激活 · 防止任何 agent（含主 agent / Codex / OpenCode / Kimi-ubuntu24）误推 main · `SKIP_BRANCH_PROTECT=1` Arbiter override · 无 husky 依赖（单行 git config）· CLAUDE.md L131 文档化机械防护。

仓库私有 + 非 GitHub Pro · branch protection API 不可用（403）· v0.2 评估升级 GitHub Pro 或仓库公开补 GitHub 端硬墙（branch protection + required reviewer + CODEOWNERS）。

### CI billing failure 应对：本地 7 gate（PR #121）

GitHub Actions billing 失败导致 main push 后所有 job 2 秒 fail（误导未来 agent）· `.github/workflows/*.yml` 全部去掉 `push: branches: [main]` 触发 · 仅保留 `workflow_dispatch:`。CI 验证规则改为本地 7 gate（cargo clippy/fmt/test + pnpm typecheck/lint + spec validator + whitespace）+ merge 后 5 min 内 `gh api .../check-runs` 回扫。

### Author 归属 100% 防御性 unset 合规

主 agent 起手 `git config --local user.name "Claude Code"` + `noreply@anthropic.com` 覆盖 · 全部 17 PR（session 19 主 agent 实施部分）无跨 agent author 错归。session 18 末主 repo `.git/config` 被 Codex worktree 污染为 "Codex CLI" 的事故未在 session 19 复现。

### v2-D.1 PR body trailer 100% 合规

session 19 全部 36 PR 三行 trailer（Implemented by / Reviewed by / Arbiter approval）齐 · 无缺失。

### ADR 增长

15 ADR · ADR-015 翻转 accepted（PR #152）· 14 历史全 accepted（含 ADR-006 升级到 Ubuntu validated · PR #138）。

### 团队收缩到 3 人

OpenCode + Codex CLI 离开（之前的 5+ 人多 agent 协作）· 团队稳态：主 agent（Claude Code）+ 本机 Kimi（远程 API · doc 专职）+ kimi-ubuntu24（远程独立电脑 · Linux 平台专项）。Sub-agent 后台并发模式（isolation worktree）替代外部 agent 编码任务。

### 本地分支全 cleanup

session 19 收尾 · 18 origin/ branches → 1（仅 main）· 4 个 stale local branches 删除 · 多次 worktree remove · PR #129 conflict 解决。

---

## 遗留进入 session 20

- **MVP-05 Phase D runtime 实测 capture**（Arbiter 本地 ~30 min · v0.1 GA gate 前推荐）· `scripts/capture/mvp-05/capture-phase-d.sh` + `measure-memory.sh` 就位 · `metrics-mvp-05.md` 待填实测数字
- **PROGRESS.md sync PR #146-#152**（5 PR 滚动窗口未补 · session 20 入口处理）
- **MVP-09 剩余 Phase B 后续**
- **MVP-10 Phase B Sentry SDK 编码**（PR #152 ADR-015 翻转后解锁 · session 20 主 agent 接）

---

← 当前进度见 [docs/PROGRESS.md](../PROGRESS.md)
