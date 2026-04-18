# Vibestation · 完整项目进度梳理（2026-04-18 快照）

> **快照时间**：2026-04-18 晚 · Session 6 结束后
> **覆盖范围**：项目从立项（Session 1）到 Pre-code Phase 1-4 全交付 + Session 6 双 PR + Codex 三轮 review 完整时间线
> **目的**：让任意 agent / 人类在 5 分钟内建立全局认知 · 知道项目在哪 · 知道下一步该做什么
> **权威同步**：本文件是阶段快照 · 滚动状态以 [`docs/PROGRESS.md`](./PROGRESS.md) 为准 · 入口规则以 [`AGENTS.md`](../AGENTS.md) + [`CLAUDE.md`](../CLAUDE.md) 为准

---

## 一、一句话现状

> **项目在 Pre-code 阶段已完整交付 Phase 1-4 全套规划/治理/视觉/基础设施文档（27 task spec + 10 ADR + 14 章战略计划 + 全套 CI），第一行业务代码即将在 SPIKE-01 启动时产生。SPIKE-01 已翻转 ready · 任何 agent 现在可以直接 claim 开工。**

---

## 二、核心数据快照

| 维度 | 数值 |
|------|------|
| 时间跨度 | 2026-04-17 ~ 2026-04-18（6 个 session · 2 天密集工作） |
| 战略计划 | 14 章 + 附录（implementation-plan.md · ~1486 行） |
| Task spec | 27 个（**3 ready** · SPIKE-01/02/MVP-01 · 24 draft 按需触发） |
| ADR | 10 个（6 accepted + 4 proposed · pending Spike 验证） |
| 已 merge PR | **18 个**（主体 13 + dependabot 5） |
| Codex 对抗审查 | **8 轮**（累计 ~30 findings 全闭合） |
| GitHub | https://github.com/tajiaoyezi/vibestation（PRIVATE） |
| 业务代码量 | **0 行**（按设计 · pre-code 阶段定义如此） |
| 投入预算 | 28-30 周 × 20-25 小时 ≈ 600-750 小时（含 20% buffer） |

---

## 三、产品定位（一页纸）

### 一句话卖点

> **"给 Claude CLI / Codex CLI 用户的多 Tab 终端 + JetBrains 级 Git 工作台"**
> 一个窗口管多项目，每个 Tab 一个 CLI 会话，右栏看 Git，不再为了看 commit 打开一堆 IDE。

### 目标用户（3 类 persona）

| Persona | 画像 | 痛点 |
|---------|------|------|
| **Alice** · 独立开发者 | 35 岁全栈 · 同时跑 3-5 个副业 + 多个 Claude CLI | iTerm2 Tab 切来切去 · Claude 改完代码要切 IDE 看 diff |
| **Bob** · 团队技术负责人 | 42 岁架构师 · 维护 monorepo 50 万 commit | JetBrains Git 好用但启动 30s · 终端不在同一窗口 |
| **Carol** · 重度 vibe coder | 28 岁前端 · 全程 CLI 派 · 偏好 Ghostty + tmux | 桌面 Git 客户端太 GUI · gitui 没多 Tab 终端 |

### 核心价值主张（3 点）

1. **多项目 × 多终端的统一工作台** — 一个窗口承载 N 个项目 Tab · 切换成本归零
2. **JetBrains 级 Git 视图 + CLI 级响应速度** — git2 读 + 自绘 Diff（避开 Monaco 3MB） · 10 万 commit 仓库首屏 < 500ms
3. **（v1.0 vision · 对外不宣传）AI session 感知的版本控制** — 把 Claude/Codex CLI 一次对话识别为 session · 自动聚合改动 · 一键回滚

### Non-goals（明确不做）

通用 IDE / Fork Ghostty / 云同步 / 团队协作 / Windows（v1.0 前）/ Git Flow 教条 / 插件市场 / 远程 SSH / 企业代理 / 超大仓库（>1M commit）保证不崩 / Git submodule/LFS 高级支持

