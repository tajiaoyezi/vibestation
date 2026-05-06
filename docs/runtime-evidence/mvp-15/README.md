# MVP-15 Phase A Runtime Evidence

## 范围

Phase A 仅实现 shiki v3+ 集成基础 + TypeScript 语言 + light/dark 主题切换。

## 截图

### 01-typescript-syntax-highlight-light.png

TypeScript diff 文件在 light 主题下的 syntax highlight 效果。

### 02-theme-switch-dark.png

同一 TypeScript diff 文件切换到 dark 主题后的效果。

## 测试覆盖

- vitest 单测：13/13 PASS
  - LRU cache：4 个测试（put/get/淘汰/clear）
  - guessLanguageFromPath：5 个测试（ts/tsx/js/py/unknown）
  - fallbackToPlainText：1 个测试
  - theme switch：2 个测试
  - failure path：1 个测试

## 备注

- Tier 1 其他语言（Rust/Python/Go/Java/Markdown/JSON/YAML/Shell）留 Phase B
- IntersectionObserver lazy load 留 Phase B
- 大文件流式加载（1MB/10MB/50MB）留 Phase C
- 性能 benchmark 留 Phase D
