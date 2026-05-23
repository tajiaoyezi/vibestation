---
id: SPIKE-07.6
status: draft
---

# SPIKE-07.6 · Codex 错误事件 corpus 补强

> **类型**：SPIKE（technical validation · benchmark）
> **状态**：draft（待详化 → ready → in-progress → done）
> **依赖**：[SPIKE-07.5](./SPIKE-07.5-structured-mode-rerun.md) `done` · [ADR-018](../adr/ADR-018-ai-aware-r1-rejudge.md) accepted
> **创建**：2026-05-23（session 34 · Claude Code · cost-aware 最简 stub）
> **优先级**：低（可选 · 非 greenlight 阻断 · v1.0 vision audit-trail 加分项）
> **估时**：~2d（详化时校准）

---

## §A · 目标

补强 **codex CLI 错误事件解析准确率** · 覆盖 SPIKE-07.5 实跑无法覆盖的 codex auth / network 错误场景。

具体：

- 录制 codex CLI 在真 OpenAI API 路径下的 auth 错误 corpus（无效 API key / quota 耗尽 / rate limit）
- 录制 codex CLI 在 network 异常下的错误 corpus（无网络 / 超时 / DNS 失败 / TLS 握手失败）
- 跑 SPIKE-07.5 已建的 IR parser 验证非退化（标准 = 30/30 100%）+ 错误事件 emit 准确率

---

## §B · 现状（残留来源）

SPIKE-07.5 已实跑闭环 · R1 greenlight（[ADR-018](../adr/ADR-018-ai-aware-r1-rejudge.md) accepted 2026-05-16 · supersede ADR-017）· 36 结构化 corpus · 非退化 30/30 = 100% · claude 18/18 = 100% · panic 0。

**ADR-018 §G 残留**：codex 错误事件 corpus 当前**未覆盖 auth / network 路径**。SPIKE-07.5 实跑用 codex OAuth backend 模拟 · 真 OpenAI API key 路径下的错误事件 emit 行为未实测。

**影响评估**：

- claude 错误能力已证（SPIKE-07.5 §F.B）· codex 仅非退化场景覆盖
- v1.0 vision MVP-18/19/20 实施已 ship · 不依赖本 SPIKE 结果
- 本 SPIKE 是 **加分项 audit** · 不阻塞任何 ship gate

---

## §C · 依赖

| 项                                 | 来源                                                                                   | 状态               |
| ---------------------------------- | -------------------------------------------------------------------------------------- | ------------------ |
| SPIKE-07.5 IR parser + corpus 框架 | [docs/spikes/code/SPIKE-07.5/](../spikes/code/SPIKE-07.5/)                             | ✅ available       |
| Codex CLI 真 OpenAI API key 模式   | 需 Arbiter 提供（OPENAI_API_KEY env · 建议 ephemeral key 或 sub-account 限额）         | ⏳ pending         |
| Codex CLI auth 错误注入能力        | 需调研（codex `--simulate-auth-error` 类 flag · 或 `OPENAI_API_KEY=invalid` 直接探测） | 🔴 prep 阶段需调研 |
| codex network 异常注入             | OS 层 firewall / dnsmasq null route / `nc -l` 假 endpoint                              | 🔴 prep 阶段需调研 |

---

## §D · 实跑前置（详化时具体化）

ready-gate 翻 `ready` 前需补足以下字段：

- [ ] §D.1 真 OpenAI API key 获取路径（Arbiter 自定 · ephemeral key 或 sub-account 限额）
- [ ] §D.2 错误注入工具链选型（mock backend vs 真 endpoint · 安全 vs 真实度权衡）
- [ ] §D.3 录制 corpus 数量基线（建议 N=10-20 · auth 5-8 + network 5-12）
- [ ] §D.4 IR parser 扩展点（是否需要新增错误 event 类型 · 还是复用 SPIKE-07.5 已建 enum）
- [ ] §D.5 acceptance 阈值（非退化 = 30/30 不动 · 新 corpus 准确率目标定义）
- [ ] §D.6 实跑命令 reproducer（同 SPIKE-07.5 模式 · `cargo run --bin spike_07_6_runner ...`）

---

## §E · 范围（详化时具体化）

- **IN**：codex CLI auth / network 错误场景 corpus 录制 + IR parse + accuracy 评估
- **OUT**：claude CLI 错误（已 SPIKE-07.5 覆盖）· UI 集成（MVP-18 Phase C 已 ship）· 端到端 IPC（MVP-18/19/20 已 ship）
- **OUT**：codex CLI 任何非错误事件（已 SPIKE-07.5 §F 覆盖）

---

## §F · Deliverables

按 [.claude/rules/spike-delivery-checklist.md](../../.claude/rules/spike-delivery-checklist.md) v2 标准（ADR-013）：

| # | 物料     | 位置                               |  必须   |
| - | -------- | ---------------------------------- | :-----: |
| 1 | 决策文档 | `docs/spikes/SPIKE-07.6-report.md` | 🔴 必须 |
| 2 | 实测源码 | `docs/spikes/code/SPIKE-07.6/`     | 🔴 必须 |
| 3 | Raw 数据 | `docs/spikes/raw/SPIKE-07.6/`      | 🔴 必须 |
| 4 | 冷备     | `spike-tmp/archive/SPIKE-07.6/`    | 🟡 推荐 |

---

## §G · 自审四问（CLAUDE.md 📝 触发器）

- **递归完备性**：本 spec 自己在 spec 索引（tasks/README.md）吗？详化为 `ready` 时同步更新索引。✅
- **反向场景**：如果不做 SPIKE-07.6 · 风险是什么？答：codex 真 OpenAI API 错误 corpus 未覆盖 · v1.0 vision ship 不阻塞但 audit-trail 缺一角 · 未来用户报告 codex 错误事件 UX 问题时可能 debug 较慢。可接受。
- **边界适用性**：本 SPIKE 适用 codex CLI 唯一 backend 吗？答：仅 OpenAI API 路径 · OAuth backend 已 SPIKE-07.5 覆盖。✅
- **YAGNI**：现阶段真需要本 SPIKE 吗？答：不阻塞 ship · ADR-018 已 supersede ADR-017 · 本 SPIKE 是"audit 加分"性质 · 优先级低。✅

---

## §H · 估时

| Phase                     | 内容                                | 估时                  |
| ------------------------- | ----------------------------------- | --------------------- |
| prep                      | 真 API key + 错误注入工具调研       | 0.5d                  |
| 录制 corpus               | auth 5-8 + network 5-12 = 10-20 个  | 0.5d                  |
| IR parse + accuracy       | 复用 SPIKE-07.5 框架 · 跑 N=20      | 0.5d                  |
| Report + 归档（4 样齐全） | 按 spike-delivery-checklist v2 标准 | 0.5d                  |
| **总计**                  |                                     | **~2d**（详化时校准） |

---

## §I · 历史

- 2026-05-23 · session 34 · Claude Code（主 agent · cost-aware 最简 stub · 110 行）· status: `draft` · 待 Arbiter 自定窗口详化为 `ready`

---

> **本 SPIKE 是可选 audit 加分项 · 非 ship 阻塞 · Arbiter 后续可自由 prioritize 或永久 defer**。
>
> v1.0 vision ship gate 已通过（SPIKE-07.5 R1 greenlight + ADR-018 + MVP-18/19/20 全 done）· 本 spec 仅为 ADR-018 §G 残留留下结构化追踪入口 · 不实施也不影响 ship。
