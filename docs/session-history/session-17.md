# Session 17 · 2026-04-23

**session**: 17  
**date**: 2026-04-23  
**pr_range**: #99-#105  
**theme**: MVP-04 Phase F 收口 + MVP-08 Phase A/B/C 落地 + PR Actions 分钟节流

---

## PR 摘要

- **PR #105 `471349f`**（**MVP-08 Phase C Diff 视图前端集成 done · 主线里程碑**）· Codex CLI · 4 路并行成果整合：OpenCode core `diff_set_view_mode` + Kimi 隔离的 `web/src/panels/Diff/`（DiffPanel 303 行 + diffApi + styles 299 行）+ Git Status 自动刷新 subscription/polling 接通 + shell PATH 修复（PTY 启动）· `crates/app/permissions/diff.toml` 加 `diff_set_view_mode` permission · `App.tsx` + `MainContent.tsx` 加主区 Diff/Terminal 切换 state（Terminal 不重挂）· Git Status 文件行 + Git Log 文件行 → 点击打开 Diff 主区 · supersede PR #104（孤立 Diff 面板分支）· Implemented + Reviewed by Codex CLI self-review · Arbiter approval 2026-04-23 22:07 CST
- **PR #103 `fb2d755`**（**入口文档对齐 main 真实进度**）· Codex CLI · 在 PR #100/#101 落地后同步 PROGRESS / CLAUDE / 入口指针 · 防止下次 agent 误读
- **PR #102 `d1c5489`**（**PR 级 GitHub Actions 自动运行关闭 · 只保留 `push main` + `workflow_dispatch`**）· Codex CLI · `.github/workflows/ci.yml` / `secret-scan.yml` / `task-spec-validator.yml` 去掉 `pull_request` 触发 · `secret-scan` 删除 `pull-requests: read` 权限 · `task-spec-validator` 保留脚本内 PR 分支逻辑便于未来恢复 · 结果：新 PR 不再消耗 GitHub Actions 分钟数，但后续 agent 必须本地先跑 gate，merge 后再回看 `main` 的 check runs
- **PR #101 `e09b9df`**（**MVP-08 Phase B Git Status Bottom Panel done**）· Codex CLI · Bottom Panel 真正承载 Git Status 只读面板 · 3 分组 `Staged / Unstaged / Untracked` + 状态码 / 相对路径 / 加减行数 + `Refresh` · 非 git workspace / 错误态有明确提示 · 新增 `GitStatusPanelSettings` / `GitStatusCollapseRequest` / `GitStatusGroup` + `git_status_get_settings` / `git_status_set_group_collapsed` · 分组折叠态写入 `app_settings` · 后续 Phase C 直接消费稳定面板状态 contract
- **PR #100 `424e894`**（**MVP-08 Phase A diff/status IPC 后端 done**）· Codex CLI · `crates/core/src/diff.rs` + `git_status.rs` 落地 · `similar` 负责文本 diff 计算 · `gix` 读取 commit / parent blob · `git2` 读取 staged / unstaged 对照源 · 6 个 Tauri commands + 8 个 ts-rs bindings + ACL/permission 补齐 · `DiffRequest.allowLargeFile` + `DiffResponse.truncatedReason/lineCount` 预留 large-file confirm 与 hard stop 协议 · `git_status_subscribe` / `git_status_unsubscribe` 先占位给 Phase D
- **PR #99 `a0c9699`**（**MVP-04 Phase F runtime 证据与量化 done**）· Codex CLI · `docs/runtime-evidence/mvp-04/` 新增 5 张截图 + `metrics-phase-f.md` · create / rename / switch / close / scrollback 证据补齐 · A.5 / E.2 切 Tab latency AX 自动化 median `20 ms` · E.4 页面内同步 JS 执行 `sync max = 3 ms` · `frame delta = 19 ms` 仅作上下文 · MVP-04 整体仍保持 `ready`，只剩 Phase D shell 兼容

## 特色

- **MVP-08 主线里程碑**：Diff 视图前端集成是 v0.1 Git 能力闭环的关键拼图 · 4 路 agent 并行成果首次完整整合
- **GitHub Actions 分钟节流**：PR 级自动运行关闭 · 从"CI 自动跑"转向"本地 gate + merge 后核对"模式 · 应对 billing 压力
- **MVP-04 Phase F 收官**：runtime 证据补齐 · 只剩 Phase D shell 兼容（低优先 follow-up）

---

← 当前进度见 [docs/PROGRESS.md](../PROGRESS.md)
