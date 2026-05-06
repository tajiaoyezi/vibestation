---
id: MVP-15
type: mvp
title: Diff 语法高亮（shiki lazy load · 对齐 W21）
status: ready
owner:
phase: v0.3
depends_on: ["MVP-08"]
blocks: []
blocked_by: []
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §W21（W21 表格 · shiki lazy load + 大文件流式）
risk_ref: 本 spec §已知风险 R1-R5（shiki WASM 包大小 / 大文件 freeze / 版本 breaking / 语言降级 UX / 主题切换闪烁）
reviewer: OpenCode
---

# MVP-15: Diff 语法高亮（shiki lazy load）

> **状态**：`draft`（v0.3 · **详化完成** · 等待 Arbiter approve 翻 `ready`）
> **依赖**：MVP-08（Diff 基础视图 · 自绘 · Phase A-D 全部 done · Phase E 部分 done · 不阻塞 MVP-15 启动）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md) · [`§W21`](../implementation-plan.md)
> **详化时间**：2026-05-06 · OpenCode spec 详化 session · chore/MVP-15-spec-detailed 分支

---

## 🎯 目标（Goal）

在 MVP-08 基础行对比的 Diff 视图上，对齐 `implementation-plan.md §W21` 的具体范围，叠加 **shiki lazy load 语法高亮 + 大文件流式加载** 两层能力。Diff 视图原有结构（split/unified、行号、增删色）完全保留，syntax highlight 作为纯装饰层注入，失败时 Diff 仍可用。

核心硬指标：`1MB 文件 diff 首屏 < 300ms`（`§W21` 验收）。

---

## 📖 背景（Context）

- **战略地位**：`implementation-plan.md §W21` 明确 v0.3 高级 Diff 范围 = `shiki lazy load` + `大文件流式加载` · 目标 `1MB 文件 diff <300ms`
- **CLAUDE.md 锁定**：#7（A 栏永久锁定）Diff 渲染 = 自建（不用 Monaco）· MVP-08 已实现基础行对比（HTML 优先）
- **技术决策已锁定**：v0.3 kickoff 评估后，`§W21` 锁定 **shiki v3+** 作为语法高亮引擎（不是 tree-sitter / highlight.js / Prism）
- **上游已落地**：MVP-08 PR #105 Diff 视图前端（`web/src/panels/Diff/`）已完成 split/unified + 行号 + 增删色 · MVP-15 只在此之上叠加 syntax highlight 装饰层
- **历史教训**：Codex PR #10 review F4 — 占位 spec 不得引入 scope creep · 本 spec 严格对齐 `§W21` 范围 · 不 pre-decide tree-sitter / word-level / LSP 等更重方案

---

## 🛠 实施进度

MVP-15 估时 **4d** · 拆 4 Phase 串行实施：

| Phase | 范围 | 状态 | PR |
|-------|------|------|----|
| Phase A · shiki 集成 + lazy load 基础 | `shiki` v3+ 包引入 · `Highlighter` 单例封装 · theme 预加载（light/dark 两套）· lazy load 核心逻辑（IntersectionObserver + 行级虚拟化）· LRU 缓存（100 文件 / 50MB）· 0 个 IPC binding（见 §G） | ✅ done · PR #252 | feat/MVP-15-phase-A-shiki |
| Phase B · Diff 视图 syntax highlight 装饰层 | `web/src/panels/Diff/` 组件改造 · 在原 `DiffLine` 渲染逻辑上注入 shiki token span · 主题 CSS variable 切换 · 纯文本降级 · 10 主流语言支持 | ✅ done · PR #255 | feat/MVP-15-phase-B-shiki-decoration |
| Phase C · 大文件流式加载 | 1MB-10MB 文件：`requestIdleCallback` 分 chunk 解析 · 10MB+：Web Worker 分 chunk · 分段大小 100KB · 主线程阻塞 ≤ 16ms | 🔄 待实施 | — |
| Phase D · runtime 证据 + 性能量化 | 1MB diff 首屏 < 300ms P99 截图（DevTools Performance）· 10MB 流式不阻塞（long task < 50ms）· 主题切换 < 50ms · 5 主流语言 × 2 主题 = 10 张 baseline screenshot · 放 `docs/runtime-evidence/mvp-15/` | 🔄 待实施 | — |

**Phase A 实施起点 checklist**（让 agent 接 spec 后 5 min 内启动）：

- [ ] `web/package.json` 新增 `shiki` v3+（`^3.0.0`）· 前端包（backend 不引入 shiki）
- [ ] 新建 `web/src/lib/shiki/` 目录：
  - `highlighter.ts` — `createHighlighter()` 单例 · `loadLanguage()` 按需加载 · `codeToTokens()` 行级 token 化
  - `theme.ts` — Calm Studio light + dark theme 映射（shiki theme 名 → CSS variable）
  - `lazy-loader.ts` — IntersectionObserver 封装 · viewport 内行触发高亮 · 滚动增量加载
  - `cache.ts` — LRU 缓存（100 文件 / 50MB）· 以 `file_path + theme + lang` 为 key
- [ ] `web/src/panels/Diff/DiffLine.tsx` 改造（或新建 `ShikiDiffLine.tsx`）：
  - 接收 `tokens?: ThemedToken[][]`（shiki 解析结果）
  - 无 tokens → 纯文本渲染（MVP-08 原有逻辑）
  - 有 tokens → 每行内按 token type 渲染 `<span class="token-{type}">`
- [ ] 语言检测：
  - 优先级：(1) 文件后缀映射 → (2) `linguist-language` 属性 → (3) 纯文本降级
  - 映射表：`web/src/lib/shiki/lang-map.ts`（Tier 1 语言：js/ts/rust/python/go/java/md/json/yaml/shell）