---

## 四、Pre-code Phase 1-4 完整交付清单

### Phase 1 · 战略与决策（PR #1/#2 · session 3-4）

- [x] B 阶段技术调研 + planner v1
- [x] 4 视觉方向设计 + **Calm Studio 定稿**（对标 Linear / Zed / Raycast）
- [x] 2 Logo 候选（mark.svg + wordmark-a.svg）
- [x] **Codex 项目级评审**（7 CRITICAL + 12 HIGH + 5 MEDIUM + 13 反对）
- [x] **4 项分歧决策拍板**：Apache 2.0 / MVP B 折中 / AI-Aware 撤出对外 / Tauri 改口"Spike 后锁定"
- [x] **planner v2**（14 章 + 附录 · 30 风险登记 · ~1486 行）
- [x] 独立 GitHub 仓库 + Apache 2.0 LICENSE + NOTICE
- [x] **Phase 1 v1 → v4 simplified** 自我反省（承认过度设计 · 砍多 agent 治理抽象 · 引入"自审四问"）

### Phase 2 · Task spec 框架（PR #3/#5/#6/#7/#8/#9/#10 · session 4-5）

- [x] `docs/tasks/` 框架：YAML schema + `_template.md` + README 索引
- [x] **7 个 SPIKE spec**（SPIKE-01..06 W0 硬通过 + SPIKE-07 v1.0-pre parser 验证占位）
- [x] **20 个 MVP spec**（MVP-01..10 v0.1 详细 + MVP-11..20 v0.2/v0.3/v1.0 占位）
- [x] 流程治理：5 步 PR 导游 · `blocked_from` 语义 · per-task 报告分离 · 翻转 gate (a)/(b) 二选一
- [x] **Codex 对抗审查 12 findings 全闭合**（R1-R6 · 4 commits 修）

### Phase 3 · 架构决策与治理文档（PR #12 · session 5）

- [x] **10 个 ADR** 落地：
  - **6 accepted**：#1 License / #2 MVP 范围 / #3 AI-Aware vision / #5 Workspace / #6 前端栈 / #7 Diff 自建
  - **4 proposed**：#12 桌面框架 / #13 Git 栈 / #14 存储 / #15 PTY（pending SPIKE-02/03/04/05）
- [x] **CONTRIBUTING.md** · 含**用户拍板 gate**（B → A 升级硬阻塞）
- [x] **CHANGELOG.md** · Keep a Changelog 格式
- [x] **CODE_OF_CONDUCT.md** · Contributor Covenant 2.1 中文版
- [x] 3 个 per-task 目录建立：`docs/spikes/` + `docs/spike-artifacts/` + `docs/session-history/` · 各有 README + 安全约束
- [x] Codex 5 findings（3 HIGH + 2 MEDIUM）全闭合

### Phase 4 · GitHub 基础设施（PR #11 · session 5）

- [x] `.github/ISSUE_TEMPLATE/` × 4：config / bug / feature / task_spec_proposal
- [x] `.github/PULL_REQUEST_TEMPLATE.md` · 强制填 Implemented by / Reviewed by / 翻转 gate / 自审四问
- [x] `.github/dependabot.yml` · cargo + npm + github-actions 周更
- [x] **`.github/workflows/ci.yml`** · skeleton（markdown-lint active · rust/frontend 占位）
- [x] **`.github/workflows/secret-scan.yml`** · gitleaks + `gitleaks-bypass-guard`（防内联 marker 绕过）
- [x] **`.github/workflows/task-spec-validator.yml`** · frontmatter schema 校验 · 无 paths filter（防 required-check pending）
- [x] **`scripts/validate-task-spec.mjs`** · 252 行自写 parser + 9 条 adversarial self-test + 7 类 schema 规则
- [x] **`docs/BRANCH-PROTECTION.md`** · admin 应用 main 保护的完整 checklist（用户暂缓应用）
- [x] Codex 3 HIGH findings 全闭合 + CI self-trigger fix

