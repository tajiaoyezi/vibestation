# Tauri 2 项目模式 · Vibestation 专属

> 本规则沉淀 Vibestation 在 Tauri 2 上踩过的坑和已采纳的 pattern。凡接触 `crates/app/` (Tauri 启动层) 或 `web/` (SolidJS 前端) 前 · 先读本规则。

## 1 · ACL 强制 permission（事故教训 · 2026-04-19 PR #28）

### 规则

**Tauri v2 的 ACL 系统要求自定义 `#[tauri::command]` 显式声明 permission · 否则 frontend `invoke()` 被 deny · runtime 显示 "IPC error: denied"**。

### 正确做法

1. **定义 permission**（在 `crates/app/permissions/*.toml`）：

```toml
"$schema" = "../gen/schemas/default_permissions.json"

[[permission]]
identifier = "allow-<command_name>"
description = "Description of what this command does"
commands.allow = ["<command_name>"]
```

2. **capability 引用**（在 `crates/app/capabilities/default.json`）：

```json
{
  "permissions": [
    "core:default",
    "allow-<command_name>"
  ]
}
```

3. **Rust side**（在 `crates/app/src/lib.rs`）：

```rust
#[tauri::command]
fn my_command() -> String { /* ... */ }

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![my_command])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}
```

### 反模式

| 反模式 | 真正该做的 |
|---|---|
| 只写 `#[tauri::command]` + `generate_handler![]` · 不定义 permission | **必加 permission toml + capability 引用** · 否则 runtime deny |
| 假设 `core:default` 覆盖自定义 command | **不覆盖** · core:default 只含 core plugin 的 default set |
| 依赖 CI build smoke 验证 | **build 过 ≠ runtime 过** · 必须本地 `pnpm tauri:dev` 开窗口看 greet 结果 |

### 为什么 Spike 骨架（如 SPIKE-02）没定义 permission 也能跑

SPIKE-02 前端**只调用 plugin API**（`@tauri-apps/plugin-clipboard-manager` · `@tauri-apps/plugin-fs`）· plugin 自带 permission 声明（`clipboard-manager:allow-write-text` 等）。SPIKE-02 的 `greet` 自定义 command **从未被前端 invoke** · 所以表现不出来 ACL deny。不能用 SPIKE-02 "能跑" 推论自定义 command 不需要 permission。

## 2 · CSP 最小化（Codex adversarial review 采纳 · PR #28）

### 规则

Tauri v2 默认 `csp: null` 是 "初次搭建" 设置 · **不是生产默认**。从 Phase A 第一行代码开始就设最小 CSP · 未来加 renderer-facing 不信任内容（terminal output / diff / markdown）时不会"温水煮青蛙"。

### 当前最小 CSP（Phase A · 生产用）

```json
"security": {
  "csp": "default-src 'self' ipc: http://ipc.localhost; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost; object-src 'none'; base-uri 'self'; frame-ancestors 'none'"
}
```

**允许的来源**：

- `'self'`：本地 bundle
- `ipc:` 和 `http://ipc.localhost`：Tauri 2 IPC 协议
- `'unsafe-inline'` for style：Solid/Vite 需要 inline style · 可 Phase B 用 nonce 收紧
- `data:` for img/font：data: URL（SVG icon 等）

**禁止**：

- `object-src 'none'` · 禁 plugin embed
- `frame-ancestors 'none'` · 禁被 iframe 嵌入
- `base-uri 'self'` · 防 base tag 劫持

### 只对 production 生效

`app.security.csp` 只对 `frontendDist` 本地文件加载生效。`pnpm tauri:dev` 时走 `http://localhost:1420`（Vite dev server）· dev server 自己的 CSP 不受影响 · Vite HMR 正常。

## 3 · Capability 最小权限（Codex review 采纳）

### 规则

**Phase A 只有 `core:default`**（暂时保留 · Phase B 前收紧到精确子集）· **不要** 加 `opener:default` 等未实际调用的 plugin permission。"未调用的 permission = 纯攻击面增加"。

