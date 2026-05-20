# MVP-05 Phase D · Pre-Capture 就绪体检

> **定位**：本文件是 MVP-05 [`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md)（PR #211 · 4 轮 codex review · 14 invariant）的**前置体检**——主 agent（CLI）能程序化验证的 Phase D 代码侧前置已全跑，结论固化在此。
> **它不是 capture 本身**：playbook 的 6 张 PNG + 30s 录屏 + F.1 内存量 + F.2/F.3 DevTools 拖拽 60FPS + F.4-F.6 webview console performance.now log 设计上就是 Arbiter 本人通过 `pnpm tauri:dev` 实跑 + `bash scripts/capture/mvp-05/capture-phase-d.sh` 自动化 capture + DevTools 实测，CLI agent 无法替代，也不得编造。
> **用途**：Arbiter 跑 Phase D capture 窗口时，先读本文件 —— 代码侧已 green 的 cargo/vitest 不必重复跑；聚焦真正需要人的 6 张截图 + 30s 录屏 + DevTools 拖拽 60FPS 验证。
>
> 体检执行：Claude Code 主 agent · 2026-05-20 · 镜像 MVP-19 #384 模式

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| 项                                          | 验证方式                                                                        | 结果                                                                                                                                                                                     |
| ------------------------------------------- | ------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **panes 模块单元测试**                      | `cargo test -p vibestation-core --lib 'panes::tests::'`                         | **63 passed · 0 failed · 0 ignored**（含 split_layout / close_pane_in_layout / update_split_ratio / apply_smart_layout 4 pure functions + envelope 序列化 + smart layout preset 全覆盖） |
| **pane_service 集成测试**                   | `cargo test -p vibestation-core --lib 'pane_service::tests::'`                  | **23 passed · 0 failed**（含 pane_init idempotent · split creates pane · split_ratio rejects invalid · close last pane errors · layout preset solo/ai_runner · legacy migration）        |
| **pane_layout_bench Criterion**             | `crates/core/benches/pane_layout_bench.rs` 已存在 · 7 micro-bench 48-210 ns     | playbook §F 仪表化已 done · Arbiter 可跑 `cargo bench --bench pane_layout_bench` 取实测 P99                                                                                              |
| **Phase A/B/C 全 9 PR done**                | git log + spec §I.0                                                             | A storage prep · B Step 1+2 panes pure + IPC + 17 unit + 7 bench · C 前端 scaffolding + 拖拽 60FPS + SmartLayoutMenu wire + §F 仪表化                                                    |
| **§F 仪表化 inline 完成**（capture 自动化） | `ls scripts/capture/mvp-05/` + `docs/runtime-evidence/mvp-05/metrics-mvp-05.md` | `capture-phase-d.sh` + `measure-memory.sh` + metrics-mvp-05.md 测量手册全在 · F.4-F.6 console.info('[mvp-05 perf] ...') inline 已就位                                                    |

### Phase C 仪表化代码侧详情（playbook §F 前置 · Arbiter 实跑可信任）

| 指标                       | 仪表化位置                                         | Arbiter 实跑方法                                                |
| -------------------------- | -------------------------------------------------- | --------------------------------------------------------------- |
| F.1 内存量                 | `bash scripts/capture/mvp-05/measure-memory.sh`    | 实测 4 Pane 内存 vs 1 Pane baseline                             |
| F.2 拖拽 60FPS             | DevTools Performance 手测                          | 拖 splitter 30s · Performance flamegraph 验证 frame rate 60FPS  |
| F.3 SmartLayout 切换流畅度 | DevTools Performance + console.info                | 切 preset 3 次 · 看 webview console `[mvp-05 perf] preset` 数字 |
| F.4 onPointerMove rAF 节流 | webview console.info inline + DevTools Performance | `[mvp-05 perf] rAF` 数字 + Performance 验 16ms throttle         |
| F.5 close 焦点切换         | webview console.info                               | console 看 `[mvp-05 perf] focus_after_close` 数字（pure JS）    |
| F.6 split → render         | webview console.info                               | console 看 `[mvp-05 perf] split_render` 数字（pure JS）         |

---

## ⚠️ 关键 gap 预警

### gap-1 · 6 张 PNG + 30s 录屏 + F.1 实测数字全部待 Arbiter capture

**坐实**：`ls docs/runtime-evidence/mvp-05/` = 仅 `CAPTURE-PLAYBOOK.md` + `metrics-mvp-05.md`（**0 张截图 / 0 段录屏 / metrics-mvp-05.md 数字全空白**）。

**影响**：spec §I.0 Phase D 翻 done 判据 = 6 张 PNG + 30s 录屏 + F.1-F.6 实测数字 · 当前 0/3 项到位。

**不是 gap 是 deferred**：这是 spec **明确设计**的 deferred capture（playbook 14 invariant 全聚焦 GUI / DevTools / 内存量 · CLI agent 无 webview / 无 Performance flamegraph 能力 · 不能替代）。需 Arbiter 启 Phase D capture 窗口（预计 30 min · capture-phase-d.sh 自动化 6 PNG + screencapture -V 30 录屏 + DevTools Performance 5-10 min）。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 [`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md) 14 invariant + spec §F 性能验收：

