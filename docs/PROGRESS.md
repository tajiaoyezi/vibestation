# 进度快照 · PROGRESS

> **定位**：当前状态面板（agent 和人类都先读本文件获取"我是谁 / 做到哪 / 下一步 / 卡点"）。
> **更新约定**：session end / 阶段切换 / 决策变化时手动更新。不要每个 commit 都更新（噪音大）。
> Session 历史归档到 `docs/session-history/`（Phase 3 已建立）——**不要**归档到 CHANGELOG（CHANGELOG 是 release-please 自动维护的发布日志）。

---

## 📊 固定状态字段

| 字段 | 值 | 更新时机 |
|------|----|---------|
| **Active branch** | `main`（PR #20 merged · SPIKE-01 Phase A macOS PASS · session 7 ongoing）| 分支切换 |
| **Latest commit** | 见 `git log --oneline -1`（不在此处硬编码）| 每次 commit |
| **Worktree status** | 见 `git status`（不在此处硬编码）| 每次 commit |
| **Unpushed branches** | 见 `git branch -vv`（不在此处硬编码）| push 后 |
| **Next concrete action** | **Phase B Ubuntu 等环境就绪**（用户暂无 Ubuntu 24 机器 · Phase B 原话 prompt 已备 · 见 `docs/spikes/SPIKE-01-report.md §5.1`）· 备选：讨论是否并行启动 SPIKE-03/04 benchmark 类（不依赖 Tauri UI · 需要用户确认修改 blocks 语义） | session end |
| **Blocked by** | SPIKE-02..06 仍 blocked 在 SPIKE-01 **full done**（Phase A macOS 过 ≠ 整体过 · 尊重 spec 3 平台规则）| 阻塞变化 |
| **Missing infra** | Ubuntu 24 LTS 环境（物理机 / VM · Phase B 前置）| Phase 完成时 |
| **Required env/accounts** | GitHub CLI ✅ · Rust toolchain (rustc 1.95.0) ✅ · Node 20.17.0 ✅ · pnpm 9.15.9 ✅ · tauri-cli 2.x ✅ · Ubuntu 24 LTS **TODO** · Apple Developer Program（W0-D6 申请 · TODO）| 新账号/工具时 |

---

## 📍 当前位置

**阶段**：**Spike W0 Day 1 · SPIKE-01 Phase A (macOS) ✅ PASS** · Phase B (Ubuntu) 待环境就绪（2026-04-18 session 7 · 首行代码入盒里程碑）
**日期**：2026-04-18（session 7）
**GitHub**：<https://github.com/tajiaoyezi/vibestation>（PRIVATE）
**已合入的 PR（按时间顺序）**：
- PR #1/#2（Phase 1）· 战略计划 v4 simplified + 决策表 + Calm Studio 视觉
- PR #3/#5（Phase 2 部分）· SPIKE-01..06 + 流程治理最小化
- PR #6/#7（Phase 2 部分）· 5 步导游 + blocked 语义 + 硬化 #2 轮
- PR #8（Phase 2 部分）· MVP-01..10 详细 spec
- **PR #11 `609bcb3`**（Phase 4 基础设施）· `.github/` + workflows + validator + BRANCH-PROTECTION
- **PR #9 `0d15fa9`**（Phase 2 tech debt）· 4 commits · 12 Codex findings 闭合
- **PR #12 `d020365`**（Phase 3 文档）· ADR × 10 + CONTRIBUTING + CHANGELOG + 3 目录 README
- **PR #10 `a157e93`**（Phase 2 占位 spec）· MVP-11..20 + SPIKE-07
- **PR #17 `68c0c21`**（session 6 onboarding 对齐 v2）· 缩范围方案 A · 11 Codex 指控全闭合
- **PR #18 `5ece9a9`**（session 6 spec 翻转）· SPIKE-01/02/MVP-01 走 (b) 路径变种 ready
- **PR #19 `b7b374e`**（session 6 Codex 二次复审漂移修）· 5 项漂移 + (b) 变种正式定义 + overview 归档
- **PR #20 `2ed80f4`**（session 7 **第一行 Rust 代码入盒**）· SPIKE-01 Phase A macOS 冷启动 202ms PASS · 骨架 + 埋点 + 测量脚本 + report + Ubuntu agent 原话 prompt

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
- 最深发现：SPIKE-04 op-log phantom data（marker-loss crash window）· SPIKE-05 后端 IPC queue 满 HOL

### Spike W0 · SPIKE-01 Phase A（session 7 · PR #20）
- [x] Tauri 2 vanilla-ts 骨架（`spike-tmp/spike-01-tauri/` · gitignored · 8.4MB binary · 8.2MB .app bundle）
- [x] Rust 端冷启动埋点（`boot_start` → `setup` callback · 差值打 stderr）
- [x] 测量脚本（mac + Ubuntu 两份 · 仓库归档 `docs/spikes/scripts/SPIKE-01/`）
- [x] **macOS 冷启动 10 次中位数 202ms**（目标 < 2000ms · 10× 余量）· Min 189 · Max 239 · Range 50ms 极稳
- [x] **Bundle 8.2MB**（目标 < 20MB · 2.4× 余量）
- [x] 人工验证 5/5 pass：渲染 / resize / 最小最大关闭 / IME 中文输入 / 5 分钟稳定
- [x] IME 录屏归档（本地 `spike-artifacts/` · gitignored · 2.4MB 大文件不入 repo）
- [x] Phase B Ubuntu 原话 prompt 准备就绪（`docs/spikes/SPIKE-01-report.md §5.1`）
- [ ] **Phase B Ubuntu X11 + Wayland**（等用户环境就绪 · 触发后完成 3 平台全过才翻 SPIKE-01 done）

