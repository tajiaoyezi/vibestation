<!--
  AGENTS.md · 通用 Agent 入口（兼容 OpenAI / Codex CLI agents.md 约定）
  本文件是工具无关的 agent 总入口。所有 agent 工具（Codex CLI / Claude Code / Cursor /
  Aider / OpenCode / Windsurf / Droid / Gemini / Kimi / 自建 …）都从这里开始。

  本文件不重复 CLAUDE.md 内容，只做"路由 + 关键约束摘录"。
  CLAUDE.md 是当前项目的事实权威单文件入口（详细规则 / 决策表 / 禁区 / 自审四问）。
-->

# Vibestation · Agent 通用入口

> 给 **Claude CLI / Codex CLI / 其他 agent CLI** 用户的多 Tab 终端 + JetBrains 级 Git 工作台
> （Tauri 2 桌面应用 · Apache 2.0 · 不签 CLA）。

**本项目不绑定具体 agent 工具。** 任何工具（Claude Code / Codex CLI / Cursor / Aider / OpenCode / Windsurf / Droid / Gemini / Kimi / 自建 …）皆可贡献。

---

## 📍 当前阶段（一句话）

**v0.3 sprint 5/5 MVP 完整代码收官 99%**（session 30 末 · MVP-17 Phase A/B/C 完整收口 · Phase A @ PR #291 + Phase B @ PR #301 + Phase C 源码 @ PR #292 + 测试重写 @ PR #297 + wiring 核心 @ PR #302 · 仅 MVP-17 E.4 settings UI + 全 5 MVP Phase D Arbiter playbook 推迟 · 4-agent dispatch pool 首次同时跑实证 = OpenCode + Codex + Droid + Cursor · 当前事实以 `docs/PROGRESS.md` 为准）。

---

## 🚀 第一次进入仓库 · 阅读顺序（5 跳 ≈ 15 分钟）

1. **本文件**（你正在读）—— 知道下一跳读什么
2. **[`CLAUDE.md`](./CLAUDE.md)** —— 项目权威规则 / 决策表 A/B/C / 禁区 / 自审四问 / 5 步 PR 流程
3. **[`docs/PROGRESS.md`](./docs/PROGRESS.md)** —— 当前阶段 / 上次 session / 下一步 / 卡点
4. **[`docs/tasks/README.md`](./docs/tasks/README.md)** —— 任务索引 + 状态流转 + `draft → ready` 翻转 gate
5. **挑一个 `status: ready` 的 task** → 按 `CLAUDE.md` 第 5 步走完整 PR 流程

> **如果 `ready` 任务为 0**：先帮某个 `draft` 走独立评审升 `ready`（流程见 `docs/tasks/README.md` 第 7 步）。

---

## 🔗 关键文档地图

| 类别                         | 路径                                                 | 用途                                                      |
| ---------------------------- | ---------------------------------------------------- | --------------------------------------------------------- |
| **权威单文件入口**           | `CLAUDE.md`                                          | 规则 / 决策 / 禁区 / 命令速查 · 所有 agent 工具的事实标准 |
| **当前位置**                 | `docs/PROGRESS.md`                                   | 阶段 / 进度 / 下一步                                      |
| **人类阶段感知手册**         | `docs/internal/SESSION-STARTUP.md`                   | 当前阶段 Playbook + FAQ                                   |
| **任务索引**                 | `docs/tasks/README.md` + `docs/tasks/<TYPE-NN>-*.md` | 30 个 task spec                                           |
| **架构决策**                 | `docs/adr/README.md` + `docs/adr/ADR-NNN-*.md`       | 14 ADR accepted                                           |
| **战略计划（14 章 + 附录）** | `docs/implementation-plan.md`                        | 完整产品定位 / 架构 / 风险登记 / Milestone                |
| **视觉原型**                 | `design/directions/1-calm-studio.html`               | Calm Studio 定稿 · 1329 行可直接 `open` 体验              |
| **贡献指南**                 | `CONTRIBUTING.md`                                    | PR 流程 + Commit 规范 + 用户拍板 gate                     |
| **分支保护**                 | `docs/BRANCH-PROTECTION.md`                          | admin checklist（当前未应用）                             |

