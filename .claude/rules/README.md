# Vibestation 项目规则索引

> 本目录存放 **Vibestation 项目级规则**（不跟随 Claude Code 全局 · 只在本 repo 生效）·
> 补充全局规则 `~/.claude/rules/` 在 Vibestation 场景的具体落地约束。
>
> 全局规则定义"怎么协作"· 本目录规则定义"在 Vibestation 怎么协作"。
>
> **新 agent 首次进项目**：先读 `CLAUDE.md`（锁定表 + 禁区 + 决策表）·
> 再按任务类型读对应规则（见下方"阅读顺序建议"）。

---

## 📂 规则索引

| 文件                                                         | 触发条件                                                  | 核心要求                                                                                                                                                                                                  | 关联全局 rule                                                                                                               | 事件源                                                                                                                |
| ------------------------------------------------------------ | --------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| [dispatch-prompt-template.md](./dispatch-prompt-template.md) | 下发 prompt 给外部 agent 执行任务时                       | 硬约束 12 条（禁止自行 accept decision / worktree 隔离 / commit 身份 3 条铁律 / runtime 证据 / Acceptance 全覆盖）· 远程 API agent 需附 spec 原文 · 本地 CLI 给路径即可                                   | `~/.claude/rules/13-cross-agent-delivery.md` + `17-dispatch-agent-capability-matrix.md` + `15-runtime-verification-gate.md` | SPIKE-04.5 §A.3 OpenCode 自行 accept + MVP-02 OpenCode 主 working tree 分支冲突 + PR #71/#82/#83 author 错归 3 次事件 |
| [spike-delivery-checklist.md](./spike-delivery-checklist.md) | Spike review accept 前 · PR 开出前 · status done 翻转前   | 3 样必交（report + code + raw · 全进 git）+ 1 样推荐（冷备 · v2 · ADR-013 降级）+ accept 原子性（不可跨 session 拆分）                                                                                    | `~/.claude/rules/13-cross-agent-delivery.md` + `09-task-workflow.md`                                                        | SPIKE-04.5 §A.3 + v1→v2 降级 ADR-013（22% 合规率实证 · session 13 audit M-1）                                         |
| [tauri-v2-patterns.md](./tauri-v2-patterns.md)               | 接触 crates/app/（Tauri 启动层）或 web/（SolidJS 前端）前 | ACL permission 强制（自定义 command 必须显式声明 · 否则 runtime deny）· CSP 最小化（生产用最小集 · 非 null）· Capability 最小权限（core:default → 精确子集 · MVP-04 Phase B 前收紧）· CLI --config 位置坑 | 无（Tauri 框架专项）                                                                                                        | PR #28 Tauri v2 ACL deny + Codex adversarial review · CSP + opener + core:default 问题                                |

> ⚠️ **2026-05-20 · runtime-evidence-location.md 已删除**（ADR-023 supersede ADR-011）：MVP / feature 类 capture 硬要求已 supersede · MVP 类 PR 不再强制 5+ 截图 / 录屏 / GUI capture。已捕证据继续保留作 ship audit。Spike 类仍按 `spike-delivery-checklist.md` 4 样齐全。详见 [ADR-023](../../docs/adr/ADR-023-capture-mandate-removed.md)。

---

## 📖 阅读顺序建议（新 agent 首次进项目）

按任务类型选择必读规则 · 3 秒定位入口：

| 任务类型                        | 必读规则                                                                        | 优先级  |
| ------------------------------- | ------------------------------------------------------------------------------- | ------- |
| **外部 agent 接 dispatch 任务** | dispatch-prompt-template.md（12 条硬约束是 BLOCK 条件）                         | 🔴 必读 |
| **Spike 实施 / review**         | spike-delivery-checklist.md（3 样必交 + accept 原子性）                         | 🔴 必读 |
| **MVP / feature 含 GUI / IPC**  | tauri-v2-patterns.md（capture mandate 已 ADR-023 supersede · 不再要求截图归档） | 🔴 必读 |
| **纯文档 chore**                | 无（CI 通过即可 · 无 runtime 要求）                                             | —       |
| **所有 agent**                  | CLAUDE.md（决策表 + 禁区 + 5 步 checklist）                                     | 🔴 必读 |

