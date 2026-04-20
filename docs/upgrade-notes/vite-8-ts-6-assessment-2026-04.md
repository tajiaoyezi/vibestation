# Vite 8 + TypeScript 6 Major Bump · 升级评估（2026-04-20）

## TL;DR

**PR #54（Vite 6→8）建议延后**。vite-plugin-solid 2.11.0 的 peerDependencies 虽声明支持 Vite 8，但内部仍调用 `transformWithEsbuild` —— 该 API 在 Vite 8 中被 deprecated，且 esbuild 不再作为 Vite 的默认依赖。当前 CI 的 `vite build` 直接报错 `Cannot find package 'esbuild'`。修复方案为显式添加 `esbuild` devDependency，但此 workaround 属于“为 deprecated API 兜底”，不如等 vite-plugin-solid 上游迁移到 `transformWithOxc` 后再升。

**PR #55（TS 5→6）建议延后**。TypeScript 6.0 将 `baseUrl` 标记为 deprecated（TS5101），直接命中 vibestation 的 `web/tsconfig.json`（`"baseUrl": "."`），导致 `tsc --noEmit` 失败。修复量小（移除 `baseUrl` + 将 `paths` 改为显式前缀），但属于“不改 tsconfig 就不能过 CI”的 breaking change。等下一个功能 PR 需要动 tsconfig 时顺便升，避免单独为 deprecation 开销一个 PR。

两个都不建议在当前状态下直接 merge。

---

## 1. 当前 web/ 栈

| 包 | 当前版本 | PR 要升到 | 备注 |
|---|---|---|---|
| vite | 6.0.3 | 8.0.8 | PR #54 dependabot |
| typescript | 5.6.2 | 6.0.3 | PR #55 dependabot |
| vite-plugin-solid | 2.11.0 | — | peerDep 已声明 Vite 8 兼容，但内部实现未跟进 |
| solid-js | 1.9.3 | — | 无 major bump |
| @tauri-apps/api | ^2 | — | 无 major bump |
| @tauri-apps/plugin-dialog | ^2.7.0 | — | 无 major bump |
| @tauri-apps/cli | ^2 | — | 无 major bump |

### CI 现状（2026-04-20）

PR #54 和 PR #55 的 **Frontend · pnpm lint + typecheck** 均失败：

- PR #54：`vite build` 失败（`transformWithEsbuild` / 缺 `esbuild`）
- PR #55：`tsc --noEmit` 失败（`baseUrl` deprecated → TS5101）

两个 PR 的 **Rust · clippy + fmt + test + Tauri build smoke** 也失败，但属于 CI 共享 job 的级联失败（Tauri build 依赖前端产物），非 Rust 代码本身问题。

---

## 2. Vite 6 → 8 breaking 分析

### 2.1 Vite 7.0 breaking（节选相关项）

| 变更 | 详情 | 对 Vibestation 命中 |
|------|------|-------------------|
| **Node.js 要求 20.19+ / 22.12+** | Vite 7 不再支持 Node 18（EOL） | ☐ no — CI 使用 Node 20+，本地 leaf 使用 Node 24 |
| **默认浏览器目标更新** | `'baseline-widely-available'` 取代 `'modules'`；Chrome 87→107, Firefox 78→104, Safari 14→16 | ☐ no — vibestation 显式设置 `"build.target": "es2022"`（vite.config.ts line 14） |
| **移除 Sass legacy API** | `css.preprocessorOptions.sass.api` 相关选项移除 | ☐ no — 无 Sass 依赖 |
| **移除 `splitVendorChunkPlugin`** | 该插件在 Vite 5.2.7 已 deprecated | ☐ no — 未使用 |
| **`transformIndexHtml` hook 格式变更** | `enforce`/`transform` → `order`/`handler` | ☐ no — 未自定义该 hook |
| **`optimizeDeps.entries` 改为 glob** | 不再接受字面路径 | ☐ no — 未使用该选项 |

