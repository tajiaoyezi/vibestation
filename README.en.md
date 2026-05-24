[中文](README.md) | English

<div align="center">
  <img src="design/logos/wordmark-a.svg" alt="Vibestation" width="280" />
</div>

<p align="center">
  <strong>Multi-tab terminal + JetBrains-grade Git workbench for CLI agent users · Tauri 2 native</strong>
</p>

<p align="center">
  <img alt="status" src="https://img.shields.io/badge/status-alpha-orange" />
  <img alt="license" src="https://img.shields.io/badge/license-Apache--2.0-blue" />
  <img alt="platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey" />
  <img alt="tauri" src="https://img.shields.io/badge/built%20with-Tauri%202-24C8DB" />
</p>

---

Vibestation is a desktop application built on **Tauri 2 + Rust**, designed for developers who use Claude CLI / Codex CLI and similar agent tools. It unifies multi-session terminals, Git log / status / diff views, and cross-project management into a single native window — reducing the cognitive overhead of switching between an IDE and a terminal.

Current release: **v0.1 alpha** (macOS / Ubuntu), under active development.

## Installation

### macOS

Download the `.dmg` from [GitHub Releases](https://github.com/tajiaoyezi/vibestation/releases), drag to Applications, then run:

```bash
xattr -cr /Applications/Vibestation.app
```

> v0.1 is not Apple-notarized, so Gatekeeper must be bypassed manually. v0.2 will add notarization and this step will no longer be needed.

### Ubuntu

```bash
# deb package (recommended)
sudo dpkg -i Vibestation_0.1.0_amd64.deb

# Or AppImage (portable, no install)
chmod +x Vibestation_0.1.0_amd64.AppImage
./Vibestation_0.1.0_amd64.AppImage
```

For detailed installation steps and troubleshooting, see the [Quickstart Guide](docs/QUICKSTART.en.md).

## Roadmap

| Milestone      | Scope                                                                                         |
| -------------- | --------------------------------------------------------------------------------------------- |
| **v0.1** alpha | Multi-tab terminal · Git read-only · Commit · Basic Diff · Single-layer Pane · Crash recovery |
| **v0.2**       | Push/Pull/Fetch · Rail graph · Branch management · Arbitrary Pane nesting                     |
| **v0.3**       | Rebase/Merge/Cherry-pick · Conflict resolution · Pop to External                              |
| **v1.0**       | Advanced workflow capabilities · See [`implementation-plan.md`](docs/implementation-plan.md)  |

## Documentation

| Topic                          | Link                                                       |
| ------------------------------ | ---------------------------------------------------------- |
| Quickstart (5 minutes)         | [docs/QUICKSTART.en.md](docs/QUICKSTART.en.md)             |
| Project overview · Repo layout | [docs/PROJECT-OVERVIEW.md](docs/PROJECT-OVERVIEW.md)       |
| Implementation plan · Roadmap  | [docs/implementation-plan.md](docs/implementation-plan.md) |
| Architecture Decision Records  | [docs/adr/](docs/adr/)                                     |
| Task index                     | [docs/tasks/](docs/tasks/)                                 |
| Progress snapshot              | [docs/PROGRESS.md](docs/PROGRESS.md)                       |
| Contributing guide             | [CONTRIBUTING.md](CONTRIBUTING.md)                         |
| Code of Conduct                | [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)                   |

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md). **Apache 2.0 · No CLA** required.

## License

Apache License 2.0 · see [LICENSE](LICENSE).
