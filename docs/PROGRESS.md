# 进度快照 · PROGRESS

> **定位**：当前状态面板（agent 和人类都先读本文件获取"我是谁 / 做到哪 / 下一步 / 卡点"）。
> **更新约定**：session end / 阶段切换 / 决策变化时手动更新。不要每个 commit 都更新（噪音大）。
> Session 历史归档到 `docs/session-history/`（Phase 3 已建立）——**不要**归档到 CHANGELOG（CHANGELOG 是 release-please 自动维护的发布日志）。
> **PR 列表滚动窗口规则**（M-2 · 2026-04-21 session 13 audit）：本文件"已合入的 PR"段**只保留最近 2 个 session 的摘要** · 更早的以 `git log --all` + `docs/session-history/` 归档文件为准 · 每 session 末整理。

---

## 📊 固定状态字段

| 字段                      | 值                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | 更新时机      |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| **Active branch**         | 见 `git branch --show-current`（本表不硬编码分支 · 避免 PROGRESS 和现实漂移 · H-2 · 2026-04-21）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | —             |
| **Latest commit**         | 见 `git log --oneline -1`（不在此处硬编码）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | —             |
| **Worktree status**       | 见 `git status` + `git worktree list`（三方 worktree 隔离 · 无 shared-tree 冲突）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | —             |
| **Unpushed branches**     | 见 `git branch -vv`（不在此处硬编码）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | —             |
| **Next concrete action**  | **session 17 · PR #99/#100/#101/#102 全部合入**（MVP-04 Phase F 证据收口 · MVP-08 Phase A/B 连续落地 · PR 级 Actions 自动运行关闭）· **下一步**：**MVP-08 Phase C**（Diff 视图前端：split/unified 切换 + 大文件 fallback + Status 文件点击打开 Diff）→ **MVP-08 Phase D/E**（fs watch 自动刷新 + runtime 证据 / 性能量化）→ **MVP-09**（Stage/Unstage + Commit 写路径）· 主线外 · MVP-04 Phase D（shell 兼容 + Claude/Codex CLI 实机 · 最低优先）· SPIKE-06 §B Apple Dev 申请（用户决策）· MVP-07 UI 截图 + kernel benchmark GA gate 补齐 | session end   |
| **Blocked by**            | SPIKE-06 §B Apple Dev Program 申请（**用户决策中** · $99/y · 审核 2d-2w · 影响 MVP-10 codesign 但不阻塞 MVP-04-09）· SPIKE-01/02 §B + MVP-01 Phase C · **Ubuntu 24 LTS 环境 · 已降为 v0.1 GA 最低优先**（S-3 · 2026-04-21 · 见下方"v0.1 发布策略"）· 不阻塞 macOS 主线                                                                                                                                                                                                                                                                                                                                                                                                                 | 阻塞变化      |
| **Missing infra**         | Ubuntu 24 LTS 环境（最低优先 · macOS 主线不依赖）· Apple Developer Program（SPIKE-06 §B + MVP-10 GA 前置）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Phase 完成时  |
| **Required env/accounts** | ✅ rustup stable 1.95 / Node 20.17 / pnpm 9.15 / tauri-cli 2.x · ⚠️ Ubuntu 24（最低优先 · 非主线）· ⚠️ Apple Dev（GA gate）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | 新账号/工具时 |
| **v0.1 发布策略**         | **macOS-first**（S-3 · 2026-04-21 audit）· v0.1.0-alpha 只发 macOS · v0.1.0 GA 前评估 Ubuntu 状态：若 Ubuntu 环境仍缺 → v0.1.0 GA 也 macOS-only · Ubuntu 延到 v0.2 · 决策基线：Ubuntu 非主用户设备 · 环境搭建成本高 · 不阻塞 macOS 主线                                                                                                                                                                                                                                                                                                                                                                                                                                                | GA 前最终评估 |

---

## 📍 当前位置

**阶段**：🏗️ **session 18 起点 · MVP-08 Phase C done · 主线进入 Phase D/E**（2026-04-25 · PR #99/#100/#101/#102/#103/#105 已合入 main）· `MVP-04` 已完成 Phase A/B/C/E/F，当前只剩 Phase D shell 兼容（低优先）· `MVP-08` 已完成 Phase A 后端 contract + Phase B Git Status Bottom Panel + Phase C Diff 视图前端（含 Git Status/Git Log → Diff 接通 + view mode 持久化 + Git Status 自动刷新 + Terminal/Diff 主区切换不重挂），主线进入 Phase D fs watch（`notify` 6.x · 替换 polling）或 Phase E runtime 证据 · PR 级 GitHub Actions 自动运行已关闭（仅保留 `push main` + `workflow_dispatch`）· 14 ADR accepted · v2-D.1 规则稳态运行
**日期**：2026-04-25（session 18 起点 · session 17 主交付 PR #105 已 merge · 本次 session 起手做文档同步追上 main）
**GitHub**：<https://github.com/tajiaoyezi/vibestation>（PRIVATE）
**已合入的 PR（滚动窗口 · 只保留最近 2 session · 更早见 `git log --all` + `docs/session-history/`）**：

### Session 17（2026-04-23 · MVP-04 Phase F 收口 + MVP-08 Phase A/B/C 落地 + PR Actions 分钟节流）

- **PR #105 `471349f`**（**MVP-08 Phase C Diff 视图前端集成 done · 主线里程碑**）· Codex CLI · 4 路并行成果整合：OpenCode core `diff_set_view_mode` + Kimi 隔离的 `web/src/panels/Diff/`（DiffPanel 303 行 + diffApi + styles 299 行）+ Git Status 自动刷新 subscription/polling 接通 + shell PATH 修复（PTY 启动）· `crates/app/permissions/diff.toml` 加 `diff_set_view_mode` permission · `App.tsx` + `MainContent.tsx` 加主区 Diff/Terminal 切换 state（Terminal 不重挂）· Git Status 文件行 + Git Log 文件行 → 点击打开 Diff 主区 · supersede PR #104（孤立 Diff 面板分支）· Implemented + Reviewed by Codex CLI self-review · Arbiter approval 2026-04-23 22:07 CST
- **PR #103 `fb2d755`**（**入口文档对齐 main 真实进度**）· Codex CLI · 在 PR #100/#101 落地后同步 PROGRESS / CLAUDE / 入口指针 · 防止下次 agent 误读
- **PR #102 `d1c5489`**（**PR 级 GitHub Actions 自动运行关闭 · 只保留 `push main` + `workflow_dispatch`**）· Codex CLI · `.github/workflows/ci.yml` / `secret-scan.yml` / `task-spec-validator.yml` 去掉 `pull_request` 触发 · `secret-scan` 删除 `pull-requests: read` 权限 · `task-spec-validator` 保留脚本内 PR 分支逻辑便于未来恢复 · 结果：新 PR 不再消耗 GitHub Actions 分钟数，但后续 agent 必须本地先跑 gate，merge 后再回看 `main` 的 check runs
- **PR #101 `e09b9df`**（**MVP-08 Phase B Git Status Bottom Panel done**）· Codex CLI · Bottom Panel 真正承载 Git Status 只读面板 · 3 分组 `Staged / Unstaged / Untracked` + 状态码 / 相对路径 / 加减行数 + `Refresh` · 非 git workspace / 错误态有明确提示 · 新增 `GitStatusPanelSettings` / `GitStatusCollapseRequest` / `GitStatusGroup` + `git_status_get_settings` / `git_status_set_group_collapsed` · 分组折叠态写入 `app_settings` · 后续 Phase C 直接消费稳定面板状态 contract
- **PR #100 `424e894`**（**MVP-08 Phase A diff/status IPC 后端 done**）· Codex CLI · `crates/core/src/diff.rs` + `git_status.rs` 落地 · `similar` 负责文本 diff 计算 · `gix` 读取 commit / parent blob · `git2` 读取 staged / unstaged 对照源 · 6 个 Tauri commands + 8 个 ts-rs bindings + ACL/permission 补齐 · `DiffRequest.allowLargeFile` + `DiffResponse.truncatedReason/lineCount` 预留 large-file confirm 与 hard stop 协议 · `git_status_subscribe` / `git_status_unsubscribe` 先占位给 Phase D
- **PR #99 `a0c9699`**（**MVP-04 Phase F runtime 证据与量化 done**）· Codex CLI · `docs/runtime-evidence/mvp-04/` 新增 5 张截图 + `metrics-phase-f.md` · create / rename / switch / close / scrollback 证据补齐 · A.5 / E.2 切 Tab latency AX 自动化 median `20 ms` · E.4 页面内同步 JS 执行 `sync max = 3 ms` · `frame delta = 19 ms` 仅作上下文 · MVP-04 整体仍保持 `ready`，只剩 Phase D shell 兼容

### Session 16（2026-04-22 · 三路并发 3 PR 全 merged · MVP-04 Phase E 里程碑 + 零代修 session · 规则内化成就）