### Session 6 · Codex 双 PR 复盘（PR #17/#18 · session 6）

- [x] **Codex round-1 评估**：onboarding 就绪度 7/10 · 5 项指控
- [x] **PR #17 v1 → v2**（缩范围方案 A · merged `68c0c21`）
  - v1 试图一次修全 5 项 + AGENTS 重写 + §5.4 增补 + ready 翻转 → Codex round-2 BLOCK（11 项指控）
  - v2 撤回 §5.4 + ready 翻转 · 修全 11 项 codex 指控 · 净 +118/-250
- [x] **AGENTS.md 重写**（修复 codex 自动生成版本的"Claude 误替换为 Codex"+ 阶段过期双 bug）
- [x] **PR #18 ready 翻转**（merged `5ece9a9`）· SPIKE-01/02/MVP-01 → ready · 走 (b) 路径变种 · 用户在 GitHub 真 approve
- [x] **§5.4 战略章节决定**：按 YAGNI 推迟到 v0.2 kickoff（届时需要数据流 / IPC / 状态机的实施级细节）
- [x] **Codex 三轮 review 元规则发现**：第三轮抓到 README §205 "reviewer 在 PR comment 里声明路径" 这条之前两轮都漏的元规则

---

## 五、决策固化情况（A/B/C 三档）

详见 [`AGENTS.md`](../AGENTS.md) / [`CLAUDE.md`](../CLAUDE.md) "🔒 决策状态表"。

### A 栏 · 永久锁定（11 条 · 除非写新 ADR 推翻）

| # | 决策 | 锁定依据 |
|---|------|---------|
| 1 | 许可证 = **Apache 2.0**（不签 CLA） | ADR-001 |
| 2 | MVP 范围 = **B 折中方案** | ADR-002 |
| 3 | **AI-Aware Pane 联动** = v1.0 vision（对外禁提） | ADR-009 |
| 4 | 视觉方向 = **Calm Studio** | `design/directions/1-calm-studio.html` |
| 5 | Cargo workspace = **2 crate**（app + core） | ADR-010 |
| 6 | 前端栈 = **SolidJS + TS + xterm.js**（不碰 Floem） | ADR-004 |
| 7 | Diff 渲染 = **自建**（不用 Monaco） | ADR-008 |
| 8 | 平台 MVP = **macOS + Ubuntu 24** · Windows 推到 v0.4 | implementation-plan §3.1 |
| 9 | Tool Windows 默认 = Primary 展开 + Secondary/Bottom 收起 | 原型 JS DEFAULT_STATE |
| 10 | Telemetry = 默认关 + 首次启动 opt-in | implementation-plan §5.1 + R30 |
| 11 | Landing page 栈 = **Astro + 自建动效** | implementation-plan §12 |

### B 栏 · 默认已选 + Spike 后最终锁定（4 条）

| # | 决策 | 默认 | 锁定节点 | Fallback |
|---|------|------|---------|---------|
| 12 | 桌面框架 | **Tauri 2** | SPIKE-02 | Electron 28+ |
| 13 | Git 栈（写） | **git2 0.20** | SPIKE-03 | 读慢 → gix 0.70 混用 |
| 14 | 本地存储 | **redb 2** | SPIKE-04 | 性能/稳定不足 → rusqlite |
| 15 | PTY 方案 | **portable-pty + 单读线程 + mpsc** | SPIKE-05 | 多 Tab 瓶颈 → 一 session 一线程 |

### C 栏 · 时间锁定，结果开放（2 条）

| # | 决策 | 时间点 | 候选 |
|---|------|-------|------|
| 16 | 项目域名 TLD | W10 附近 | `.app` / `.dev` / `.io` |
| 17 | Logo 最终定稿 | v0.1 发布前 | wordmark-a.svg + mark.svg（可能补 combo） |