- [ ] IPC 需求（见 §G）：若 backend 需透传文件路径 / language guess · 复用 MVP-08 已有 `DiffRequest` / `DiffResponse` · 不新增 IPC command（见 §G.5 复用决策）
- [ ] shiki 主题 CSS variable 定义：`web/src/styles/shiki-theme.css`（与 Calm Studio design token 对齐）
- [ ] **shiki 初始化代码示例**：
  ```typescript
  // web/src/lib/shiki/highlighter.ts
  import { createHighlighter, type Highlighter } from 'shiki';
  
  let highlighter: Highlighter | null = null;
  
  export async function getHighlighter(): Promise<Highlighter> {
    if (!highlighter) {
      highlighter = await createHighlighter({
        themes: ['github-light', 'github-dark'],
        langs: ['javascript', 'typescript', 'rust', 'python', 'go', 'java', 'markdown', 'json', 'yaml', 'shell'],
      });
    }
    return highlighter;
  }
  ```
- [ ] **IntersectionObserver 封装示例**：
  ```typescript
  // web/src/lib/shiki/lazy-loader.ts
  export class ShikiLazyLoader {
    private observer: IntersectionObserver;
    private pendingLines = new Set<HTMLElement>();
    
    constructor(private highlighter: Highlighter) {
      this.observer = new IntersectionObserver(
        (entries) => this.handleIntersection(entries),
        { rootMargin: '200px' }
      );
    }
    
    observe(lineEl: HTMLElement) {
      this.observer.observe(lineEl);
    }
    
    private handleIntersection(entries: IntersectionObserverEntry[]) {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          this.pendingLines.add(entry.target as HTMLElement);
          this.processBatch();
        }
      }
    }
    
    private processBatch() {
      requestIdleCallback(() => {
        const batch = Array.from(this.pendingLines).slice(0, 50);
        for (const lineEl of batch) {
          this.highlightLine(lineEl);
          this.pendingLines.delete(lineEl);
        }
      });
    }
  }
  ```
- [ ] **语言映射表示例**：
  ```typescript
  // web/src/lib/shiki/lang-map.ts
  export const EXT_TO_LANG: Record<string, string> = {
    '.js': 'javascript',
    '.ts': 'typescript',
    '.tsx': 'tsx',
    '.rs': 'rust',
    '.py': 'python',
    '.go': 'go',
    '.java': 'java',
    '.md': 'markdown',
    '.json': 'json',
    '.yaml': 'yaml',
    '.yml': 'yaml',
    '.sh': 'shell',
    '.bash': 'shell',
    '.zsh': 'shell',
  };
  
  export const FILENAME_TO_LANG: Record<string, string> = {
    'Dockerfile': 'dockerfile',
    'Makefile': 'makefile',
    'CMakeLists.txt': 'cmake',
  };
  
  export function detectLanguage(filePath: string): string | null {
    const ext = filePath.slice(filePath.lastIndexOf('.'));
    if (EXT_TO_LANG[ext]) return EXT_TO_LANG[ext];
    
    const basename = filePath.split('/').pop() || '';
    if (FILENAME_TO_LANG[basename]) return FILENAME_TO_LANG[basename];
    
    return null;
  }
  ```

**下次 agent 起点**：等 Arbiter approve PR · 翻 `ready` · 派 Phase A 实施 agent（首选 OpenCode · 如不可用走 Claude Code / Kimi）。

**依赖关系说明**：MVP-15 依赖 MVP-08 Phase A-D done（Diff 视图基础结构 + IPC 已通）· 已满足 · 无前置阻塞。MVP-15 内部 4 phase 串行。文件域与 MVP-16（rebase/merge）**完全隔离**（MVP-15 只动 `web/src/panels/Diff/` + `web/src/lib/shiki/` + `web/src/styles/`）· 可并行启动。

---

## 🎨 功能范围（Scope）

**Do**（严格对齐 `§W21`）：

- **shiki lazy load 语法高亮**（主线 · `§W21` 指定方案）
  - 仅对 viewport 可见行加载 shiki theme + parse（IntersectionObserver）
  - 滚动时增量加载（进入 viewport 的行触发 parse）
  - 缓存 theme + parse 结果（LRU · max 100 文件 / 50MB）
  - 主流语言覆盖（Phase A 范围）：
    - Tier 1（MVP）：JS / TS / Rust / Python / Go / Java / Markdown / JSON / YAML / Shell
    - Tier 2（v0.4+）：Swift / Kotlin / C++ / Ruby / PHP / SQL / HTML / CSS / Vue / Svelte
  - 不支持语言降级：纯文本（不崩溃 · console.warn + UI 顶部 chip "Plain text"）
- **大文件流式加载**（主线 · `§W21` 指定行为）
  - 1MB 文件 diff 首屏 < 300ms（`§W21` 硬指标）
    - 算法：先 render diff 结构（无高亮）· 再 lazy 加载 shiki
    - 性能预算：先呈现 + 后润色（Progressive Enhancement）
  - 10MB+ 文件流式：
    - 分段加载（每段 100KB · 由滚动触发）
    - 主线程阻塞 ≤ 16ms（60fps · 用 `requestIdleCallback` 或 Web Worker）
  - 切换主题（light / dark）瞬时生效：
    - shiki theme cache 预加载两套
    - DOM 不重建 · 仅替换 `data-shiki-theme` 属性 · CSS variable 切换
- **Diff 视图原结构保留**（MVP-08 已有）
  - split/unified 模式 · 行号 · 增删色（绿/红/灰）全部保留
  - syntax highlight 是 token 级颜色叠加 · 不改变 diff 结构
  - 选择 / 复制：shiki 渲染的 token span 必须可选可复制（inline span 不影响）

**Don't**（明确排除 · 防 scope creep）：

