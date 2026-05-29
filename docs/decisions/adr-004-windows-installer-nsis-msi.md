# ADR `004`: Windows 安装包 NSIS 主 + MSI 辅（unsigned MVP）

**Status**: Accepted
**Date**: 2026-05-29
**Category**: 部署发布
**Related**: PRD §Decisions Log D4

## Context

`crates/app/tauri.conf.json` 的 `bundle.targets`（约行 44）当前只有 `["dmg", "appimage", "deb"]`（macOS + Linux），**无 Windows target** —— Windows 上 `tauri build` 产不出任何安装包，无法分发（survey Tauri 子系统标 blocker）。

约束与现状：

- Tauri 2 原生支持 `nsis`（.exe，bundler 内置）与 `msi`（WiX toolset）两种 Windows target。
- PRD §Constraints 发布约束：先 unsigned（对齐 macOS alpha 的 unsigned 策略，签名推 Windows GA）；保持 mac `.dmg` + Linux `.deb/.AppImage` 不动。
- PRD §Users 次要用户："小团队 IT / 部署方需要 `.msi`（适合 GPO 企业部署）或 `.exe`（NSIS，适合个人安装）"。
- `crates/app/icons/icon.ico` 已就位、`main.rs` 已设 `windows_subsystem="windows"`（survey already_windows_ok）—— bundle 前置基本就绪。
- CI 环境约束（PRD R2）：windows-latest runner 需 MSVC + WebView2 + WiX，`tauri build` 可能因缺依赖或 WiX 配置失败。

本 ADR 定 Windows 安装包格式与签名策略；CI 矩阵怎么跑由 task-5.2 / ADR 之外的 CI 决策承载。

## Decision

`tauri.conf.json` 的 `bundle.targets` **同产 `nsis`（.exe，主）+ `msi`（WiX，辅）**，先 **unsigned**：

- `nsis` 为主分发格式：体积小巧、对未签名场景友好，适合个人开发者下载安装。
- `msi`（WiX）为辅：供小团队/企业 GPO 部署。
- 两者 Tauri 都原生支持，CI 同跑（windows-latest）。
- MVP 阶段 **unsigned**（对齐 macOS alpha）；SmartScreen 警告在 Release notes 写 bypass 指引；Authenticode 签名推 Windows GA（OQ1 由 Arbiter 在 GA 前决定证书来源）。
- 平台 target 条件化：Windows job 不产 `dmg`/`appimage`（按平台跳过），保持 mac/Linux 现有 target 不变。

## Rationale

- **覆盖两类分发场景**：NSIS 小巧友好作主满足个人安装；MSI 供企业 GPO 部署 —— 两者并存覆盖 PRD 列出的两类次要用户需求。
- **原生支持、零额外栈**：Tauri 都原生支持 nsis/msi，不引入第三方打包工具，CI 同跑成本可控。
- **对齐既有 unsigned 策略**：macOS alpha 已是 unsigned 分发，Windows MVP 同策略一致，把签名复杂度（证书来源 / CI secrets）推到 GA，降低 MVP 风险面。

## Alternatives

- **(a) 仅 MSI**：拒绝 —— MSI 体积大，对未签名场景不友好（SmartScreen + MSI 双重摩擦），个人下载体验差。
- **(b) 仅 NSIS**：拒绝 —— 缺企业 GPO 部署能力，把小团队 IT / 部署方用户挡在门外。
- **(c) 两者（NSIS 主 + MSI 辅）**（**选定**）：NSIS 小巧友好作主、MSI 供企业部署，Tauri 都原生支持，CI 同跑。

## Consequences

**正面**：

- `pnpm tauri:build --bundles nsis`（及 msi）在 windows-latest 产出可安装 `.exe`/`.msi`（PRD Success Metric "三平台 CI 全绿" 的 Windows bundle 项）。
- 同时满足个人安装（NSIS）与企业 GPO 部署（MSI）。
- mac `.dmg` + Linux `.deb/.AppImage` 不受影响（平台 target 条件化）。

**负面 / 风险**：

- **unsigned → SmartScreen 警告**：用户首次运行会看到 SmartScreen 拦截（OQ1）。缓解：Release notes 写 bypass 指引；签名推 GA。
- **CI 环境复杂度**（PRD R2，概率中 / 影响高）：windows-latest 需 MSVC + WebView2 + WiX，`tauri build` 可能因缺依赖/WiX 配置失败。缓解：先 `--no-bundle` 验编译链路，再分步加 nsis、msi；用官方 tauri-action 或显式装 WiX；bundle 失败时先只 gate build+test，bundle 作单独可选 job。
- WebView2 Runtime 分发策略未定（OQ4）：默认依赖系统 evergreen，固定版本随包（体积大）待最低支持 Windows 版本确认。
- 单实例强制（OQ2）：MVP 不做 NSIS / 命名管道单实例，待用户反馈。

## Rollback Or Migration Plan

- **回滚**：从 `bundle.targets` 移除 `nsis`/`msi` 即退回无 Windows 安装包状态；不影响 mac/Linux target、不改源码逻辑、不改 DB/IPC。若 MSI(WiX) 在 CI 持续失败，可先只保留 `nsis` 主格式发布，MSI 作后续单独 job 补齐。
- **迁移**：无数据迁移。unsigned → signed 是后续 GA 的增量（加证书 + CI secrets），不影响已安装用户的应用数据（`%APPDATA%` 不变）。

## Follow-ups

- task-5.1（windows-bundle）落地 `tauri.conf.json` 的 `nsis`+`msi` target + 平台 target 条件化 + 窗口装饰条件化。
- task-5.2（windows-ci-matrix）落地 windows-latest CI：先 `--no-bundle` 验编译，再分步加 bundle（PRD R2 缓解）。
- OQ1：Authenticode 证书来源 —— Arbiter 在 Windows GA 前决定。
- OQ2：单实例强制（NSIS / 命名管道）—— 待用户反馈。
- OQ4：WebView2 随包 vs 系统 evergreen —— 待最低支持 Windows 版本确认。
- Release notes 模板补 SmartScreen bypass 指引（unsigned MVP）。
