# MVP-10 Phase D · Linux AppImage 实测总结

**平台**：Ubuntu 24.04.4 LTS (Linux 6.17.0-22-generic · x86_64)
**CPU**：AMD Ryzen 7 9800X3D 8-Core Processor
**Rust**：rustc 1.95.0 (2026-04-14) · Node v22.22.2 · pnpm 9.15.9
**测量时间**：2026-04-26
**实施 agent**：Claude Code（Ubuntu 独立项目目录 · session 20 · D3 task）

## AppImage 产物

| 属性 | 值 |
|------|-----|
| 文件名 | `Vibestation_0.1.0_amd64.AppImage` |
| 路径 | `target/release/bundle/appimage/Vibestation_0.1.0_amd64.AppImage` |
| 体积 | **7.61 MB**（7,988,416 bytes） |
| 压缩 | squashfs gzip · 42.84% 压缩率（17.8 MB → 7.43 MB） |
| 类型 | ELF 64-bit LSB executable · AppImage |

## §E Acceptance

- **§E.1**：7.61 MB < 80 MB ✅（10.5× 余量）
- **§E.2**：sha256 → `478b197ac8c2ade10764026382f0d10a95111b531d130afdc9c962fa6fa602ea`（见 `02-sha256.txt`）✅
- **§E.3 X11**：AppImage 启动成功 · GUI 窗口可见 · 截图 `03-x11-startup.jpg`（1920×1080 · 135 KB）✅
- **§E.3 Wayland**：skip · 当前 session 为 X11（`XDG_SESSION_TYPE=x11`）· Ubuntu 24 GDM 登录界面可选 Wayland 但当前 session 不支持实时切换 · 推主 agent macOS 环境补测或另配 Wayland session
- **§E.4 GPG**：skip · spec 标可选 · v0.2 推荐

## --version 支持

当前 Tauri 2 构建的 binary 不支持 `--version` CLI flag（启动即打开 GUI 窗口）。这是 Tauri 2 默认行为，非 Vibestation 特有。v0.1 GA 可在 AppRun wrapper 内嵌版本号，或通过 `--help`/`--version` 解析（需改 `crates/app/src/main.rs` 加 clap/argparse）。当前不影响功能交付。

## 构建注意事项

1. **linuxdeploy 依赖**：Tauri bundler 自动下载 `linuxdeploy-x86_64.AppImage` 到 `~/.cache/tauri/`。若 FUSE 不可用，需手动 `--appimage-extract` 并 setup PATH。
2. **icon 命名**：AppImage 构建时 desktop file 的 `Icon=vibestation-app` 需与 AppDir 根目录下的 `vibestation-app.png` 匹配。当前 `tauri.conf.json` 生成的 icon 名为 `Vibestation.png`，需 symlink 补丁。
3. **deb 打包正常**：`--bundles deb` 在 Ubuntu 24 LTS 上完全通过，产物 5.5 MB。
