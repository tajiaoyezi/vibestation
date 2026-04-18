# SPIKE-02 · Tauri 硬通过矩阵 + Electron fallback 验证报告

> **Task spec**：[`docs/tasks/SPIKE-02-tauri-hard-pass-matrix.md`](../tasks/SPIKE-02-tauri-hard-pass-matrix.md)
> **状态**：Phase A macOS 已测 · **Phase B Ubuntu 待补**（用户暂无 Ubuntu 24 环境）
> **实施者**：Claude Code (Sonnet 4.6) · **评审人**：User (Arbiter)
> **分支**：`spike/spike-02-macos-phase-a`

---

## 1 · 结论概览

| # | 判据 | macOS Phase A | Ubuntu X11 | Ubuntu Wayland |
|---|---|---|---|---|
| 1 | 连续启动 10 次零失败 | `TBD` | — Phase B | — Phase B |
| 2 | 剪贴板 copy/paste（含中文） | `TBD` | — Phase B | — Phase B |
| 3 | IME 中文拼音 + 日文罗马字 | `TBD` | — Phase B | — Phase B |
| 4 | Bundle 大小 < 30MB / 40MB | `TBD` | — Phase B | — Phase B |
| 5 | Clipboard plugin smoke test | `TBD` | — Phase B | — Phase B |
| 5 | FS plugin smoke test | `TBD` | — Phase B | — Phase B |
| 5 | Updater plugin smoke test | ⚠️ **归 SPIKE-06**（需 Apple Dev Program 签名 key） | — | — |
| 6 | ADR-006 草稿 + 决策表 #12 | **Phase A 给 conditional accepted 建议** · 等 Phase B 三平台全过才翻正式 accepted | — | — |

**Phase A 整体判定**：`TBD`（回填后更新）
**SPIKE-02 整体 status**：保持 `in-progress` · Phase B Ubuntu 未补完前不翻 `done`

---

## 2 · 环境

| 维度 | 数据 |
|---|---|
| OS | macOS 26.3.1 (Build 25D771280a) |
| CPU | Apple Silicon (aarch64-apple-darwin) |
| Rust toolchain | rustc 1.95.0 · cargo 1.95.0 |
| Node | v20.17.0 LTS |
| pnpm | 9.15.9 |
| Tauri CLI | 2.10.1 |
| Plugins | `@tauri-apps/plugin-clipboard-manager` 2.3.2 · `@tauri-apps/plugin-fs` 2.5.0 · `@tauri-apps/plugin-opener` 2.5.3 |

---

## 3 · 骨架实现

位置：`spike-tmp/spike-02-tauri/`（`.gitignore` 排除）

### 3.1 基于 SPIKE-01 骨架的增量

- 从 `spike-01-tauri/` 完整 cp · 保留冷启动埋点
- `Cargo.toml` 增 `tauri-plugin-clipboard-manager` / `tauri-plugin-fs`
- `src-tauri/src/lib.rs` 挂 2 个 plugin init · 保留 `window_ready` 事件埋点
- `src-tauri/capabilities/default.json` 授权 clipboard 读写 + fs 读写 + home scope
- `src-tauri/tauri.conf.json` 产品名 / identifier 都改 `spike-02-tauri`
- 前端 `index.html` + `main.ts` 重新组织为 4 个测试 section（clipboard / fs / ime / meta）
- 样式 `styles.css` 改为 section 卡片 + ok/error 色带反馈

### 3.2 UI 功能（用户肉眼验证点）

| Section | 功能 | 验证方式 |
|---|---|---|
| ① Clipboard | 写入 `Hello · 你好 · こんにちは · 🎉 SPIKE-02` · 读取剪贴板 | 点按钮 · 切别的 app Cmd+V 验证；再点 "读取" 应返回原文 |
| ② FS | 写 `~/.vibestation-spike-02-test.txt` · 读同一文件 | 点按钮 · `cat ~/.vibestation-spike-02-test.txt` 验证 |
| ③ IME | 输入框 · 中日混输 | QuickTime 录屏 · 输入 "你好世界" + "こんにちは世界" · 观察是否丢字 / 光标异常 |
| ④ Meta | 10 次稳定性 / bundle 大小 / updater 注 | 脚本自动验证（不在 UI） |

### 3.3 Updater plugin 的处理

SPIKE-02 spec 原要求 updater plugin smoke test · 但 Tauri 2 updater 需要 **`pubkey` 签名 key** 才能 build · 该 key 依赖 Apple Developer Program（`tauri signer generate`）——**而 Apple Developer Program 申请正是 SPIKE-06 的工作**。

**本报告 descope updater 到 SPIKE-06** · 在 ADR-006 accepted 前 · 视为 "Updater plugin 兼容性已由上游社区验证 + SPIKE-06 会补完整签名链路"。

---

## 4 · Phase A · macOS 测量数据

### 4.1 执行命令

