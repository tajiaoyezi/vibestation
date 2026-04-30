---
id: MVP-20
type: mvp
title: PTY 预热池 · 新 tab 瞬时出 prompt
status: ready
owner:
phase: v0.2
depends_on: ["MVP-04"]
blocks: []
estimate: 1.5d
plan_ref: implementation-plan.md §3.2 · §5.3
risk_ref:
reviewer: tajiaoyezi (Arbiter · 单人项目 v2-D.1 self-review)
---

# MVP-20: PTY 预热池 · 新 tab 瞬时出 prompt

> **状态**：`draft` → `ready` → `in-progress` → `done`
> **依赖**：MVP-04 PTY runtime（已 done）
> **战略依据**：[`implementation-plan.md §3.2`](../implementation-plan.md)（核心架构 · PTY 子系统）

---

## 🎯 目标（Goal）

新增 tab 时 · 用户从点击到看见 prompt 的可见延迟从 **当前 700-2500ms** 降到 **≤ 200ms（warm 命中）/ ≤ 当前水平（cold 兜底）**。

## 📖 背景（Context）

- MVP-04 已落地 PTY runtime · 但每个新 tab 都是 cold spawn：fork shell → 加载 .zshrc / .bashrc / oh-my-zsh / starship → 等 prompt
- 用户实测 macOS + zsh + omz 环境下 · 新增 tab 主观感受"卡 1-2 秒"
- shell 启动开销是用户环境决定的（与 vibestation 无关）· 但用户感知是 app 慢
- 业界做法：tmux / kitty / wezterm / Warp 都有类似预热机制（warp 文档明确提"warm shell pool"）

---

## 🎨 功能范围（Scope）

**Do**：

- 后台维护 1 个（默认 · 可配 1-2-3）idle PTY · 启动时立即预热 · take 后异步补充
- 用户点 + 新 tab 时优先从池里取 idle PTY · 取不到回退到现有 cold spawn 路径
- **池作用域 = per-app 全局单例**（非 per-workspace · workspace 切换不销毁 pool · idle PTY 的 cwd 由 attach 时 cd 修正）
- attach 时若 cwd 不匹配 · 通过 PTY stdin 注入 `cd -- 'path'; clear\n` 切换目录
  - 用 `--` 终止 shell 选项解析（兼容 fish/zsh/bash · 防 path 以 `-` 开头）
  - 用 `;` 分号而非 `&&`（fish shell 不支持 `&&` · 用 `;` 三大主流 shell 全兼容）
  - 路径含单引号时降级为 `cd -- "path"` 双引号（双引号路径含 `"` 时再降级 cold spawn · 极少见）
- **idle stdout 处理**：预热 PTY 注册到 PtyManager reader thread · `PtySession.tab_id` 用 placeholder `__idle_<uuid>__` · 前端 event listener 按真实 tab_id filter · placeholder 自动被忽略 · 不污染前端事件总线
  - 实现要点：`PtySession.tab_id` 字段改为 `parking_lot::Mutex<String>` 或同等可变结构 · take 时 rename 为真实 tab_id
  - take 前 drain mpsc backlog（防 omz greeting 残留显示）
- **预热并发**：spawn 在独立 thread 执行（不阻塞 main thread / Tauri webview）· 用户可立即操作 app
- idle PTY 超过 5 分钟未使用 → kill + 重新预热（防止 shell 状态老化）
- shell 不匹配（用户改了默认 shell 或 tab 显式指定不同 shell）→ 直接 cold spawn · 不维护多 shell 池
  - **default shell 变更检测**：Settings change → pool 立即全 kill 现有 idle + 用新 shell 触发预热（不等懒加载）
- Settings 加 toggle：`pty_pool_enabled: bool`（默认 true）+ `pty_pool_size: 1 | 2 | 3`（默认 1）
- DB schema 不动（pool 是运行时状态 · 不持久化）
- App 退出 / workspace 切换 / settings 关闭 pool 时正确 reap idle PTY · 不留 zombie

**Don't**（显式排除 · 避免 scope creep）：

- ❌ 不做多 shell 类型池（zsh + bash + fish 各一个）· 单池只追当前 effective_shell
- ❌ 不做 cwd 预测预热（基于 workspace recent 预热在常用目录）· v0.3 再考虑
- ❌ 不做"用户输入命令再启动 shell"（lazy spawn）· 这是另一种架构
- ❌ 不改 PTY reader thread / event routing 架构 · MVP-04 ADR-003 既定不动
- ❌ 不动 pane_pty（先验证 tab_pty 路径 · pane_pty 在 Phase B 一起接入）
- ❌ 不做 Linux / Windows 平台特异化 · portable-pty 已抽象 · 跨平台逻辑等价

## 🖼 UI 引用（UI Reference）

- Settings 面板新增分区 `Terminal · Performance`：
  - `[ ] 启用 PTY 预热池`（toggle）· 描述："新 tab 启动加速 · 后台预备 shell"
  - `池容量 [1] [2] [3]`（segmented · 默认 1 · 仅 toggle on 时可见）
