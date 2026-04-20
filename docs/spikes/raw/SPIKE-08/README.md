# SPIKE-08 · Raw Data 索引

> 所有结论都必须能从本目录回溯。`docs/spikes/SPIKE-08-report.md` 的数字、pass/fail 判定、CI 推荐都以这里为证据源。

## §A Contract Layer

- `contract-export.log`
  - baseline `cargo build`/bindings 生成日志。
- `h2-contract-regression-build.log`
  - Rust 字段 rename 后重新 `cargo build` 的成功日志。
- `h2-contract-regression.log`
  - 重新 codegen 后 `pnpm typecheck` 的真实 FAIL log。
- `ts-rs-cargo-info.txt`
  - `ts-rs` crate 元数据（版本/feature）。
- `tauri-specta-cargo-info.txt`
  - `tauri-specta` crate 元数据（版本/feature，当前为 `2.0.0-rc.24`）。
- `ts-rs-repo.json`
  - `Aleph-Alpha/ts-rs` GitHub 仓库维护态快照。
- `tauri-specta-repo.json`
  - `specta-rs/tauri-specta` GitHub 仓库维护态快照。
- `specta-repo.json`
  - `specta-rs/specta` GitHub 仓库维护态快照。
- `ts-rs-cargo-tree.txt` / `ts-rs-cargo-tree.lines`
  - 最小 Tauri 2 + `ts-rs` sample 的依赖树与行数。
- `tauri-specta-cargo-tree.txt` / `tauri-specta-cargo-tree.lines`
  - 最小 Tauri 2 + `tauri-specta` sample 的依赖树与行数。

## §B Runtime Layer

- `playwright-browser-smoke.log`
  - B.2 baseline：`Playwright + Vite dev server` golden path PASS。
- `playwright-browser-trace.zip`
  - B.2 baseline trace。
- `playwright-browser-final.png`
  - B.2 baseline 成功截图。
- `playwright-browser-timing.log`
  - B.2 单次耗时（`/usr/bin/time -p`）。
- `playwright-browser-10x.log`
  - B.2 10 连跑 flake 统计。
- `vite-browser.log`
  - baseline Vite dev server 输出。
- `vite-browser-10x.log`
  - 10 连跑期间的 Vite 输出。
- `h2-runtime-regression.log`
  - 前端旧 key 取值后，browser E2E 的真实 FAIL log。
- `h2-runtime-regression-trace.zip`
  - runtime regression trace。
- `h2-runtime-regression.png`
  - runtime regression 失败截图。
- `tauri-playwright-dev.log`
  - B.3 `cargo tauri dev --features e2e-testing` 输出。
- `tauri-playwright-smoke.log`
  - B.3 社区 recipe smoke 的 FAIL log。
- `tauri-playwright-results/`
  - B.3 Playwright 失败附件（screenshot + trace + error context）。
- `tauri-playwright-npm-meta.json`
  - `@srsholmes/tauri-playwright` npm 发布时间/version 元数据。
- `linux/`（等待 GitHub Actions run `24653654459` 结束后补齐）
  - 已被 GitHub artifact 两次下载替代，见下方。

## GitHub Runner / CI

- `linux-gh-run.log`
  - GitHub Actions run `24653654459` 完整日志（第一次 B.1 fail：缺 `WebKitWebDriver`）。
- `linux-gh-artifact/`
  - run `24653654459` 上传的 raw artifact（`tauri-driver.log` / `xvfb.log` / `tauri-build-linux.log`）。
- `linux-gh-run-rerun.log`
  - GitHub Actions run `24653923822` 完整日志（补装 `webkit2gtk-driver` 后再次 fail）。
- `linux-gh-artifact-rerun/`
  - run `24653923822` 上传的 raw artifact；第二次已经进入 `tauri-driver`，但报 `Connection refused`。
