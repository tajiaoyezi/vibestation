[English](./QUICKSTART.en.md) | 中文

# Vibestation 快速上手

## 引言 / 适用对象

Vibestation 是一款为 CLI agent 用户打造的多 Tab 终端 + JetBrains 级 Git 工作台，基于 Tauri 2 + Rust 构建，提供原生桌面体验。本指南将帮助你在 5–10 分钟内完成安装并跑通核心流程。

本指南适合以下用户：

- 日常使用 Claude CLI、Codex CLI 等命令行工具的开发者
- 需要在一个窗口内管理多个终端会话和项目的用户
- 想要在终端内直接查看 Git log、status 和 diff 的工程师
- 对 Tauri / Rust 桌面应用感兴趣的早期尝鲜者

## 系统要求

| 平台   | 最低要求                                                            |
| ------ | ------------------------------------------------------------------- |
| macOS  | macOS 13 (Ventura) 及以上 · Apple Silicon (arm64) 或 Intel (x86_64) |
| Ubuntu | Ubuntu 24.04 (24 LTS) 及以上 · x86_64                               |

> Windows 支持计划在 v0.4 提供，当前仅支持 macOS 和 Ubuntu 24 LTS。

## 安装

### macOS · DMG

1. 从 [GitHub Releases](https://github.com/tajiaoyezi/vibestation/releases) 下载最新 `.dmg` 文件
2. 双击打开 DMG，将 Vibestation 拖入 Applications 文件夹
3. 首次启动前，在终端执行以下命令移除隔离属性：

```bash
xattr -cr /Applications/Vibestation.app
```

> **为什么需要这一步？** v0.1 版本未经 Apple 公证，macOS Gatekeeper 会阻止未签名应用的启动。执行 `xattr -cr` 可移除隔离标记，让系统放行。后续版本将完成公证流程，届时此步骤将不再需要。

### Ubuntu · deb

1. 从 [GitHub Releases](https://github.com/tajiaoyezi/vibestation/releases) 下载 `.deb` 包
2. 安装：

```bash
sudo dpkg -i Vibestation_0.1.0_amd64.deb
```

3. 卸载（如需）：

```bash
sudo dpkg -r vibestation
```

或使用 apt：

```bash
sudo apt remove vibestation
```

### Ubuntu · AppImage

1. 从 [GitHub Releases](https://github.com/tajiaoyezi/vibestation/releases) 下载 `.AppImage` 文件
2. 添加执行权限：

```bash
chmod +x Vibestation_0.1.0_amd64.AppImage
```

3. 运行：

```bash
./Vibestation_0.1.0_amd64.AppImage
```

> AppImage 无需安装，直接运行即可。适合不想通过包管理器安装的用户。

## 首次启动核对清单

启动 Vibestation 后，请逐项确认：

- [ ] 应用窗口正常显示，无崩溃或白屏
- [ ] 左侧侧栏可见，显示工作区导航
- [ ] 底部终端区域已加载默认 Shell（如 bash / zsh）
- [ ] 顶部标题栏显示当前项目路径
- [ ] 快捷键 `Cmd+T`（macOS）/ `Ctrl+T`（Ubuntu）可创建新 Tab

## 上手 5 步

### 步骤 1 · 打开应用

启动 Vibestation 后，你将看到默认的深色布局：左侧侧栏展示工作区导航，底部是终端区域，右侧可展开 Git 视图。

![Vibestation 默认深色布局，主侧栏展开](assets/onboarding/hero/01-default-layout-dark.jpg)

> 首次打开时，Vibestation 会自动检测当前目录并加载对应的 Shell 配置。如果显示异常，请检查「系统要求」是否满足。

### 步骤 2 · 创建 Tab + 跑 CLI

使用快捷键 `Cmd+T`（macOS）或 `Ctrl+T`（Ubuntu）创建新终端 Tab。每个 Tab 拥有独立的 Shell 会话，你可以在不同 Tab 中分别运行 Claude CLI、Codex CLI 或其他命令行工具。

![创建多个终端 Tab](assets/onboarding/terminal/01-multi-tab-create.png)

> 你也可以通过侧栏的「+」按钮或右键菜单创建 Tab。每个 Tab 可独立设置工作目录。

### 步骤 3 · 看 Git log

点击侧栏的 Git 图标或使用快捷键打开 Git 视图，即可浏览当前仓库的提交历史。每条 commit 显示作者、时间、消息摘要。

![Git 提交详情视图](assets/onboarding/git/01-commit-detail-loaded.jpg)

> Git 视图会自动检测当前 Tab 工作目录下的 Git 仓库。如果未显示，请确认目录包含 `.git`。

### 步骤 4 · 查看 Diff

在 Git 视图中点击某条 commit，右侧将展示该提交的文件变更详情，包含新增 / 删除 / 修改行的叠加视图。

![Diff 叠加视图](assets/onboarding/git/02-diff-overlay-opened.jpg)

> Diff 视图支持语法高亮，可直观查看代码变更。后续版本将支持暂存区 diff 和行内 diff。

### 步骤 5 · 切换主题

Vibestation 支持深色和浅色主题。通过设置面板或快捷键 `Cmd+,`（macOS）/ `Ctrl+,`（Ubuntu）打开设置，在「外观」选项中切换。

![切换深色主题](assets/onboarding/theme/01-theme-switch-dark.png)

> 主题切换即时生效，无需重启应用。你也可以跟随系统主题自动切换。

## 常见问题

### 启动时 macOS 提示「无法打开应用」

这是 Gatekeeper 对未公证应用的拦截。在终端执行 `xattr -cr /Applications/Vibestation.app` 即可放行。参见安装章节中 macOS · DMG 的详细说明。

### Ubuntu 上 AppImage 双击无反应

AppImage 需要执行权限。在终端运行 `chmod +x Vibestation_0.1.0_amd64.AppImage`，然后通过 `./Vibestation_0.1.0_amd64.AppImage` 启动。

### Git 视图显示为空

请确认当前 Tab 的工作目录是一个 Git 仓库（包含 `.git` 目录）。Vibestation 会自动检测 Git 仓库并展示提交历史。

## 下一步

- 🤝 参与贡献：阅读 [CONTRIBUTING.md](../CONTRIBUTING.md)
- 📊 了解开发进度：查看 [docs/PROGRESS.md](./PROGRESS.md)
- 🗺️ 查看产品路线图：跳转 [README.md 路线图](../README.md#路线图)