- 位置：`web/src/panels/Settings/SettingsPanel.tsx` · 现有 Terminal 分区下面新加 Performance section

## ✅ Acceptance

evaluator 按此逐项对照：

- [ ] **A1a · warm 命中核心延迟**：从前端 `invoke("tab_pty_spawn", ...)` 调用到 xterm 首次 `onData` 事件触发 ≤ 200ms（macOS + zsh + omz 环境 · pool enable + 池容量 1 · Performance.now() 标记两端时间戳）
- [ ] **A1b · warm 命中 end-to-end 延迟**：从用户点 + 按钮到 prompt 显示在屏幕（含 cd 注入 + chpwd hook） ≤ MVP-04 cold spawn baseline 的 50%（同台机器同 shell · 实测对比数据）
- [ ] **A2 · cold 兜底等价性**：pool disable 时 · 新增 tab spawn 延迟 P99 与 MVP-04 baseline 差异 ≤ 10%（同台机器同 shell · 各采样 ≥ 20 次取 P99）
- [ ] **A3 · shell 不匹配 cold path**：用户在 Settings 修改 default shell 并关闭 settings 窗口后 · 现有 idle pool 立即 kill + 触发新 shell 预热；首次新增 tab 走新 shell warm path（zsh → bash 切换录屏 + idle pool 状态日志验证）
- [ ] **A4 · cwd 切换正确**：idle PTY 在 $HOME · 用户在 workspace `/Users/.../my-project` 下新增 tab · 看到 prompt 的 cwd 是 `my-project` 而非 `~`
- [ ] **A5 · idle 老化回收**：idle PTY 超过 5 min 未用 → 自动 kill + 补新 idle（单元测试 + log 验证）
- [ ] **A6 · zombie 检测**：app 启动 + 关 settings + 退出 · 完整生命周期内 idle shell 数与 sessions HashMap 一致 · 无泄漏（单元测试 reap 验证）
- [ ] **A7 · 设置实时生效**：Settings toggle off → 立即 kill 现有 idle pool；toggle on → 立即开始预热 · 不需重启 app（实测录屏）
- [ ] **A8 · 池容量调整生效**：1 → 2 立即补到 2；2 → 1 立即 kill 1 个多余 idle
- [ ] **A9 · 跨平台编译通过**：macOS + Linux 都能 `cargo build --workspace` + `cargo test --workspace` 全过 · CI 绿
- [ ] **A10 · runtime 证据**：3 段录屏放 `docs/runtime-evidence/mvp-20/` · 含主观感受 + 时间戳数据：
  - `01-warm-hit.mp4`：zsh + omz 环境 · pool enable · 点 + 到 prompt 出现（标 A1a / A1b 时间戳）
  - `02-cold-path.mp4`：pool disable · 同环境点 + 到 prompt（baseline 对照）
  - `03-settings-toggle.mp4`：on → off 看 idle PTY 熄灭日志 · off → on 看预热重启 + 容量 1→2 立即补

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | `pty_pool.rs` 内部状态机 | `cargo test -p vibestation-core pty_pool::` · 覆盖 take/refill/expire/mismatch/disable 5 条核心路径 |
| 集成 | `tab_pty_spawn` IPC 走 pool | `cargo test --test tab_pty_pool_integration` · mock pool 与 cold path 切换 |
| Runtime | macOS 实测 · 主观体感 | dev mode 录屏 + Performance.now() 时间戳标记 |
| 跨平台 | Linux CI 不变慢 | GitHub Actions Linux job 跑 cargo test · 性能数据可选 |

## 💾 数据模型变更

无 DB schema 变更。Settings 字段 `pty_pool_enabled` / `pty_pool_size` 加入 `app_settings` 表（已有的 settings 表不需 migration · 用现有 KV 存储 · 默认值在 Rust 端 fallback）。

## ⚠️ 已知风险

- **R1 · idle shell 状态污染**：用户 .zshrc 里有 timeout 后台任务 / 全局 trap / set -e · idle 时可能产生不可预期状态 · 5 分钟回收只能缓解
  - 缓解：Settings 提供 toggle off · 怀疑环境异常时一键关
- **R2 · cd 注入的 prompt 重绘延迟**：omz / starship 的 chpwd hook 触发重画 · 50-300ms · 仍快于 cold spawn 但有可见 flash
  - 缓解：take 时发 `cd -- 'path'; clear\n` · clear 隐藏中间状态 · A1b end-to-end 门槛已计入此延迟
  - bash PROMPT_COMMAND 类似情况 · 无 omz 也可能触发；fish 无 chpwd hook · 此延迟在 fish 下接近 0
- **R3 · 单引号路径转义**：workspace 路径含单引号或特殊字符时 cd 命令注入失败
  - 缓解：用 `cd -- 'path'` 单引号兜底 · 路径含 `'` 时降级 `cd -- "path"` 双引号；含 `"` 时再降级 cold spawn（极少见 · 单元测试覆盖）