---

## ⚡ 关键约束（详细见 CLAUDE.md "🚫 禁区"）

- ❌ **禁止 push 到 main**：所有变更走 feature 分支 + PR + 独立评审
- ❌ **禁止对外文案提及 v1.0 vision 具体名词**（具体禁词见 `CLAUDE.md §禁区` + [ADR-009](docs/adr/ADR-009-ai-aware-v1-vision.md) · 本条不 spell out 以避免 AGENTS 自身违规）
- ❌ **禁止硬编码** API Key / 密码 / Token / 个人邮箱 / 生产域名
- ❌ **禁止跳过 CI 必过项**：`gitleaks` / `task-spec-validator` / `cargo clippy -D warnings` / `cargo fmt --check` / `pnpm lint` / `pnpm typecheck`
- ⚠️ **改 `CLAUDE.md` 决策表 A 栏前**：(1) 开新 ADR · (2) 独立评审通过 · (3) 用户拍板 gate

---

## 🤝 多 Agent 协作（简版）

每个 commit 必须：

1. 走 feature branch（命名 `<scope>/<task-id>`，如 `spike/SPIKE-01-tauri-three-platform-boot`）
2. Commit 消息 = Conventional Commits + 中文描述 + `Co-authored-by: <Agent Name> <email>` trailer
3. PR description 必填 `Implemented by: <agent-id>` + `Reviewed by: <≠ Implementer>`
4. 独立评审 ≠ 原实现者
5. 翻转 gate（reviewer 自己 push 翻转 commit 推荐 · 防作者私自改 spec）

---

## 📝 Agent 身份示例（commit trailer）

| 工具                       | trailer                                               |
| -------------------------- | ----------------------------------------------------- |
| Claude Code                | `Co-authored-by: Claude Code <noreply@anthropic.com>` |
| Codex CLI                  | `Co-authored-by: Codex CLI <noreply@openai.com>`      |
| OpenCode                   | `Co-authored-by: OpenCode <noreply@opencode.ai>`      |
| Cursor                     | `Co-authored-by: Cursor <noreply@cursor.com>`         |
| Droid (Factory.ai)         | `Co-authored-by: Droid <noreply@factory.ai>`          |
| Kimi (Moonshot · 远程 API) | `Co-authored-by: Kimi <noreply@moonshot.ai>`          |
| 其他                       | `Co-authored-by: <Tool Name> <email>`                 |
| 人类                       | `Co-authored-by: <Name> <email>`                      |

**4-agent dispatch pool 能力分工**（session 30 实证 · 见 `.claude/rules/dispatch-prompt-template.md` §2.9 Agent 能力矩阵）：

- **Codex CLI** · Rust 后端 / Tauri lifecycle / 复杂集成测试 · 历史最稳
- **OpenCode** · 机械重构（binding rebase · 删 mock · grep 可验证）· 文档 sync · style 调整 · ❌ 不分配测试重写 / 复杂逻辑（N=3 §2.10 违规史 · task 类型受限 · N=4 触发永久转出）
- **Cursor** · IDE 插件型 · React/Solid 测试 + 复杂组件逻辑 · jest-dom + vitest 接入
- **Droid** · 纯文档（PROGRESS sync · spec frontmatter 翻转）· 新加入 · session 30 首次走全流程
- **Kimi** · 远程 API · 需 prompt 内附 spec 原文（无 worktree access）· spec review / draft 任务

---

**本文件保持极简，详细规则永远以 [`CLAUDE.md`](./CLAUDE.md) 为权威。两份文件冲突时以 `CLAUDE.md` 为准。**

---

## 🪟 S2V Windows 适配工作流（feat/windows-support 分支 · `/s2v-init` 生成）

> 本段由 `/s2v-init` 为 **Windows 适配工作流** 追加，与上方项目通用入口**并存**。仅 Windows 适配 task（`docs/specs/tasks/task-*.md`）走本段的 S2V 流程；项目既有 MVP 体系（`docs/tasks/`）不受影响。规范快照在 `docs/s2v/`，adapter 在 `docs/s2v-adapter.md`。

