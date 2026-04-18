# ADR-004: 前端栈 = SolidJS + TypeScript + Vite + xterm.js

**状态**：accepted
**日期**：2026-04-18（Phase 1 锁定 · Phase 3 ADR 建立）
**决策者**：项目发起人 · 多 agent 评审
**对应 `CLAUDE.md` 决策表**：#6

---

## 背景与问题

Vibestation 前端需要满足：
- 高频 UI 更新（xterm 每秒多次渲染 · Git Log 滚动加载）
- 小 bundle（Tauri 场景下前端资源随 app 分发 · 目标 < 150kb gzipped）
- Rust 后端 via Tauri IPC 集成顺畅
- JetBrains 级视觉（Calm Studio · 对标 Linear/Zed/Raycast）

候选栈影响：bundle 大小 / 运行时开销 / 开发体验 / 生态成熟度。

## 决策驱动因素

- **D1 · 性能**：xterm 渲染热路径不允许 React 级别 reconciliation 开销
- **D2 · Bundle**：Tauri 目标 dmg < 30MB · 前端 < 150kb
- **D3 · 类型安全**：Rust ↔ 前端类型必须端到端 · JS 不可选
- **D4 · 生态**：xterm.js / diff crate JS 绑定 / Tauri JS SDK 必须兼容
- **D5 · 长期维护**：v0.2+ 3-5 年范围 · 框架不能短命

## 考虑的选项

### 前端框架

- **A · React**：生态第一 · 但 virtual DOM 对高频更新不友好 · bundle 偏大
- **B · Vue 3**：composition API 好 · 但 Rust 社区用得少 · Tauri 生态偏 React/Solid
- **C · SolidJS**：fine-grained reactivity · 无 virtual DOM · 极快 · bundle 最小（< 10kb 核心）
- **D · Svelte 5**：runes 很新 · 编译期优化 · 但 Tauri 集成样例少 · 风险未知
- **E · Leptos（Rust）**：纯 Rust · 无需 JS · 但 WASM 运行时体积 + xterm 集成成本高
- **F · Floem（Rust native）**：已考虑但 `CLAUDE.md` 明确"不碰"（GUI Rust 生态不成熟）

### 语言

- **TypeScript**：社区默认 · 与 Rust 类型的 manual sync 可接受
- **JavaScript**：拒绝 · 项目规模下必然出错

### 构建工具

- **Vite**：现代标准 · Tauri 官方支持
- **Rspack**：更快但生态小
- **esbuild 直用**：偏底层 · 不建议

### 终端渲染

- **xterm.js 5.5**：事实标准 · 被 VSCode / zed 用 · 成熟
- **Alacritty 前端**：仅 Rust native · 与 JS 前端不兼容

## 决策

**选择**：
- **框架**：选项 C · **SolidJS**（最新稳定版）
- **语言**：**TypeScript** 严格模式
- **构建**：**Vite**（Tauri 2 官方默认）
- **终端**：**xterm.js 5.5** + `@xterm/addon-fit` + `@xterm/addon-web-links`
- **Floem 禁区**：`CLAUDE.md §禁区` 明确不碰

**理由**：
1. **SolidJS fine-grained reactivity**：signal/store 模型与 xterm 高频渲染完美契合 · 不触发 virtual DOM diff
2. **Bundle 最小**：SolidJS 核心 < 10kb · 比 React/Vue 小 5-10 倍 · 给 xterm 和业务代码留更多预算
3. **与 Rust 类型易于对齐**：小框架 · API 少 · 维护 Rust ↔ TS 类型 bridge 简单
4. **生态充分**：Tauri 2 官方 Solid 模板 · solid-js 成熟 5+ 年 · 不是小众实验框架

## 后果

### 正面

- **性能上限高**：signal 级更新 · 4 Tab 同时 `yes` 压测 FPS 不掉
- **包小**：前端 gzipped 估算 < 100kb · 符合 `implementation-plan.md §10.2` 预算
- **类型安全**：`createSignal<T>()` · `Resource<T>` 全类型化
- **学习曲线**：来自 React 的贡献者 15 分钟入门 · 心智模型类似 hooks 但更简单

### 负面

- **社区规模**：SolidJS << React · 招聘 / 外包找 Solid 开发者难度略高（本项目当前单人 · 不关键）
- **第三方库少**：比 React 少 10 倍 · 大部分 React 库可以手改为 Solid · 但非零成本
- **JetBrains 级 UI 组件**：无现成 "shadcn-solid" 类套件 · 手写（本项目视觉已设计完整）

### 风险

- **SolidJS 1.x → 2.0 大版本**：若未来升级 breaks · 参考 React 16→18 级别迁移成本 · 当前 1.9 稳定 · 2.0 传言 compose API 兼容
- **xterm.js 依赖**：xterm 5.x 稳定 · 但若停更 → 必须 fork 或换 Alacritty native

## 与 `implementation-plan.md` 的映射

- 对应章节：§3.1（技术栈选型）· §10.2（性能指标：bundle 预算）
- 对应风险：R12（Tauri 桌面框架 CRITICAL · 本 ADR 的前端栈依赖 Tauri 选型）

## 相关

- `CLAUDE.md` 决策表：#6
- 前端代码目录（Spike W0 后建立）：`app/web/` 或 `src/`
- 相关 ADR：ADR-006（桌面框架 Tauri 2 · 上游决策）

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code（Phase 3 · 把 Phase 1 锁定决策正式化为 ADR）