- tree-sitter 方案（未经 implementation-plan 批准 · 若 v0.4+ 评估换 tree-sitter · **先更新 `§W21` + 开新 ADR** 再改本 spec）
- word-level diff（超出 `§W21` 范围 · v0.3+ 不做）
- LSP 语义高亮（v0.3+ 不做）
- 交互式编辑（Diff 视图只读 · 见 MVP-08）
- Monaco editor（`§10.1` 硬禁区 · MVP-08 §H.5 已禁）
- 修改 MVP-08 已有 backend 代码（MVP-15 纯前端装饰层 · backend 不改 diff 计算逻辑）
- 服务端渲染 shiki（`§W21` 锁定浏览器端 lazy load）

---

## 🖼 UI 引用

- **Diff 视图基础**：MVP-08 已完成 `web/src/panels/Diff/` · split/unified 两种模式
- **Syntax highlight 叠加**：在原有 diff 行内 · 按 shiki token type 渲染颜色
  - 示例：删除行（红底）内 · `const` 关键字蓝色 · `foo` 标识符默认色 · `"bar"` 字符串绿色
  - 保持 diff 底色（红/绿/灰）为 dominant · token 颜色 secondary · 避免视觉冲突
- **不支持语言 chip**：Diff 视图顶部 toolbar 右侧 · 细字灰色 chip "Plain text"
  - hover tooltip："此文件类型暂不支持语法高亮 · 作为纯文本显示"
  - 不弹 toast（避免烦扰）
- **主题切换**：与 Calm Studio light/dark 全局主题联动
  - shiki theme 名：`github-light` / `github-dark`（或自定义 Calm Studio 适配 theme）
  - 切换时 diff 视图内所有 token 颜色瞬时更新（CSS variable · 不重 parse）
- **截图归档**：详化时实施 PR 补到 `docs/runtime-evidence/mvp-15/`（按 `.claude/rules/runtime-evidence-location.md` R1 命名）

---

## ✅ Acceptance

### A. shiki 集成基础

- [ ] A.1 `shiki` v3+ 成功安装 · `pnpm -C web install shiki@^3.0.0` 后 `pnpm -C web typecheck` 通过
- [ ] A.2 `createHighlighter()` 单例初始化 < 100ms（DevTools Performance 测 `import('shiki')` 到 `createHighlighter` resolve · 测 3 次取 P99）
- [ ] A.3 theme 预加载：light + dark 两套 theme 在 app 启动时并行加载 · 不阻塞首次 diff 打开
- [ ] A.4 语言加载：Tier 1 语言（10 种）首次 loadLanguage < 50ms/种（DevTools Network 测 WASM 下载 + parse · 测 3 次取 P99）

### B. Lazy Load 语法高亮

- [x] B.1 仅 viewport 内行触发 shiki parse（IntersectionObserver · rootMargin 上下各 200px 预加载缓冲区 · DiffLine.tsx onMount + IO observe · DiffLine.test.tsx mock 验证 viewport 外不 highlight）
- [x] B.2 滚动时增量加载：新进入 viewport 的行 IO entries.isIntersecting 触发 highlight + unobserve · 单测 mock 验证（16ms P99 留 Phase D DevTools Performance 量化）
- [x] B.3 LRU 缓存生效（Phase A LRUCache 已落地 · multi-lang 单测验证多 lang 独立 entry · 同 lang/theme/code → cache hit）
- [x] B.4 缓存驱逐（Phase A LRUCache 已含 maxFiles + maxSizeBytes 双驱逐 · 原 shiki.test.ts L30-L42 已验证）
- [x] B.5 语言检测准确：`.ts` `.tsx` → typescript · `.js` `.jsx` → javascript · `.rs` → rust · `.py` → python · `.go` → go · `.java` → java · `.md` `.markdown` → markdown · `.json` → json · `.yaml` `.yml` → yaml · `.sh` `.bash` `.zsh` → shell（与 shiki lang ID 对齐 · multi-lang.test.ts 验证）· 无匹配返回 null → PlainTextChip 显示
- [x] B.6 不支持语言降级：guessLanguageFromPath null → DiffLine highlighted=null → fallbackToPlainText（HTML escape） · PlainTextChip toolbar 显示 "Plain text" · UI 不崩溃（DiffLine.test.tsx + PlainTextChip.test.tsx 双测）

### C. 大文件流式加载

- [ ] C.1 **1MB 文件 diff 首屏 < 300ms**（`§W21` 硬指标 · Chrome DevTools Performance 测从点击 Status 文件到 DOM commit 完成 · 含 IPC + diff 计算 + 无高亮渲染 + viewport 内行 shiki parse · 测 5 次取 P99）
- [ ] C.2 1MB 文件算法：先 render diff 结构（无高亮）→ 再对 viewport 内行 lazy load shiki → 用户感知"先看到内容，再变彩色"
- [ ] C.3 10MB 文件流式：分段加载（每段 100KB）· 由滚动触发 · 未加载段显示纯文本（不空白）
- [ ] C.4 主线程阻塞 ≤ 16ms（Chrome DevTools Performance 录 long task · 10MB 文件滚动时任何 frame 不掉 60fps · 测 3 次取 P99）
- [ ] C.5 Web Worker 兜底：10MB+ 文件 shiki parse 在 Worker 执行 · 主线程仅接收 token 数组 · Worker 通信 < 20ms
- [ ] C.6 50MB 文件：打开时提示 "Large file ({size}) · 语法高亮已禁用" · 纯 diff 结构渲染（MVP-08 已有逻辑）· 不尝试 shiki parse

### D. 主题切换

- [x] D.1 切换 light/dark · DiffLine createEffect track useShikiTheme() · setShikiTheme 触发 signal 变化 · 自动重 highlight（DiffLine.test.tsx light vs dark innerHTML 不同验证）· < 50ms 留 Phase D DevTools 量化
- [x] D.2 DOM 不重建：setShikiTheme 仅写 `data-shiki-theme` attribute + signal · DiffLine 只重渲 innerHTML（不重 mount span · ref 不变）· 滚动位置保留（无 scroll reset）
- [x] D.3 两套 theme cache 预加载：createHighlighterCore themes 含 githubLight + githubDark · 启动时一次性加载 · 切换时零网络