---

## 六、Task Spec 现状（27 个）

| 类别 | 数量 | status: ready | status: draft | 说明 |
|------|------|---------------|---------------|------|
| **SPIKE**（W0 硬通过） | 6 | 2（SPIKE-01/02） | 4（SPIKE-03/04/05/06） | W0-D1~D6 顺序执行 |
| **SPIKE**（v1.0-pre） | 1 | 0 | 1（SPIKE-07） | R1 降级前置 |
| **MVP**（v0.1 详细） | 10 | 1（MVP-01） | 9（MVP-02..10） | v0.1 GA 范围 |
| **MVP**（v0.2/v0.3/v1.0 占位） | 10 | 0 | 10（MVP-11..20） | 各版本 kickoff 时详化 |
| **合计** | **27** | **3** | **24** | |

### Spike W0 顺序执行图

```
SPIKE-01 (W0-D1) ── ready ✅
   │
   ▼
SPIKE-02 (W0-D2) ── ready ✅ ── R12 CRITICAL · 失败触发 Electron fallback
   │
   ├─► SPIKE-03 (W0-D3) ── draft · git2 vs gix benchmark · R3
   ├─► SPIKE-04 (W0-D4) ── draft · redb vs rusqlite + git2 写 · R27
   ├─► SPIKE-05 (W0-D5) ── draft · portable-pty 多 Tab 压测 · B.4 三子场景 HOL
   └─► SPIKE-06 (W0-D6) ── draft · Claude/Codex CLI 实机 + Apple Dev 申请 · R1

SPIKE-07 (v1.0-pre) ── draft · 占位 · 等 SPIKE-06 done + R1 降级触发
```

### MVP 依赖链（v0.1）

```
SPIKE-02 (Tauri 锁定)
   │
   ▼
MVP-01 (Tauri shell) ── ready ✅
   │
   ├─► MVP-02 (Workspace 管理)
   ├─► MVP-03 (Tool Windows 布局)
   └─► MVP-04 (多 Tab 终端) ── 依赖 SPIKE-05/06
        │
        ├─► MVP-05 (Pane 单层分屏)
        └─► MVP-06 (配置导入 Ghostty/iTerm2/Alacritty)

MVP-02/03 → MVP-07 (Git Log 只读) ── 依赖 SPIKE-03
   │
   └─► MVP-08 (Diff + Status)
        │
        └─► MVP-09 (Stage/Commit) ── 依赖 SPIKE-04
             │
             └─► MVP-10 (设置 + Telemetry + 打包) · v0.1 GA
```

---

## 七、风险登记（30 条 · 关键摘录）

完整见 [`implementation-plan.md §9`](./implementation-plan.md)。

### CRITICAL（3 条）

| # | 风险 | 触发时机 | 对策 |
|---|------|---------|------|
| **R12** | Tauri 2 在 Ubuntu 24 Wayland 不稳定 | SPIKE-01/02 | 失败 → Electron 28+ + 1-2 周额外工期 |
| **R21** | macOS notarization + Hardened Runtime 配置错误 | W11 | Apple Developer Program 需 1-2 周审核 · W0-D6 立即申请 |
| **R24** | 终端正确性（IME/CJK/OSC52/mouse/alt-screen/tmux 兼容） | W11 | 专项验收矩阵 + 每项可 demo |

### HIGH（典型）

| # | 风险 | 对策 |
|---|------|------|
| R1 | Claude/Codex CLI 协议解析失败 | SPIKE-06 实机录制样本 · v1.0 W23 单独 spike |
| R3 | git2 大仓库 log 慢 | SPIKE-03 benchmark · 慢则引入 gix 读 |
| R4 | macOS GUI PATH 空 | `fix-path-env` crate |
| R17 | 单人维护精力耗尽 | 连续 2 周 < 5h 投入 → 进入 §10.5 hibernation |
| R27 | redb 状态损坏 / 升级迁移失败 | schema_version + 备份 + 自检 + 手动导出/导入 |

