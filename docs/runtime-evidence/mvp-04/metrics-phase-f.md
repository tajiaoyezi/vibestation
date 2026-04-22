# MVP-04 Phase F · runtime 证据与性能量化

## Runtime 证据

- `01-create-loading-card.png`：多 Tab 创建 + shell loading 态
- `02-rename-tab.png`：tab rename 提交后状态
- `03-switch-scrollback.png`：切回已有 tab 后的 scrollback 画面
- `04-close-tab.png`：关闭临时 tab 后剩余 tab 状态
- `05-performance-overlay.png`：页面内注入的性能角标（隐藏窗口直抓 + 延迟测量）

## 量化结果

### A.5 / E.2 · Tab switch latency（AX 自动化）

- 2-tab 稳态切换样本：`[12, 24, 20, 21, 20, 20, 24, 19]` ms
- median：`20 ms`
- 结论：满足 A.5 `< 100ms` 与 E.2 `< 50ms`

### E.4 · 主线程阻塞（页面内同步执行 + rAF 辅助采样）

- 干净版页面角标结果：
  - A.5/E.2 median：`0 ms`
  - A.5/E.2 p95：`0 ms`
  - E.4 sync max：`3 ms`
  - E.4 sync p95：`3 ms`
  - frame max：`19 ms`
- 说明：
  - `sync max / p95` 统计的是 `target.click()` 同步 JS 执行时长，更贴近 spec 的“单帧 JS 执行”口径
  - `frame max` 记录 `requestAnimationFrame` delta，包含 compositor / offscreen window 调度，不直接等同于纯 JS 执行时长
  - 以 `sync max = 3 ms` 判定 E.4 通过；`frame max = 19 ms` 作为附带上下文保留

## 当前判断

- 运行时证据已齐
- A.5 / E.2 已量化并通过
- E.4 已按更贴近 spec 语义的同步 JS 指标量化并通过（`sync max = 3 ms`）
