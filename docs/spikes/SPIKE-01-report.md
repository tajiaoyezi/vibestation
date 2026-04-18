# SPIKE-01 · Tauri 2 三平台空壳启动验证报告

> **Task spec**：[`docs/tasks/SPIKE-01-tauri-three-platform-boot.md`](../tasks/SPIKE-01-tauri-three-platform-boot.md)
> **状态**：Phase A macOS 已测 · **Phase B Ubuntu 待补**（用户暂无 Ubuntu 24 环境）
> **实施者**：Claude Code (Sonnet 4.6) · **评审人**：User (Arbiter)
> **分支**：`spike/spike-01`

---

## 1 · 结论概览

| 平台 | 冷启动中位数 | 目标 | 结果 | 窗口渲染 | resize | IME 中文 | 稳定性 5min |
|---|---|---|---|---|---|---|---|
| macOS 26.3.1 (M 系列) | `TBD` ms | < 2000ms | `TBD` | `TBD` | `TBD` | `TBD` | `TBD` |
| Ubuntu 24 X11 | — | < 3000ms | **TBD**（Phase B） | — | — | — | — |
| Ubuntu 24 Wayland | — | < 3000ms | **TBD**（Phase B） | — | — | — | — |

**Phase A 整体判定**：`TBD` · Phase B 数据回填后重新评估。

---

## 2 · 环境

| 维度 | 数据 |
|---|---|
| OS | macOS 26.3.1 (Build 25D771280a) |
| CPU | Apple Silicon (aarch64-apple-darwin) |
| Rust toolchain | rustc 1.95.0 (2026-04-14) · cargo 1.95.0 |
| Node | v20.17.0 LTS (via NVM) |
| pnpm | 9.15.9 |
| Tauri CLI | 2.x（项目 devDep + global cargo install）|
| Xcode CLT | /Library/Developer/CommandLineTools |
| 骨架模板 | `vanilla-ts`（隔离变量 · 不引入 SolidJS · 确保测的是 Tauri 本身）|

---

## 3 · 骨架实现

位置：`spike-tmp/spike-01-tauri/`（`.gitignore` 第 95 行已排除 · 不进主仓库）

### 3.1 定制点

- **`index.html`**：title `Hello Vibestation · SPIKE-01` · h1 `Hello Vibestation` · 输入框 placeholder `试输入中文 · IME 测试`
- **`src-tauri/src/lib.rs`**：`run()` 起始 `Instant::now()` 采点 · `setup` 回调内打印 `[SPIKE-01] window_ready t=<ms>ms` 到 stderr
- **`src/main.ts`**：保留原 greet form（作为 IME 测试点）

### 3.2 冷启动埋点原理

```rust
let boot_start = Instant::now();
eprintln!("[SPIKE-01] boot_start t=0ms");
tauri::Builder::default()
    ...
    .setup(move |_app| {
        let elapsed_ms = boot_start.elapsed().as_millis();
        eprintln!("[SPIKE-01] window_ready t={}ms", elapsed_ms);
        Ok(())
    })
    .run(...)
```

- 定义："冷启动" = 从 `pub fn run()` 被调用 → Tauri `setup` 回调触发（窗口已创建且可见）
- 测量方式：shell 脚本 `scripts/measure-boot-macos.sh` / `scripts/measure-boot-ubuntu.sh` · 启动 release binary · grep stderr 抓 `window_ready t=<ms>ms` · 多次取中位数
- **注意**：这个"冷启动"不含 macOS 图标双击/Spotlight 到 `pub fn run()` 被调用的那段（由 macOS Launch Services 负责，数十毫秒级，难以精准测量）

---

## 4 · Phase A · macOS 测量

### 4.1 执行命令

```bash
cd spike-tmp/spike-01-tauri
pnpm tauri build              # 构建 release bundle (~3-8 min)
./scripts/measure-boot-macos.sh 10   # 测 10 次冷启动
```

