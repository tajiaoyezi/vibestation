<div align="right">

[English](README.en.md) · **中文**

</div>

<div align="center">
  <img src="design/logos/wordmark-a.svg" alt="Vibestation" width="320" />

  <p>
    <strong>🌊 为 CLI agent 用户打造的多 Tab 终端 + JetBrains 级 Git 工作台</strong>
    <br />
    <sub>Tauri 2 原生 · macOS / Ubuntu · Apache 2.0 · 无 CLA</sub>
  </p>

  <p>
    <a href="https://github.com/tajiaoyezi/vibestation/releases"><img alt="status" src="https://img.shields.io/badge/status-v0.1%20alpha-orange?style=flat-square" /></a>
    <a href="LICENSE"><img alt="license" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" /></a>
    <img alt="platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey?style=flat-square" />
    <img alt="tauri" src="https://img.shields.io/badge/built%20with-Tauri%202-24C8DB?style=flat-square&logo=tauri" />
    <a href="https://github.com/tajiaoyezi/vibestation/stargazers"><img alt="stars" src="https://img.shields.io/github/stars/tajiaoyezi/vibestation?style=flat-square&logo=github" /></a>
    <a href="https://github.com/tajiaoyezi/vibestation/commits/main"><img alt="last commit" src="https://img.shields.io/github/last-commit/tajiaoyezi/vibestation?style=flat-square" /></a>
  </p>

  <p>
    <a href="docs/QUICKSTART.md"><strong>5 分钟上手 →</strong></a>
    &nbsp;·&nbsp;
    <a href="docs/PROJECT-OVERVIEW.md">项目概览</a>
    &nbsp;·&nbsp;
    <a href="CONTRIBUTING.md">参与贡献</a>
  </p>
</div>

---

Vibestation 是一款基于 **Tauri 2 + Rust** 构建的桌面应用，专为使用 Claude CLI / Codex CLI 等 agent 工具的开发者打造。它把多会话终端、Git log / status / diff 视图、跨项目管理整合到一个原生窗口里，减少在 IDE 和终端之间来回切换的认知成本。

当前版本 **v0.1 alpha** · macOS / Ubuntu 双平台 · 开发活跃中 🚧

<br />

## 📦 安装

<details open>
<summary><strong>🍎 macOS</strong></summary>

<br />

从 [GitHub Releases](https://github.com/tajiaoyezi/vibestation/releases) 下载 `.dmg`，拖到 Applications 后执行：

```bash
xattr -cr /Applications/Vibestation.app
```

> ⚠️ v0.1 未经 Apple notarize，需手动放行 Gatekeeper。v0.2 升级 notarize 后自动免除。

</details>

<details>
<summary><strong>🐧 Ubuntu</strong></summary>

<br />

```bash
# deb 包（推荐）
sudo dpkg -i Vibestation_0.1.0_amd64.deb

# 或 AppImage（便携，免安装）
chmod +x Vibestation_0.1.0_amd64.AppImage
./Vibestation_0.1.0_amd64.AppImage
```

</details>

📖 详细安装步骤与常见问题见 [**快速上手指南 →**](docs/QUICKSTART.md)

<br />

## 🗺️ 路线图

| 里程碑      | 主线能力                                                                      |
| :---------- | :---------------------------------------------------------------------------- |
| 🌱 **v0.1** | 多 Tab 终端 · Git 只读 · Commit · 基础 Diff · 单层 Pane · 崩溃恢复            |
| 🌿 **v0.2** | Push / Pull / Fetch · Rail graph · 分支管理 · Pane 任意嵌套                   |
| 🌳 **v0.3** | Rebase / Merge / Cherry-pick · 冲突解决 · Pop to External                     |
| 🚀 **v1.0** | 高级工作流能力 · 详见 [`implementation-plan.md`](docs/implementation-plan.md) |

<br />

## 📚 文档导航

| 我想…                    | 看这里                                                       |
| :----------------------- | :----------------------------------------------------------- |
| ⚡ 5 分钟跑起来          | [`docs/QUICKSTART.md`](docs/QUICKSTART.md)                   |
| 🏗️ 了解架构 / 仓库结构   | [`docs/PROJECT-OVERVIEW.md`](docs/PROJECT-OVERVIEW.md)       |
| 🗺️ 实施计划 / 路线图细节 | [`docs/implementation-plan.md`](docs/implementation-plan.md) |
| 📐 架构决策记录 (ADR)    | [`docs/adr/`](docs/adr/)                                     |
| ✅ 任务索引              | [`docs/tasks/`](docs/tasks/)                                 |
| 📊 当前进度快照          | [`docs/PROGRESS.md`](docs/PROGRESS.md)                       |
| 🤝 贡献指南              | [`CONTRIBUTING.md`](CONTRIBUTING.md)                         |
| 🏛️ 行为准则              | [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)                   |

<br />

## 🤝 贡献

欢迎贡献 —— 无论是代码、文档、bug 报告还是功能建议。

✨ **Apache 2.0 · 无 CLA**，提交无需签署任何额外协议。
🤖 **不绑定 agent 工具**：Claude Code / Codex CLI / Cursor / Aider / 人类 / 自建 agent 均可参与。

详见 [**CONTRIBUTING.md →**](CONTRIBUTING.md)

<br />

## 📄 许可证

[Apache License 2.0](LICENSE) · Copyright © 2026 Vibestation Contributors

<div align="center">
  <br />
  <sub>用 ❤️ 与 🦀 打造 · 为 CLI agent 用户而生</sub>
</div>