- Vite 7 Migration Guide: https://v7.vite.dev/guide/migration
- Vite 7 CHANGELOG: https://github.com/vitejs/vite/blob/v7.0.0/packages/vite/CHANGELOG.md

### 2.2 Vite 8.0 breaking（节选相关项）

| 变更 | 详情 | 对 Vibestation 命中 |
|------|------|-------------------|
| **默认浏览器目标再次更新** | Chrome 107→111, Firefox 104→114, Safari 16.0→16.4 | ☐ no — 显式 `"build.target": "es2022"` |
| **Rolldown 成为默认 bundler** | 取代 esbuild（dev transform + dep optimize）+ Rollup（production build） | ⚠️ 间接影响 — 依赖优化和打包行为由 Rolldown 接管，但兼容性层自动转换大部分配置 |
| **Oxc 取代 esbuild 做 JS transform** | `esbuild` 选项 deprecated，自动转译为 `oxc` | ☐ no — vibestation 无自定义 `esbuild` 选项 |
| **Oxc Minifier 取代 esbuild minifier** | `build.minify: 'esbuild'` deprecated，需安装 esbuild 作为 devDep | ⚠️ 命中 — vibestation 使用 `"minify": "esbuild"`（vite.config.ts line 15），Vite 8 会自动 fallback 到 Oxc，但该选项本身已 deprecated |
| **Lightning CSS 取代 esbuild 做 CSS minify** | 默认使用 Lightning CSS | ☐ no — 无特殊 CSS minify 需求 |
| **CommonJS interop 一致性变化** | dev 和 build 的 `default` import 行为统一；可能导致 `Cannot read properties of undefined` | ⚠️ 低风险 — vibestation 依赖均为 ESM-first（solid-js、@tauri-apps/api），但需 build 后 smoke test 确认 |
| **`build.rollupOptions` → `build.rolldownOptions`** | 重命名 | ☐ no — 未使用 `rollupOptions` |
| **移除 `import.meta.hot.accept` URL fallback** | 需传递 id 而非 URL | ☐ no — 未使用 HMR API |
| **`transformWithEsbuild` deprecated** | 需单独安装 esbuild；推荐迁移到 `transformWithOxc` | ☑️ **直接命中** — vite-plugin-solid 2.11.0 内部调用 `transformWithEsbuild`，Vite 8 下 `vite build` 报错 `Cannot find package 'esbuild'` |
| **esbuild 不再为 Vite 默认依赖** | 变为 optional dependency | ☑️ **直接命中** — 同上，esbuild 不在 node_modules 中 |
| **部分 Rollup plugin hook 不再支持** | `shouldTransformCachedModule`、`resolveImportMeta`、`renderDynamicImport`、`resolveFileUrl` | ⚠️ 需确认 — vite-plugin-solid 是否使用这些 hook（从报错栈看未涉及） |

- Vite 8 Migration Guide: https://vite.dev/guide/migration
- Vite 8 CHANGELOG: https://github.com/vitejs/vite/blob/v8.0.8/packages/vite/CHANGELOG.md
- Vite 8 Announcement: https://vite.dev/blog/announcing-vite8

### 2.3 Vite 8 对 Vibestation 的直接影响评估

**命中项（需要修改才能过 CI）：**

1. **`transformWithEsbuild` / 缺 esbuild**（最高优先级）
   - 报错：`Error: Failed to load 'transformWithEsbuild'. It is deprecated and it now requires esbuild to be installed separately.`
   - 根因：vite-plugin-solid 2.11.0 的 renderChunk 阶段调用 `transformWithEsbuild`，而 Vite 8 不再自带 esbuild
   - 修复方案 A：在 `web/package.json` devDependencies 中添加 `"esbuild": "^0.25.0"`（或其他兼容版本）
   - 修复方案 B：等待 vite-plugin-solid 上游升级到 `transformWithOxc`
   - CI 日志证据：https://github.com/tajiaoyezi/vibestation/actions/runs/24643958809/job/72052975125