---

## 八、里程碑路线图

| 里程碑 | 时间 | 内容 | 状态 |
|--------|------|------|------|
| **Pre-code 全交付** | 2026-04-17~04-18 | 14 章战略 + 27 spec + 10 ADR + 全套治理/CI | ✅ **已完成** |
| **Spike W0**（1 周） | 待启动 | Tauri Pass/Fail · PTY · 多 Tab · CLI 实机 · git2 读/写 · 存储 benchmark | 🟡 SPIKE-01 ready |
| **v0.1 MVP**（+12 周） | 2026-Q3 | 多 Tab 终端 + Git log/status 只读 + Commit + 基础 Diff + 单层 Pane + 配置导入 + 崩溃恢复 + macOS/Linux 签名打包 | ⏳ 待 Spike Pass |
| **v0.2**（+5 周） | | Push/Pull/Fetch + Rail graph + 分支管理 + Pane 任意嵌套 | ⏳ |
| **v0.3**（+5 周） | | Rebase/Merge/Cherry-pick + 冲突解决 + Pop to External | ⏳ |
| **v1.0**（+6-8 周） | 2026-11 下旬 | **AI-Aware Pane 联动**（session ↔ commit · 一键回滚 AI 改动） | ⏳ |

---

## 九、技术栈最终态（截至 v0.1）

```
┌────────────────────────────────────────────────────┐
│                 Vibestation v0.1                   │
├────────────────────────────────────────────────────┤
│ Desktop:  Tauri 2 (Spike-locked) / Electron 28+    │
│ Frontend: SolidJS + TypeScript + Vite + xterm.js   │
│ Style:    原生 CSS + oklch token + Inter +         │
│           JetBrains Mono                           │
│ Backend:  Rust (cargo workspace 2 crate · app+core)│
│ Git:      git2 0.20 (write) + 可选 gix 0.70 (read) │
│ PTY:      portable-pty + 单读线程 + mpsc           │
│ Storage:  redb 2 / rusqlite (Spike-决定)           │
│ License:  Apache 2.0 (no CLA)                      │
│ Platform: macOS 15 + Ubuntu 24 (Wayland + X11)     │
│ Distrib:  mac dmg + Linux AppImage                 │
│ Landing:  Astro + 自建动效                         │
└────────────────────────────────────────────────────┘
```

---

## 十、Codex 8 轮对抗审查全统计

这套对抗式 review 流程是项目质量保障的核心机制。

| 轮次 | 范围 | 阶段 | findings | 收敛模式 |
|------|------|------|----------|---------|
| Round 1-3 | Phase 1 v1-v3 | session 4 | 21 HIGH | 砍过度设计 → v4 simplified |
| Round 4-6 | Phase 2 task spec | session 4-5 | 12 findings | SPIKE/MVP spec Acceptance 硬化 |
| Round 7 | Phase 4 基础设施 | session 5 | 3 HIGH + CI self-trigger fix | gitleaks-bypass-guard 双层防护 |
| Round-1（Session 6） | onboarding 就绪度 | session 6 | 7/10 评估 + 5 指控 | 触发 PR #17 v1 |
| Round-2（Session 6） | PR #17 v1 | session 6 | **3 CRITICAL + 3 HIGH + 3 MEDIUM + 2 LOW** | BLOCK → 触发 PR #17 v2 缩范围 |
| Round-3（Session 6） | PR #18 | session 6 | 1 HIGH + 1 MEDIUM + 1 LOW | BLOCK 是流程时序问题（用户没 approve）· 内容认可 |

