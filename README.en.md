<div align="right">

**English** · [中文](README.md)

</div>

<div align="center">
  <img src="design/logos/wordmark-a.svg" alt="Vibestation" width="320" />

  <p>
    <strong>🌊 Multi-tab terminal + JetBrains-grade Git workbench for CLI agent users</strong>
    <br />
    <sub>Tauri 2 native · macOS / Ubuntu · Apache 2.0 · No CLA</sub>
  </p>

  <p>
    <a href="https://tajiaoyezi.github.io/vibestation/"><img alt="website" src="https://img.shields.io/badge/🌐_website-vibestation-7c3aed?style=flat-square" /></a>
    <a href="https://github.com/tajiaoyezi/vibestation/releases"><img alt="status" src="https://img.shields.io/badge/status-v0.1%20alpha-orange?style=flat-square" /></a>
    <a href="LICENSE"><img alt="license" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" /></a>
    <img alt="platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey?style=flat-square" />
    <img alt="tauri" src="https://img.shields.io/badge/built%20with-Tauri%202-24C8DB?style=flat-square&logo=tauri" />
    <a href="https://github.com/tajiaoyezi/vibestation/stargazers"><img alt="stars" src="https://img.shields.io/github/stars/tajiaoyezi/vibestation?style=flat-square&logo=github" /></a>
    <a href="https://github.com/tajiaoyezi/vibestation/commits/main"><img alt="last commit" src="https://img.shields.io/github/last-commit/tajiaoyezi/vibestation?style=flat-square" /></a>
  </p>

  <p>
    <a href="https://tajiaoyezi.github.io/vibestation/"><strong>🌐 Visit Website</strong></a>
    &nbsp;·&nbsp;
    <a href="docs/QUICKSTART.en.md"><strong>⚡ Quickstart in 5 min</strong></a>
    &nbsp;·&nbsp;
    <a href="docs/PROJECT-OVERVIEW.md">Project overview</a>
    &nbsp;·&nbsp;
    <a href="CONTRIBUTING.md">Contribute</a>
  </p>
</div>

---

Vibestation is a desktop application built on **Tauri 2 + Rust**, designed for developers who use Claude CLI / Codex CLI and similar agent tools. It unifies multi-session terminals, Git log / status / diff views, and cross-project management into a single native window — reducing the cognitive overhead of switching between an IDE and a terminal.

Current release **v0.1 alpha** · macOS / Ubuntu · under active development 🚧

<br />

## 📦 Installation

<details open>
<summary><strong>🍎 macOS</strong></summary>

<br />

Download the `.dmg` from [GitHub Releases](https://github.com/tajiaoyezi/vibestation/releases), drag to Applications, then run:

```bash
xattr -cr /Applications/Vibestation.app
```

> ⚠️ v0.1 is not Apple-notarized, so Gatekeeper must be bypassed manually. v0.2 will add notarization and this step will no longer be needed.

</details>

<details>
<summary><strong>🐧 Ubuntu</strong></summary>

<br />

```bash
# deb package (recommended)
sudo dpkg -i Vibestation_0.1.0_amd64.deb

# Or AppImage (portable, no install)
chmod +x Vibestation_0.1.0_amd64.AppImage
./Vibestation_0.1.0_amd64.AppImage
```

</details>

📖 Detailed steps & troubleshooting: [**Quickstart Guide →**](docs/QUICKSTART.en.md)

<br />

## 🗺️ Roadmap

| Milestone   | Scope                                                                                         |
| :---------- | :-------------------------------------------------------------------------------------------- |
| 🌱 **v0.1** | Multi-tab terminal · Git read-only · Commit · Basic Diff · Single-layer Pane · Crash recovery |
| 🌿 **v0.2** | Push / Pull / Fetch · Rail graph · Branch management · Arbitrary Pane nesting                 |
| 🌳 **v0.3** | Rebase / Merge / Cherry-pick · Conflict resolution · Pop to External                          |
| 🚀 **v1.0** | Advanced workflow capabilities · see [`implementation-plan.md`](docs/implementation-plan.md)  |

<br />

## 📚 Documentation

| I want to…                          | Link                                                                          |
| :---------------------------------- | :---------------------------------------------------------------------------- |
| 🌐 Visit the website                | [tajiaoyezi.github.io/vibestation](https://tajiaoyezi.github.io/vibestation/) |
| ⚡ Get running in 5 minutes         | [`docs/QUICKSTART.en.md`](docs/QUICKSTART.en.md)                              |
| 🏗️ Understand architecture / layout | [`docs/PROJECT-OVERVIEW.md`](docs/PROJECT-OVERVIEW.md)                        |
| 🗺️ Implementation plan / roadmap    | [`docs/implementation-plan.md`](docs/implementation-plan.md)                  |
| 📐 Architecture Decision Records    | [`docs/adr/`](docs/adr/)                                                      |
| ✅ Task index                       | [`docs/tasks/`](docs/tasks/)                                                  |
| 📊 Current progress snapshot        | [`docs/PROGRESS.md`](docs/PROGRESS.md)                                        |
| 🤝 Contributing guide               | [`CONTRIBUTING.md`](CONTRIBUTING.md)                                          |
| 🏛️ Code of Conduct                  | [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)                                    |

<br />

## 🤝 Contributing

Contributions are welcome — code, docs, bug reports, feature ideas, all of it.

✨ **Apache 2.0 · No CLA** required — submit without signing any extra agreement.
🤖 **Tool-agnostic**: Claude Code / Codex CLI / Cursor / Aider / humans / your own agent are all welcome.

See [**CONTRIBUTING.md →**](CONTRIBUTING.md)

<br />

## 📄 License

[Apache License 2.0](LICENSE) · Copyright © 2026 Vibestation Contributors

<div align="center">
  <br />
  <sub>Built with ❤️ and 🦀 · for CLI agent users</sub>
</div>