2. **`build.minify: 'esbuild'` deprecated**
   - vibestation 当前配置：`"minify": "esbuild"`（vite.config.ts line 15）
   - Vite 8 仍支持但 deprecated；长期应改为 `"minify": true`（Oxc 默认）或显式 `"minify": "terser"`
   - 修复量：1 行配置

**未命中项（无需修改）：**

- `build.target`、`server.port`、`clearScreen`、`plugins`、`paths` 等均不受影响
- 无 `rollupOptions`、`manualChunks`、`sass`、`ssr`、`legacy` 等配置
- 无自定义 Vite plugin（仅使用 vite-plugin-solid）

---

## 3. vite-plugin-solid 兼容性

### 3.1 peerDependencies 分析

```json
// vite-plugin-solid 2.11.0（当前安装版本）
{
  "vite": "^3.0.0 || ^4.0.0 || ^5.0.0 || ^6.0.0 || ^7.0.0 || ^8.0.0",
  "solid-js": "^1.7.2"
}
```

```json
// vite-plugin-solid 2.11.12（最新稳定版）
{
  "vite": "^3.0.0 || ^4.0.0 || ^5.0.0 || ^6.0.0 || ^7.0.0 || ^8.0.0",
  "solid-js": "^1.7.2"
}
```

```json
// vite-plugin-solid 3.0.0-next.5（最新预发布）
{
  "vite": "^3.0.0 || ^4.0.0 || ^5.0.0 || ^6.0.0 || ^7.0.0 || ^8.0.0",
  "solid-js": ">=2.0.0-beta.0 <2.0.0-experimental.0",
  "@solidjs/web": ">=2.0.0-beta.0 <2.0.0-experimental.0"
}
```

### 3.2 关键发现：peerDep 兼容 ≠ 实现兼容

虽然 2.11.0/2.11.12 的 `peerDependencies.vite` 已包含 `^8.0.0`，但**实现层面未跟进 Vite 8 的 API 变化**：

- 内部代码在 renderChunk 阶段调用 `transformWithEsbuild`（Vite 8 deprecated）
- 未使用新的 `transformWithOxc` API
- 这导致即使 peerDep 解析通过，`vite build` 仍会失败

### 3.3 上游状态

- vite-plugin-solid CHANGELOG: https://github.com/solidjs/vite-plugin-solid/blob/main/CHANGELOG.md
- 最新稳定版：2.11.12（2026-04 前后）
- 最新预发布：3.0.0-next.5（面向 Solid 2.0，与当前 solid-js 1.9.x 不兼容）
- **尚无正式版移除 `transformWithEsbuild` 依赖**（截至 2026-04-20）
- 判定：**peerDep 层面兼容 7/8，实现层面部分兼容** —— 需额外安装 esbuild 作为 workaround

### 3.4 结论

| 维度 | 评估 |
|------|------|
| peerDep 声明 | ✅ 已支持 Vite 8 |
| 实现兼容 | ⚠️ 部分兼容 — `transformWithEsbuild` 未迁移 |
| 是否需改 plugin 版本 | 否 — 2.11.0→2.11.12 无相关修复 |
| 是否需额外依赖 | 是 — 需显式安装 `esbuild` |
| 长期方案 | 等上游迁移到 `transformWithOxc` |

---

## 4. TypeScript 5 → 6 breaking 分析

### 4.1 TS 6.0 官方来源

- Announcing TypeScript 6.0: https://devblogs.microsoft.com/typescript/announcing-typescript-6-0/
- TS 6.0 Documentation: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-6-0.html
- TS 6.0 Gist Migration Guide: https://gist.github.com/privatenumber/3d2e80da28f84ee30b77d53e1693378f

### 4.2 默认行为变更（vibestation 已显式覆盖 = 安全）