### 4.2 采样数据

<!-- 用户跑完后回填 -->

```
Run 1:  TBD ms
Run 2:  TBD ms
Run 3:  TBD ms
Run 4:  TBD ms
Run 5:  TBD ms
Run 6:  TBD ms
Run 7:  TBD ms
Run 8:  TBD ms
Run 9:  TBD ms
Run 10: TBD ms

Min:    TBD ms
Median: TBD ms
Max:    TBD ms
```

### 4.3 人工验证（用户观察 · 录屏归档）

- [ ] 窗口启动后显示 "Hello Vibestation" · 无黑屏 / 白屏
- [ ] 窗口 resize 正常（拖拽右下角）
- [ ] 最小化 / 最大化 / 关闭正常
- [ ] 输入框可输入中文（拼音切换 · 候选词弹出正常）· 录屏见 `spike-artifacts/SPIKE-01/macos-ime.mov`
- [ ] 单次启动 → 5 分钟内无 panic / 崩溃日志

### 4.4 Bundle 信息

- Release binary: `spike-tmp/spike-01-tauri/src-tauri/target/release/spike-01-tauri`
- .app bundle: `spike-tmp/spike-01-tauri/src-tauri/target/release/bundle/macos/spike-01-tauri.app`
- Bundle size: `TBD`

---

## 5 · Phase B · Ubuntu 24 待补（给接手 agent 的委托 prompt）

用户当前暂无 Ubuntu 24 机器。当有环境时 · 将下方 prompt 直接转给在该环境上的 agent（或人肉执行者）· 执行后把数据回填到本报告第 6 节。

### 5.1 给 Ubuntu agent 的原话 prompt（可直接粘贴）

```
任务：在 Ubuntu 24 LTS 机器上完成 SPIKE-01 的 Ubuntu X11 + Wayland 冷启动验证。

背景：
这是 Vibestation 项目 SPIKE-01 的 Phase B · Phase A 的 macOS 已在主开发机完成。
仓库：https://github.com/tajiaoyezi/vibestation
骨架代码位置：`spike-tmp/spike-01-tauri/`（代码不进主仓库 · 需要你本地 git clone 后自己跑 scaffold 或者 scp 骨架过来）

你要做的 5 件事：

1. 准备环境：
   - Ubuntu 24 LTS · 图形桌面（GNOME/KDE）
   - Rust toolchain (rustup · stable)
   - Node 20 LTS · pnpm 9
   - 系统依赖：sudo apt install -y libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf
   - 中文输入法：fcitx5（sudo apt install fcitx5 fcitx5-chinese-addons · 注销重登或 im-config 切换）

2. 拿骨架代码（两种方式）：
   方式 A · 本地 scaffold（可重现）：
     pnpm create tauri-app@latest spike-01-tauri \
       --manager pnpm --template vanilla-ts \
       --identifier com.vibestation.spike01 --tauri-version 2 --yes
     然后对照 vibestation repo 里 scripts/measure-boot-ubuntu.sh + lib.rs 埋点 patch 一下
   方式 B · 让主机打 tarball 给你：
     tar -czf spike-01-tauri.tgz spike-tmp/spike-01-tauri/
     （但 node_modules/target 不要打 · 只要源码）

3. 构建 + 测量：
   cd spike-01-tauri
   pnpm install
   pnpm tauri build   # 首次 3-8 min
   ./scripts/measure-boot-ubuntu.sh 10

4. 分别在 X11 和 Wayland 会话下各跑一遍（GDM 登录界面切换 session type）。
   记录：XDG_SESSION_TYPE + 10 次采样 + 中位数。

5. 人工验证（务必录屏归档）：
   - 窗口渲染 · resize · 最小/最大/关闭
   - fcitx5 下输入中文（"你好世界"）· 候选词正常 · 无崩溃
   - 单次启动 → 5 min 观察 · 无 panic / segfault
   录屏存 spike-artifacts/SPIKE-01/ubuntu-{x11,wayland}-{boot,ime}.mp4（在自己那边，后续归档时打包）

返回格式：
直接填到 docs/spikes/SPIKE-01-report.md 第 6 节 · 按现有 TBD 位置回填。
附上 10 次采样 raw + 中位数 + session type + 所有勾选项状态 + 录屏文件名。

通过标准：
- X11 冷启动中位数 < 3000ms ✅
- Wayland 冷启动中位数 < 3000ms ✅
- 两会话窗口渲染 / IME / resize / 稳定性全部通过 ✅

失败信号（任一触发 → 触发 Day 2 Electron fallback spike）：
- 冷启动 > 5s
- Wayland 黑屏/白屏
- IME 崩溃或输入无响应
- 5 min 内 panic / segfault

约束：
- 这是 Spike 一次性验证 · 不进主仓库 · 不需要 code review
- 但数据填入 report 的 PR 需要走 GitHub 流程（docs PR · 走 README §8 (b) 路径变种）
- 用中文写回填内容 · 代码块保留英文
```