### E. UI / 视觉

- [x] E.1 Diff 视图原结构不动：DiffPanel 改动仅 toolbar-right 加 PlainTextChip · DiffLine inline span 注入不改 split/unified / 行号 / 增删色
- [ ] E.2 Token 颜色与 diff 底色兼容（WCAG AA 对比度 ≥ 4.5:1）· explicit skip "Phase D 视觉量化 · 实机 capture light/dark × 5 lang baseline 后做 Lighthouse contrast audit"
- [x] E.3 选择 / 复制：shiki 输出 inline span 不阻断浏览器原生 selection · 用户 Cmd+A + Cmd+C 时浏览器拼接 textContent · 与 diff 底色一致
- [x] E.4 字体：DiffLine span class `vs-diff-line-content` 继承 `var(--font-mono)`（MVP-08 styles.css）· shiki span 不改 font-family
- [x] E.5 无后缀文件：guessLanguageFromPath 无匹配返回 null → PlainTextChip 显示 + 纯文本（escape）渲染（PlainTextChip.test.tsx Dockerfile 用例验证）
- [ ] E.6 混合语言文件（如 `.vue`）· explicit skip "shiki 内置识别 · Phase D dev mode 实机验证 · 当前 Tier 1 不含 vue · v0.4+ Tier 2"
- [x] E.7 二进制文件：MVP-08 已有 "Binary file" 提示 · MVP-15 不处理（无二进制语法高亮 · 复用 MVP-08 binary 路径）

### F. 性能基准

- [ ] F.1 1MB JS 文件 diff 首屏 < 300ms（P99 · 5 次采样 · DevTools Performance）
- [ ] F.2 10MB 文件流式不阻塞：主线程 long task < 50ms（DevTools Performance · 3 次采样取 P99）
- [ ] F.3 主题切换 < 50ms（DevTools Performance · 3 次采样取 P99）
- [ ] F.4 LRU 缓存命中：同一文件再次打开 < 5ms（DevTools Performance · 3 次采样取 P99）
- [ ] F.5 内存占用：打开 10 个 1MB diff 文件后 · shiki cache + theme + parse 结果总内存 < 100MB（Chrome DevTools Memory 面板 snapshot）

### G. 边界 / 错误处理

- [ ] G.1 shiki 加载失败（网络断 / CDN 不可达）：降级纯文本 · console.error + UI chip "Plain text · 加载失败" · 不白屏
- [ ] G.2 语言识别错误（如 `.js` 被识别成 Java 因为文件内容像 Java）：允许误识别 · 用户不可手动纠正（v0.4+ 评估语言选择器）· 纯文本降级始终可用
- [ ] G.3 空文件：diff 显示空（MVP-08 已有）· syntax highlight 无内容可高亮 → 空显示 · 不崩溃
- [ ] G.4 单行超大文件（1 行 10MB）：按 100KB 分段截断显示 · 提示 "Line too long · truncated" · 不 freeze
- [ ] G.5 Web Worker 创建失败（浏览器禁用 Worker）：fallback `requestIdleCallback` · 仍满足 ≤ 16ms 主线程预算

---

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元（web）| shiki 适配层 + lazy loader + LRU cache + lang-map | `vitest` · `web/src/lib/shiki/__tests__/` |
| 集成 | shiki parse 结果与 DiffLine 组件结合 · token → span 渲染正确 | `vitest` + `testing-library` |
| E2E（Playwright）| golden path：打开 diff → 验证 syntax highlight 颜色 → 滚动加载 → 切换主题 | `web/tests/e2e/diff-syntax-highlight.spec.ts` |
| 性能 | 1MB / 10MB diff 首屏 benchmark | Playwright + `performance.now()` · 或 vitest bench |
| 视觉回归 | 5 主流语言 × 2 主题 = 10 screenshots | Playwright screenshot diff · baseline 存 `web/tests/e2e/snapshots/` |
| 手动 QA | 不支持语言（`.xyz`）· 无后缀文件 · 50MB 大文件 · 主题切换 | 手动 capture |

### C.1 · fixture 准备

所有性能测试 fixture 用预生成文本文件（不依赖真实 repo）：

```bash
# scripts/fixtures/generate-syntax-highlight-fixtures.sh
# 生成 1MB / 10MB / 50MB JS 文件（含典型语法结构）
# 生成 1MB Rust / Python / Go / Java / Markdown 文件
# 生成无后缀文件（名为 "Dockerfile" 实际为 shell 脚本）
# 生成 .xyz 文件（不支持语言）

# 用法：
# ./scripts/fixtures/generate-syntax-highlight-fixtures.sh
# 输出到 web/tests/fixtures/syntax-highlight/
```

### C.2 · vitest bench 模板

```typescript
// web/src/lib/shiki/__tests__/bench/highlighter.bench.ts
import { bench, describe } from 'vitest';
import { getHighlighter } from '../highlighter';

describe('shiki lazy load', () => {
  bench('parse 1MB JS file', async () => {
    const highlighter = await getHighlighter();
    const code = await fetch('/fixtures/1mb.js').then(r => r.text());
    await highlighter.codeToTokens(code, { lang: 'javascript', theme: 'github-light' });
  }, { time: 5 });

  bench('theme switch', async () => {
    const highlighter = await getHighlighter();
    // 预 parse 好的 tokens · 仅切换 theme
    highlighter.setTheme('github-dark');
  }, { time: 5 });
});
```

跑 `pnpm -C web bench` · P99 数字写入 PR description。

### C.3 · 视觉回归测试流程

