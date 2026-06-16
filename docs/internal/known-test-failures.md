# 已知测试失败（Pre-existing · 非回归）

> 本文档记录仓库中**已知的、非回归性的**测试/lint 失败，避免每次跑测试时重复调查。
> 这些失败早于当前活跃开发，根因已确认，修法已记录但未执行（优先级/工作量原因）。

---

## 1. `tests/scripts/*.test.ts` · vitest 环境不匹配（2 文件）

**症状**：
```
Error: No such built-in module: node:
Test Files  1 failed (1)
     Tests  no tests
```

**涉及文件**：
- `web/tests/scripts/setup-git-hooks.test.ts`
- `web/tests/scripts/validate-runtime-evidence.test.ts`

**根因**：这两个测试文件测的是 Node.js 脚本（`scripts/setup-git-hooks.mjs` / `scripts/validate-task-spec.mjs`），但 vitest 默认用 **jsdom 环境**（浏览器模拟），Rolldown 转换 `#!/usr/bin/env node` shebang + `node:` 内置模块导入时失败。

**建议修法**：在 `web/vitest.config.ts` 里把 `tests/scripts/**` 排除出 jsdom 环境（用 `test.environmentMatchGlobs` 或 `exclude` + 单独的 Node 环境测试配置）。工作量 ~1h，低风险。

**当前处理**：跑 `pnpm vitest run` 时忽略这 2 个文件失败。FEAT-02.5 §10 已记录。

---

## 2. `pnpm lint` · 94 files Prettier warnings

**症状**：
```
[warn] 94 files ... Code style issues found
```

**根因**：仓库历史遗留——大量 `.ts`/`.tsx`/`.css` 文件在 CRLF（Windows）+ 不同时期不同格式化下产生 prettier 偏差。**非任何单个 PR 引入**。

**已知 Windows CRLF 假失败**：部分文件在 Windows 上因 `.gitattributes` 的 `* text=auto eol=crlf` 与 prettier 的 `endOfLine: lf` 冲突（PR #452 已确认 #452 假失败）。

**当前处理**：
- PR 流程中只对**改动文件**跑 `npx prettier --check <files>`（单文件检查不受全局噪音影响）
- 全量 `pnpm lint` 的 94 files warn 是已知的 · 不阻塞 PR
- session 36 #439 曾部分清理 markdown-lint（行尾空格），但全量清理需批量操作 + 跨平台验证

**建议修法**：单独开一个 housekeeping PR 批量 `prettier --write`（需在 Linux/macOS 环境验证 EOL 一致性，避免 Windows CRLF 反复）。

---

## 3. SolidJS `cleanups created outside a createRoot` runtime warning

**症状**：浏览器 devtools console 出现：
```
cleanups created outside a `createRoot` or `render` will be ignored.
```

**根因**：SolidJS 内部 reactive primitive 在某些路径下于 root 外创建 onCleanup（可能是 module 级 listener 或 lazy init 时序）。需运行时断点定位，非静态可查。

**当前处理**：warning 不影响功能（cleanup 被忽略 = 该 listener 不会在 unmount 时清理，但 Provider 常驻挂载不 unmount，实际无泄漏）。标记为 follow-up。

**建议修法**：用 SolidJS DevTools 浏览器扩展定位 warning 来源，包裹 `createRoot` 或改 listener 注册时机。

---

## 如何使用本文档

跑测试/lint 遇到失败时，先查本文件：
1. 如果匹配已知项 → 确认是 pre-existing，不是你的改动引入的
2. 在 PR body 的 gate raw output 里注明「已知失败见 `docs/internal/known-test-failures.md`」
3. **不要**在 feature PR 里顺手修这些（除非该 PR 本身就是 housekeeping）

---

> 维护：发现新的 pre-existing 失败时追加到本文档。修好某个失败时从本文档删除 + 在 commit 里注明 closes 该项。
