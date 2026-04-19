# SPIKE-02 · Tauri 2 硬通过矩阵 · 源码归档

对应 report：[`docs/spikes/SPIKE-02-report.md`](../../SPIKE-02-report.md)
对应 spec：[`docs/tasks/SPIKE-02-tauri-hard-pass-matrix.md`](../../../tasks/SPIKE-02-tauri-hard-pass-matrix.md)
对应决策：[`CLAUDE.md` 决策表 #12（桌面框架 = Tauri 2 · SPIKE-02 是 SPIKE-01 之后的硬通过验证）](../../../../CLAUDE.md)

## 来源

- **实施 agent**：Claude Code (Sonnet 4.6 · 2026-04-19)
- **评审人 / 实测执行**：User (Arbiter · macOS 26.3.1 M 系列)
- **原始归档**：`spike-tmp/spike-02-tauri/`（gitignored · 含完整 `node_modules/` + `target/` 共 ~1.1 GB · 本归档剥离后 ~528 KB）
- **本次归档**：2026-04-19 session 10 末（FU-4 · rule 13 历史欠账修复 · session 7 当时未归档）

## 与 SPIKE-01 的关系

| 维度 | SPIKE-01 | SPIKE-02 |
|---|---|---|
| 目标 | 单机三平台空壳启动通过 | 验证 Tauri 2 + 2 个 plugin（clipboard + fs）硬通过矩阵 |
| 骨架 | vanilla-ts 纯净（隔离变量） | vanilla-ts + 2 plugin |
| 测什么 | 冷启动 + IME + resize + 5min 稳定 | 10× 稳定性 + clipboard 跨 app + fs 读写 + bundle size + IME |
| 决策权重 | Tauri 整体可行性 | Tauri 2 plugin 子系统可行性 |

SPIKE-02 是 SPIKE-01 的扩展验证 · 共同支撑决策表 #12（Tauri 2 锁定 · Electron fallback 关闭）。

## 结构

```
SPIKE-02/
├── README.md
├── .gitignore
├── package.json · pnpm-lock.yaml      # 含 @tauri-apps/plugin-clipboard-manager + plugin-fs
├── tsconfig.json · vite.config.ts
├── index.html                          # 含 clipboard 读写 / fs 读写 UI
├── src/
│   ├── main.ts                         # plugin invoke 入口
│   ├── styles.css
│   └── assets/
├── scripts/
│   ├── measure-boot-macos.sh           # 同 SPIKE-01 (测 plugin 加载后冷启动)
│   ├── measure-boot-ubuntu.sh          # Phase B 待跑
│   ├── measure-10x-stability-macos.sh  # 10 次连续启动 zero crash
│   └── check-bundle-size.sh            # bundle size 自动检查
└── src-tauri/
    ├── Cargo.toml · Cargo.lock         # 含 tauri-plugin-clipboard-manager + tauri-plugin-fs
    ├── build.rs
    ├── tauri.conf.json                 # plugin 注册
    ├── capabilities/default.json       # ACL · 含 clipboard-manager:default + fs:default
    ├── icons/                          # vanilla-ts 默认（非项目最终 icon）
    └── src/                            # main.rs + lib.rs · plugin 初始化
```

排除（同 SPIKE-01 · 见 `.gitignore`）：
- `node_modules/`（64 MB · 用 `pnpm install` 重建）
- `src-tauri/target/`（1.0 GB · 用 `pnpm tauri build` 重建）
- `src-tauri/gen/` · `dist/` · `.vscode/`

## 如何复现

准备：macOS 26+ + Rust stable 1.95+ + Node 20.17 + pnpm 9.15+ + Tauri CLI 2.x

```bash
cd docs/spikes/code/SPIKE-02
pnpm install
pnpm tauri build
chmod +x scripts/*.sh

# 4 个测量维度（report §4.2-4.6）
./scripts/measure-boot-macos.sh 10        # § 冷启动
./scripts/measure-10x-stability-macos.sh  # § 10× 稳定性
./scripts/check-bundle-size.sh            # § bundle 大小
# clipboard / fs / IME 是 UI 交互测试 · 启动 .app 后人工 + 录屏验证（详见 report §4.3-4.5）
```

预期：
- 冷启动 median ≈ 212ms（report 实测）
- 10/10 稳定（连续 10 次启动零 crash）
- .app ≈ 10 MB · .dmg ≈ 4 MB

## 关键结论（对照 raw 数据）

- **Raw 数据归档**：[`docs/spikes/raw/SPIKE-02/`](../../raw/SPIKE-02/)（含 README 说明）
- **重要**：同 SPIKE-01 · Phase A raw 是嵌入式 raw（直接写在 report §4.2 / §4.6）· 未独立产出 JSON
- Report §4.2 "Min: 187ms · Median: 212ms · Max: 229ms · Range: 42ms" 来自 `measure-10x-stability-macos.sh` stdout
- Report §4.6 "bundle .app 10MB · .dmg 4MB" 来自 `check-bundle-size.sh`

## 复现验证清单

- [ ] `pnpm install` 成功（lockfile 决定的依赖版本一致）
- [ ] `pnpm tauri build` 成功
- [ ] `measure-10x-stability-macos.sh` 10/10 启动成功 · median < 1000ms（report 实测 212ms · 留 5× 余量）
- [ ] `check-bundle-size.sh` .app < 20 MB · .dmg < 10 MB（report 实测 10 MB / 4 MB · 留 2× 余量）
- [ ] clipboard 读写 UI 跨 Cmd+V 可达（含中日英 + emoji UTF-8 完整）
- [ ] fs 读写文件 UI 可在 terminal `cat` 验证

## 注意

- 本目录是**决策依据归档** · 不直接进生产
- 生产 Tauri 配置由 [`crates/app/`](../../../../crates/app/)（MVP-01 Phase A · PR #28）维护
- `clipboard-manager` / `fs` plugin 在生产代码中 **暂未启用** · 是 SPIKE-02 验证 plugin 体系本身可用 · 实际使用看 MVP-XX (Tool Window 阶段)
- 2 项降级（已记入 report）：updater 推到 SPIKE-06（依赖 Apple Dev key）· 日文 IME 全平台 skip（用户决策）
- Phase B (Ubuntu) 数据未补 · 待用户提供 Ubuntu 24 环境后回填到 report §6