### 当前状态（Phase A）

```json
{
  "permissions": [
    "core:default",
    "allow-greet"
  ]
}
```

- `core:default` 包含 Tauri 核心 default set（app/event/image/menu/path/resources/tray/webview/window）· Phase B 收紧
- `allow-greet` 显式允许 greet 命令

### Phase B 前要做的收紧（TODO · 触发时机 @ MVP-04 Phase B 启动）

⏰ **触发时机**（本 TODO 从未明确 deadline · session 13 audit L-1 补）：

MVP-04 Phase B（PTY runtime 启动）**第一次要加 `fs:*` 或 `process:*` permission 时** · 必须**同一 PR 内**完成本收紧动作。不能继续延后到 Phase C/D。

**责任人**：MVP-04 Phase B 的 implementer agent（OpenCode / Codex / Claude 均可）· PR body Test Plan 必含 checkbox "Tauri capability 已从 core:default 收紧到精确子集"。

**动作分析** · `core:default` 展开的 9 个子 set · 只保留实际需要的：

- `core:app:default`（app info）· 大概率需要
- `core:window:default`（window 操作）· MVP 可能需要 close/minimize/resize
- `core:webview:default`（webview）· 取决于 Phase B 功能
- `core:event:default`（事件）· IPC 事件系统可能需要
- `core:path:default` / `core:resources:default` / `core:menu:default` / `core:tray:default` / `core:image:default`：按需加

实际做法：删 `core:default` · 尝试 `core:app:default` + `core:window:default` · 跑 dev 看哪里 deny · 逐个加回最小必需。

### 反模式

| 反模式 | 真正该做的 |
|---|---|
| 从 Tauri 模板 cp capability · 包含 `opener:default` 等无用 permission | 初始化时就删 · 未来用时再加具体 allowlist（非 default） |
| 每加一个 feature 就粗暴加 `*:default` | 具体 permission identifier · 比如 `fs:allow-read-text-file` 比 `fs:default` 小 |

## 4 · Tauri CLI --config 参数位置（语法坑）

### 规则

Tauri CLI 的 `--config` 必须在 **subcommand 之后** · 不能在之前：

```
❌ tauri --config path/to/tauri.conf.json build
✅ tauri build --config path/to/tauri.conf.json
```

### `package.json` scripts 推荐写法

```json
{
  "scripts": {
    "tauri:dev": "tauri dev --config crates/app/tauri.conf.json",
    "tauri:build": "tauri build --config crates/app/tauri.conf.json",
    "tauri:build:smoke": "tauri build --config crates/app/tauri.conf.json --debug --no-bundle"
  }
}
```

**不要** `"tauri": "tauri --config X"` 然后 `pnpm tauri build` · 会 "unexpected argument '--config'"。

## 5 · Icons 占位 → 真实替换路径

### Phase A

使用 SPIKE-02 骨架的 `icons/`（Tauri 默认 256 像素企鹅图标等）· 不在 Phase A 替换真实 icon。

### Phase B

从 `design/logos/mark.svg` 生成真实 icon 组：

```bash
# 用 tauri-cli 生成（推荐）
pnpm tauri icon design/logos/mark.svg

# 或手动生成 32x32 / 128x128 / 128x128@2x / icon.icns / icon.ico
```

## 关联

- [全局] `~/.claude/rules/15-runtime-verification-gate.md` · Tauri / GUI 项目的 runtime 验证责任
- [全局] `~/.claude/rules/14-ci-pnpm-pattern.md` · pnpm + Tauri 在 CI 的 corepack pattern
- 事故记录：
  - PR #28 · 2026-04-19 · Codex adversarial review 2 轮（CSP + CI build smoke + opener + Cargo.lock + ACL + core:default）
  - 教训：CI 全绿不等于 product ready · Tauri v2 ACL 默认 deny 自定义 command