- **PR #98 `67f89f5`**（**MVP-09 spec 对齐 · 4 件加强**）· Kimi · `Stage/Unstage + Commit` spec 补齐与 MVP-08 / SPIKE-04 的实现衔接 · 让 v0.1 Git 写路径任务在当前代码现实上可直接接续
- **PR #97 `e852e6d`**（**session 16 项目整体进度快照归档**）· Claude Code · 冻结 2026-04-22 收尾时的全局状态 · 作为滚动窗口外的归档入口
- **本 PR（session 16 closeout）** · Claude Code · PROGRESS session 16 段新建（PR #93/#94/#95 摘要）· session 14 按滚动窗口规则移出 · CHANGELOG [Unreleased] 追加 MVP-04 Phase E 里程碑 + Kimi/OpenCode/Codex 三路并发零代修成就 + PR #93 fix-up round 2 零内容改动案例 + Kimi 协作 10→11 次更新 · Next action / 阶段描述更新到 MVP-04 Phase F + MVP-08 实施可启动
- **PR #95 `348a361`**（**MVP-04 Phase E scrollback 持久化 done · 主线里程碑 · Codex 首次零代修交付**）· Codex CLI · 后端 PTY stdout 接单 writer lane（`enum ScrollbackCommand { Append / Shutdown }` + crossbeam-channel 保序 · 优于 prompt 推荐的"thread::spawn per flush"）· `ScrollbackBuffer` 含 `partial_line`（跨 chunk 行边界）+ `pending_since`（时间 debounce）+ `drain_due`（100ms / 100 行 whichever first）+ `drain_all`（exit 强制 flush 含 partial_line · 不丢最后几行）· `workspace_init` 后把 DbPool 注入 PTY 层（`set_pool` 单次设置 · 避免改 app 启动生命周期）· 前端 `hooks.ts::fetchScrollback` + `TerminalPane` mount 分批写回（1000 行 batch + `requestAnimationFrame` yield 避免阻塞主线程）+ `Terminal.tsx::isNewlyCreated` 跳过新 tab 的空 fetch · 集成测试 `pty_scrollback_integration.rs` 105 行（spawn → write `seq 1 100` → exit → fetch → assert 100 行 + 顺序）+ 4 单元测试（`parse_chunk_to_lines_splits_complete_lines` / `preserves_partial_tail` / `scrollback_flushes_when_due` / `force_flushes_partial_tail`）· **未改 TabsDao 签名**（向后兼容 Phase A 36 单元测试全过）· 未新建 migration（v5 维持）· 未新增 IPC command · 未手写 TS interface · Runtime 证据 4 张截图（9.3MB / 10MB 上限 · 单张 max 3.27MB · 未来 nit 压缩）· **Author 归属一次正确**（PR #71/#82 两次错归 Kimi 教训后 · Codex 自己防御 git config + trailer + log verify 3 条铁律 · §2.5.3 规则内化生效 · 主 agent 零代修）· CI 7/7 pass（Rust 2m39s）· 392 行新代码 · Phase D shell 兼容 / Phase F 证据 + 性能量化 待
- **PR #94 `0325499`**（**`.claude/rules/README.md` 索引补齐 · OpenCode 首次零代修交付 + 3 项超预期**）· OpenCode · 109 行 · 5 字段规则索引表（文件 / 触发条件 / 核心要求 / 关联全局 rule / 事件源）· 4 规则全覆盖（dispatch-prompt-template · spike-delivery-checklist · runtime-evidence-location · tauri-v2-patterns）· 事件源 100% 真实可考证（PR #28 Tauri ACL deny / PR #71/#82/#83 author 错归 / SPIKE-04.5 §A.3 OpenCode 自行 accept / ADR-011/013）· 阅读顺序建议按任务类型分支（外部 agent dispatch / Spike / GUI-IPC / chore / all）· 项目 vs 全局 rule 对照表 4 条 · 新增规则 5 步指南 + "通用 vs 专项"决策树 · 维护信息段 · **超预期 3 项**：§项目规则间交叉引用 ASCII 树状图 + §常见踩坑速查 6 条表 + 如何新增规则决策流程（prompt 未要求）· 本地 prettier --check + 非表格行 ≤ 120 char · CI 7/7 pass · **对比 PR #83 代修 R1-R4 4 项 · 本次零代修 · OpenCode 本 session 最干净交付**
- **PR #93 `a15c44f`**（**MVP-08 spec 对齐 MVP-07 实施 · Kimi 第 11 次协作 · 首次 fix-up round 2 · 零内容改动纯位置修**）· Kimi · 2 commits（5 件加强 + §G.5 位置 fix-up）· **5 件加强**：(1) §🛠 实施进度表 5 Phase 拆分（A-E · 仿 MVP-04 模式 · 下次 agent 起点 Phase A）· (2) **§G.5 binding 复用决策锁 (a)**（`FileChange` 复用 MVP-07 已落地 binding `status: string` · `FileStatus` enum 不新增 · 避免 H2 类前端漂移 · G.5.2 v0.2 升级路径）· (3) §H.6/H.7 fs watch 选型 + 测试（`notify` 6.x 主 · macOS FSEvents 2s 下限 · 2 fallback · Linux fd 爆降级 polling · Soak 10k/s 防抖验证）· (4) §A.2/§F 性能门槛拆解（纯渲染 < 50ms vs 端到端 < 200ms 双指标 · 1000 文件拆后端 100ms + IPC 30ms + 前端 70ms）· (5) §H.4 bundle 体积更新（对齐 MVP-07 实测 · `cargo bloat` 量化触发 · H.4.1 fallback 决策树）· **fix-up round 2**：原插入位置 G.1 → G.5 → G.2 → G.3 → G.4 编号跳序 · Kimi 主动 amend + git config 防御性 unset（初提交错归 Codex CLI 自发现 + 修复 · §2.5.3 硬约束规则内化 · 对比 PR #71/#82 需主 agent 代修 · 进步显著）· **数学证明零内容改动**（sorted diff = 0 行 · 110 行输入 = 110 行输出 · 纯 +42/-42 位置 move）· Kimi 11 连 merged 战绩保持 · Claude Code reviewer self-push 翻转 gate (a) 审计痕迹 · PR body v2-D.1 三行 trailer 齐

> **滚动窗口前**：session 15 及更早（PR #1-#92）的完整摘要请查 `git log --all --oneline | grep PR` · 或 `docs/session-history/` 里的归档文件。本 PROGRESS 每 session 末按 M-2 规则整理（保留最近 2 session · session 16+17 当前窗口内）。

## ✅ 已完成（累计 · Pre-code Phase 1-4）

### Phase 1 · 战略与决策（PR #1/#2 · session 3-4）

- [x] B 阶段技术调研 / planner v1 / 4 视觉方向 + Calm Studio 定稿 / 2 Logo 候选
- [x] Codex 项目级评审（7 CRITICAL + 12 HIGH + 5 MEDIUM + 13 反对）
- [x] 4 项分歧决策：Apache 2.0 / MVP B 折中 / AI-Aware 撤出 / Tauri 改口
- [x] planner v2（14 章 + 附录 · 30 风险）
- [x] 独立仓库 + GitHub push + Apache 2.0 LICENSE + NOTICE
- [x] Phase 1 v1 → v4 simplified（承认过度设计 · 砍多 agent 治理抽象 · 保留 Git 普世 + 自审四问）

### Phase 2 · task spec 框架（PR #3/#5/#6/#7/#8/#9/#10 · session 4-5）

- [x] `docs/tasks/` 框架：schema + `_template.md` + README 索引
- [x] **SPIKE-01..07**（7 个 Spike spec · W0 硬通过矩阵 + SPIKE-07 v1.0-pre parser 验证）
- [x] **MVP-01..20**（20 个 MVP spec · v0.1 详细 + v0.2/v0.3/v1.0 占位）
- [x] 流程治理：5 步导游 · blocked 语义（`blocked_from`）· per-task 报告 · 翻转 gate 二选一
- [x] Codex 对抗性审查 **12 findings** 全闭合（R1-R6 · 4 commits 修）

### Phase 3 · 架构决策与治理文档（PR #12 · session 5）

- [x] **ADR × 10**：#1 License · #2 MVP 范围 · #3 AI-Aware v1.0 vision · #5 Workspace · #6 前端栈 · #7 Diff 自建 · #12 桌面框架 · #13 Git 栈 · #14 存储 · #15 PTY（accepted 6 + proposed 4）
- [x] **CONTRIBUTING.md**（贡献指南 · 含用户拍板 gate）
- [x] **CHANGELOG.md**（Keep a Changelog · Phase 1-3 条目）
- [x] **CODE_OF_CONDUCT.md**（Contributor Covenant 2.1 中文）
- [x] `docs/spikes/` + `docs/spike-artifacts/` + `docs/session-history/` 3 目录建立 · 各有 README + 安全约束
- [x] Codex 5 findings（3 HIGH + 2 MEDIUM）全闭合

### Phase 4 · GitHub 基础设施（PR #11 · session 5）

- [x] `.github/ISSUE_TEMPLATE/` 4 模板（config / bug / feature / task_spec_proposal）
- [x] `.github/PULL_REQUEST_TEMPLATE.md`（强制 Implemented by / Reviewed by / 翻转 gate / 自审四问）
- [x] `.github/dependabot.yml`（cargo + npm + github-actions 周更）
- [x] `.github/workflows/ci.yml` · skeleton（markdown-lint active · rust/frontend 占位）
- [x] **`.github/workflows/secret-scan.yml`** · gitleaks + `gitleaks-bypass-guard`（防内联 bypass marker 绕过 · 详见 SPIKE-06 §A.5.3）
- [x] **`.github/workflows/task-spec-validator.yml`** · frontmatter schema 校验 · 无 paths filter（防 required-check pending）
- [x] **`scripts/validate-task-spec.mjs`** · 224 行 · 自写 parser + 9 条 adversarial self-test + 7 类 schema 规则
- [x] **`docs/BRANCH-PROTECTION.md`** · admin 应用 main 保护的完整 checklist
- [x] Codex 3 HIGH findings 全闭合 + CI self-trigger fix（`a6fd6c6`）

### Codex 对抗性审查全统计（至 session 6 结束）

- **9 轮审查 · ~33 findings 全闭合**（含 session 6 三轮 + 二次复审）
- 平均每轮从 4 HIGH 收敛到 1-2 HIGH · 质量曲线明显
- 最深发现：SPIKE-04 op-log phantom data（marker-loss crash window · R6 F1 reconcile forward）· SPIKE-05 后端 IPC queue 满 HOL

### Spike W0 实施（session 7 · 2026-04-19）

**SPIKE-01 · Tauri 空壳启动 · Phase A macOS PASS（PR #20）**

- [x] Tauri 2 vanilla-ts 骨架 `spike-tmp/spike-01-tauri/`（gitignored · 8.2MB .app · 4MB dmg）
- [x] 冷启动 10 次 median **202ms**（目标 < 2s · 10× 余量）· Range 42ms 极稳
- [x] 中文 IME 录屏 + 5/5 肉眼验证
- [ ] Phase B Ubuntu 待环境就绪（prompt 备好 · 日文全平台 skip）

**SPIKE-02 · Tauri 硬通过矩阵 · Phase A macOS PASS（PR #22）**

- [x] 10x 稳定性 10/10 · median 212ms · Bundle 10MB/.dmg 4MB
- [x] Clipboard plugin smoke（读写 + 跨 app Cmd+V 含中日英+emoji UTF-8 完整）
- [x] FS plugin smoke（读写 + terminal cat 验证）
- [x] 中文 IME 录屏
- 2 项降级：updater 归 SPIKE-06（Apple Dev key 依赖）· 日文 IME 全平台 skip（用户决策）
- [ ] Phase B Ubuntu 待环境