| 变更 | TS 6.0 新默认 | vibestation 当前值 | 命中？ |
|------|--------------|-------------------|--------|
| `strict` | `true` | `"strict": true` | ☐ no — 已一致 |
| `module` | `esnext` | `"module": "ESNext"` | ☐ no — 已一致 |
| `target` | `es2025`（浮动） | `"target": "ES2022"` | ☐ no — 显式覆盖 |
| `moduleResolution` | `bundler` | `"moduleResolution": "Bundler"` | ☐ no — 已一致 |
| `noUncheckedSideEffectImports` | `true` | 未设置 | ⚠️ 可能产生新 error，需 `tsc --noEmit` 验证 |
| `libReplacement` | `false` | 未设置 | ☐ no — 无副作用 |
| `types` | `[]` | `["vite/client"]` | ☐ no — 已显式设置 |
| `rootDir` | `.`（tsconfig.json 所在目录） | 未设置 | ⚠️ 可能影响 emit 路径，但 `tsc --noEmit` 不 emit |

### 4.3 Deprecated 选项（vibestation 相关）

| 变更 | vibestation 使用情况 | 命中？ |
|------|---------------------|--------|
| **`baseUrl` deprecated** | `"baseUrl": "."`（tsconfig.json line 19） | ☑️ **直接命中** — `tsc --noEmit` 报错 TS5101 |
| `target: es5` deprecated | `"target": "ES2022"` | ☐ no |
| `moduleResolution: node` deprecated | `"Bundler"` | ☐ no |
| `module: amd/umd/system/none` deprecated | `"ESNext"` | ☐ no |
| `esModuleInterop: false` 禁止 | `"esModuleInterop": true` | ☐ no |
| `allowSyntheticDefaultImports: false` 禁止 | `"allowSyntheticDefaultImports": true` | ☐ no |
| `alwaysStrict: false` 禁止 | 未设置（默认 true with strict） | ☐ no |
| `outFile` removed | 未使用 | ☐ no |
| `downlevelIteration` deprecated | 未使用 | ☐ no |
| legacy `module` namespace syntax | 未使用 | ☐ no |
| `asserts` on imports deprecated | 未使用 | ☐ no |

### 4.4 `baseUrl` deprecation 详细分析

**报错（CI 日志）：**

```
tsconfig.json(19,5): error TS5101: Option 'baseUrl' is deprecated and will stop functioning in TypeScript 7.0.
```

**vibestation 当前配置：**

```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  }
}
```

**迁移方案（TS 6.0 推荐）：**

```json
{
  "compilerOptions": {
    // 移除 "baseUrl": "."
    "paths": {
      "@/*": ["./src/*"]  // 添加显式前缀
    }
  }
}
```

**工作量**：1 行删 + 1 行改，低。

### 4.5 其他潜在影响

- **`noUncheckedSideEffectImports` 默认 `true`**：可能导致 `import "some-polyfill"` 或 `import "./styles.css"` 被标记为 error（如果 TS 认为该 import 只有副作用且模块未声明）。vibestation 无此类 import，风险低。
- **`dom` lib 自动包含 `dom.iterable`**：vibestation 已显式包含 `"DOM.Iterable"`，无变化。
- **Temporal / `RegExp.escape` / `Map.getOrInsert` 新类型**：不影响现有代码。

---

## 5. 推荐

### 5.1 PR #54（Vite 6 → 8）

