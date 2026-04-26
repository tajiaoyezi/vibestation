<!--
  README — 开发中（未发布）
  当前版本记录仓库现状与高层定位，不展开 session 级执行细节。
-->

# Vibestation

> 给 Claude CLI / Codex CLI 用户的多 Tab 终端 + JetBrains 级 Git 工作台。
> 一个窗口管多项目，每个 Tab 一个 CLI 会话，右栏看 Git，不再为了看 commit 打开一堆 IDE。

**项目状态**：开发中 · 尚未发布可用二进制 · 终端主链已完成 `MVP-04` Phase A/B/C/E/F，Git 读链已完成 `MVP-07` + `MVP-08` Phase A/B，当前主线为 `MVP-08` Phase C（Diff 视图前端）。

---

## 仓库结构

```
vibestation/
├── LICENSE                 Apache License 2.0
├── NOTICE                  Apache 2.0 归属声明
├── README.md               本文件
├── .gitignore              Rust / Node / Tauri / OS
├── crates/
│   ├── app/                            Tauri 启动层 / IPC / permissions / capabilities
│   └── core/                           业务核心（workspace / PTY / git / diff / layout）
├── web/
│   ├── src/                            SolidJS 前端（Terminal / Git Log / Git Status）
│   └── package.json
├── docs/
│   ├── PROGRESS.md                     当前进度 / 下一步 / 滚动窗口
│   ├── SESSION-STARTUP.md              人类启动手册
│   ├── tasks/                          task spec 索引与实施规格
│   ├── adr/                            accepted ADR
│   ├── runtime-evidence/               runtime 截图 / 指标记录
│   ├── implementation-plan.md          v2 实施计划（14 章 + 附录）
│   ├── codex-review-and-response.md    Codex 独立评审与应对
│   └── tech-research.md                CodexMonitor / lapce / gitui 预研
└── design/
    ├── index.html          视觉方向总览（4 个方向）
    ├── directions/
    │   ├── 1-calm-studio.html      主风格（定稿）
    │   ├── 2-terminal-native.html
    │   ├── 3-codex-inspired.html
    │   └── 4-vscode-dense.html
    └── logos/
        ├── wordmark-a.svg
        └── mark.svg
```

## 快速查看当前设计

```bash
open design/index.html
```

打开 `design/directions/1-calm-studio.html` 可直接体验定稿方向的完整原型（双主题切换、可 toggle 工具窗口、Pane 分屏演示）。

## 规划成果（规划期交付物）

| 文档 | 内容 | 行数 |
|------|------|------|
| [`docs/implementation-plan.md`](docs/implementation-plan.md) | 产品定位 / 4 crate → 2 crate 架构 / 数据模型 / IPC / 30 风险 / 降级树 / 终端正确性矩阵 / 安全边界 / 分发运营 | 14 章 + 附录 |
| [`docs/codex-review-and-response.md`](docs/codex-review-and-response.md) | Codex 独立评审（7 CRITICAL · 12 HIGH · 5 MEDIUM · 13 强烈反对）+ Claude 元评论 + 用户 4 项决策 | 157 |
| [`docs/tech-research.md`](docs/tech-research.md) | 三项目深度预研与可借鉴点（PTY 多会话表 / AsyncLog 双速率 / logwalker 时间堆 / workspace 配置）| — |

## 当前已锁定决策

| 维度 | 值 |
|------|---|
| 许可证 | **Apache License 2.0**（无 CLA）|
| 技术栈 | **Tauri 2 + Rust + SolidJS + xterm.js** |
| 持久化 | **rusqlite 0.31+ + r2d2_sqlite** |
| Git 栈 | **git2 0.20 写 + gix 0.70 读** |
| 平台 | **macOS-first**；Ubuntu 24 为低优先级补测项；Windows 11 推到 v0.4 |
| 视觉方向 | Calm Studio（柔和 oklch + Inter + JetBrains Mono 双字体）|

## 安装（v0.1 alpha）

> **重要提示**：v0.1 alpha 版本 **未经过 Apple notarize**（推迟至 v0.2 升级）· macOS 用户首次启动需手动放行 Gatekeeper · 这是预期行为 · 不是 bug。

### macOS（Apple Silicon / Intel）