> 全局规则 `~/.claude/rules/` 由 Claude Code 自动加载 · 不需手动读取。

---

## 🔗 项目规则 vs 全局规则

项目规则都是全局规则在 Vibestation 场景的**具体落地**。对照表：

| 项目规则                    | 上位全局规则                                                                            | Vibestation 专项约束                                                                                                |
| --------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| dispatch-prompt-template.md | `~/.claude/rules/13-cross-agent-delivery.md` + `17-dispatch-agent-capability-matrix.md` | 硬约束 12 条 · 远程/本地 CLI/IDE 插件分三类适配 · commit 身份 3 条铁律（2.5.1 config + 2.5.2 trailer + 2.5.3 验证） |
| spike-delivery-checklist.md | `~/.claude/rules/13-cross-agent-delivery.md` + `09-task-workflow.md`                    | 3 样必交进 git · v2 冷备降级（ADR-013 · 22% 合规率实证）· accept 原子性（不可跨 session）                           |
| tauri-v2-patterns.md        | 无（Tauri 框架专项）                                                                    | ACL permission 强制 · CSP 最小集 · Capability 精确子集 · `--config` 位置语法坑                                      |

> ~~runtime-evidence-location.md~~ · **已删除**（ADR-023 supersede ADR-011 · 2026-05-20）· 详见 [ADR-023](../../docs/adr/ADR-023-capture-mandate-removed.md)

---

## 📋 项目规则间交叉引用

3 条项目规则（runtime-evidence-location.md 已 ADR-023 删除 · 2026-05-20）的交叉依赖：

```
dispatch-prompt-template.md（下发层）
  ├── §2.3 引用 → spike-delivery-checklist.md（Spike 4 样齐全）
  └── §2.4 引用 → ~/\{global\}/16-multi-agent-worktree-sync.md（worktree 隔离）

tauri-v2-patterns.md（框架层）
  └── 独立 · 无项目内交叉引用
```

---

## ⚠️ 常见踩坑速查

| 踩坑                                 | 规则                                | 正确做法                                                          |
| ------------------------------------ | ----------------------------------- | ----------------------------------------------------------------- |
| 外部 agent 自行标 "Arbiter 选定 X"   | dispatch §2.1                       | 硬约束禁止 · 只能建议 · Arbiter 在 PR comment 明确 approve 才生效 |
| CI 绿就认为 runtime 过了             | dispatch §2.14（reviewer dev mode） | CI 绿 ≠ runtime 过 · GUI/IPC reviewer 启 dev mode 跑 critical UX  |
| Spike 代码不进 git · 放 /tmp         | spike-delivery §3 样必交            | 代码是证据 · /tmp 3 天清 · accept 和归档同一原子动作              |
| Tauri 自定义 command 不加 permission | tauri-v2-patterns §1                | 必须加 permission toml + capability 引用 · 否则 runtime deny      |
| Tauri CSP 设 `null`                  | tauri-v2-patterns §2                | 生产用最小集 · `null` 只是初建默认 · Day 1 起设最小               |

---

## ➕ 如何新增规则

1. 判断是**通用规则**（任何项目适用）还是 **Vibestation 专项**：
   - 通用 → 加到 `~/.claude/rules/` 全局（走 Arbiter approval）
   - 专项 → 本目录新建 `.md`（走本 README 追加索引）
2. 命名规范：`kebab-case.md`（如 `git-workflow.md`）· 不用 emoji / 中文
3. 文件结构：触发场景 → 硬规则 → 反模式 → 关联 → 事件记录（仿现有 4 个规则）
4. 走 PR + Arbiter approval（即使是单人项目 v2-D.1 也要）
5. 更新本 README 索引表（追加行 · 不删已有行）

**判断流程**：

```
新规则想法
  ├── 任何 Rust/Solid/Tauri 项目都用得上？—— 是 → ~/.claude/rules/ 全局
  └── 只有 Vibestation 才需要？—— 是 → .claude/rules/ 项目级（本目录）
```

---

## 🗂 维护信息

- **当前规则数**：4 个（session 16 · 2026-04-22）
- **最后更新**：session 16 · 2026-04-22 · OpenCode
- **下次更新触发**：新增项目规则 / 现有规则重大改动 / 事件源追加