- **结论**：⏸️ **延后（wait-for-plugin-fix）**
- **理由**：
  1. vite-plugin-solid 2.11.0 内部调用 `transformWithEsbuild`，Vite 8 中该 API deprecated 且需单独安装 esbuild（[Vite 8 Migration Guide](https://vite.dev/guide/migration)）
  2. CI 直接报错 `Cannot find package 'esbuild'`（[CI 日志](https://github.com/tajiaoyezi/vibestation/actions/runs/24643958809/job/72052975125)）
  3. 虽然可通过添加 `esbuild` devDependency workaround，但这属于“为 deprecated API 兜底”，技术债务
- **风险**：中 — workaround 简单（+1 devDep），但引入对 deprecated API 的依赖，且 Rolldown/Oxc 的 CJS interop 变化可能在 production build 中产生运行时差异（需 smoke test）
- **若延后的 re-evaluation trigger**：
  - vite-plugin-solid 发布 2.12.x+ 移除 `transformWithEsbuild` 调用（迁移到 `transformWithOxc`）
  - 或：vibestation 功能 PR 中需要动 vite.config.ts 时，可顺手将 minify 改为 `"minify": true`（Oxc）+ 添加 esbuild workaround
  - 或：Vite 8.1+ 发布，观察社区迁移经验更丰富后再评估

### 5.2 PR #55（TypeScript 5 → 6）

- **结论**：⏸️ **延后（bundle-with-tsconfig-work）**
- **理由**：
  1. `baseUrl` deprecated（TS5101）直接命中当前 `web/tsconfig.json`，`tsc --noEmit` 失败（[CI 日志](https://github.com/tajiaoyezi/vibestation/actions/runs/24643963742/job/72052988945)）
  2. 修复简单（1 行删 + 1 行改），但属于“不改配置就不能过 CI”的强制变更
  3. 单独为一个 deprecation fix 开 PR 价值低，适合捆绑到下一个需要动 tsconfig 的功能 PR
- **风险**：低 — 修复明确，无副作用（`paths` 显式前缀是 TS 6.0 推荐做法）
- **若延后的 re-evaluation trigger**：
  - 下一个需要修改 `web/tsconfig.json` 的功能 PR（例如添加新 `types`、调整 `lib`、引入新路径 alias）
  - 或：dependabot 再次推送 TS 6 patch（6.0.4+）时，评估是否捆绑升级
  - 注意：长期建议不晚于 TypeScript 7.0 发布前完成迁移（`baseUrl` 在 TS 7.0 将彻底移除）

---

## 6. 附录 · 快速修复参考（供主 agent 后续使用）

### PR #54 若决定修复

```diff
// web/package.json devDependencies
+  "esbuild": "^0.25.0",
```

或修改 vite.config.ts：

```diff
  build: {
    target: "es2022",
-   minify: "esbuild",
+   minify: true,  // 使用 Vite 8 默认的 Oxc minifier
    sourcemap: false,
  },
```

### PR #55 若决定修复

```diff
// web/tsconfig.json
-   "baseUrl": ".",
    "paths": {
-     "@/*": ["src/*"]
+     "@/*": ["./src/*"]
    }
```

---

## 7. 参考链接

| 资源 | URL |
|------|-----|
| Vite 7 Migration Guide | https://v7.vite.dev/guide/migration |
| Vite 8 Migration Guide | https://vite.dev/guide/migration |
| Vite 8 Announcement | https://vite.dev/blog/announcing-vite8 |
| Vite CHANGELOG (main) | https://github.com/vitejs/vite/blob/main/packages/vite/CHANGELOG.md |
| Vite 8.0.0 CHANGELOG | https://github.com/vitejs/vite/blob/v8.0.0/packages/vite/CHANGELOG.md |
| vite-plugin-solid CHANGELOG | https://github.com/solidjs/vite-plugin-solid/blob/main/CHANGELOG.md |
| vite-plugin-solid npm | https://www.npmjs.com/package/vite-plugin-solid |
| TypeScript 6.0 Announcement | https://devblogs.microsoft.com/typescript/announcing-typescript-6-0/ |
| TypeScript 6.0 Release Notes | https://www.typescriptlang.org/docs/handbook/release-notes/typescript-6-0.html |
| TS 6.0 Gist Migration Guide | https://gist.github.com/privatenumber/3d2e80da28f84ee30b77d53e1693378f |
| PR #54 CI 失败日志 | https://github.com/tajiaoyezi/vibestation/actions/runs/24643958809/job/72052975125 |
| PR #55 CI 失败日志 | https://github.com/tajiaoyezi/vibestation/actions/runs/24643963742/job/72052988945 |