```typescript
// web/tests/e2e/diff-syntax-highlight.spec.ts
import { test, expect } from '@playwright/test';

const LANGUAGES = ['javascript', 'typescript', 'rust', 'python', 'go'];
const THEMES = ['light', 'dark'];

for (const lang of LANGUAGES) {
  for (const theme of THEMES) {
    test(`${lang} ${theme} syntax highlight`, async ({ page }) => {
      await page.goto(`/diff?file=fixtures/1mb.${lang}&theme=${theme}`);
      await page.waitForSelector('.token-keyword', { timeout: 5000 });
      
      // 截图对比
      const diffView = page.locator('.diff-view');
      expect(await diffView.screenshot()).toMatchSnapshot(`${lang}-${theme}.png`);
      
      // 验证 token 颜色
      const keyword = page.locator('.token-keyword').first();
      const color = await keyword.evaluate(el => getComputedStyle(el).color);
      expect(color).not.toBe('rgb(0, 0, 0)'); // 不是默认黑色
    });
  }
}
```

### C.4 · E2E 性能测试流程

```typescript
// web/tests/e2e/diff-performance.spec.ts
import { test, expect } from '@playwright/test';

test('1MB JS diff first paint < 300ms', async ({ page }) => {
  await page.goto('/');
  
  const startTime = await page.evaluate(() => performance.now());
  await page.click('[data-testid="status-file-1mb.js"]');
  await page.waitForSelector('.diff-view', { timeout: 5000 });
  const endTime = await page.evaluate(() => performance.now());
  
  const duration = endTime - startTime;
  expect(duration).toBeLessThan(300);
});

test('10MB file scroll no long task', async ({ page }) => {
  await page.goto('/diff?file=fixtures/10mb.js');
  await page.waitForSelector('.diff-view');
  
  // 注入 PerformanceObserver 监听 long task
  await page.evaluate(() => {
    window.longTasks = [];
    const observer = new PerformanceObserver((list) => {
      window.longTasks.push(...list.getEntries());
    });
    observer.observe({ entryTypes: ['longtask'] });
  });
  
  // 滚动到底部
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  await page.waitForTimeout(2000);
  
  const longTasks = await page.evaluate(() => window.longTasks);
  const maxDuration = Math.max(...longTasks.map(t => t.duration), 0);
  expect(maxDuration).toBeLessThan(50);
});
```

---

## 💾 数据模型变更

无新 table · 无 schema 变更。

shiki 相关状态全部运行时内存（session 级别 · 关掉即丢）：

| 状态 | 位置 | 持久化 |
|------|------|--------|
| Theme 偏好（light/dark） | `app_settings` 表（MVP-03 已建）· key `"theme"` | ✅ 跨 session |
| shiki `Highlighter` 单例 | 前端内存（`web/src/lib/shiki/highlighter.ts`） | ❌ session |
| LRU cache（parse 结果） | 前端内存（`web/src/lib/shiki/cache.ts`） | ❌ session |
| 语言映射表 | 前端常量（`web/src/lib/shiki/lang-map.ts`） | ❌ 代码内 |

**禁止**：不在 rusqlite 缓存 shiki parse 结果（parse 是 CPU 密集型 · 缓存收益 < 序列化成本）。

**禁止**：不持久化 shiki theme / language WASM 到本地存储（每次 session 重新 load · 利用浏览器 HTTP cache）。

---

## ⚠️ 已知风险

- **R1 · shiki WASM 包大小**：每个 theme + 语言独立 wasm · 总 5MB+ · 首次 load 慢
  - mitigation：lazy load（按需下载语言 wasm）+ CDN cache + 浏览器 HTTP cache（v0.4 评估 service worker 预缓存）
  - 监控：DevTools Network 面板记录首次 load 总大小 · 若 > 3MB（单语言 + theme）→ 评估更细粒度 lazy load
- **R2 · 大文件 freeze**：10MB+ 单行渲染（无换行符）· shiki parse 可能阻塞主线程
  - mitigation：行级虚拟化 + 主线程 budget 检查（> 16ms 自动切 Worker）+ Web Worker 兜底
  - 监控：DevTools Performance long task 标记
- **R3 · shiki 版本升级 breaking change**：v3 → v4 可能改 API（`codeToTokens` 签名 / theme 格式）
  - mitigation：lock `shiki` 版本（`package.json` `^3.0.0` · 不自动升 major）+ 适配层隔离（`web/src/lib/shiki/highlighter.ts` 封装所有 shiki API 调用 · 升级时只改一处）
- **R4 · 不支持语言用户感知**：toast 太烦 · 静默又困惑
  - mitigation：UI 顶部细字 chip "Plain text"（hover 提示原因）· 不弹 toast · 用户明确知道"这是纯文本，不是 bug"
- **R5 · 主题切换时 token 颜色闪烁**：CSS variable 切换虽然快 · 但 browser repaint 可能导致瞬间颜色错乱（light token 在 dark 底色上）
  - mitigation：切换前预计算新 theme 的 token 颜色 → 批量更新 DOM `style` 属性（而非依赖 CSS variable 级联）→ 或切换时加 50ms `opacity: 0.5` transition 遮罩闪烁
  - 验收：DevTools 录屏确认切换过程无可见闪烁（肉眼观察 + 录屏逐帧检查）

---

## 📝 Notes

