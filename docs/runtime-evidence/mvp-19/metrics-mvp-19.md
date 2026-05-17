# MVP-19 Phase E §F.4 · 性能测量记录

> **状态**：测量框架就位 · 实际数字待 Arbiter 本地 capture（Phase E 收口前）
> **测量基础设施**：performance.now()（若 Phase E 仪表化） + DevTools Performance 面板 + 手动 timing
> **生成时间**：MVP-19 Phase E playbook 同步（2026-05-17）
> **Spec 来源**：`docs/tasks/MVP-19-session-commit-binding.md` §F.4

---

## §F.4 性能验收建议（spec 原文）

- 绑定计算单次目标 < 20ms（本地 500 commit 样本）。
- Git Log 列表渲染徽章不引入全表重绘。
- Session 详情首次打开目标 < 200ms（缓存命中 < 80ms）。

---

## 测量手册（Arbiter 本地 10-15 min · 跑完后填实测数字）

### F.4.1 绑定计算单次 < 20ms（500 commit 样本）

**测量方法**（若后端已暴露 timing）：

- 在 `session_commit_links` 评分函数入口/出口加 `performance.now()`（或等价 Rust `Instant`）
- 用 500 commit 合成 fixture 跑 5 次 · 取 P99

**实测**：

| Run     | commit 样本 | 耗时 (ms)    | 备注        |
| ------- | ----------- | ------------ | ----------- |
| 1       | 500         | <TBD>        |             |
| 2       | 500         | <TBD>        |             |
| 3       | 500         | <TBD>        |             |
| 4       | 500         | <TBD>        |             |
| 5       | 500         | <TBD>        |             |
| **P99** | —           | **<TBD> ms** | 目标 < 20ms |

F.4.1 判定：<PASS_or_FAIL>

### F.4.2 Git Log 徽章渲染不引入全表重绘

**测量方法**：

- 打开含 100+ commit 的 Git Log
- DevTools Performance 录制 2s
- 观察 "徽章挂载/更新" 期间 React re-render 次数或 layout/reflow 波及范围
- 期望：仅局部 commit 行更新 · 不触发整个虚拟列表重绘

**实测**：

```
F.4.2 Run 1: 重绘范围 ___ 行 / 全表重绘? (yes/no)
F.4.2 Run 2: ...
F.4.2 Run 3: ...
```

Pass 判据：**no 全表重绘**（§F.4 "不引入全表重绘"）

F.4.2 判定：<PASS_or_FAIL>

### F.4.3 Session 详情首次打开 < 200ms（缓存命中 < 80ms）

**测量方法**：

- 冷启动详情（清缓存或新 session）
- 用 `performance.now()` 包 "点击徽章 → 详情 mount 完成"（或 DevTools User Timing）
- 再测一次（缓存命中）
- 各跑 3 次取 P99

**实测**：

| 场景     | Run 1 | Run 2 | Run 3 | P99   | 目标    |
| -------- | ----- | ----- | ----- | ----- | ------- |
| 首次打开 | <TBD> | <TBD> | <TBD> | <TBD> | < 200ms |
| 缓存命中 | <TBD> | <TBD> | <TBD> | <TBD> | < 80ms  |

F.4.3 判定：<PASS_or_FAIL>

---

**注意**：以上数字为 Phase E 收口 gate 必须项 · 实测后填入本文件 + 截图/录屏证据一并提交 `docs/runtime-evidence/mvp-19/`。

**自检**（填完后）：

```bash
grep -nE "<TBD>|<PASS_or_FAIL>" docs/runtime-evidence/mvp-19/metrics-mvp-19.md
# 应只剩未跑的占位 · 跑完后 0 处
```