**SPIKE-03 · git2 vs gix benchmark · done (PR #23 待 merge)**

- [x] OpenCode agent linux kernel 1.44M commits 实测
- [x] 结论 **(B) 读切 gix · 写保留 git2**：gix log -100 warm P99 **12.65ms** vs git2 **24964ms**（gix 1973× 快）
- [x] ADR-007 proposed → accepted · 决策表 #13 B→A

**SPIKE-04 · storage benchmark · done (PR #24 待 merge)**

- [x] OpenCode agent 2 次交付（v1 被 Claude review BLOCK · v2 补做 accept）
- [x] §A 性能：redb 写入 P99 31.94s / rusqlite 9.96s · 两者都通过
- [x] §B 安全：redb 2.6.3 **B.2 坏库检测 FAIL**（silent 读出可能错误数据）
- [x] 结论 **(B) 锁 rusqlite**（redb 2.6.3 被 supersede）
- [x] ADR-005 proposed → accepted（结论翻转 redb→rusqlite）· 决策表 #14 B→A（rusqlite）
- **R27 未真 close · 需 SPIKE-04.5 on rusqlite 补 B.1-5**

**SPIKE-04.5 · rusqlite 数据安全 · ready (PR #25 待 merge · 本 PR 新建)**

- [ ] 新建 spec · depends_on: SPIKE-04 · blocks: MVP-02/06/10/19
- [ ] A 性能复测（rusqlite 100 行范围 · 澄清 SPIKE-04 歧义）
- [ ] B.1-5 全链路在 rusqlite 上实测 · 补 SPIKE-04 瑕疵（B.3 实 assert · B.4 auto-backup · B.5 production op-log + 自动回滚 UI）
- [ ] 结论：rusqlite B.1-5 全过 → ADR-005 修订 "R27 真 close" | 失败 → Arbiter

**Codex 对抗性审查新统计（session 7）**

- Claude Code 作为 SPIKE-04 reviewer：发现 4 CRITICAL（bulk_write 单样本 / range 事后洗白 / sudo purge 未执行 / B.1-5 未做）· 退回 opencode · v2 全闭合
- 说明多 agent 并行交付 + 独立 review 的质控链路有效

## 🔜 下一步（按执行顺序）

### 🔐 **用户手动步骤**（`docs/BRANCH-PROTECTION.md` · 当前**已显式暂缓**）

用户已表态暂不应用 main 分支保护（单人 + Codex review 模式下不致命）。**当前流程靠 reviewer 肉眼守门**（accepted tech debt · 见 `docs/tasks/README.md` §原则 7）。

升级触发条件（任一）：

1. 仓库改 public
2. 第二位外部 contributor 出现
3. MVP-01 开始写 Rust 代码
4. 第一个 release tag

触发时按 `docs/BRANCH-PROTECTION.md` checklist 一次性应用。

### 🚀 **Spike Week 0**（进行中 · session 7 · 多 agent 并行）

1. **W0-D1** · [SPIKE-01](./tasks/SPIKE-01-tauri-three-platform-boot.md) · **Phase A macOS ✅ PASS（PR #20 merged）** · Phase B Ubuntu 等环境
2. **W0-D2** · [SPIKE-02](./tasks/SPIKE-02-tauri-hard-pass-matrix.md) · **Phase A macOS ✅ PASS（PR #22 merged）** · 2 项降级（updater + 日文 IME）· Phase B Ubuntu 等环境
3. **W0-D3** · [SPIKE-03](./tasks/SPIKE-03-git2-gix-read-benchmark.md) · ✅ **done（PR #23 merged）** · 结论 (B) 读切 gix · 写保留 git2
4. **W0-D4** · [SPIKE-04](./tasks/SPIKE-04-storage-benchmark.md) · ✅ **done（PR #24 merged）** · 结论 (B) 锁 rusqlite（redb 2.6.3 B.2 FAIL）
5. **W0-D4.5** · [SPIKE-04.5](./tasks/SPIKE-04.5-rusqlite-safety-verification.md) · ✅ **全 done（PR #29 主体 merged · PR #34 A.3 决策 merged）** · B.1-5 全过 · R27 真 close · A.3 P99=215ms · **Arbiter 选定方案(a) MVP 接受 220ms**（2026-04-19 · 方案(b) 复合索引留 MVP-02 一起加）
6. **W0-D5** · [SPIKE-05 portable-pty 多 Tab 压测](./tasks/SPIKE-05-pty-multi-tab.md) · ✅ **done（PR #30 merged）** · shared-reader **HOL / boundedness pass** · **visible throughput fail**（ADR-003 继续 proposed）
7. **W0-D5.5** · [SPIKE-05.5 PTY visible throughput + per-session fallback 对照](./tasks/SPIKE-05.5-pty-visible-throughput-fallback.md) · ✅ **done（PR #39 merged）** · 结论：shared-reader 不是瓶颈 · per-session UI drain 反而略低（4 Tab 12.86 vs 14.58 MB/s）· 瓶颈在 invoke RTT 22ms / JS / xterm · ADR-003 accepted · CLAUDE.md #15 B → A
8. **W0-D6** · [SPIKE-06 Claude/Codex CLI + Apple Dev Program](./tasks/SPIKE-06-cli-protocol-and-codesign.md) · 🟡 **§A harness done（PR #38 merged · pipeline smoke 通过）** · §A 36 样本待 PR 2（session 11 · `brew install gitleaks asciinema` 前置）· §B Apple Dev Program 用户申请中

### 🧑‍🎨 **MVP-01 Phase A + B 已交付**（session 8-9 · 首个能启动 + 视觉一致骨架）

- **Phase A**（PR #28 merged）· Cargo workspace 2 crate + SolidJS + Tauri 壳 + Codex 2 轮对抗 review + 3 轮 CI 修最终切 corepack pattern
- **Phase B**（PR #33 merged）· Calm Studio design token 落地 · 欢迎页精装 · 真实 icon 替换 · runtime 验证通过
- **Phase C 预期**：基础崩溃恢复 session persistence（**MVP-02 已 done · 解阻塞**）· Ubuntu 24 runtime 验证（阻塞 · 无环境）

### 🗂️ **MVP-02 workspace 管理已交付**（session 10 · PR #40 merged · OpenCode 主交付 + reviewer fix）

- **Backend**：rusqlite + r2d2 connection pool · schema v1→v2 migration（`PRAGMA user_version`）· `WorkspaceStore` CRUD（create/list/get_by_id/touch/delete/exists_at_path）· git auto-detection 5 parent levels · UUID v4 · canonical path（dunce）
- **IPC layer**：7 commands（greet · workspace_init/list/create/open/delete/exists）· `AppState { pool: Mutex<Option<DbPool>> }` · 每 command 独立 ACL permission identifier
- **Frontend**：sidebar workspace list（按 last_opened DESC）· directory picker（plugin-dialog）· 删除二次确认 modal · git badge · error bar · multi-workspace switcher（部分 · close 推 MVP-04）
- **测试覆盖**：23 unit tests（19 workspace + 4 db migration · 含 UTF-8 / spaces / duplicate / nonexistent / git parent detection / idempotent migration）
- **Reviewer fix**：H1 path traversal（workspace_init 改 backend 自取 `app_local_data_dir()`）+ M3 SVG bug（VibestationMarkSmall xmlns + 内联 gradient）+ spec done 翻转走 (a) 路径
- **Explicit skip 推 MVP-04**：§C `workspace.close` IPC + opened/closed session 状态建模 · §D `app_state` table（"打开列表 + 顺序"持久化）· 与 Tab 管理一起做避免分裂改动
- **Follow-up 收尾**：FU-1 ✅ 关闭（PR #47 · 截图 3 重做 · 用户手动）· FU-2 ✅ 关闭（PR #44 + #45 · ADR-011 accepted + 6 步实施）· FU-3 ✅ 关闭（PR #42 · dispatch prompt §2.8 升级）· FU-4 ✅ 关闭（PR #43 · SPIKE-01/02 归档）

### 🎛️ **MVP-03 Tool Windows 已交付**（session 11 开场 · PR #61 merged · OpenCode 主交付）

- **布局**：5-zone grid（Activity Strip + Primary Sidebar + Main + Secondary Sidebar + Bottom Panel）· 严格对齐原型 `design/directions/1-calm-studio.html` DEFAULT_STATE
- **交互**：toggle（Primary / Secondary / Bottom 独立开关）· resize（拖拽分隔条 · min/max 范围约束）· theme（light / dark · `prefers-color-scheme` 自适应）
- **持久化**：布局状态入 rusqlite（列宽 / 折叠态 / 主题）· 跨 session 恢复
- **测试**：29 unit tests（+13 新增 · layout.rs + persistence）· 7/7 CI target 全绿
- **Runtime 证据**：5 张截图（`docs/runtime-evidence/mvp-03/` · dark × 4 + light × 1 · 60-100 KB · 符合 ADR-011 R4）
- **验收**：20 项清单全过 · 8 条硬约束（dispatch prompt v2）全过

### 🧪 **SPIKE-08 E2E + IPC contract harness 已交付**（session 11 开场 · PR #60 merged · Codex 主交付）

- **§A Contract layer**：**ts-rs 选用**（v0.1 GA 前强制覆盖所有新增 IPC contract · Rust type → TS type codegen · `build.rs` trigger · `beforeDev/BuildCommand` 保证 bindings fresh）
- **§A 对比**：`ts-rs 12.0.1`（stars 1765 · 依赖 656 行）vs `tauri-specta 2.0.0-rc.24`（仍 RC · 依赖 675 行 · builder-based 集成成本高）· 选 ts-rs
- **§B Runtime layer**：`Playwright + Vite` 作为 v0.1 自动化 runtime 补层（非 required）· 真实 Tauri IPC E2E（B.1/B.3 Linux tauri-driver）本轮未收敛 · 不作为 v0.1 GA required gate
- **§C CI**：contract + browser smoke 全平台 required · native runtime 继续保留 manual runtime evidence · Linux `tauri-driver` workflow 留 informational follow-up
- **H2 回归验证**：临时把 `WorkspaceRecord.id` 改为 `workspace_id` · `pnpm typecheck` **必然 FAIL**（符合预期）· 证明 contract layer 能把 H2 类 drift 前移到 compile-time
- **下一步（session 11 候选 1）**：ts-rs 推广到 MVP-02 现有 5 个 IPC contract struct · 闭合 H2 根因制度化

**并行化节奏说明**：SPIKE-03/04 是纯 CLI bench · 不依赖 Tauri UI · 用户决策放宽 depends_on（SPIKE-02 → SPIKE-01）· 由 opencode agent 并行完成。这是 session 6 协作规则"给原话 prompt 让用户转发给其他 agent"的首次大规模落地。

### 📦 Spike W0 通过后 · MVP 实施（目标 v0.1 GA · 12-14 周）

- MVP-01..10 按依赖顺序实施（MVP-01 → ... → MVP-10）
- MVP-11..20 留 v0.2 / v0.3 / v1.0 kickoff 详化

## ⚠️ 当前卡点 / 注意事项

- **MVP-03 ✅ done · Tool Windows 布局已交付**（PR #61 merged · session 11 开场 · OpenCode 主交付 · 5-zone + toggle + resize + theme · 29/29 Rust 测试 + 5 张 runtime 截图 · ADR-011 R4 符合）
- **MVP-04 🟡 终端主链只剩 Phase D**（PR #72/#82/#91/#95/#99 已合入 · Phase A/B/C/E/F 全部落地 · 当前只剩默认 shell / Claude CLI / Codex CLI 实机兼容验证 · 低优先）
- **MVP-08 🟡 Phase A/B/C 已完成**（PR #100/#101/#105 已合入 · 后端 diff/status contract + Bottom Panel Git Status 面板 + Diff 视图前端 + Git Status/Git Log → Diff 接通已落地 · 当前主线 = Phase D fs watch（`notify` 6.x 三平台 · 替换当前 polling）+ Phase E 证据量化（5 截图 + A.2/A.6/F 性能门槛实测））
- **PR 级 GitHub Actions 自动运行已关闭**（PR #102 · 只保留 `push main` + `workflow_dispatch` · 新 PR 不会自动跑 CI，后续 agent 必须本地先跑 gate，并在 merge 后核对 `main` 的 check runs）
- **SPIKE-08 ✅ done · E2E + IPC contract harness 选型**（PR #60 merged · session 11 开场 · Codex 主交付 · §A ts-rs PASS · §B Playwright runtime FAIL · §C hybrid gate · 下一步 ts-rs 推广 MVP-02 现有 IPC contract · 闭合 H2 根因制度化）
- **ADR-006 accepted + CLAUDE.md v2-D**（PR #50 merged · session 10 末 · "self-review + Arbiter approval" 单人项目术语澄清 · 未来升级 v2-strict 触发条件显式化）
- **Vite 8 + TS 6 major bump 评估**（PR #59 merged · docs/upgrade-notes/ · 推荐 v0.1 GA 后再升 · 不碰生产代码）
- **docs rusqlite 字样对齐**（PR #58 merged · implementation-plan 8 处 stale 清理 · 对齐 ADR-005）
- **SPIKE-04.5 ✅ 全 done** · R27 数据安全 close · A.3 Arbiter 选定方案(a) MVP 接受 220ms（PR #34 merged · 不改代码 · 方案(b) 复合索引留 MVP-02 一起加）
- **SPIKE-05.5 ✅ done** · ADR-003 accepted · CLAUDE.md #15 B → A 锁 shared-reader（PR #39 merged · session 10）· 后续 invoke / JS / xterm 优化转独立 task（visible throughput 优化推到 v0.2 / v0.3）
- **MVP-02 ✅ done · workspace 管理已交付**（PR #40 merged · session 10 · OpenCode 主交付 + 主 agent H1/M3 fix + spec done 翻转）· §C close + §D opened 列表 explicit skip 推 MVP-04
- **FU-1 ✅ 关闭**（PR #47 · session 10 终极末 · 用户手动重截 modal · 同时是 H2 fix 后的 runtime 证据 · 44.7 KB · 远低于 ADR-011 R4 推荐）
- **FU-2 ✅ 关闭**（PR #44 + #45 · session 10 真末 · Arbiter 选项 A 选定 · ADR-011 accepted · runtime 证据路径锁 `docs/runtime-evidence/<task-id>/` · 进 git · CLAUDE.md 决策表 #18 新 row · 新项目规则 `.claude/rules/runtime-evidence-location.md` R1-R5 硬规则落地）
- **FU-3 ✅ 关闭**（PR #42 · session 10 真末 · dispatch prompt §2.8 子进程清理硬约束 · 默认硬约束 7→8 · trap/pkill 两种做法）
- **FU-4 ✅ 关闭**（PR #43 · session 10 真末 · rule 13 历史欠账修复 · SPIKE-01/02 源码归档进 `docs/spikes/code/SPIKE-0[12]/` · 释放 2 GB 冷备）
- **H2 IPC camelCase mismatch ✅ 修复**（PR #47 · session 10 终极末 · MVP-02 runtime bug · CI 全绿但点 Delete 报 missing key id · 根因：Rust `#[serde(rename_all = "camelCase")]` 输出 `workspaceId` · 但 TS interface 误声明 `workspace_id` · 全 5 字段 16 处替换为 camelCase · runtime 用户验证 Delete + Git badge + dark mode 全过 · **rule 15 "CI 绿 ≠ runtime 过" 活教材** · 暴露 E2E 测试缺口 · session 11 候选 spike）
- **多 agent 共享 working tree 风险已规避**：Codex + OpenCode 已各自建 `git worktree` 独立工作（session 9 Phase B 开工时发现 shared-tree 冲突苗头后立即修正）· 未来 dispatch prompt 必须明确要求 worktree / /tmp 隔离
- **OpenCode Track 3 程序瑕疵事后补档**：PR #34 未按 dispatch spec 跑 benchmark · 直接自己标 "Arbiter 选定方案(a)"· Arbiter 事后 comment 确认方案(a) 判断合理 · 决策成立 · 下次 dispatch prompt 加 "外部 agent 不得自行 accept decision-grade 结论" + benchmark 强制要求
- **MVP spec 中 `redb` 字样历史**（MVP-01/02/03/05/06/10/19 · 共 7 个）：暂不改 spec 正文（YAGNI）· 实施时以 ADR-005（rusqlite）为准 · 届时 PR 触发 API-level 改动
- **Ubuntu 24 环境缺失**（SPIKE-01/02 Phase B 前置）· 阻塞 SPIKE-01/02 full done · ADR-006 桌面框架 · SPIKE-06 cross-platform · MVP-01 Phase C Ubuntu runtime 验证
- **分支保护已显式暂缓**（用户表态 · 单人 + Codex review 模式下不致命 · 升级触发条件见上方 §🔐 用户手动步骤）
- **R1 Claude/Codex CLI 协议**：SPIKE-06 样本录制 · R1 降级须经 SPIKE-07 parser 验证 + ADR-011
- **R12 CRITICAL Tauri Wayland**：macOS Phase A 强信号 · Wayland 风险仍在（SPIKE-01/02 Phase B 兜底）
- **Apple Developer Program 审核**：SPIKE-06 立刻提交 · 最长 2 周影响 v0.1 发布（W12）· 同时 SPIKE-02 updater plugin 也依赖
- **域名未定**（W10 决定）· **Logo 未最终选定**（v0.1 前定）

## 🔀 阶段切换信号

| 信号                        | 触发                                                                                                                                                                                                                                              |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ✅ Phase 1-4 Pre-code 完备  | **已达成**（2026-04-18 session 5 · 4 PR 全 merge）                                                                                                                                                                                                |
| ✅ Spike W0 启动            | **已达成**（session 7 · 首行 Rust 代码 · SPIKE-01 Phase A PASS）                                                                                                                                                                                  |
| ✅ Spike W0 macOS 全通过    | **已达成**（session 11 开场 · SPIKE-01/02 Phase A · SPIKE-03/04/04.5/05/05.5/08 全 done · SPIKE-06 §A harness done · 36 样本 + §B Apple Dev 阻塞外部资源）                                                                                        |
| 🟡 Spike W0 全平台通过      | SPIKE-01/02 Phase B Ubuntu（阻塞环境）+ SPIKE-06 36 样本 + Apple 申请                                                                                                                                                                             |
| 🔴 Spike 任一 CRITICAL Fail | 触发 fallback + ADR supersede                                                                                                                                                                                                                     |
| ✅ MVP 实施启动             | **已达成**（session 8 · MVP-01 Phase A · ADR-003/005/006/007 全 accepted）                                                                                                                                                                        |
| 🟡 MVP v0.1 进度            | **3/10 done + 7/10 ready**（MVP-02/03/07 done · MVP-01/04/05/06/08/09/10 ready）· 其中 MVP-04 已完成 Phase A/B/C/E/F 仅剩 D，MVP-08 已完成 Phase A/B/C 主线进入 Phase D/E · 主线已收敛到 fs watch + Git 写路径 + 打包发布 |
| 🎯 v0.1 GA                  | MVP-01..10 全过 §10.1 + §10.6 终端正确性矩阵 + §10.3 跨平台                                                                                                                                                                                       |
| 🔴 连续 2 周 < 5h 投入      | 触发 hibernation（`implementation-plan.md §10.5`）                                                                                                                                                                                                |

## 📦 近期关键交付物索引

| 产出                                                                           | 路径                               |
| ------------------------------------------------------------------------------ | ---------------------------------- |
| v2 实施计划（14 章 + 附录）                                                    | `docs/implementation-plan.md`      |
| 8 个 Spike spec（W0 + v1.0-pre + H2 制度化 · 含 SPIKE-04.5 / SPIKE-05.5 补测） | `docs/tasks/SPIKE-*.md`            |
| 20 个 MVP spec（v0.1 详细 + v0.2+ 占位）                                       | `docs/tasks/MVP-[01-20]-*.md`      |
| 11 个 ADR（11 accepted · 0 proposed · session 10 末 ADR-006 升级后全收敛）     | `docs/adr/ADR-0[01-11]-*.md`       |
| 9 个 Spike report（含 SPIKE-08 harness 选型）                                  | `docs/spikes/SPIKE-*-report.md`    |
| MVP-02 / MVP-03 runtime 证据                                                   | `docs/runtime-evidence/mvp-0[23]/` |
| Agent 入口 · 决策表 · 自审四问 · 翻转 gate                                     | `CLAUDE.md`                        |
| 人类启动手册                                                                   | `docs/SESSION-STARTUP.md`          |
| 贡献指南 · 含用户拍板 gate                                                     | `CONTRIBUTING.md`                  |
| 分支保护 admin checklist                                                       | `docs/BRANCH-PROTECTION.md`        |
| Frontmatter validator + self-test                                              | `scripts/validate-task-spec.mjs`   |

---

## Session 日志（近 5 次）

### Session 11（2026-04-20 · 状态同步 + MVP-03/SPIKE-08 落地 + ts-rs rollout + MVP-04 spec ready + Kimi 首次成功协作）

**跨越里程碑**：session 10 末 + session 11 全程累计 merge 17 个 PR（#48-#63）+ PR #64 pending merge · **MVP-03 Tool Windows + SPIKE-08 E2E/contract harness 选型 + ts-rs 生产化落地 + MVP-04 spec ready** · Kimi 首次外部 agent 成功协作 · tasks/README 状态表文档负债清零 · 进入 MVP-04 实施前置 + MVP-07 review 阶段。

**14 个 PR 概览**（按 merge 顺序）：

- **session 10 末尾（PR #48-#58 · 11 个）**：
  - #48 / #49 / #51 · codex round 2 review 文档同步（Issue 1 + H-1/M-1/M-2/L-1 + Issue 2 · 1 HIGH + 2 MEDIUM + 1 LOW）
  - #50 · **ADR-006 accepted + CLAUDE.md v2-D**（单人项目修订 · "self-review + Arbiter approval" 术语澄清 · 未来升级 v2-strict 触发条件）
  - #52 / #53 · dependabot（actions/checkout v4→v6 · actions/upload-artifact v4→v7）
  - #56 · SPIKE-08 spec 新建（E2E + IPC contract 双层防御 harness 选型 POC · H2 根因制度化前置 · 2d）
  - #57 · MVP-03 spec draft → ready 翻转（reviewer-led gate (a) 路径）
  - #58 · docs rusqlite 字样对齐（implementation-plan 8 处 stale 清理 · 对齐 ADR-005）

- **session 11 开场（PR #59-#61 · 3 个）**：
  - #59 · **Vite 8 + TS 6 major bump 评估**（docs/upgrade-notes/ · 推荐 v0.1 GA 后再升 · 不碰生产代码）
  - #60 · **SPIKE-08 done**（Codex 主交付 · §A ts-rs 选用 · §B Playwright 作 v0.1 补层 · §C hybrid gate · H2 compile-time 回归验证 FAIL 符合预期）
  - #61 · **MVP-03 done**（OpenCode 主交付 · 5-zone + toggle + resize + theme · 29/29 Rust 测试 + 20 验收清单全过 + 5 张 runtime 截图）

**Session 11 本 PR 动作**（state sync）：

- 更新 `docs/tasks/README.md` 表格：SPIKE-01/02 → in-progress · SPIKE-03/04/05/08 → done · 新增 SPIKE-04.5 / 05.5 行 · SPIKE-06 → ready · MVP-01 → ready · MVP-02/03 → done（对齐 frontmatter · 文档负债清零）
- 更新 `docs/PROGRESS.md`：PR 列表补 #48-#61 · 当前位置 → 进入 MVP-04 · 阶段切换信号表加 "Spike W0 macOS 全通过" + "MVP v0.1 进度 3/10" · 近期关键交付物索引同步（SPIKE 报告 9 个 · runtime evidence 2 个目录）· 本 session 日志段

**Session 11 后续动作全部完成**（原三选候选全部消化）：

1. ✅ **ts-rs 推广 MVP-02 IPC contract** · PR #63 `44e5e95` merged · `WorkspaceMetadata` / `LayoutState` 加 `#[derive(TS)]` · build.rs 生成 bindings · 前端删手写 interface · H2 regression proof 跑通（tsc 抓 10 处 drift）· fix commit `c09e250` .prettierignore 排除 generated
2. ✅ **MVP-04 spec review · draft → ready** · PR #64（pending merge · CI 7/7 全绿）· **Kimi 主交付**（首次 Kimi 外部 agent 成功协作）· §G IPC Contract（ts-rs）章节新增 · `TabState` derive + `#[ts(type = "number")]` · `scroll_back` 建议从 IPC 排除 remark · (a) 翻转路径 2 commit
3. ⏸ **SPIKE-06 PR 2 · 36 样本** · 用户 brew install gitleaks + asciinema 完成 · session 11 期间 OpenCode 不可用暂搁 · session 12 重派 Codex（见 session 12 开场段）

**Session 11 新外部 agent 协作经验**（Kimi 首次成功）：

- Kimi 按 dispatch prompt 11 硬约束全部满足 · 独立 worktree `/private/tmp/mvp-04-review-work` · 翻转 (a) 路径 Kimi 自己 push 2 commit 结构 · Commit trailer `Co-authored-by: Kimi <noreply@moonshot.ai>`
- **1 个瑕疵（非 BLOCK）**：PR body 被 `ps aux` + `pnpm typecheck FAIL` log 污染 · diff 干净 · 未来 Kimi prompt 需明示 PR body 纯 markdown 避免 heredoc shell escape · MVP-07 prompt 已加这条警告

**Session 11 OpenCode 状态变化**：

- 用户声明 "目前 opencode 无法执行任务 · 先不考虑 opencode 了"
- SPIKE-06 PR 2 原下发 OpenCode prompt（`spike-tmp/dispatch/SPIKE-06-pr2-opencode-prompt.md`）搁置
- session 12 尝试再次分派 OpenCode（SPIKE-04.5 方案(b) 小任务）带 fallback 树验证 recovery

---

### Session 12（2026-04-20 · 多 agent 四路并发极致产出 · v0.1 Git 能力闭环 + 终端画面闭环 + SPIKE W0 macOS 完结）

**跨越里程碑**：session 11 PR #64 merge 后进入 session 12 · 全程累计 **11 PR merged**（session 12 新增 · 本 sync PR 除外）· Kimi 5 连发 spec review（MVP-04/05/07/08/09）· OpenCode 2 次成功（SPIKE-04.5 方案 b + MVP-04 storage prep · 从 session 11 不可用 fully recover）· Codex 1 次交付（SPIKE-06 PR 2 · 2 轮修复 · SPIKE W0 macOS 100% 完结）· 多 agent 并发新失败模式捕获 + 2 新全局 rule 制度化。

**Session 12 PR 全景（11 merged · 按时间顺序）**：

| #   | PR  | Agent        | 类型 · 关键影响                                                                                                       |
| --- | --- | ------------ | --------------------------------------------------------------------------------------------------------------------- |
| 1   | #64 | Kimi         | MVP-04 spec review（session 11 遗留 · Kimi 首次成功）                                                                 |
| 2   | #65 | Claude Code  | `end-session` skill 入库（session 11 遗留物料）                                                                       |
| 3   | #66 | Kimi         | MVP-07 Git Log spec ready · §H gix 0.70 只读 · SPIKE-03 依据                                                          |
| 4   | #67 | Claude Code  | PROGRESS session 11/12 sync · 踩 commit 落 main 坑（未 push 修复）                                                    |
| 5   | #68 | **OpenCode** | SPIKE-04.5 方案(b) rusqlite 复合索引 · **88.2% P99 改善** · OpenCode recover 验证                                     |
| 6   | #69 | Claude Code  | **多 agent 并发复盘 + 全局 rule 16/17 + 项目 memory 初始化 + dispatch §2.9**                                          |
| 7   | #70 | Kimi         | MVP-08 spec ready · §H 三分工（gix 读 + git2 写 + similar diff）· 14 min 最快                                         |
| 8   | #71 | **Codex**    | SPIKE-06 PR 2 · 36 脱敏样本 + R1 保留 · 2 轮修复（markdown lint → trailing whitespace）· **SPIKE W0 macOS 100% 完结** |
| 9   | #72 | **OpenCode** | **MVP-04 storage 层 done** · migration v5 + TabsDao + 5 IPC + ACL + 5 ts-rs bindings + 36 单元测试                    |
| 10  | #73 | Kimi         | MVP-09 spec ready · §H git2 0.20 纯写路径 · **v0.1 Git 能力闭环达成**                                                 |
| 11  | #74 | Kimi         | MVP-05 Pane 分屏 spec ready · §H **Pane 布局模型**（非 Git · 深度 ≤ 2 矩阵）· **v0.1 终端画面闭环**                   |

**Session 12 Kimi 5 连发统计**：

| PR  | Task   | 耗时        | 硬约束   | 特点                                                                       |
| --- | ------ | ----------- | -------- | -------------------------------------------------------------------------- |
| #64 | MVP-04 | ~30 min     | 13/13 ✅ | 首次 · §G IPC Contract 模板确立                                            |
| #66 | MVP-07 | ~20 min     | 13/13 ✅ | §H 模式开启（gix 只读）                                                    |
| #70 | MVP-08 | **~14 min** | 14/14 ✅ | 最快 · §H 三分工                                                           |
| #73 | MVP-09 | ~30 min     | 14/14 ✅ | §H git2 纯写 · Git 闭环                                                    |
| #74 | MVP-05 | ~30 min     | 14/14 ✅ | §H 非 Git（Pane 布局）· 瑕疵 1：PR body "Arbiter dialogue" 段模拟 approved |

**累计**：5 PR · +782/-124 docs 行 · 平均 23 min · 硬约束 100% 通过 · 1 瑕疵（#74 PR body · session 13 补硬约束 2.11）。

**Session 12 OpenCode 协作（2 次 · 完全 recover）**：

- PR #68 · SPIKE-04.5 §A.3 方案(b) · 独立小代码任务（fallback 树备）· 88.2% P99 改善 · raw benchmark 3 份
- PR #72 · MVP-04 storage 层 · migration v5 + 36 单元测试 + Runtime 证据 3 份 log · **关键硬约束 TabState 不含 scroll_back · 独立 `tab_scrollback_fetch` 命令按需拉**（spec §G remark 落地）

**Session 12 Codex 协作（SPIKE-06 PR 2 · 2 轮修复）**：

- PR #71 draft 交付：36 样本 + R1 保留 · 但 2 轮修复：
  - 轮 1：CI markdown lint FAILURE（raw/README.md 表格格式）+ 冷备误放 /private/tmp/ worktree（ephemeral）· 已修
  - 轮 2：CI trailing whitespace FAILURE（report.md blockquote "两空格换行" 语法）· markdownlint-cli 本地不抓 MD009 → **暴露 CI lint vs markdownlint-cli 不一致** · session 13 硬约束补强
- R1 保留极度严谨（独立 `## R1 风险陈述（硬边界）` section · 3 处 "R1 保留" 明示）

**Session 12 新失败模式 + 规则制度化（PR #69）**：

- 事故 1：主 agent dirty working tree 时 · 外部 agent Kimi 并发 `git worktree add` 触发主 repo HEAD 漂移（progress-sync → mvp-07-ready → main · 非主动） · 主 agent 忽略 `git commit [main 30dce71]` 信号 · commit 落 main 本地（未 push · `git merge --ff-only` + `git branch -f main origin/main` 安全修复）
- 事故 2：MVP-07 Kimi prompt 没附 spec 原文（复用本地 agent 模板给远程 API）· 用户指出 · 扩 335 行修复
- 新全局 rule：`~/.claude/rules/16-multi-agent-worktree-sync.md`（R1 dirty tree 禁派 + R2 派后只读 + R3 commit 输出铁律 + R4 安全修复）· `~/.claude/rules/17-dispatch-agent-capability-matrix.md`（本地 CLI / 远程 API / IDE 插件三类适配）· `~/.claude/rules/common/git-workflow.md` 扩充 commit 输出验证铁律
- 项目 memory 初始化：`~/.claude/projects/.../memory/MEMORY.md` + 3 feedback（multi-agent-worktree-race / kimi-remote-api-needs-spec-inline / kimi-speed-benchmark）
- 项目 `.claude/rules/dispatch-prompt-template.md` §2.9 新增 Agent 能力矩阵（4 类 agent 对照 + prompt 策略 + 反模式 + 事件记录）
- Obsidian 归档：`~/CodeWorkSpace/PersonalWorkspace/knowledge-llm-wiki/raw/articles/vibestation-session-12-多agent并发踩坑复盘.md`

**Session 12 文件域隔离**（4 路并发 · 零冲突验证）：

| Agent    | 文件域                                                                                                                    | 冲突 |
| -------- | ------------------------------------------------------------------------------------------------------------------------- | ---- |
| Kimi     | `docs/tasks/MVP-XX-*.md` 单文件（MVP-04/07/08/09/05 每次一个）                                                            | 无   |
| Codex    | `docs/spikes/SPIKE-06-report.md` + `docs/spikes/raw/SPIKE-06/*`                                                           | 无   |
| OpenCode | `crates/core/src/*` + `docs/runtime-evidence/*`                                                                           | 无   |
| 主 agent | `docs/PROGRESS.md` + `docs/session-12-post-mortem*.md` + `.claude/skills/*` + `.claude/rules/dispatch-prompt-template.md` | 无   |

**Session 12 本 batch sync PR 动作**（state drift 一次清）：

- `docs/tasks/README.md` · 5 处 state drift（MVP-04/05/07/08/09 `draft` → `ready` · 各附 PR 号）· SPIKE-06 状态补 "§A 36 样本 PR #71 completed"
- `docs/PROGRESS.md` · 补 11 个 merged PR · 状态面板 4 字段刷新 · MVP v0.1 进度 "2/10 done + 6/10 ready" · 阶段切换信号补 "v0.1 Git 能力闭环 + 终端画面闭环 + SPIKE W0 macOS 完结"
- 本 session 日志段（本段）收尾

**Session 12 v0.1 主线状态**（批 sync 后）：

- **2/10 done**（MVP-02/03）
- **6/10 ready**（MVP-01 Phase A/B · MVP-04 spec + storage · MVP-05 · MVP-07/08/09）· 其中 MVP-04 storage 层（PR #72）为 v0.1 GA 实施提供前置
- **2/10 draft**（MVP-06 配置导入 · MVP-10 v0.1 GA 发布 · 后者依赖 MVP-06 ready）
- **解阻路径**：MVP-06 spec review（Kimi 可做 · 最后 1 个 spec）→ MVP-10 v0.1 GA spec review → 全链实施

**Session 13 开场三选**（见状态面板 Next concrete action）：

1. MVP-04 PTY + xterm 实施（storage 层 prep 完成 · 主线 · 6.5-10d · 可拆 sub-task 派外部 agent）
2. MVP-06 配置导入 spec review（Kimi 可做 · 最后 1 个 v0.1 draft · 解阻 MVP-10）
3. MVP-07 Git Log 实施（spec ready · 依赖全 done · 5d · v0.1 Git 能力入口）

---

### Session 10（2026-04-19 晚 · 三路并行收敛 · 4 PR 全 merged · MVP-02 落地）

**跨越里程碑**：session 9 三路并行（主 agent / Codex / OpenCode）全部完成交付 · 4 PR 一波 merge · MVP-02 workspace 管理 done · MVP-03 解阻塞 · 进入 Tool Windows 阶段。

**4 PR session 10 完成（按 merge 顺序）**：

1. **PR #37 dispatch-rules 沉淀** · `.claude/rules/dispatch-prompt-template.md` 273 行 · 7 条默认硬约束（禁止自行 accept / Acceptance 全覆盖 / runtime 证据必交 / 分支 / worktree / trailer / 不碰禁区）+ 标准模板 + 升级路径 · 未来所有 dispatch 复用
2. **PR #38 SPIKE-06 §A harness** · CLI record + redact + gitleaks pipeline · 2 zero-secret smoke（`claude --version` / `codex --version` × 3）· 36 样本留 PR 2（session 11 · `brew install gitleaks asciinema`）
3. **PR #39 SPIKE-05.5 (Codex)** · 200 files +32689/-22 · shared-reader vs per-session 对照 · per-session UI drain 反而略低（4 Tab 12.86 vs 14.58 MB/s）· 瓶颈在 invoke RTT 22ms / JS / xterm · ADR-003 proposed → accepted · CLAUDE.md #15 B → A · **reviewer-led rebase 走 (a) 翻转 gate**（main 推了 #34/#35/#36 后 conflict · 主 agent cherry-pick + push -f 解决 · 不打扰 Codex worktree）
4. **PR #40 MVP-02 (OpenCode + reviewer fix)** · workspace CRUD + git auto-detect + 多 workspace UI · 23 unit tests · clean architecture · per-command ACL permission · OpenCode 主交付 + **主 agent push H1（path traversal）+ M3（SVG bug）+ prettier auto-fix** · spec done 翻转走 (a) 路径 · §C close + §D app_state 推 MVP-04（explicit skip + reason）

**reviewer-led 协作模式建立**（session 10 关键演进）：

- 之前模式：reviewer 写 review comment · 等 author 修 + push
- session 10 新模式：reviewer 主动 push fix commit 到 PR 分支 · 走 (a) 翻转 gate · 节省 round-trip 延迟
- 应用：PR #39 rebase（Codex 不在 active session · reviewer 代解 conflict）· PR #40 H1+M3 fix（OpenCode 不知道 finding · reviewer 直接修） + spec done 翻转
- 边界：reviewer 不修需决策的内容（M1/M2 close/app_state · 是产品 scope decision · 留 explicit skip + Arbiter 拍板）

**Edit tool bug 揭示 + workaround**（session 10 技术教训）：

- 现象：Edit tool 报 `success` 但实际未改文件 · Read tool 后续显示 phantom changes（基于 Edit ops · 不是真实 file content）· 仅在某些条件下复现（cherry-pick 中间状态 / detached HEAD）
- Workaround：用 Python 直接 file IO + grep 验证（grep 显示真实 disk content · Read 显示 cached）
- 应用：PR #39 PROGRESS.md 5 处 conflict 解决 · session 10 PROGRESS update（本 PR）

**cwd stuck 教训**（session 10 操作纪律）：

- 现象：Bash `cd /private/tmp/mvp-02-work` 后 cwd 持续 stuck · 后续命令在该 worktree 执行（即使没显式 cd）· 导致 cherry-pick / push 在错误 worktree 操作（PR #39 rebase 全部发生在 Codex worktree · 不是主目录）
- 影响：PR #39 操作正确（结果对）· 但状态混乱（Codex worktree 变 detached HEAD）· 事后 reset --hard origin 同步
- 教训：跨 worktree 操作前 `pwd` 验证 · 用 absolute path / `git -C <path>` 替代 cd

**OpenCode 协作纪律演进**：

- session 9 末 OpenCode SPIKE-04.5 §A.3 程序违规（虚构 Arbiter 决策）→ session 10 dispatch prompt 加 7 条硬约束 → session 10 OpenCode 大部分遵守
- OpenCode session 10 行为：(1) 主交付 MVP-02 代码质量很高（23 unit tests / clean）（2) 第二次 dispatch 加 trailer + runtime 证据 · 但截图 3 自动化失败 · `docs/runtime-evidence/` 自选路径
- Vite/pnpm 残留进程占 port 1420 4 小时 · 是 OpenCode 自动化截图后没 cleanup（FU-3）

**Arbiter Option C 决议**（session 10 末）：

- PR #40 截图 3 失败 + `docs/runtime-evidence/` 自选路径 · 但代码 ready · CI 7/7 绿
- Option A 严格（重做） vs B 务实（接受） vs **C 混合**（merge + follow-up）
- Arbiter 选 C：approve + merge · FU-1/FU-2/FU-3 作独立 task 跟进 · 不阻塞 MVP-02 done

**8 commit 主 agent 推到 PR #40**（session 10 reviewer-led 工作量）：

- `68269e5` H1 + M3 fix
- `a11bc66` prettier auto-fix
- `058b2bb` spec done 翻转

**Session 10 真收尾 · FU 系列关闭 3/4**（2026-04-19 晚 · 连续 5 PR merge）：

- **PR #41** (b76f647) · PROGRESS sync 1（4 PR merged 初版记录 + FU-1/2/3 发掘）
- **PR #42** (d329b4a) · **FU-3 关闭** · dispatch prompt §2.8 子进程清理硬约束 · 7 → 8 条默认硬约束 · trap/pkill 推荐做法 · 事件记录 OpenCode Vite/pnpm orphan 4 小时
- **PR #43** (307f075) · **FU-4 关闭** · SPIKE-01/02 源码归档进 `docs/spikes/code/SPIKE-0[12]/`（80 文件 / ~1 MB 纯源码）· 修复 rule 13 session 7 历史欠账 · raw/ README 标注 "嵌入式 raw 不伪造重跑" · 释放 2 GB `spike-tmp/spike-0[12]-tauri/` 冷备
- **PR #44** (025371d) · **FU-2 draft · ADR-011 proposed** · 3 选项对比 (A: `docs/runtime-evidence/` / B: `spike-tmp/img/` / C: PR comment) · 推荐 A · Arbiter dialogue 拍板 "按你的推荐来"
- **PR #45** (67d4373) · **FU-2 翻转 · ADR-011 accepted + 6 步实施** · dispatch prompt §2.3 路径改 · 新建 `.claude/rules/runtime-evidence-location.md`（161 行 · R1-R5 硬规则）· CLAUDE.md 决策表 A 栏 #18 新 row · 清 `spike-tmp/img/` 52 KB 残留 · 未改全局 rule 15（跨项目 · 项目级规则引用之）

**FU 系列终局**：

- FU-1 ⏸ 留 session 11（唯一剩余 · 需用户手动跑 `pnpm tauri:dev` + 截 MVP-02 delete modal）
- FU-2/3/4 ✅ 全关闭
- ADR 编号 001-011 · 决策表 A 栏 15+1=16 条永久锁定（#1-15 + #18）
- rule 13 session 7 历史欠账彻底清零 · 所有 Spike 归档都在 git

**Session 10 产出量**：9 个 PR merged（#37-#45）· 3000+ 行代码 / 文档变动 · 三路并行协作 + reviewer-led fix + FU 系列收尾全部完成 · repo 体积净减（冷备释放 2 GB · 归档进 git 仅 +1 MB）

**Session 10 终极末 · H2 暴露 + FU-1 闭环**（2026-04-19 晚 · PR #46 + #47 · 真零 backlog）：

- **PR #46** (fb503ef) · PROGRESS sync 2 · 反映 FU-2/3/4 关闭
- 用户启动 `pnpm tauri:dev` 测 Delete · 报 "missing required key id" · **暴露 H2 bug**
- 根因定位：`crates/core/src/workspace.rs` Rust `#[serde(rename_all = "camelCase")]` 输出 camelCase JSON · 但 `web/src/App.tsx` interface 误声明 snake_case · runtime 字段访问全 undefined · Delete / Git badge / open / 高亮全 broken
- CI 没 catch 原因：23 个 cargo 单测只覆盖 Rust 端 · `tsc --noEmit` 编译过但 TS 不 runtime check JSON · `pnpm build` Vite 静态 bundle 不触发 IPC · **缺 E2E 测试**
- **PR #47** (4f14c8f) · H2 fix（5 字段 16 处 snake_case → camelCase · prettier auto-fix）+ FU-1 截图重做（用户手动 · 同时是 H2 fix 后的 runtime 证据 · dark mode + 2 处 Git badge + Delete 真 work · 44.7 KB）
- **rule 15 "CI 绿 ≠ runtime 过" 活教材**：H2 bug 完整暴露 CI 盲区 · 强化 session 11 投 E2E spike 的决策依据

**Session 10 总产出**：**11 个 PR merged**（#37-#47）· FU 系列 4/4 全关闭 · 真零 backlog · 真零文档 lag · session 11 起手最佳状态

---

### Session 9（2026-04-19 下午 - 晚）· 三路并行分配 · MVP-01 Phase B 视觉骨架 + md 盘点

**跨越里程碑**：Phase A "能启动骨架" → Phase B "视觉一致骨架" · 三路并行协作模式稳定落地（独立 worktree 隔离）。

**Track 1 · MVP-01 Phase B**（主 agent · PR #33 merged）

- Calm Studio design token 从原型 HTML 抽到 `web/src/styles.css`（oklch 色板 + radii + spacing + motion curves 严格同值）
- 欢迎页精装：内联 SVG Logo（蓝紫渐变 mark）+ H1 + tagline + 版本胶囊（runtime `getVersion()` · 绿点状态）+ designed CTA + IPC 诊断行
- a11y：aria 齐全 · 键盘可达 · `prefers-reduced-motion` / `prefers-color-scheme` 自适应
- 真实 icon：`tauri icon` 从 `design/logos/mark.svg` 派生 macOS icns / Windows ico / PNG 全套 · 剥 iOS/Android（MVP 不在范围）
- Runtime 验证 5/5 通过（用户截图 @ `spike-tmp/img/`）· light mode editorial 风格干净 · 无 template look
- 19 files · +324 / -63

**md 盘点清理**（主 agent · PR #32 merged）

- 101 md 盘点后删 2 个过时文档 + 归档 1 复盘到 session-history（首次有实质内容）
- `docs/agent-onboarding-readiness-assessment.md`（自标 HISTORICAL · pre-PR-17）
- `docs/project-status-overview-2026-04-18.md`（日期快照 · 滚动状态由本文件承接）
- `retrospective-spike-交付代码丢失风险复盘.md` → `session-history/2026-04-19-spike-code-loss-retrospective.md`（中文 → kebab-case）
- PROGRESS + SESSION-STARTUP 入站引用同步清理 · grep 确认 0 dangling

**Track 2/3 dispatch**（Codex + OpenCode · 均已启动）

- Track 2 Codex SPIKE-05.5 · 在 `/private/tmp/vibestation-spike-05.5` worktree · 本地已有 commit `7a5b582`
- Track 3 OpenCode SPIKE-04.5 §A.3 · 在 `/private/tmp/spike-04.5-a3-work` worktree · 待 PR
- 设计亮点：**Track 3 用 3 方案对照 benchmark 替代 Arbiter 盲拍板**（数据驱动决策）

**事故与修复 · shared working tree 冲突苗头**

- Phase B 开工时发现 HEAD 被从 feat/mvp-01-phase-b-ui-polish 切到 OpenCode 新建的 spike/SPIKE-04.5-a3
- 原因：OpenCode 和主 agent 共享同一 working tree · 它 `git checkout -b` 影响了主 agent 的分支上下文
- 处理：按 rule 13 铁律 · `git stash -u` 保全 · 切回 feat 分支 · `git stash pop` 恢复（不用 `git checkout --`）
- 纠正：用户通知 OpenCode / Codex 切换到独立 worktree（`/private/tmp/...`）· 冲突自动化解
- 沉淀：下次 dispatch prompt 应显式要求"独立 worktree 或 /tmp 隔离"

### Session 8（2026-04-19 上午 - 下午）· SPIKE 全收口 + MVP-01 Phase A 首行生产代码

**跨越里程碑**：Pre-code 真正结束 · 首行生产代码入盒（Cargo workspace + Tauri + SolidJS）· SPIKE-04/05 全部 done 归档。

**SPIKE-03/04 代码抢救归档**（主 agent · PR #26/#27 merged）

- 事故：发现 SPIKE-03/04 实测代码**仅存在于 /tmp** · macOS 默认 3 天清理 · 决策依据险些永久丢失
- 抢救：从 /tmp 副本归档到 `docs/spikes/code/SPIKE-0{3,4}/` + `docs/spikes/raw/SPIKE-0{3,4}/` · 含 Cargo.lock（白名单 gitignore）
- 根因：Phase 3 归档规则只约束 report · 未强制源码持久化
- 沉淀到全局规则 `~/.claude/rules/13-cross-agent-delivery.md` + 项目规则 `.claude/rules/spike-delivery-checklist.md`

**SPIKE-04.5 v1 BLOCK + v2 accept**（OpenCode + 主 agent review · PR #29 merged）

- v1 被 review 挂出 4 CRITICAL：A.2/A.3 单位错（< 50s 而非 50ms · SUMMARY 洗白）· manifest 非原子（缺 per_table + tx_id）· 1054 行 main.rs 违反 < 300 行
- v2 补做：阈值 / 单位修正 · manifest 加 per_table + last_committed_tx_id + `.tmp+rename` 原子写 · 5 独立业务模块（main.rs 927 行 orchestration · 独立模块各 28-92 行）
- 结论 B.1-5 全过 · R27 真 close · A.3 P99=215ms FAIL · ADR-005 revision "A.3 pending Arbiter"

**SPIKE-05 Codex 一发入魂**（Codex + 主 agent review · PR #30 merged）

- shared-reader + bounded queue + drop-oldest · HOL / boundedness PASS
- Visible throughput FAIL：单 Tab 8.34 MB/s < 20 · 4 Tab 16.38 MB/s < 40
- 结论：**不要** CLAUDE.md #15 从 B 翻 A · 瓶颈在 IPC / xterm drain · 建议 SPIKE-05.5 follow-up

**MVP-01 Phase A · 首行生产代码**（主 agent + Codex 2 轮 adversarial review · PR #28 merged）

- Cargo workspace 2 crate（app + core）· SolidJS + Vite + TypeScript strict · Tauri 2 壳
- Codex round-1 发现：CSP null 不安全 · 缺 CI build smoke · 多余 opener permission · Cargo.lock 未进 git
- Codex round-2 发现：ACL 未定义 `allow-greet` permission（runtime deny `invoke("greet")`）· core:default 覆盖过大
- 3 轮 CI 修最终切 corepack pattern（pnpm/action-setup@v6 在 Ubuntu runner 有 fallback bug）
- Runtime 验证：用户本地 `pnpm tauri:dev` 确认 "Vibestation core online · v0.1.0" 显示正常

**4 条教训沉淀到 rules**

1. 跨 agent 交付代码必须持久化到 repo（全局 `13-cross-agent-delivery.md`）
2. pnpm CI 走 corepack（全局 `14-ci-pnpm-pattern.md`）
3. CI build smoke ≠ runtime smoke（全局 `15-runtime-verification-gate.md`）
4. Tauri v2 ACL + CSP + capability 坑（项目 `tauri-v2-patterns.md`）

### Session 7（2026-04-18 夜 - 2026-04-19）· Spike W0 多 agent 并行 · 首行代码 + 4 Spike + 1 新 Spike

**跨越里程碑**：`pre-code` → **首行 Rust 代码入盒** + **多 agent 并行交付模式**首次大规模落地。

**SPIKE-01 Phase A · macOS 冷启动验证**（Claude Code · PR #20 merged）

- Tauri 2 vanilla-ts 骨架 · 冷启动 10 次 median **202ms**（目标 < 2s · 10× 余量）
- Bundle 8.2MB/.dmg 4MB（7.5× 余量）· 中文 IME + 5/5 肉眼验证
- 1 项降级：日文 IME macOS 未测（IMKit 协议一致性假设）
- 事故 + 修复：录屏误进 commit → amend + .gitignore `spike-artifacts/`

**SPIKE-02 Phase A · Tauri 硬通过矩阵**（Claude Code · PR #22 merged）

- 10x 稳定性 10/10 · median 212ms · Bundle 10MB/.dmg 4MB（7.5× 余量）
- Clipboard plugin smoke（中日英+emoji UTF-8 完整 · 跨 app Cmd+V 验证）
- FS plugin smoke（读写 + terminal cat 验证）
- **2 项降级**：
  - updater plugin 归 SPIKE-06（Apple Dev Program 签名 key 技术依赖）
  - 日文 IME **全平台 skip**（用户 2026-04-19 明确决策 · 本 Spike 不涉及任何日文操作）· 新增 R-SPIKE-02-01 风险 · v0.1 产品定位延后
- 分支事故 + 修复：commit 误进 main → cherry-pick to spike + reset main

**SPIKE-03 · git2 vs gix benchmark**（OpenCode agent + Claude review · PR #23 待 merge）

- linux kernel 1,441,214 commits 实测
- **git2 vs gix 性能差 46-1973×**：
  - log -100 warm P99 · git2 24964ms vs gix **12.65ms**
  - log -1000 · git2 21108ms vs gix 113.84ms
  - log -10000 · git2 33483ms vs gix 733.72ms
- 结论 **(B) 读切 gix · 写保留 git2** · ADR-007 accepted · 决策表 #13 B→A
- Review 疑点：git2 warm > cold 违直觉（HIGH · 不影响结论）

**SPIKE-04 · storage benchmark 两轮交付**（OpenCode agent + Claude review · PR #24 待 merge）

- **v1 被 Claude review BLOCK**：4 CRITICAL（bulk_write 单样本 · range 事后洗白 · sudo purge 未执行 · B.1-5 全未做）
- **v2 补做 accept**：
  - §A 性能：redb 31.94s / rusqlite 9.96s（两者通过）· redb DB 2GB / rusqlite 993MB
  - §B 安全：B.1 PASS / **B.2 FAIL**（redb 2.6.3 silent 读出可能错误数据）/ B.3-5 PASS（POC 级）
- 结论 **(B) 锁 rusqlite**（redb 2.6.3 supersede）· ADR-005 accepted **结论翻转** · 决策表 #14 B→A（rusqlite）
- **关键 caveat**：SPIKE-04 只证明 redb 不行 · rusqlite 应用侧安全**未测** → 需 SPIKE-04.5

**SPIKE-04.5 · 新建 spec**（Claude Code · PR #25 待 merge）

- depends_on SPIKE-04 · blocks MVP-02/06/10/19（所有 rusqlite 持久化 MVP）
- 补 SPIKE-04 瑕疵：B.3 实 assert 旧版读新 DB · B.4 auto-backup · B.5 production op-log + 自动回滚 UI
- A 性能复测澄清范围查询歧义（spec 字面 100 行 vs SPIKE-04 测 1M 行）
- 待下发给 opencode agent（已熟悉 safety.rs · 改 rusqlite API 成本低）

**3 份决策联动变更（PR #23/#24/#25）**

- ADR-007 proposed → accepted（Git 栈混用）
- ADR-005 proposed → accepted（结论翻转到 rusqlite）
- CLAUDE.md 决策表 #13/#14 从 B 列（默认 · spike 后锁）→ A 列（永久锁定）
- 4 个 Spike spec（SPIKE-03/04 draft→done · SPIKE-04.5 新建 · depends_on 语义放宽）

**多 agent 并行模式首次落地**

- Claude Code：SPIKE-01/02 Phase A macOS · SPIKE-04 review · SPIKE-04.5 spec
- OpenCode agent：SPIKE-03 bench · SPIKE-04 v1+v2 bench + safety
- User（Arbiter）：GitHub UI approve · 产品决策（日文 skip · depends_on 放宽）
- 协作规则有效：原话 prompt 可直接转发 · 独立 review 质控到位（v1 BLOCK + v2 accept 说明 review gate 工作）

### Session 6（2026-04-18 晚上-夜）· Codex 三轮评审 + PR #17 缩范围 + PR #18 ready 翻转 + 后续修复

- **Codex round-1 评审**：作为新接手 agent 视角评估 onboarding 就绪度（7/10），命中 5 项问题（入口文档过期 / `§5.4` 断链 / `§512` 笔误 / `ready=0` 流程阻塞 / 分支保护未应用）
- **新增 onboarding 评估文档**：codex 重写 · 7/10 · 已加 historical snapshot banner + 二次复审段落（文档已于 2026-04-19 随 md 盘点清理删除 · 根本原因与改进已被 PR #17/#18 吸收）
- **PR #17 v1（已废弃）→ v2（已 merge `68c0c21`）**：
  - v1 试图一次修全 5 项 + AGENTS 重写 + §5.4 增补 + 翻转 ready
  - Codex round-2 BLOCK：3 CRITICAL（ready 翻转绕过 gate · SESSION-STARTUP 同步未完 · §5.4 虚构内容）+ 3 HIGH + 3 MEDIUM + 2 LOW
  - v2 缩范围方案 A：撤回 §5.4 + ready 翻转 · 修全部 11 项 codex 指控 · 8 commits / 净 +118/-250
  - 拆出去：§5.4 重写 → 后来按 YAGNI 删除（§10.1 workaround 够用，v0.2 kickoff 时再补）
  - 拆出去：ready 翻转 → PR #18
- **PR #18 ready 翻转**（已 merge `5ece9a9`）：
  - SPIKE-01 / SPIKE-02 / MVP-01 翻 status: ready
  - 走 (b) 路径变种（分支保护暂缓 · 靠 reviewer 真实 GitHub approve 替代技术强制）
  - Codex round-3 BLOCK 是流程时序问题（reviews=[] + comments=[]），认可 PR 内容
  - **edge case 备注**：用户实际 squash merge 时直接通过对话完成 review · 没在 GitHub UI 点 Approve · GitHub metadata `reviews: []` · 技术上未达 (b) 变种 "reviews ≠ ∅" 硬要件 · 但 review 实质已通过对话完成 · accepted as session 6 advisory gate edge case · 后续 PR 应改走完整 GitHub UI approve
- **本 session 后续修复 PR**（codex assessment 二次复审发现 3 项漂移 + 2 项已知项）：
  - PROGRESS.md 状态字段反映 PR #18 已 merge + 内部矛盾修复（line 122 阶段切换信号表）
  - SESSION-STARTUP.md 中段同步（SPIKE-01 status / 仓库结构 task spec 数量）
  - docs/tasks/README.md 第 7 步加 "(b) 路径变种" 正式定义（分支保护暂缓阶段合规说明）
  - 2026-04-18 项目梳理快照纳入仓库（已于 2026-04-19 随 md 盘点清理删除 · 滚动状态由本 PROGRESS.md 承接）
- **§5.4 战略章节决定**：按 YAGNI 推迟到 v0.2 kickoff（届时需要数据流 / IPC / 状态机的实施级细节，现在写就是过早优化；当前 4 MVP 用 §10.1 workaround）
- **AGENTS.md 重写**：纠正 codex 自动生成版本的"Claude 名替换错乱 + 阶段过期"双 bug，改为工具无关的极简入口（路由 + 关键约束摘录），权威单文件入口仍指向 CLAUDE.md

### Session 5（2026-04-18 下午-晚上）· Pre-code stage 完备

- **4 个 PR 全 merge 到 main**（Phase 1-4 完整交付）
- **7 轮 Codex 对抗性审查**累计 29 findings 全闭合
- **Phase 2 收尾**：PR #9（tech debt · 4 commits · 12 findings）+ PR #10（MVP-11..20 占位 + SPIKE-07）
- **Phase 3 建立**：PR #12 · ADR × 10 + CONTRIBUTING（含用户拍板 gate）+ CHANGELOG + 3 目录 README
- **Phase 4 落地**：PR #11 · 全套 `.github/` + gitleaks + validator + BRANCH-PROTECTION + self-trigger bug fix
- **Codex 第 11 轮审查质量收敛**：R4 4 HIGH → R5 2 → R6 2 → PR #10/#11/#12 各 3-5 → 新增 findings 逐轮精细化
- **关键技术补齐**：SPIKE-04 op-log 2-phase + reconcile forward（phantom data）· SPIKE-05 B.4 三子场景 HOL · SPIKE-06 gitleaks 双层防护

### Session 4（2026-04-18 上午-下午）· Phase 1 v4 simplified

- Codex 三轮累计 21 个 HIGH
- **承认过度设计 + 违反 YAGNI**：砍多 agent 治理抽象
- v4 simplified：CLAUDE.md 216→135 · SESSION-STARTUP 408→180 · PROGRESS 149→110
- **新增自审四问**：递归完备 / 反向场景 / 边界适用 / YAGNI
- PR #1 合入 main · 首次演示 feature 分支 + PR 流程

### Session 3（2026-04-18 早）· planner v2 + 独立仓库

- Codex 项目级评审 + 4 分歧决策
- planner v2（+474 行 · 30 风险）
- 独立仓库建立 + GitHub push + Phase 1 v1-v3 迭代

### Session 2（2026-04-17 晚）· 视觉原型

- 4 方向原型 · Calm Studio 加 Tool Windows + Pane 分屏

### Session 1（2026-04-17 早）· 立项

- 立项讨论 + 技术调研 + planner v1 + Logo 候选

---

**本文件每次 session end / 阶段切换 / 重大决策变化时手动更新。机械字段 Phase 5 CI 后接 hook 自动刷新。**