- MVP-15 是**纯前端装饰层** · 模式（shiki 浏览器端 lazy load + IntersectionObserver + LRU）和 MVP-08 后端完全隔离
- **语言覆盖扩展**：Tier 2 语言（Swift / Kotlin / C++ 等）推到 v0.4+ · 仅需在 `lang-map.ts` 加映射 + loadLanguage · 无架构改动
- **自定义 theme**：v0.4+ 评估为 Calm Studio 设计自定义 shiki theme（当前用 `github-light` / `github-dark` 近似）
- **shiki 服务端渲染（SSR）**：明确不做 · `§W21` 锁定浏览器端 lazy load · SSR 需 backend 引入 shiki → 增加 bundle + 复杂化 IPC
- **与 MVP-08 边界**：MVP-15 不改 MVP-08 已有 Diff 数据流（`DiffRequest` → backend `diff_compute` → `DiffResponse` → 前端渲染）· 仅在前端渲染层注入 syntax highlight
- **shiki bundle 体积监控**：Phase A 实施后必须跑 `pnpm -C web build` · 检查 `dist/assets/index-*.js` 增量 · 若 shiki WASM 导致总包 > 5MB → 评估更激进的分割策略（按语言拆 chunk）
- **内存泄漏防范**：`ShikiLazyLoader` 组件 unmount 时必须 `observer.disconnect()` · LRU cache 在 workspace 切换时清空（避免跨 workspace 泄漏）
- **无障碍（a11y）**：syntax highlight 不改变 DOM 语义 · ` DiffLine` 仍保持 `role="listitem"` · 屏幕阅读器不受影响
- **国际化**："Plain text" chip · "Large file" 提示文案走 i18n 系统（`web/src/i18n/`）· 不硬编码中文

---

## 🔗 相关

- `CLAUDE.md` #7 Diff 自建（不用 Monaco）· #13 Git 栈（不影响本 task · MVP-15 纯前端）
- ADR-007 Git 栈混用决策（MVP-15 不涉及 backend 改动）
- `implementation-plan.md` §10.1 v0.3 Diff 高级 · §W21
- 上游：MVP-08（Diff 基础视图 · 自绘 · Phase A-D done）
- 下游：无（自成一功能）
- 对应 `CLAUDE.md` 决策表：#7（Diff 自建 · [ADR-008](../adr/ADR-008-diff-renderer-custom.md)）

---

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)。所有 IPC struct 必须遵循 ADR-014 §规范 5 条 + H2 regression proof 6 步。

MVP-15 是**纯前端装饰层** · backend 不改 diff 计算逻辑 · IPC 需求极简。

### G.1 本 MVP 涉及的 IPC struct 清单

**决策：MVP-15 不新增 IPC command** · 复用 MVP-08 已有 binding · 仅在前端扩展字段。

| 已有 binding | MVP-15 使用 | 决策 | 理由 |
|---|---|---|---|
| `DiffRequest`（MVP-08） | 复用 · 前端调用时附加 `lang_hint?: string` | ✅ **复用** · 不新建 | MVP-08 已有 `file_path` · 前端可从后缀推导语言 · 作为 hint 传 backend（backend 可忽略） |
| `DiffResponse`（MVP-08） | 复用 · `DiffLine.content` 字段不变 | ✅ **复用** · 不新建 | syntax highlight 在前端解析 `content` → shiki tokens · backend 不感知 |
| `GitStatusResponse`（MVP-08） | 复用 · 文件列表来源 | ✅ **复用** | MVP-15 从 Status 面板点击文件 → 复用 MVP-08 链路 |

**实际新增 binding 数：0**（MVP-15 纯前端 · 无 backend 改动）。

> 若 Phase A 实施时发现必须 backend 提供 language guess（如根据文件内容用 `tree-sitter` 检测语言）· 则新增 1 个 IPC：
> - `LanguageDetectRequest` · `LanguageDetectResponse` · 但此方案需 Arbiter 评估 · 当前 spec 锁定前端自主检测（后缀 + 文件名映射）。

### G.2 若新增 IPC 的 derive 模板（预留 · 当前不实施）

```rust
// 仅当 Arbiter 批准 backend 语言检测时启用
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LanguageDetectRequest {
    pub workspace_id: String,
    pub file_path: String,
    pub content_sample: Option<String>, // 前 1KB 内容 · 用于内容识别
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LanguageDetectResponse {
    pub language: Option<String>, // "javascript" / "rust" / null
    pub confidence: f32,          // 0.0-1.0
}
```

### G.3 强制规范