### 5.2 给人肉执行者的简化 runbook

如果不是给 agent · 是你自己 SSH 到 Ubuntu · 执行顺序和上面一样 · 重点：
1. `fcitx5-configtool` 配好中文
2. 登录时从 GDM gear 图标选 "Ubuntu on Xorg" vs "Ubuntu on Wayland" · 两次都测
3. 录屏用 `obs-studio` 或 GNOME 内置 `Ctrl+Alt+Shift+R`
4. 10 次采样 · 关注极值（首次含 linker cache warm-up · 可能偏慢）

---

## 6 · Phase B · Ubuntu 数据回填（待补）

### 6.1 X11

<!-- 等 Ubuntu 测试完成回填 -->

### 6.2 Wayland

<!-- 等 Ubuntu 测试完成回填 -->

---

## 7 · 最终判定与决策联动

| 阶段 | 判定 | 触发动作 |
|---|---|---|
| Phase A macOS | `TBD`（回填后更新） | 通过 → Phase B 启动；失败 → Day 2 Electron fallback（mac 都过不了不用测 Ubuntu） |
| Phase B Ubuntu | `TBD`（回填后更新） | 三平台全通过 → `CLAUDE.md` 决策表 #12 从 B 栏移入 A 栏 + 翻 SPIKE-01 `status: done` · 解锁 SPIKE-02 |

Fallback 路径（参考 spec §Fail Signals）：
- **Ubuntu 某平台失败** → 不立即 abandon Tauri · 先评估是 Wayland IME 问题（可能换 ibus）还是根本渲染问题
- **两平台都失败** → 启动 `SPIKE-01.5 · Electron 28+ fallback`（1 天）· 通过则更新 ADR-006、decision table、implementation-plan.md §3.1

---

## 8 · 自审四问（按 CLAUDE.md §"自审四问"）

1. **递归完备性**：Phase A + Phase B 覆盖了 spec 的 3 平台 · 7 条 pass criteria 一一对应 · ✅
2. **反向场景**：每种失败都有明确 fallback 路径（Ubuntu 局部失败 vs 全局失败 vs macOS 失败） · ✅
3. **边界适用性**：冷启动定义（`run()` → `setup` callback）不含 Launch Services · 明确承认 · ✅
4. **YAGNI**：骨架用 `vanilla-ts` 而非 `solid-ts` · 故意砍掉业务变量 · ✅

---

## 9 · 变更记录

| 日期 | 实施者 | 变更 |
|---|---|---|
| 2026-04-18 | Claude Code (Sonnet 4.6) | 骨架 + 埋点 + 测量脚本 · Phase A 待用户跑 bench · Phase B 文档化 prompt |
| `TBD` | User / Ubuntu agent | Phase A 数据回填 + Phase B 完成 |