- **R4 · idle PTY 占用 fd / 内存**：每个 idle ~5MB + 1 fd · 默认池 1 · 极端情况 3 · 影响小
  - 缓解：池容量上限 3 · 5 min 超时回收
- **R5 · take 与 refill 并发竞态**：用户连续狂点 +new tab 时 idle 可能瞬间排空 · refill 任务来不及补
  - 缓解：take 内部用 `Mutex<VecDeque>` 保证原子 · refill 用独立线程异步触发 · 不阻塞 take
- **R6 · fork 失败 / fd 耗尽 graceful degradation**：系统资源不足导致预热 spawn 失败时不能 panic / 影响主流程
  - 缓解：spawn 失败 → log warn + 降级 cold spawn 路径 + 5s 后重试 refill；连续 3 次失败则自动 disable pool（运行时 · 不持久化到 settings）+ toast 提示用户"PTY 预热池已自动停用 · 请检查系统资源"
- **R7 · 系统休眠 / app background 对 idle PTY 的影响**：macOS App Nap / Linux suspend → resume 后 idle PTY 可能因超时被回收（预期行为）
  - 缓解：不特殊处理 · 唤醒后 idle 缺失会触发 refill 自动补上 · 用户首次 take 走 cold path（此时刚刚醒 · 用户感知与 cold start 一致 · 无额外退化）

---

## 📝 Notes / 讨论

### 实施 Phase 拆分（按 Kimi review 建议拆 Phase A · 降低单 PR diff 量）

- **Phase A1** · `crates/core/src/pty_pool.rs` 核心结构 + take/refill/kill API + 基础单元测试（**Codex CLI fast** · 独立 worktree · 3-4h · 第一个独立 PR）
  - `PoolConfig` / `PtyPool` struct + idle queue + refill async hook
  - `take(req)` / `kill_all()` / `len()` / `set_size()` API
  - `PtySession.tab_id` 改为可变（`parking_lot::Mutex<String>`）+ rename helper
  - 单元测试：take warm hit / take cold fallback / refill / size adjust（≥ 4 个）
- **Phase A2** · 生命周期管理：app exit / settings toggle / shell change / idle expire（**Codex CLI fast** · 独立 worktree · 3-4h · 接续 A1 第二个 PR）
  - 5 min idle expire timer（用 `crossbeam_channel::recv_timeout` · 不引入 tokio）
  - settings toggle on/off 触发 kill_all / start_warmup
  - default shell change 触发 kill_all + 新 shell 预热
  - 单元测试：expire / toggle / shell change（≥ 3 个）
- **Phase A3** · cd 注入 + 路径转义 + 跨 shell 兼容（**Codex CLI fast** · 独立 worktree · 2-3h · 接续 A2 第三个 PR）
  - `inject_cd_clear(session, target_cwd)`：`cd -- 'path'; clear\n` · 含单/双引号 fallback
  - 单元测试：normal path / path with single quote / path with double quote / path with `--` prefix（≥ 4 个）
  - 跨 shell 行为模拟测试（mock shell · 不真跑 zsh/bash/fish）
- **Phase B** · `tab_pty_spawn` / `pane_pty_spawn` 接入 pool.take（**主 agent** · 3-4h · 依赖 A1+A2+A3 全 merge）
- **Phase C** · Settings UI + ts-rs binding（**主 agent** · 3-4h · **依赖 A1 完成**：`PoolConfig` struct ts-rs export 需先在 A1 落地 · 然后 C 才能并发 B）
- **Phase D** · Runtime evidence 录屏 + spec 翻 done（**主 agent** · 2h · 依赖 B+C merge）

### 跨环境兼容性原则

预热池设计**不绑定** shell 类型 / init 文件 / prompt 行为。运行时通过 `effective_shell_for_spawn`（已存在 · 见 `pty.rs:454`）确定预热目标 shell · 用户改默认 shell 自动失配走 cold path。

### 关联

- ADR：暂不开（Phase D 完成后看是否值得 ADR · 此功能不修改 #15 PTY 决策）
- CLAUDE.md 决策表：不影响（#15 PTY 方案保持）
- 相关 PR：待 Phase A/B/C/D 各开一个

---

**填写完毕后自审**：

1. ✅ **递归完备性**：A1-A10 全可量化（毫秒数 + 录屏 + 单测）· Phase 拆分明确
2. ✅ **反向场景**：cold path 兜底 · pool disable 等价当前 · shell 不匹配自动降级
3. ✅ **边界适用性**：跨 shell（zsh/bash/fish）· 跨 OS（macOS/Linux）· 跨 settings（on/off）· 跨容量（1/2/3）全覆盖
4. ✅ **YAGNI**：不做多 shell 池 / cwd 预测 / lazy spawn · 只解最痛点（warm hit）
