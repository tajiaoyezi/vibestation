# 已知测试失败（Pre-existing · 非回归）

> 本文档记录仓库中**已知的、非回归性的**测试/lint 失败，避免每次跑测试时重复调查。
> 这些失败早于当前活跃开发，根因已确认，修法已记录但未执行（优先级/工作量原因）。

---

## 1. `tests/scripts/*.test.ts` · 已修复 vitest 环境配置（2026-06-15）· 剩余 10 个真实断言失败

**已修复**（PR #478）：vitest 主配置 `exclude` 掉 `tests/scripts/**`（不再假失败 "No such built-in module: node:"）+ 新增 `vitest.config.scripts.ts`（node 环境 · 无 solid plugin）+ `package.json` 加 `test:scripts` 命令。

**现状**：`pnpm vitest run`（前端套件）不再包含 scripts 测试。`pnpm test:scripts`（脚本套件）14 tests · 4 passed / 10 failed。

**剩余 10 个失败**（真实断言 · 非 transform 问题）：`spawnSync` 在测试环境的路径/退出码断言失败（可能是 CI 环境依赖 · 如 `validate-runtime-evidence` 需要真实的 runtime-evidence 目录结构）。后续单独排查。

**跑法**：
```bash
# 前端套件（不含 scripts）
pnpm --filter @vibestation/web test

# 脚本套件（独立 node 环境）
pnpm --filter @vibestation/web test:scripts
```

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