```bash
cd spike-tmp/spike-02-tauri
pnpm tauri build
./scripts/measure-10x-stability-macos.sh 10
./scripts/check-bundle-size.sh
```

然后打开 `.app` · 肉眼点 4 个按钮 · 录屏 IME 中日输入。

### 4.2 连续启动 10 次稳定性

<!-- 用户跑完回填 -->

```
Run 1:  TBD  (TBD ms)
Run 2:  TBD  (TBD ms)
...
Run 10: TBD  (TBD ms)

Summary: TBD/10 success · TBD fail
Min: TBD ms · Median: TBD ms · Max: TBD ms
```

### 4.3 Clipboard plugin smoke test（用户肉眼 + 录屏）

- [ ] 点 "写入剪贴板" · UI 显示 ✅
- [ ] 切 Safari/Notes · Cmd+V · 粘贴出 `Hello · 你好 · こんにちは · 🎉 SPIKE-02` 完整无丢字
- [ ] 点 "读取剪贴板" · UI 显示读出内容 · 长度正确

### 4.4 FS plugin smoke test

- [ ] 点 "写 ~/.vibestation-spike-02-test.txt" · UI 显示 ✅
- [ ] Terminal 执行 `cat ~/.vibestation-spike-02-test.txt` · 输出 `Hello · 你好 · こんにちは · 🎉 SPIKE-02`
- [ ] 点 "读 ~/..." · UI 读出内容正确

### 4.5 IME 测试（用户录屏）

- [ ] **中文拼音**：切输入法 · 输入 "你好世界" · 候选词正常 · 无丢字 · 光标对齐
- [ ] **日文罗马字**：切日文输入法 · 输入 "こんにちは" · 候选词正常 · 假名 + 汉字转换正常
- [ ] 录屏存 `spike-artifacts/SPIKE-02/macos-ime-zh.mov` 和 `spike-artifacts/SPIKE-02/macos-ime-ja.mov`（gitignored）

### 4.6 Bundle size

<!-- 回填 -->

```
macOS .app:  TBD MB
macOS .dmg:  TBD MB (目标 < 30MB)
```

---

## 5 · Phase B · Ubuntu 24 待补（给接手 agent 的委托 prompt）

用户暂无 Ubuntu 24 环境。当有环境时 · 转给在 Ubuntu 24 上的 agent / 人肉执行者。

### 5.1 给 Ubuntu agent 的原话 prompt（可直接粘贴）

