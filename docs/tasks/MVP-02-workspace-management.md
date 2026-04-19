---
id: MVP-02
type: mvp
title: Workspace 管理 + 项目识别 + 多 workspace 并存
status: done
owner: OpenCode
phase: W2-W3
depends_on: ["MVP-01"]
blocks: ["MVP-03", "MVP-04", "MVP-07"]
blocked_by: []
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §3.1
risk_ref:
reviewer: Claude Code (Opus 4.7 · session 10 · Arbiter Option C · 2026-04-19)
---

# MVP-02: Workspace 管理 + 项目识别

> **状态**：`done`（OpenCode 实施 PR #40 · Arbiter Claude Code Opus 4.7 · session 10 · Option C 2026-04-19 · H2 camelCase fix + FU-1 截图 PR #47）
> **依赖**：MVP-01（Tauri 骨架）/ **阻塞**：MVP-03（Tool Windows 显示 workspace 列表）· MVP-04（Tab 属于 workspace）· MVP-07（Git Log 读 workspace 下 repo）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md)

---

## 🎯 目标（Goal）

实现 workspace 创建 / 选择目录 / 自动识别 git 仓库 / 多 workspace 并存的能力，为所有后续功能提供容器。

## 📖 背景（Context）

- Workspace = Vibestation 的顶级组织单位，对应"一个项目"或"一组相关项目"
- 用户从欢迎页（MVP-01）点 "Create first workspace" → 进入本 spec 流程
- 多 workspace 并存是 `implementation-plan.md §10.1` 必做项（"多 workspace 同时打开，Tab 切换状态独立"）
- 自动识别 git 仓库 = 判断目录是否含 `.git` 子目录（顶级或父级递归向上查找）

---

## 🎨 功能范围（Scope）

**Do**：
- 创建 workspace：选择本地目录（系统文件对话框）→ 输入 workspace 名称（默认取目录名）→ 保存到 rusqlite
- 自动识别 git 仓库：目录含 `.git` → 标记为 `has_git: true` 并记录 repo root 路径
- Workspace 列表：显示所有已创建的 workspace（名称 + 路径 + git 标记）
- 打开 workspace：选中后进入 workspace 主视图（空白占位，MVP-03 接管布局）
- 多 workspace 并存：可以同时打开多个 workspace，通过 Tab 或 sidebar 切换
- 关闭 workspace：仅从当前 session 关闭，不删除记录
- 删除 workspace：永久从 rusqlite 移除（需二次确认对话框）

**Don't**（推后或不做）：
- Workspace 分组 / 标签（v0.2+）
- Workspace 内的分支树显示（→ MVP-03 Primary Sidebar）
- 跨 workspace 搜索（v0.2+）
- Workspace 导入/导出（v0.2+）

## 🖼 UI 引用（UI Reference）

- Workspace 列表：`design/directions/1-calm-studio.html` Primary Sidebar 顶部区域（workspace switcher）
- 创建对话框：Calm Studio 风格的模态对话框（浅灰背景 + 圆角 + 主色 CTA）
- 空状态：若无 workspace，欢迎页显示"Create first workspace"（继承 MVP-01）

## ✅ Acceptance

### A. 创建 workspace

- [ ] 从欢迎页或已有 workspace 的菜单触发"Create workspace"
- [ ] 打开系统文件对话框（`tauri-plugin-dialog`），限制为"选择目录"
- [ ] 目录选中后：自动填充 workspace 名称为目录名（可编辑）
- [ ] 确认创建 → workspace 写入 rusqlite `workspaces` table
- [ ] UUID v4 作为 `workspace_id`（由 core crate 生成）
- [ ] 创建后立即打开该 workspace

### B. 自动识别 git 仓库