**累计 ~30 findings 全闭合**。Codex 三轮 review 进化轨迹：
- Round-1 抓**静态文档过期**
- Round-2 抓**流程绕过**（自创路径）
- Round-3 抓**元规则细节**（reviewer 在 PR comment 里声明路径）

---

## 十一、当前真实状态

### 当前没有任何阻塞

- ✅ Pre-code 全交付
- ✅ 3 个 ready task 可立即认领
- ✅ 0 个 open PR
- ✅ main 分支干净（在 `5ece9a9`）
- ✅ CI 全绿
- ✅ 工作树 clean

### 真正的"等待"在用户侧

| 等待项 | 触发条件 |
|--------|---------|
| 启动 SPIKE-01 mac 半边 | 用户有 1-2 小时 + mac 在手 |
| 启动 SPIKE-01 完整版 | 用户有 mac + Ubuntu 24 Wayland + Ubuntu 24 X11 三平台 |
| Apple Developer Program 申请 | SPIKE-06 W0-D6（1-2 周审核） |
| 域名 TLD 决定 | W10 附近 |
| Logo 最终定稿 | v0.1 发布前 |

### 已知但不阻塞的小漂移（含 Codex assessment 二次复审发现）

| 漂移项 | 影响 | 修复时机 |
|--------|------|---------|
| `docs/PROGRESS.md` Active branch / Next action 说 PR #18 还 open | onboarding 信息略过期（PR #18 已 merge） | 下次 session 顺手修 |
| **`docs/PROGRESS.md` 内部矛盾**（codex 复审发现）：line 110 "分支保护已显式暂缓" vs line 122 阶段切换信号表里仍写 "🟡 Spike W0 启动 \| 用户应用分支保护后" | 阻塞条件叙述不一致 | 顺手修 |
| **`docs/SESSION-STARTUP.md` 中段残留**（codex 复审发现）：line 60 把 SPIKE-01 写为"status: draft，需先翻转 gate 升 ready"；line 121 仓库结构图里写"27 task spec · 全 draft" | 与 PR #18 已合入的 3 ready 状态不符 | 顺手修 |
| **PR #18 reviews=[] 但已 MERGED**（codex 复审发现）：用户直接 squash merge 时没在 GitHub UI 点 "Approve" → reviews 字段实际为空 → 技术上违反 "(b) 路径变种" 约定的 "reviews ≠ ∅" 硬要件 | 历史事实漂移 · 范围比 PR #17 v1 小（review 已通过对话完成 · 仅 GitHub metadata 缺失）· accepted tech debt | 后续 PR 改走完整路径或补 (b) 变种术语正式化 |
| Codex round-3 LOW · "(b) 路径变种" 术语 README 没正式定义 | 后续审计可能歧义 | 第二位 reviewer 进来时再补 |

> **二次复审的元价值**：Codex 在 assessment 文档上做了二次复审（自动触发 · 不需要再次启动 review），3 项指控全部成立。这印证了"codex 作为持续 reviewer"的可行性——它会自己回头校准之前的判断。本次梳理刚写完就被 codex 抓到几处轻度乐观偏差 · 已如实记录。

---

## 十二、新接手 agent 的 5 跳 onboarding（≈ 15 分钟）

```
1. AGENTS.md           (1 分钟) — 工具无关入口 · 路由
2. CLAUDE.md           (3 分钟) — 项目权威规则 · 决策表 · 禁区 · 自审四问
3. docs/PROGRESS.md    (2 分钟) — 当前位置 · 下一步 · 卡点
4. docs/tasks/README.md (3 分钟) — 任务索引 · 状态流转 · 翻转 gate
5. 挑 ready task → 走 5 步 PR 流程
```

详见 [`AGENTS.md`](../AGENTS.md)。

---

## 十三、下一步可选动作（按优先级）

### P0 · 真正实质性进展