**Collaboration Tier = solo**

<!-- solo：单分支(feat/windows-support)无人值守 · 主 agent 兼 Arbiter + 调度 subagent 实施 · 直接在分支三段 commit，不开 per-task worktree/PR。整个分支最终作为一个 PR 合入 main。 -->

### 必守清单（任何 tier 不可降级）

1. **SDD**：phase spec / task spec 必写（`docs/specs/`）
2. **BDD**：用户可见行为有 `.feature`（`test/features/`）
3. **TDD Iron Law**：先写失败测试（RED），再写实现（GREEN）——没有 RED 的 commit 禁止 GREEN
4. **§2.5 三段 commit 节律**：每个 task 至少 RED(`test`) + GREEN(`feat`)，REFACTOR(`refactor`) 可选，§10 回填(`docs`)
5. **ADR**：架构/依赖/协议/安全/数据决策必写（`docs/decisions/`）
6. **Verification**：每个 task done 必跑 task §9 实际列出的验证项（unit-test 强制）
7. **§7 追踪表**：每 task 维护 AC ↔ SCEN ↔ TEST 映射
8. **卡住协议**：AC 失败 ≥3 次写 `BLOCKED-task-<X.Y>.md`

### task 启动 SOP（每个 task 开始前）

1. **基线绿**：动手前确认无遗留红测试（`cargo test --workspace`；首个编译-修复 task 因 Windows 编译失败而基线红，是该 task 要解决的红 → 跳过基线绿并在 §10 备注）
2. **读规格**：AGENTS.md → `docs/s2v-adapter.md` → 本 task spec → §5.1 Required Reading 上游 + `.feature` + 相关 ADR
3. **PREFLIGHT Ready Gate**：task spec Status 必须 Ready（无 `<TBD-by-user>` 残留、§6 AC 非空、§7 非空）才进 RED。无人值守模式下由主 agent 以 Arbiter 身份完成 Draft→Ready 审核（基于 Windows 缺口调研证据填实 §3/§5.2/§5.3）
4. **RED → GREEN → REFACTOR**：三段 commit，每段 commit 后校验 `[branch]`（应为 `feat/windows-support`）
5. **§9 Verification 全套**：unit-test 强制；前端 task 跑 `pnpm lint`/`pnpm typecheck`/`pnpm vitest run`
6. **回填 §10 Completion Notes**（6 项 schema）+ Status → Done + 同步 adapter Task 索引

### §2.5 Commit 节律

| 阶段 | type | 示例 |
|---|---|---|
| RED | `test` | `test(pty): 加 SCEN-1.1.1~1.1.3 共 3 个 RED 测试（Windows cfg 编译）` |
| GREEN | `feat` | `feat(pty): cfg 分离 Unix reader + Windows ConPTY 路径，编译通过` |
| REFACTOR | `refactor` | `refactor(pty): 提取 platform_reader helper` |
| §10 回填 | `docs` | `docs(spec): 回填 task-1.1 §10 + Status → Done` |

Scope = 模块名（pty / app-settings / external_term / config_import / fs_watch / web / tauri-bundle / ci / spec / adapter / adr）。

### git 协作（solo · 本工作流）

- 直接在 `feat/windows-support` 分支 `git commit` ✅；**禁止直推 main**（Vibestation 禁区 + `.githooks/pre-push`）
- `git reset --hard` / `git push --force*` 禁止默认（用 `git revert` / `git branch -f`）
- 整个分支最终作为**一个 PR** 合入 main（v2-D.2：PR body 含 `Implemented by` / `Reviewed by` / `Arbiter approval` 三行）

### 卡住协议

AC 连续失败 ≥3 次且已试 systematic-debugging + 查上游 spec/ADR → 写 `BLOCKED-task-<X.Y>.md`（卡住 AC / 已尝试 / 当前假设 / 决策需求 A/B/C / 测试代码状态）→ commit → 等 Arbiter 决策。

> 完整 solo SOP 见 `docs/s2v/templates-used/agents-solo.md`；S2V 22 章规范见 `docs/s2v/standard.md`。
