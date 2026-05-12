# MVP-15 Phase D §G 边界 / 错误处理 · 运行时证据

## 平台信息

| 项目 | 值 |
|------|-----|
| 平台 | macOS |
| Node | 20.17.0 |
| pnpm | 9.15.9 |
| vitest | 4.1.5 |
| 测试时间 | 2026-05-12T20:55:40+08:00 |

## Acceptance 状态

| 项 | 状态 | 说明 |
|----|------|------|
| G.1 shiki 加载失败 → fallbackToPlainText + console.warn + chip | ✅ 4 cases | `shiki-load-failure.test.ts` · mock `createHighlighterCore` reject · 验证降级路径 |
| G.2 语言识别错误 · 误识别 .js 为 java | ⏸️ skip | spec L243 explicit skip · 用户不可手动纠正 · v0.4+ 评估 |
| G.3 空文件 → 空显示不崩溃 | ✅ 5 cases | `empty-file.test.ts` · 空字符串 / 空白 / 换行 / 缓存 |
| G.4 单行超大文件 → 截断 + 提示 | ⚠️ 暴露缺陷 | `single-large-line.test.ts` · 4 cases · **Phase C 未实施单行截断逻辑** · 留 v0.3 sprint fix track |
| G.5 Worker 创建失败 → fallback idleCallback | ✅ 4 cases | `worker-disabled-fallback.test.ts` · mock `highlightInWorker` reject · 验证 idle fallback |

## 测试文件清单

```
web/tests/utils/shiki/edge-cases/
├── shiki-load-failure.test.ts      # G.1 · 4 cases
├── empty-file.test.ts               # G.3 · 5 cases
├── single-large-line.test.ts        # G.4 · 4 cases（含缺陷暴露）
└── worker-disabled-fallback.test.ts # G.5 · 4 cases
```

## 缺陷记录

### G.4 截断逻辑缺失

**当前行为**：`scheduler.ts` 仅按 `fileSize` 参数分三档（<1MB / 1-10MB / >=10MB），未对单行长度做截断。

**期望行为**：spec G.4 要求单行 10MB → 按 100KB 分段截断 + "Line too long · truncated" 提示。

**建议 fix track**：在 `scheduleHighlight` 中加入代码长度检查，若单行超过 `CHUNK_SIZE_BYTES`（100KB），截断并附加提示 chip。

## vitest 原始输出

```
$ pnpm -C web exec vitest run tests/utils/shiki/edge-cases/

 RUN  v4.1.5 /private/tmp/MVP-15-phase-D-edge-work/web

 Test Files  4 passed (4)
      Tests  17 passed (17)
   Start at  20:55:40
   Duration  2.55s (transform 116ms, setup 0ms, import 324ms, tests 2.08s, environment 1.49s)
```