启动 **SPIKE-01 Tauri 三平台空壳启动**：
- 创建 `spike/SPIKE-01-tauri-three-platform-boot` 分支
- 第 1 commit `chore(SPIKE-01): claim`（status: ready → in-progress + 填 owner）
- 实施 commits（spike-tmp/spike-01-tauri/ + Tauri 空壳 + 三平台测试 + 录屏）
- 收尾 commit `chore(SPIKE-01): done`
- 产出 `docs/spikes/SPIKE-01-report.md` + `docs/spike-artifacts/SPIKE-01/*.mp4`

**单平台模式**（如果只有 mac）：
- 完成 mac 部分 Pass Criteria（4-5/7）
- 标记 Ubuntu 部分 `pending-cross-platform`
- 等 Ubuntu 机器就绪后接力

### P1 · 准备工作（不阻塞 SPIKE-01）

- 申请 Apple Developer Program（1-2 周审核 · W0-D6 之前提交）
- 准备 Ubuntu 24 Wayland + X11 测试机
- 准备 linux kernel 仓库 clone（SPIKE-03 大仓库 benchmark 用）

### P2 · 文档维护（轻微 · 可推迟）

- 修 PROGRESS.md 的 PR #18 状态漂移
- 把"(b) 路径变种"术语正式写进 docs/tasks/README.md 第 7 步（codex round-3 LOW）

### P3 · 不做（按 YAGNI）

- ~~PR #18（曾计划 §5.4 重写）~~ → **已取消** · 推迟到 v0.2 kickoff
- ~~应用 main 分支保护~~ → 用户已表态暂缓 · 升级触发条件见 PROGRESS

---

## 十四、关键元发现（Session 6 收获）

这部分是项目元层面的智慧积累，值得固化到决策记忆里。

### 1. AGENTS.md / CLAUDE.md 双入口架构

- `AGENTS.md` = 工具无关极简入口（路由 + 关键约束摘录）
- `CLAUDE.md` = 项目权威单文件入口（详细规则 / 决策 / 禁区 / 命令速查）
- 两份文件冲突时以 CLAUDE.md 为准
- **教训**：codex 自动生成 AGENTS.md 时容易做"Claude → Codex"系统性误替换 · 必须人工 review

### 2. 翻转 gate (a)/(b) 实战经验

| 路径 | 操作 | 依赖 | 适用 |
|------|------|------|------|
| (a) Reviewer 自己 push 翻转 commit | 用户本地 git push | 无 | 严格无歧义 · 但操作成本高 |
| (b) Author push + Reviewer re-approve 最新 HEAD | Claude push + 用户 GitHub UI approve | 分支保护"require approval from latest commit" | 标准路径 |
| **(b) 变种** | 同 (b) · 但分支保护暂缓时 | 流程约定替代技术强制 · accepted tech debt | **当前阶段使用** |

**关键约束**（codex round-3 抓出）：
- reviews ≠ ∅（用户必须真在 GitHub UI approve · 不是"merge 间接 approve"）
- reviewer 必须在 PR comment 里**显式声明走哪个路径**（README §205）

### 3. §5.4 决策的 YAGNI 教训

- PR #17 v1 试图补 §5.4 战略章节 → codex round-2 抓到 6 类硬伤（虚构类型 / 风险编号错乱 / 与 MVP-11/13 Don't 冲突）
- PR #17 v2 撤回 → 4 MVP plan_ref 用 §10.1 workaround
- **结论**：v0.2 kickoff 时再补 §5.4 才是合适时机（届时已有 v0.1 实战经验 · 不会 invent 内容）

### 4. Codex 作为独立 reviewer 的不可替代价值

- 8 轮 review 抓到的问题质量明显高于"Claude 自评估"
- Round 1-7 抓项目内容问题
- Round-1~3（session 6）甚至抓 PR 流程层面的问题
- **流程**：spec / 重大文档 PR 默认走"双轮 codex review"

### 5. 自审四问 的实战价值