- [ ] 所有 IPC struct + enum 必须 `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
- [ ] 前端**禁止**手写 `interface DiffRequest { ... }`——所有类型必须从 `./bindings/*` import
- [ ] `.prettierignore` 已排除 `web/src/bindings/`（防止 prettier 与生成格式冲突）

### G.4 H2 类 regression proof

复用 MVP-08 §G.4 模式 · 流程：

1. 临时在 `DiffLine` 的某个字段上加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'DiffLine'`——FAIL 证明防御生效
5. **回滚**：撤销 `#[ts(rename = ...)]` · 确认 `pnpm typecheck` 恢复 PASS

> 本 proof 只需做一次 · 结果写入 PR description 或 `docs/runtime-evidence/mvp-15/`。

### G.5 · 与上游已落地 binding 的复用决策

MVP-15 实施前必须明确复用 / 新增边界：

| MVP-08 已有 binding | MVP-15 使用场景 | 决策 | 理由 |
|---|---|---|---|
| `DiffRequest` | 触发 diff 计算 · MVP-15 附加 `lang_hint`（前端可选字段 · backend 忽略） | ✅ **复用** | 不改 Rust struct · 前端调用时加字段即可（TypeScript 允许额外字段） |
| `DiffResponse` | diff 结果 · `DiffLine.content` 被 shiki parse | ✅ **复用** | backend 不改输出 · 前端消费时扩展 |
| `DiffLine` | 每行内容 · MVP-15 前端解析 `content` → tokens | ✅ **复用** | 不新增字段 · tokens 是前端运行时计算 |
| `GitStatusResponse` | 文件列表来源 | ✅ **复用** | MVP-08 已有链路 |

### G.6 · MVP-15 新增 binding 清单

**实际新增：0 个**（MVP-15 纯前端 · 无 backend 改动）。

> 预留：若 v0.4+ 评估 backend 语言检测 → 新增 `LanguageDetectRequest` / `LanguageDetectResponse`（2 个 binding）。

---

## §H. 决策锁定（MVP-15 专有 · 防 v0.3 实施期反复讨论）

MVP-15 是**纯前端装饰层** · 对齐 `implementation-plan.md §W21`（2026-04-19 accepted）· 必须明确：

### H.1 技术栈：shiki v3+

**决策**：使用 `shiki` v3+ 作为语法高亮引擎（浏览器端）。

| 选项 | 优点 | 缺点 | v0.3 评估 |
|------|------|------|-----------|
| (a) **shiki v3+**（**v0.3 选定**） | 浏览器原生 WASM · 语言覆盖广 · VS Code 同款 grammar · 社区活跃 | WASM 包大（5MB+）· 首次 load 慢 | ✅ `§W21` 锁定 |
| (b) tree-sitter | 解析准确 · 可增量更新 · 适合大文件 | 浏览器端需 WASM + JS binding · 语言 grammar 维护成本高 · 未经 `§W21` 批准 | ⛔ 需先改 `§W21` + ADR |
| (c) highlight.js | 包小（~50KB）· 无 WASM | 语言覆盖少 · 解析质量低 · 无懒加载 | ❌ 不满足 Tier 1 覆盖 |
| (d) Prism.js | 包小 · 插件多 | 解析质量低 · 无 WASM · 语言覆盖少 | ❌ 不满足 |

**依据**：
- `§W21` 明确锁定 shiki · 不预 decision tree-sitter
- shiki v3 浏览器友好（WASM lazy load）· 支持 100+ 语言 · 与 VS Code 共享 TextMate grammar

### H.2 不碰的库

- **不碰 tree-sitter**：未经 `implementation-plan.md §W21` 批准 · 不试水
- **不碰 Monaco**：`CLAUDE.md §10.1` 硬禁区 · 3MB bundle 超限
- **不碰 Prism / highlight.js**：解析质量不满足 Tier 1 语言需求
- **不碰服务端渲染 shiki**：`§W21` 锁定浏览器端 lazy load

### H.3 渲染策略：先 diff 结构 + 后 syntax 高亮

**决策**：Progressive Enhancement — 先 render diff 结构（无高亮）→ 再对 viewport 内行 lazy load shiki。

| 选项 | 优点 | 缺点 | v0.3 评估 |
|------|------|------|-----------|
| (a) **先 diff + 后 shiki**（**v0.3 选定**） | 用户先看到内容 · 再变彩色 · 感知快 | 两次渲染（纯文本 → 高亮）· 有轻微 flicker | ✅ 满足 `§W21` 1MB < 300ms |
| (b) 等 shiki 全量 parse 完再 render | 无 flicker · 一次到位 | 首屏慢（1MB parse 可能 > 1s）· 用户白等 | ❌ 不满足硬指标 |
| (c) 后端 pre-parse | 前端零计算 · 直接渲染 tokens | 增加 IPC payload（tokens 数组比纯文本大 5-10x）· backend 引入 shiki | ❌ `§W21` 锁定前端 |

**依据**：
- `§W21` 硬指标 1MB < 300ms 要求先呈现
- MVP-08 已有 diff 渲染 < 200ms · 叠加 shiki lazy load viewport 内行 < 100ms · 总和 < 300ms

### H.4 缓存策略：LRU 100 文件 / 50MB

**决策**：运行时内存 LRU 缓存 · session 级别 · 关掉即丢。

| 选项 | 优点 | 缺点 | v0.3 评估 |
|------|------|------|-----------|
| (a) **LRU 内存缓存**（**v0.3 选定**） | 快（< 5ms 命中）· 实现简单 | session 丢失 · 重启后重新 parse | ✅ 复杂度优先 |
| (b) IndexedDB 持久化 | 跨 session 保留 · 二次打开更快 | 序列化开销大（tokens 数组 → IndexedDB）· 容量管理复杂 | ⏸ v0.4 评估 |
| (c) 不缓存 | 内存最小 | 重复 parse 浪费 CPU | ❌  unacceptable |

**依据**：
- parse 结果是纯内存对象 · IndexedDB 序列化收益 < 成本
- 100 文件 / 50MB 预算：单文件平均 500KB parse 结果 · 覆盖日常场景

### H.5 主题切换：CSS variable + data-attribute

**决策**：DOM 不重建 · 仅替换 `data-shiki-theme` 属性 · CSS variable 切换 token 颜色。

| 选项 | 优点 | 缺点 | v0.3 评估 |
|------|------|------|-----------|
| (a) **CSS variable + data-attribute**（**v0.3 选定**） | 快（< 50ms）· 滚动位置保留 · 无 flicker | 需要预定义所有 token 的 light/dark 颜色映射 | ✅ 满足 D.1 |
| (b) 重 parse + 重 render | 实现简单（直接调 shiki 换 theme） | 慢（> 1s for 1MB）· 滚动位置丢失 | ❌ 不满足 |
| (c) 两套 DOM 并存（light + dark） | 切换瞬时 | 内存翻倍 · DOM 膨胀 | ❌  unacceptable |

**依据**：
- shiki theme 对象预加载两套 · 切换时零网络请求
- CSS variable 重算由 browser 优化 · 比 JS 重 render 快 10x+

### H.6 Web Worker / requestIdleCallback：10MB+ 用 Worker · 1MB-10MB 用 idle callback

**决策**：分档处理 · 小文件主线程 · 大文件 Worker。

| 文件大小 | 策略 | 理由 |
|----------|------|------|
| < 1MB | 主线程直接 parse | 内容少 · parse < 16ms · 不阻塞 |
| 1MB - 10MB | `requestIdleCallback` 分 chunk | 内容中等 · idle 时间片解析 · 不抢用户交互 |
| > 10MB | Web Worker 分 chunk | 内容大 · 必须 offload · 主线程零阻塞 |

**依据**：
- 60fps = 16ms/frame · 主线程 parse 必须 ≤ 16ms
- Worker 通信开销 ~20ms · 仅在大文件时值得

### H.7 视觉回归：5 主流语言 × 2 主题 baseline screenshot

**决策**：Playwright screenshot diff · 10 张 baseline。

| 语言 | light theme | dark theme |
|------|-------------|------------|
| JavaScript | ✅ | ✅ |
| TypeScript | ✅ | ✅ |
| Rust | ✅ | ✅ |
| Python | ✅ | ✅ |
| Go | ✅ | ✅ |

**依据**：
- 覆盖 Tier 1 主流语言 · 验证 shiki grammar 正确性
- 2 主题验证 CSS variable 切换无颜色错乱
- baseline 存 `web/tests/e2e/snapshots/` · CI 自动 diff

### H.8 与 MVP-08 边界：MVP-15 不改 MVP-08 已有 Diff 数据流

**决策**：MVP-15 仅装饰层 · 失败时 Diff 仍可用。

| 场景 | MVP-08 责任 | MVP-15 责任 |
|------|-------------|-------------|
| Diff 计算（similar 算法） | ✅ | ❌ |
| Diff 渲染（split/unified + 行号 + 增删色） | ✅ | ❌ |
| Syntax highlight 叠加 | ❌ | ✅ |
| 主题切换（全局 light/dark） | ✅（框架级） | ✅（shiki theme 联动） |
| 大文件降级（> 1MB 提示） | ✅ | ❌ |
| 大文件语法高亮禁用（> 50MB） | ❌ | ✅ |

**依据**：
- MVP-08 已 production 可用 · MVP-15 不应破坏已有功能
- shiki parse 失败 → catch error → 纯文本降级（MVP-08 原有逻辑接管）

---

**自审四问**（2026-05-06 · OpenCode 详化）：

1. **递归完备性**：spec 自己在 spec 里吗？YAGNI 适用 spec 自己吗？
   - Spec 覆盖了 shiki lazy load + 大文件流式 + UI/视觉 + 性能 + 测试 + IPC + 决策锁定 + 风险 · 结构完整 ✅
   - YAGNI 适用：tree-sitter / word-level / LSP / Monaco 都明确推后 ✅
2. **反向场景**：
   - 50MB 文件 → 纯 diff 结构 · 语法高亮禁用 ✅
   - 无后缀文件 → 文件名匹配 → 失败则纯文本 ✅
   - 语言识别错误 → 允许误识别 · 纯文本降级始终可用 ✅
   - shiki 加载失败 → 降级纯文本 · 不白屏 ✅
   - 主题切换失败 → 保持当前主题 · 不崩溃 ✅
3. **边界适用性**：
   - 1KB / 1MB / 10MB / 50MB 文件全覆盖（C.1-C.6）
   - 单语言 / 混合语言（.vue）/ 二进制 / 空文件全覆盖（E.5-E.7）
   - 支持语言 / 不支持语言 / 无后缀语言全覆盖（B.5-B.6）
4. **YAGNI**：
   - tree-sitter → 明确推后（需改 `§W21` + ADR）
   - word-level diff → 明确推后（v0.3+ 不做）
   - LSP 语义高亮 → 明确推后（v0.3+ 不做）
   - 服务端渲染 shiki → 明确推后（`§W21` 锁定前端）
5. **对齐上游 binding**：MVP-08 `DiffRequest` / `DiffResponse` / `DiffLine` / `GitStatusResponse` 全部复用 · 新增 binding = 0（§G.5-G.6）
6. **§H 决策锁定全覆盖**：H.1 技术栈 / H.2 不碰列表 / H.3 渲染策略 / H.4 缓存策略 / H.5 主题切换 / H.6 Worker 分档 / H.7 视觉回归 / H.8 与 MVP-08 边界 · 防 v0.3 实施期反复讨论
7. **runtime evidence 路径已锁定**：§Phase D 明确 `docs/runtime-evidence/mvp-15/`（按 `.claude/rules/runtime-evidence-location.md` R1）

---

## 详化完成度评估（Arbiter 审 PR 时参考）

| 12 段必含 | 状态 | 备注 |
|----------|------|------|
| 1. frontmatter | ✅ | id / type / title / status:draft / depends_on / phase / estimate / plan_ref / risk_ref / reviewer 占位 |
| 2. 🎯 目标 Goal | ✅ | 一句话核心 + plan_ref link + 硬指标 |
| 3. 📖 背景 Context | ✅ | implementation-plan + CLAUDE.md + 路线图 W21 + 上游 MVP-08 已落地 |
| 4. 🛠 实施进度表 | ✅ | Phase A/B/C/D 拆分 + Phase A 起点 checklist |
| 5. 🎨 功能范围 Scope | ✅ | Do 6 项 / Don't 6 项 · 含 Tier 1/2 语言清单 |
| 6. 🖼 UI 引用 | ✅ | MVP-08 基础 + syntax highlight 叠加 + chip + 主题切换 |
| 7. ✅ Acceptance | ✅ | A-G 7 大组 / 35+ 项 checkbox · 每项含具体测法（P99 / 文件大小 / 语言） |
| 8. 🧪 测试策略 | ✅ | 单元 / 集成 / E2E / 性能 / 视觉回归 / 手动 QA + fixture 脚本 + bench 模板 |
| 9. 💾 数据模型变更 | ✅ | 无新表 · 运行时内存状态 + 持久化 theme 偏好 |
| 10. §G IPC Contract | ✅ | 0 新增 binding（纯前端）+ G.5 复用决策 + G.6 预留 |
| 11. §H 决策锁定 | ✅ | H.1-H.8 8 子段 · 含技术栈表 / 渲染策略表 / 缓存策略表 / 主题切换表 / Worker 分档表 / MVP-08 边界表 |
| 12. ⚠️ 已知风险 + Notes + 相关 + 自审四问 | ✅ | 5 风险 + 5 Notes + 6 相关 + 7 条自审 |

**完成度**：12/12 = **100%**（建议 Arbiter approve PR 后翻 status: ready）。

**遗留问题**：无 · 所有决策已锁定 · 没有"v0.3 启动后再讨论"的悬空项。
