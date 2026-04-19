# SPIKE-01 · Tauri 2 三平台空壳启动 · 源码归档

对应 report：[`docs/spikes/SPIKE-01-report.md`](../../SPIKE-01-report.md)
对应 spec：[`docs/tasks/SPIKE-01-tauri-three-platform-boot.md`](../../../tasks/SPIKE-01-tauri-three-platform-boot.md)
对应决策：[`CLAUDE.md` 决策表 A 栏 #19（桌面框架 = Tauri 2 · accepted with Ubuntu caveat · session 10 末升级 · 原 B 栏 #12）](../../../../CLAUDE.md) · [ADR-006](../../../adr/ADR-006-desktop-framework.md)

## 来源

- **实施 agent**：Claude Code (Sonnet 4.6 · 2026-04-18)
- **评审人 / 实测执行**：User (Arbiter · macOS 26.3.1 M 系列)
- **原始归档**：`spike-tmp/spike-01-tauri/`（gitignored · 含完整 `node_modules/` + `target/` 共 ~975 MB · 本归档剥离后 ~512 KB）
- **本次归档**：2026-04-19 session 10 末（FU-4 · rule 13 历史欠账修复 · session 7 当时未归档）

## 结构

```
SPIKE-01/
├── README.md            # 本文件 · 归档元数据
├── .gitignore           # 原 vanilla-ts 模板 .gitignore
├── package.json · pnpm-lock.yaml   # 前端依赖（版本冻结）
├── tsconfig.json · vite.config.ts  # 前端构建配置
├── index.html
├── src/                 # 前端源码（vanilla-ts · 隔离 SolidJS 变量）
│   ├── main.ts
│   ├── styles.css
│   └── assets/
├── scripts/             # 测量脚本
│   ├── measure-boot-macos.sh    # 冷启动 N 次中位数（用 nohup + grep window_ready）
│   └── measure-boot-ubuntu.sh   # Ubuntu 同等测量（Phase B 待跑）
└── src-tauri/           # Rust 后端
    ├── Cargo.toml · Cargo.lock  # 版本冻结（rule 13 要求 binary crate 锁文件入 git）
    ├── build.rs
    ├── tauri.conf.json
    ├── capabilities/default.json   # Tauri 2 ACL（仅 core:default · 测的是骨架）
    ├── icons/                      # vanilla-ts 默认 icon 组（非项目最终 icon）
    └── src/                        # main.rs + lib.rs · window_ready 计时打点
```

排除（不进 git · 见 `.gitignore`）：
- `node_modules/`（55 MB · 用 `pnpm install` 重建）
- `src-tauri/target/`（920 MB · 用 `pnpm tauri build` 重建）
- `src-tauri/gen/`（Tauri 自动生成 schema · 首次 build 自动产出）
- `dist/`（Vite 输出 · build 产物）

## 如何复现

准备：macOS 26+ + Rust stable 1.95+ + Node 20.17 + pnpm 9.15+ + Tauri CLI 2.x

```bash
cd docs/spikes/code/SPIKE-01
pnpm install
pnpm tauri build           # release build · 产物在 src-tauri/target/release/bundle/macos/
chmod +x scripts/measure-boot-macos.sh
./scripts/measure-boot-macos.sh 10   # 跑 10 次 · 输出每次毫秒数 + median
```

预期：median ≈ 200ms · range < 100ms（参考 report §4.2 实测 202ms · 50ms range）

## 关键结论（对照 raw 数据）

- **Raw 数据归档**：[`docs/spikes/raw/SPIKE-01/`](../../raw/SPIKE-01/)（含 README 说明）
- **重要**：Phase A raw 是 **嵌入式 raw**（直接写在 report §4.2 · 10 次单值序列）· 当时未独立产出 JSON 文件 · raw 目录有 README 说明此情况
- Report §4.2 的 "Median: 202 ms" / "Range: 50 ms" 等数字来源 = `measure-boot-macos.sh` 的 stdout 抓取 + 人工统计

## 复现验证清单

- [ ] `pnpm install` 成功
- [ ] `pnpm tauri build` 成功 · 输出 `src-tauri/target/release/bundle/macos/spike-01-tauri.app`
- [ ] `./scripts/measure-boot-macos.sh 10` 中位数 < 1000ms（report 实测 202ms · 留 5× 余量）
- [ ] `.app` bundle 大小 ≈ 8 MB（report 实测 8.2 MB）
- [ ] 启动后窗口显示 "Hello Vibestation" 字样

## 注意

- 本目录是**决策依据归档** · 不直接进生产代码
- 生产 Tauri 配置由 [`crates/app/`](../../../../crates/app/)（MVP-01 Phase A 起·见 PR #28）独立维护
- Phase B (Ubuntu) 数据未补 · 待用户提供 Ubuntu 24 环境后回填到 report §6
- 归档 icon 组是 Tauri vanilla-ts 模板默认 · **不**是项目最终 icon（最终 icon 在 PR #33 落地于 `crates/app/icons/`）
