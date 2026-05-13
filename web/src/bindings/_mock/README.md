# MOCK IPC binding · MVP-17 Phase C 临时使用

session 29（2026-05-12）OpenCode 实施 MVP-17 Phase C frontend 时 ·
Phase A（Codex CLI）+ Phase B（主 agent）尚未 merge · 真实 ts-rs binding
未生成。本目录 11 个 stub 文件复制 spec §G.1 描述的 struct shape · 让 Phase C
能够：

1. 编写 UI 组件（无类型错误）
2. 写 vitest 单测（mock IPC mode）
3. 自审 lint + typecheck pass

## 何时删除

Phase A + Phase B 都 merge 后（PR #28x + #28x · 顺序不限）·
主 agent / 后续 dispatch agent 跑：

```bash
# 删 mock 目录
rm -rf web/src/bindings/_mock/

# Phase A/B merge 后 cargo build 自动生成 web/src/bindings/*.ts（真 binding）
cargo build --workspace

# 把所有 import 从 _mock 改成根目录
grep -rln "_mock/" web/src/ | xargs sed -i.bak 's|bindings/_mock/|bindings/|g'
rm -f web/src/**/*.bak
```

## 来源

[MVP-17 spec §G.1](../../../docs/tasks/MVP-17-external-terminal-pane-detach.md)
