[中文](README.md) | English

**alpha** · **Apache 2.0** · **macOS / Linux**

# Vibestation

Multi-tab terminal + JetBrains-grade Git workbench for CLI agent users · Built with Tauri

## Representative Screenshots

![Vibestation default dark layout with sidebar open](docs/assets/onboarding/hero/01-default-layout-dark.jpg)

![Vibestation all panels open dark view](docs/assets/onboarding/hero/02-all-panels-open-dark.jpg)

## Why Vibestation

- **Multi-tab Terminal** — Create multiple terminal tabs in one window, each with an independent CLI session, compatible with Claude CLI / Codex CLI and similar tools
- **Workbench-grade Git** — Built-in Git log / status / diff views — no need to switch to an IDE to check commits
- **Cross-project Management** — Manage multiple projects in a single window, with each tab mapped to a different project directory
- **Tauri Native Experience** — Built on Tauri 2 + Rust, macOS cold start < 200ms, low memory footprint
- **Apache 2.0 · No CLA** — Contributor-friendly open source license, no CLA required

## Current Status & Version

Vibestation is in **v0.1 alpha** stage, under development, and not yet released as a stable binary. Multi-tab terminal and Git read-only views are functional; more features are actively being developed.

## Installation

### macOS

Download the `.dmg` from [GitHub Releases](https://github.com/tajiaoyezi/vibestation/releases), drag to Applications, then run:

```bash
xattr -cr /Applications/Vibestation.app
```

> v0.1 is not Apple-notarized, so you need to manually bypass Gatekeeper. v0.2 will add notarization and this step will no longer be needed.

### Ubuntu

**deb package (recommended)**:

```bash
sudo dpkg -i Vibestation_0.1.0_amd64.deb
```

**AppImage (portable)**:

```bash
chmod +x Vibestation_0.1.0_amd64.AppImage
./Vibestation_0.1.0_amd64.AppImage
```

For detailed installation steps and troubleshooting, see the [Quickstart Guide](docs/QUICKSTART.en.md).

## Screenshot Gallery

### Terminal

![Creating multiple terminal tabs](docs/assets/onboarding/terminal/01-multi-tab-create.png)

![Switching terminal tabs](docs/assets/onboarding/terminal/02-tab-switch.png)

### Git

![Git commit detail view](docs/assets/onboarding/git/01-commit-detail-loaded.jpg)

![Diff overlay view](docs/assets/onboarding/git/02-diff-overlay-opened.jpg)

### Theme & Platform

![Light theme view](docs/assets/onboarding/theme/02-light-theme.jpg)

![Ubuntu AppImage launch screen](docs/assets/onboarding/platform/01-ubuntu-appimage-launch.png)

## Roadmap

| Milestone | Scope                                                                                                                                          |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| **v0.1**  | Multi-tab terminal · Git log/status read-only · Commit · Basic Diff · Single-layer Pane · Config import · Crash recovery · macOS-first release |
| **v0.2**  | Push/Pull/Fetch · Rail graph · Branch management · Arbitrary Pane nesting                                                                      |
| **v0.3**  | Rebase/Merge/Cherry-pick · Conflict resolution · Pop to External                                                                               |
| **v1.0**  | Advanced workflow capabilities · See [`implementation-plan.md`](docs/implementation-plan.md)                                                   |

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Apache License 2.0 — No CLA required. See [LICENSE](LICENSE) for details.

## Learn More

For developer-oriented repository structure, planning outcomes, locked decisions, and non-goals, see [docs/PROJECT-OVERVIEW.md](docs/PROJECT-OVERVIEW.md).