## 🔜 下一步（按执行顺序）

### 🔐 **用户手动步骤**（`docs/BRANCH-PROTECTION.md` · 当前**已显式暂缓**）

用户已表态暂不应用 main 分支保护（单人 + Codex review 模式下不致命）。**当前流程靠 reviewer 肉眼守门**（accepted tech debt · 见 `docs/tasks/README.md` §原则 7）。

升级触发条件（任一）：
1. 仓库改 public
2. 第二位外部 contributor 出现
3. MVP-01 开始写 Rust 代码
4. 第一个 release tag

触发时按 `docs/BRANCH-PROTECTION.md` checklist 一次性应用。

### 🚀 **Spike Week 0**（进行中 · SPIKE-01 Phase A 已 PASS）

1. **W0-D1** · [SPIKE-01](./tasks/SPIKE-01-tauri-three-platform-boot.md) · **Phase A macOS ✅ PASS（PR #20 merged）** · **Phase B Ubuntu 等环境就绪**
2. **W0-D2** · [SPIKE-02 Tauri 硬通过矩阵](./tasks/SPIKE-02-tauri-hard-pass-matrix.md)（R12 CRITICAL · blocked by SPIKE-01 full done）
3. **W0-D3** · [SPIKE-03 git2 vs gix benchmark](./tasks/SPIKE-03-git2-gix-read-benchmark.md)（linux kernel 仓库 · R3 · blocked by SPIKE-01 full done · ⚠️ 可讨论是否并行解锁 · bench 类不依赖 Tauri UI）
4. **W0-D4** · [SPIKE-04 redb vs rusqlite + git2 写](./tasks/SPIKE-04-storage-benchmark.md)（R27 · B.5 含 reconcile forward · blocked by SPIKE-01 full done · ⚠️ 同上）
5. **W0-D5** · [SPIKE-05 portable-pty 多 Tab 压测](./tasks/SPIKE-05-pty-multi-tab.md)（B.4 前端/后端/hidden-tab 三子场景 HOL · blocked by SPIKE-01 full done）
6. **W0-D6** · [SPIKE-06 Claude/Codex CLI 实机 + Apple Dev Program](./tasks/SPIKE-06-cli-protocol-and-codesign.md)（R1 保留 · 样本录制 · gitleaks 硬阻塞 · blocked by SPIKE-01 full done）

**并行化讨论点**：SPIKE-03/04 是 CLI benchmark（git2 / redb / storage）· 不依赖 Tauri UI · 不依赖 Ubuntu GUI · 仅依赖 Rust toolchain（已就绪）。严格按 spec `blocks` 关系 · 这两个 spec 目前 blocked 在 SPIKE-01 full done；但 **macOS Phase A 的强信号已经足够证明 Tauri 可用** · 反向推翻风险低。若用户决定并行解锁 SPIKE-03/04 · 需修改对应 spec 的 `depends_on` 语义（从 "SPIKE-01 full done" 改为 "SPIKE-01 Phase A done"）。

### 📦 Spike W0 通过后 · MVP 实施（目标 v0.1 GA · 12-14 周）

- MVP-01..10 按依赖顺序实施（MVP-01 → ... → MVP-10）
- MVP-11..20 留 v0.2 / v0.3 / v1.0 kickoff 详化

## ⚠️ 当前卡点 / 注意事项

- **Ubuntu 24 环境缺失**（Phase B 前置 · 阻塞 SPIKE-01 full done · 从而阻塞下游 5 个 Spike）· 缓解：用户补物理机 / VM / 或 SSH 远程 Ubuntu · 原话 prompt 已备即拿即用
- **分支保护已显式暂缓**（用户表态 · 单人 + Codex review 模式下不致命 · 升级触发条件见上方 §🔐 用户手动步骤）
- **R1 Claude/Codex CLI 协议**：SPIKE-06 样本录制 · R1 降级须经 SPIKE-07 parser 验证 + ADR-011
- **R12 CRITICAL Tauri Wayland**：Phase A macOS PASS 强信号 · 但 Wayland + webkit2gtk + IME 仍是关键未验收点 · 不要因 macOS 过就提前放松
- **R27 存储 silent loss**：SPIKE-04 B.5 reconcile forward 设计须实机验证
- **Apple Developer Program 审核**：W0-D6 立刻提交 · 最长 2 周影响 v0.1 发布（W12）
- **域名未定**（W10 决定）· **Logo 未最终选定**（v0.1 前定）

## 🔀 阶段切换信号

