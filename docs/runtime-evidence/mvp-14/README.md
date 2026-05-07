# MVP-14 Phase A · Runtime Evidence

按 ADR-011 R1 · 路径锁 `docs/runtime-evidence/<task-id>/`（目录非单文件）。

## 文件索引

| 文件 | 内容 |
|---|---|
| `01-cargo-test-panes.txt` | panes 模块 89 单测 raw output |
| `02-cargo-test-pane-service.txt` | pane_service 24 单测 raw output |
| `03-h2-regression-proof.txt` | spec §G.4 6 步 H2 drift 验证 console log（step 3 typecheck FAIL · step 5 PASS） |
| `04-bindings-listing.txt` | 12 ts-rs binding 物理列表 + count |

## H2 Anchor 设计

`web/src/panels/Pane/_contract-typecheck-mvp14.ts` · type-only 哨兵 · 不被运行时代码 import · 仅作 contract drift 检测（仿 PR #256 droid `web/src/panels/GitLog/RailGraph/_contract-typecheck.ts` 模式）。