- [ ] 创建或打开 workspace 时，扫描目录是否含 `.git`
- [ ] `.git` 存在 → `has_git: true`，记录 `repo_root = path/to/workspace`
- [ ] `.git` 不存在 → 向上递归最多 5 层，找到则 `has_git: true` + `repo_root = 父目录`
- [ ] 递归 5 层无 `.git` → `has_git: false`
- [ ] Workspace 列表 UI 上，`has_git: true` 显示 git 图标徽章

### C. 多 workspace 并存

- [ ] 同时打开 ≥ 2 个 workspace
- [ ] Primary Sidebar 顶部显示 workspace switcher（已打开的列表）
- [ ] 切换 workspace 时，Tab 状态（MVP-04）/ Git 视图状态（MVP-07）独立保留
- [ ] 关闭 workspace（从 session 移除）不影响其他 workspace

### D. 持久化

- [ ] 所有 workspace 元数据写入 rusqlite `workspaces` table（schema 见下方）
- [ ] 应用退出 → 重启后，打开的 workspace 列表 + 顺序恢复（继承 MVP-01 崩溃恢复机制）
- [ ] 已关闭但未删除的 workspace 仍在 rusqlite 里，欢迎页可以"打开最近 workspace"

### E. 删除 workspace

- [ ] Workspace 菜单 → "Delete workspace"
- [ ] 二次确认对话框："确定删除 X？（文件不会删，仅从 Vibestation 移除）"
- [ ] 确认后从 rusqlite 移除
- [ ] 若是最后一个 workspace → 回到欢迎页

### F. 边界情况

- [ ] 选择不存在或无权限的目录 → 明确错误提示
- [ ] 路径含中文/空格 → 正确处理（UTF-8）
- [ ] 重复添加同一目录 → 提示"该 workspace 已存在，要打开它吗？"

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元（core）| workspace 元数据 CRUD + git 识别逻辑 | `cargo test -p vibestation-core workspace` |
| 集成 | rusqlite 读写 + Tauri IPC 对话框 | `cargo test --features integration` |
| E2E | 完整流程：创建 → 打开 → 切换 → 删除 | Playwright 脚本 |
| 手动 QA | 中文路径 / 无权限目录 / 网络盘 | §8.3 QA 清单 |

## 💾 数据模型变更

扩展 MVP-01 建立的 `workspaces` table：

```rust
struct WorkspaceMetadata {
    workspace_id: String,       // UUID v4
    name: String,
    path: String,                // 绝对路径
    has_git: bool,
    repo_root: Option<String>,   // 若 has_git=true
    created_at: i64,             // Unix timestamp
    last_opened: i64,
}
```

`schema_version = 2`（从 MVP-01 的 1 升级，需 migration 脚本）。

## ⚠️ 已知风险

- **网络挂载盘性能**：NFS / SMB 挂载目录访问慢 → 扫描 `.git` 需 timeout
- **路径规范化**：macOS / Linux 路径大小写敏感性不同 → core 统一用 canonical path
- **权限变化**：workspace 创建后路径变为无权限 → 打开时 graceful fail，标记状态

## 📝 Notes / 讨论

- Workspace 不绑定到 git repo（可以是非 git 目录，如 docs 项目）；但 git 功能（MVP-07..09）只在 `has_git: true` 时启用
- MVP-02 不做"workspace 内 project 嵌套"——一个 workspace 一个 project（v0.2 再讨论 monorepo）

## 🔗 相关

- 上游：MVP-01
- 下游：MVP-03（sidebar 显示 workspace）· MVP-04（Tab 归属 workspace）· MVP-07（Git 读 `repo_root`）
- `implementation-plan.md` 章节：§10.1

---

**自审四问**：
1. 递归完备性：CRUD + 边界 + 持久化全覆盖 ✅
2. 反向场景：目录权限变化 / 网络盘慢 都有 graceful 处理 ✅
3. 边界适用性：中文路径 / 大小写敏感 显式测试 ✅
4. YAGNI：不做 monorepo / 分组 / 导入导出（v0.2+）✅