| 信号 | 触发 |
|------|------|
| ✅ Phase 1-4 Pre-code 完备 | **已达成**（2026-04-18 session 5 · 4 PR 全 merge）|
| ✅ Spike W0 启动 | **已达成**（2026-04-18 session 7 · 第一行 Rust 代码入盒 · SPIKE-01 Phase A macOS PASS · PR #20）|
| 🟡 Spike Pass | W0-D6 全过（SPIKE-01 full done 需补 Phase B Ubuntu / 然后 02-06 顺序 · Apple 申请并行）|
| 🔴 Spike 任一 CRITICAL Fail | 触发 fallback + ADR supersede |
| 🟡 MVP 实施启动 | Spike W0 + ADR-003/005/006/007 proposed → accepted |
| 🎯 v0.1 GA | MVP-01..10 全过 §10.1 + §10.6 终端正确性矩阵 + §10.3 跨平台 |
| 🔴 连续 2 周 < 5h 投入 | 触发 hibernation（`implementation-plan.md §10.5`）|

## 📦 近期关键交付物索引

| 产出 | 路径 |
|------|------|
| v2 实施计划（14 章 + 附录）| `docs/implementation-plan.md` |
| 7 个 Spike spec（W0 + v1.0-pre）| `docs/tasks/SPIKE-0[1-7]-*.md` |
| 20 个 MVP spec（v0.1 详细 + v0.2+ 占位）| `docs/tasks/MVP-[01-20]-*.md` |
| 10 个 ADR（6 accepted + 4 proposed）| `docs/adr/ADR-0[01-10]-*.md` |
| Agent 入口 · 决策表 · 自审四问 · 翻转 gate | `CLAUDE.md` |
| 人类启动手册 | `docs/SESSION-STARTUP.md` |
| 贡献指南 · 含用户拍板 gate | `CONTRIBUTING.md` |
| 分支保护 admin checklist | `docs/BRANCH-PROTECTION.md` |
| Frontmatter validator + self-test | `scripts/validate-task-spec.mjs` |

---

## Session 日志（近 5 次）

### Session 7（2026-04-18 夜）· Spike W0 启动 · SPIKE-01 Phase A PASS（第一行代码里程碑）
- **第一行 Rust 代码入盒**：`spike-tmp/spike-01-tauri/`（gitignored）· Tauri 2 vanilla-ts 骨架 · 8.4MB binary
- **macOS 冷启动实测**：10 次采样 median **202ms**（目标 < 2000ms · **10× 余量** · Min 189 / Max 239 / Range 50ms 极稳）
- **Bundle 8.2MB**（目标 < 20MB · 2.4× 余量）
- **人工验证 5/5 pass**：窗口渲染 / resize / 最小最大关闭 / IME 中文输入（拼音 + 候选词正常）/ 5 分钟稳定性
- **IME 录屏**：本地存 `spike-artifacts/SPIKE-01/macos-ime.mov`（2.4MB · gitignored · 中途 amend 修了一次录屏误入 commit 的事故 · 补了 `.gitignore` `spike-artifacts/` 规则）
- **PR #20 `2ed80f4`** · 3 commits：`ad1852e` claim + `1085a7b` 骨架 + `bff7a2e` 数据回填 + .gitignore 修复
- **环境栈就绪**：rustup stable (rustc 1.95.0 · 2026-04-14) + NVM Node 20.17.0 + pnpm 9.15.9 (绕开 corepack 0.29.3 签名 bug) + tauri-cli 2.x + Xcode CLT
- **Phase B Ubuntu 策略**：用户暂无 Ubuntu 24 环境 → 在 report §5.1 写好**给接手 agent 的原话 prompt**（自包含背景 / 任务 / 步骤 / 返回格式）· 未来任意 Ubuntu agent / 人肉执行者都可直接粘贴复用
- **SPIKE-01 整体 status 保持 in-progress**：尊重 spec "3 平台全过" 规则 · 单平台过不算整体过 · 下游 SPIKE-02..06 仍 blocked 在 full done
- **并行化讨论待决**：SPIKE-03/04 是 CLI benchmark · 不依赖 Tauri UI · 严格按 blocks 关系被阻 · macOS Phase A 强信号下可讨论修改 depends_on 语义解锁
- **用户新增协作规则**：agent 在完成阶段 / 需要分工时 · 必须给出**原话 prompt** · 用户直接转达给其他 agent（Claude / Codex / Cursor / 人肉均可）· Phase B Ubuntu prompt 是本规则落地的第一个产出

### Session 6（2026-04-18 晚上-夜）· Codex 三轮评审 + PR #17 缩范围 + PR #18 ready 翻转 + 后续修复
- **Codex round-1 评审**：作为新接手 agent 视角评估 onboarding 就绪度（7/10），命中 5 项问题（入口文档过期 / `§5.4` 断链 / `§512` 笔误 / `ready=0` 流程阻塞 / 分支保护未应用）
- **新增 onboarding 评估文档**：`docs/agent-onboarding-readiness-assessment.md`（codex 重写 · 7/10 · 已加 historical snapshot banner + 二次复审段落）
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
  - project-status-overview-2026-04-18.md 纳入仓库（项目梳理报告归档）
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