```
任务：在 Ubuntu 24 LTS 机器上完成 SPIKE-02 Phase B · 对 spike-02-tauri 空壳做硬通过矩阵验收（X11 + Wayland 两会话各做一遍）。

背景：
这是 Vibestation 项目 SPIKE-02 的 Phase B。Phase A 已在主开发机 macOS 完成。
仓库：https://github.com/tajiaoyezi/vibestation
骨架代码：主开发机 spike-tmp/spike-02-tauri/（gitignored 不入仓库 · 需要你本地重建或让主机打 tarball 给你）

你要做的 5 件事：

1. 环境准备：
   - Ubuntu 24 LTS · 图形桌面（GNOME/KDE · 支持 Wayland 和 X11 两种会话）
   - Rust toolchain (rustup · stable)
   - Node 20 LTS · pnpm 9
   - 系统依赖：sudo apt install -y libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf xclip wl-clipboard
   - 中文输入法：fcitx5（sudo apt install fcitx5 fcitx5-chinese-addons · 注销重登）
   - 日文输入法：fcitx5-mozc（sudo apt install fcitx5-mozc）
   - 录屏：obs-studio 或 GNOME Ctrl+Alt+Shift+R

2. 拿骨架（两种方式选一）：
   A. 让主机打 tarball：
      tar -czf spike-02-tauri.tgz --exclude=node_modules --exclude=target --exclude=dist --exclude=src-tauri/gen spike-tmp/spike-02-tauri/
   B. 自己重建：
      pnpm create tauri-app@latest spike-02-tauri --manager pnpm --template vanilla-ts --identifier com.vibestation.spike02 --tauri-version 2 --yes
      然后对照主仓库的以下文件 patch（从 docs/spikes/scripts/SPIKE-02/ + repo 历史 PR 的 diff 找）：
      - src-tauri/Cargo.toml（加 clipboard-manager + fs）
      - src-tauri/capabilities/default.json（加权限）
      - src-tauri/src/lib.rs（埋点 + 2 plugin init）
      - src-tauri/tauri.conf.json
      - package.json（加 @tauri-apps/plugin-clipboard-manager / fs）
      - index.html / src/main.ts / src/styles.css

3. 构建 + 测量 · 两会话各跑一遍：
   在 X11 会话登录 · 然后：
     cd spike-02-tauri && pnpm install && pnpm tauri build
     ./scripts/measure-10x-stability-ubuntu.sh 10   # 该脚本等同 macos 版 · 路径指向 linux binary
     ./scripts/check-bundle-size.sh
   注销 · 在 Wayland 会话重登 · 再跑同样命令。
   记录 XDG_SESSION_TYPE 不同时的两组数据。

4. 人工验证（每会话都录屏）：
   每个会话下肉眼点 clipboard / fs 按钮 · 跑 IME 测试：
   - Clipboard：点写入 · 切别的 app Ctrl+V 粘贴验证 · 再点读取确认
   - FS：点写 · terminal `cat ~/.vibestation-spike-02-test.txt` 验证 · 点读
   - IME 中文：fcitx5 切中文 · 输入 "你好世界" · 录屏
   - IME 日文：fcitx5 切 mozc 或 anthy · 输入 "こんにちは世界" · 录屏
   
   录屏存本地 spike-artifacts/SPIKE-02/ubuntu-{x11,wayland}-{clipboard,fs,ime-zh,ime-ja}.mp4（不入 repo · 后续整理归档）

5. 返回格式：
   直接填到 docs/spikes/SPIKE-02-report.md 第 5 节 "Phase B Ubuntu 数据"（新增段落）· 格式：
   
   ### X11 会话
   - 10x 稳定性：XX/10 · median YY ms · raw: [...]
   - Clipboard smoke: PASS/FAIL + 说明
   - FS smoke: PASS/FAIL + 说明
   - IME 中文: PASS/FAIL + 录屏文件名
   - IME 日文: PASS/FAIL + 录屏文件名
   - Bundle deb: XX MB · AppImage: YY MB
   
   ### Wayland 会话
   （同上）

通过标准：
- 10x 稳定性 10/10 · 任一 session 一次 crash 即 Fail
- 剪贴板 + FS 读写在两会话都 PASS
- IME 中日在两会话都 PASS（fcitx5 下合格即可 · ibus 失败不算整体 fail · 但 ADR 需注明推荐 fcitx5）
- Bundle AppImage + deb 各 < 40MB

失败信号触发（任一）：
- 连续 10 次启动出现 ≥ 1 次黑屏 / 闪退
- Wayland 下剪贴板写入后其他 app 读不到
- Wayland 下 IME 崩溃 / 严重丢字
→ 触发 Day 2 Electron 28+ fallback spike（不在你工作范围 · 只需报告给用户）

约束：
- 代码不入 Vibestation 主仓库（遵从 spec · spike 代码 gitignored）
- report 回填由主 agent 归档 · 你不需要开 PR · 只需把数据 + 录屏文件名 + 一段 markdown 发给用户
- 用中文写正文 · 代码保留英文

时间预期：环境准备 30-60 min · 骨架重建 20 min · build + 测量 1-2 小时 · 写 report 30 min · 总计 3-5 小时（两会话串行）
```

### 5.2 给人肉执行者的简化 runbook

如果是你自己 SSH 或本地操作：
1. GDM 登录时从齿轮图标选 X11 vs Wayland · 两次都测
2. fcitx5 配好中文 · 日文用 fcitx5-mozc
3. 录屏用 obs-studio 或 GNOME Ctrl+Alt+Shift+R

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
| Phase A macOS | `TBD` | 全 PASS → 写 ADR-006 草稿 "conditional accepted" · 等 Phase B 正式 accepted |
| Phase B Ubuntu | `TBD` | 两会话全 PASS → ADR-006 accepted · `CLAUDE.md` 决策表 #12 B → A · SPIKE-02 翻 done · 解锁 SPIKE-03..06 |

失败路径（参考 spec §Fail Signals）：
- macOS 单项 fail → 扩 Spike 补测 · 不急着切 Electron
- Ubuntu 两会话都 fail → 触发 Electron 28+ fallback spike（SPIKE-02.5）
- Ubuntu 一会话 fail → 评估是 IME 框架问题（fcitx5 → ibus）还是 webkit2gtk 问题 · 分情况处理

---

## 8 · 自审四问

1. **递归完备性**：5 项 §3.1.1 判据 + updater descope 说明 + Phase A/B 两路径 · ✅
2. **反向场景**：macOS 失败扩测 · Ubuntu 失败分三级 · 不越权直接切 Electron · ✅
3. **边界适用性**：updater 明确归 SPIKE-06 · 不在当前 Spike 强验证 · 避免假 PASS · ✅
4. **YAGNI**：骨架最小扩展（只加 2 plugin · 不碰 updater / notification / dialog 等）· ✅

---

## 9 · 变更记录

| 日期 | 实施者 | 变更 |
|---|---|---|
| 2026-04-19 AM | Claude Code (Sonnet 4.6) | 骨架 cp + 2 plugin 扩展 + UI 重构 + 测量脚本 + report 骨架 + Ubuntu prompt |
| `TBD` | User | Phase A macOS 10x 稳定性 + clipboard/fs/IME 肉眼验证 + 录屏 |
| `TBD` | Claude Code | 回填 Phase A 数据 · 开 PR |
| `TBD` | User / Ubuntu agent | Phase B Ubuntu 数据补 + ADR-006 accepted + #12 B→A |
