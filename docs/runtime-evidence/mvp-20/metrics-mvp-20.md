# MVP-20 Phase E §F.1 / §L · 性能与跨平台测量记录

> **状态**：测量框架就位 · 实际数字待 Arbiter 本地 capture（Phase E 收口前）
> **测量基础设施**：Criterion bench（`cargo bench -p vibestation-core --bench rollback`）+ GitHub Actions Linux runner
> **生成时间**：MVP-20 Phase D/E playbook 同步（2026-05-17）
> **Spec 来源**：`docs/tasks/MVP-20-ai-one-click-rollback.md` §F.1（Criterion）+ §L.1（macOS vs Linux）

---

## §F.1 Criterion 性能验收（spec 原文）

- 单 commit revert P99 < 100ms
- 5 commit session revert P99 < 500ms
- 20 commit session revert P99 < 2s

---

## 测量手册（Arbiter 本地 15-20 min · 跑完后填实测数字）

### F.1.1 单 commit revert P99

**测量方法**：

```bash
cargo bench -p vibestation-core --bench rollback -- single
```

**实测**（5 run）：

| Run     | 耗时 (ms)    | 备注         |
| ------- | ------------ | ------------ |
| 1       | <TBD>        |              |
| 2       | <TBD>        |              |
| 3       | <TBD>        |              |
| 4       | <TBD>        |              |
| 5       | <TBD>        |              |
| **P99** | **<TBD> ms** | 目标 < 100ms |

F.1.1 判定：<PASS_or_FAIL>

### F.1.2 5 commit session revert P99

**测量方法**：

```bash
cargo bench -p vibestation-core --bench rollback -- 5-commit
```

**实测**（5 run）：

| Run     | 耗时 (ms)    | 备注         |
| ------- | ------------ | ------------ |
| 1       | <TBD>        |              |
| ...     | <TBD>        |              |
| **P99** | **<TBD> ms** | 目标 < 500ms |

F.1.2 判定：<PASS_or_FAIL>

### F.1.3 20 commit session revert P99

**测量方法**：

```bash
cargo bench -p vibestation-core --bench rollback -- 20-commit
```

**实测**（5 run）：

| Run     | 耗时 (ms)    | 备注      |
| ------- | ------------ | --------- |
| 1       | <TBD>        |           |
| ...     | <TBD>        |           |
| **P99** | **<TBD> ms** | 目标 < 2s |

F.1.3 判定：<PASS_or_FAIL>

---

## §L.1 跨平台 smoke 对照表（spec 原文）

| 行为                                             | macOS（本机） | Linux Ubuntu 24（runner） | 差异说明 |
| ------------------------------------------------ | ------------- | ------------------------- | -------- |
| `git revert` 顺序执行                            | <PASS/FAIL>   | <PASS/FAIL>               |          |
| 文件锁 / 并发 revert                             | <PASS/FAIL>   | <PASS/FAIL>               |          |
| `Repository::cleanup_state()` + REVERT_HEAD 恢复 | <PASS/FAIL>   | <PASS/FAIL>               |          |
| 路径大小写敏感（冲突场景）                       | <PASS/FAIL>   | <PASS/FAIL>               |          |
| SQLite WAL 模式下 DB 一致性                      | <PASS/FAIL>   | <PASS/FAIL>               |          |
| 冲突解决后 Continue 续跑                         | <PASS/FAIL>   | <PASS/FAIL>               |          |

**总判定**：<PASS_or_FAIL · 待 Arbiter 跑完填>

---

**注意**：以上数字 + 表格为 Phase E 收口 gate 必须项 · 实测后填入本文件 + 截图/录屏证据一并提交 `docs/runtime-evidence/mvp-20/`。

**自检**（填完后）：

```bash
grep -nE "<TBD>|<PASS_or_FAIL>" docs/runtime-evidence/mvp-20/metrics-mvp-20.md
# 应只剩未跑的占位 · 跑完后 0 处
```
