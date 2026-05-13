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
| **人类阶段感知手册**         | `docs/SESSION-STARTUP.md`                            | 当前阶段 Playbook + FAQ                                   |
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