1. **`pnpm tauri:dev`** 启动应用 · 准备 workspace 含 ≥4 Pane 的 fixture（spec §F 测试矩阵 100 Pane 不 mandate · 4 Pane = §F.5 close 焦点切换最小验证集）
2. **6 张 PNG**（playbook §1）：覆盖 split 操作 / SmartLayout preset / 拖拽 splitter / close pane / focus 切换 / Quad layout
3. **30s 录屏**（playbook §2）：键盘 + 鼠标拖拽完整流程 · 用 `bash scripts/capture/mvp-05/capture-phase-d.sh` 或 `screencapture -V 30 -x <wid>`
4. **F.1 内存量实测**（playbook §3）：`bash scripts/capture/mvp-05/measure-memory.sh` · 测 4 Pane 内存 vs 1 Pane baseline · 填 `metrics-mvp-05.md`
5. **F.2-F.3 DevTools Performance**（playbook §4）：拖拽 splitter 30s + 切 SmartLayout preset 3 次 · 验 frame rate 60FPS · 填实测数字
6. **F.4-F.6 webview console**（playbook §5）：直接看 console.info `[mvp-05 perf]` log 数字 · pure JS 自动产数 · 复制到 metrics-mvp-05.md
7. **PR + R1-R5**：`docs/runtime-evidence/mvp-05/01-*.png` ... `06-*.png` 顺序前缀 · `rollback-flow.mov` · 单文件 ≤ 500KB · 总目录 ≤ 3 MB

### 14 invariant 关键提醒（playbook §0）

- I3 / I4：split 截图必伴随 `pane list` 终端输出（验真 4 pane 而非 mock）
- I5 / I6：拖拽截图必须有 motion blur 或 cursor trail（不能截静态截图冒充拖拽）
- I7-I14：详细见 playbook · Arbiter 写完后跑 self-check grep

---

## 结论

MVP-05 Phase D 验收项中：

- **代码侧验证全过**（panes 63 / pane_service 23 + bench harness 7 · 0 failed · §F 仪表化 inline 就位）✅
- **既有 evidence 仅 CAPTURE-PLAYBOOK + metrics 模板**（6 PNG + 录屏 + 数字全空）🔴
- **Arbiter 30 min capture 窗口**（playbook 14 invariant · capture-phase-d.sh 自动化部分 + DevTools Performance + webview console.info）🔴

MVP-05 spec 维持 `ready`（Phase A/B/C 代码 done · Phase D capture 待 Arbiter）。

**关联**：spec [`docs/tasks/MVP-05-pane-split-single-level.md`](../../tasks/MVP-05-pane-split-single-level.md) §F 测试矩阵 / §I 实施进度 · [`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md) 14 invariant · [`metrics-mvp-05.md`](./metrics-mvp-05.md) 测量手册 · `scripts/capture/mvp-05/` 自动化脚本 · `.claude/rules/runtime-evidence-location.md` R1-R5 · 模板镜像 [MVP-19 PRE-CAPTURE-READINESS](../mvp-19/PRE-CAPTURE-READINESS.md) PR #384