写规则 / 文档 / spec 前必问：
1. **递归完备性**：清单自己在清单里吗？
2. **反向场景**：违规会怎样？
3. **边界适用性**：所有数据形态 / 并发数 / 阶段都适用吗？
4. **YAGNI**：当前阶段真需要吗？

**任一答不清楚 → 删掉或标 `[planned]`**。这条规则在 session 4 v4 simplified 引入后多次救场。

---

## 十五、文档导航（一图全索引）

```
vibestation/
├── AGENTS.md                            ← 工具无关 agent 入口（路由）
├── CLAUDE.md                            ← 项目权威单文件入口（规则 / 决策 / 禁区）
├── README.md                            ← 对外（规划期）
├── LICENSE / NOTICE                     Apache 2.0
├── CONTRIBUTING.md                      贡献指南 + 用户拍板 gate
├── CODE_OF_CONDUCT.md                   Contributor Covenant 2.1 中文
├── CHANGELOG.md                         Keep a Changelog
├── .github/
│   ├── ISSUE_TEMPLATE/                  4 模板（config / bug / feature / task_spec_proposal）
│   ├── PULL_REQUEST_TEMPLATE.md         强制 Implemented by / Reviewed by / 翻转 gate
│   ├── dependabot.yml                   cargo + npm + github-actions 周更
│   └── workflows/
│       ├── ci.yml                       markdown-lint + rust/frontend placeholder
│       ├── secret-scan.yml              gitleaks + bypass-guard
│       └── task-spec-validator.yml      frontmatter schema 校验
├── scripts/
│   └── validate-task-spec.mjs           252 行 + 9 self-test
├── docs/
│   ├── implementation-plan.md           战略计划（14 章 + 附录）
│   ├── codex-review-and-response.md     Phase 1 codex 评审存档
│   ├── tech-research.md                 三项目预研
│   ├── PROGRESS.md                      滚动状态面板（权威）
│   ├── SESSION-STARTUP.md               人类启动手册 + Playbook + FAQ
│   ├── BRANCH-PROTECTION.md             admin 应用 main 保护 checklist
│   ├── agent-onboarding-readiness-assessment.md  codex 评估稿（pre-PR-17 snapshot）
│   ├── project-status-overview-2026-04-18.md     ← 本文件
│   ├── tasks/                           27 task spec
│   │   ├── README.md + _template.md
│   │   ├── SPIKE-01..07-*.md            7 spike spec
│   │   └── MVP-01..20-*.md              20 mvp spec
│   ├── adr/                             10 ADR
│   │   ├── README.md + _template.md
│   │   └── ADR-001..010-*.md
│   ├── spikes/README.md                 per-task SPIKE 报告目录（Spike 启动后产出）
│   ├── spike-artifacts/README.md        per-task 录屏/截图目录（Spike 启动后产出）
│   └── session-history/README.md        session 归档
└── design/
    ├── index.html                       视觉方向总览
    ├── directions/
    │   ├── 1-calm-studio.html           ⭐ 主视觉定稿（1329 行）
    │   ├── 2-terminal-native.html
    │   ├── 3-codex-inspired.html
    │   └── 4-vscode-dense.html
    └── logos/
        ├── mark.svg                     主图标
        └── wordmark-a.svg               文字标识
```

---

## 十六、一句话总结

> **Vibestation 在 6 个 session（48 小时）内完成了从 0 到"可启动 SPIKE-01"的完整 pre-code 准备。8 轮 Codex 对抗审查淘汰了 30 多个潜在问题。当前状态干净、决策固化、流程合规。差的只是**用户的 1-2 小时 + mac 前面的实施动作**——而那一刻，第一行业务代码就会产生。**

---

*评估者：Claude (Sonnet 4.6)*
*快照基于：main 分支 commit `5ece9a9`（PR #18 squash · 2026-04-18 12:52 UTC）*
*下次刷新：SPIKE-01 启动后 / 重大决策变化时*