1. 从 [GitHub Releases](https://github.com/tajiaoyezi/vibestation/releases) 下载对应架构的 `.dmg`：
   - Apple Silicon：`Vibestation_0.1.0_aarch64.dmg`
   - Intel：`Vibestation_0.1.0_x64.dmg`
2. 双击 `.dmg` · 拖动 `Vibestation.app` 到 `Applications`
3. 打开 Terminal · 执行：
   ```bash
   xattr -cr /Applications/Vibestation.app
   ```
4. 双击 `Vibestation.app` 启动 · 完成

> **为什么需要这一步**：macOS Gatekeeper 默认拒绝运行未经 Apple 公证的应用 · `xattr -cr` 命令清除 `com.apple.quarantine` 扩展属性 · 让 macOS 跳过在线公证检查。命令是安全的（只影响该 app 的隔离标记 · 不修改 app 内容）。
>
> v0.2 升级 notarize 后这一步会自动免除。升级触发条件：(1) README 反馈"装不上"超 5 次 · (2) 公开 landing page 上线 · (3) macOS 用户基础超 100 任一即触发。

### Linux（Ubuntu 24 LTS）

直接下载 + 运行（无需 bypass · Linux 没有 Gatekeeper）：

#### 方式 A · `.deb`（推荐 · 系统集成 + 自动卸载）

```bash
# 下载
curl -LO https://github.com/tajiaoyezi/vibestation/releases/download/v0.1.0/Vibestation_0.1.0_amd64.deb
# 安装
sudo dpkg -i Vibestation_0.1.0_amd64.deb
# 启动（终端或应用菜单）
vibestation
# 卸载
sudo dpkg -r vibestation
```

#### 方式 B · `.AppImage`（便携 · 单文件）

```bash
curl -LO https://github.com/tajiaoyezi/vibestation/releases/download/v0.1.0/Vibestation_0.1.0_amd64.AppImage
chmod +x Vibestation_0.1.0_amd64.AppImage
./Vibestation_0.1.0_amd64.AppImage
```

X11 + Wayland 双 backend 验证通过（ADR-006 · 30 cold boot 0 fail · IME fcitx5 PASS）。

### 已知限制（v0.1 alpha）

- **macOS** unsigned · 需手动 bypass Gatekeeper（见上）· v0.2 升级 notarize
- **Windows** 暂不支持 · 推到 v0.4
- **CLI 集成深度** v0.1 仅"多 Tab 里跑 CLI"· AI-aware 联动是 v1.0 vision

## 路线图（高层）

| 里程碑 | 周 | 内容 |
|--------|----|------|
| Spike W0（已完成） | 1 周 | Tauri / PTY / Git / 存储 / CLI 实机验证与 ADR 锁定 |
| v0.1 MVP（进行中） | +12 周 | 多 Tab 终端 · Git log/status 只读 · Commit · 基础 Diff · 单层 Pane · 配置导入 · 崩溃恢复 · macOS-first 打包发布 |
| v0.2 | +5 周 | Push/Pull/Fetch · Rail graph · 分支管理 · Pane 任意嵌套 |
| v0.3 | +5 周 | Rebase/Merge/Cherry-pick · 冲突解决 · Pop to External |
| v1.0 | +6-8 周 | 高级工作流能力（范围详见 [`implementation-plan.md`](docs/implementation-plan.md)）|

**总预算**：28-30 周 × 20-25 小时 ≈ 600-750 小时（含 20% buffer）。若投入减半，触发 [`docs/implementation-plan.md#105-降级树`](docs/implementation-plan.md) 降级策略。

## 贡献

贡献流程已就绪。详见：
- [`AGENTS.md`](AGENTS.md) · 任意 agent CLI 通用入口
- [`CLAUDE.md`](CLAUDE.md) · 项目权威单文件入口（规则 / 决策 / 禁区 / 5 步 PR 流程）
- [`CONTRIBUTING.md`](CONTRIBUTING.md) · 详细贡献指南
- [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) · Contributor Covenant 2.1 中文版

**不要求贡献者签署 CLA**（Apache 2.0 本身已含 patent grant）。

## 非目标

MVP 明确不做：Windows · 云同步 · 团队协作 · 插件市场 · 远程 / SSH / devcontainer · Git worktree/submodule/LFS 的高级支持。详见实施计划 §1.4。

## 许可证

Apache License 2.0 — 详见 [`LICENSE`](LICENSE)。
