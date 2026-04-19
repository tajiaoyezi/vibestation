# Spike 交付物归档 Checklist（项目级）

> 本规则是全局 [`~/.claude/rules/13-cross-agent-delivery.md`](~/.claude/rules/13-cross-agent-delivery.md) 在 vibestation 项目的具体落地。
> 触发场景：任何 Spike（无论主 agent 自己跑 · 还是下发给 opencode / codex / 其他 agent 跑）accept 前 · 主 agent 必须完整过此 checklist。

---

## 触发条件

本 checklist 触发于：

- 任何 Spike 的 review accept 瞬间（主 agent 做出"结论合规"判定的同一个动作内）
- Spike PR 开出之前（PR body 的 Test Plan 必须勾选下列检查项）
- Spike 任务 `status: done` 翻转之前（spec frontmatter 改动提交之前）

三个时间点缺一不可。

## "4 样齐全" 归档位置（强制）

每个 Spike accept 前必须确认以下 4 样都在正确位置：

| # | 物料 | 位置 | 是否进 git | 备注 |
|---|---|---|:---:|---|
| 1 | **决策文档** | `docs/spikes/SPIKE-XX-report.md` | ✅ | 结论 / 数据 / v1→v2 追溯 / 瑕疵归属 |
| 2 | **实测源码** | `docs/spikes/code/SPIKE-XX/` | ✅ | 含 `src/` + `Cargo.toml` + `Cargo.lock`（已 gitignore 白名单）+ `README.md` |
| 3 | **Raw 数据** | `docs/spikes/raw/SPIKE-XX/` | ✅ | JSON / log / benchmark 输出 + `README.md` 索引 |
| 4 | **冷备**（含 build 产物） | `spike-tmp/archive/SPIKE-XX/` | ❌ | gitignored · 含 `target/` / 大 DB 测试文件 · 本地保留 |

**缺任一项 · accept 不成立**。

## 每样物料的具体要求

### 1. 决策文档 `docs/spikes/SPIKE-XX-report.md`

- 必须包含：Pass/Fail 判定 · 原始数据引用（指向 raw 文件）· 结论（Fallback 路径选择）
- Report 引用的每个具体数字 · 必须能在 `docs/spikes/raw/SPIKE-XX/` 文件内溯源
- 若有 v1→v2 迭代 · 必须在同一 report 追溯 v1 被 BLOCK 的原因

### 2. 实测源码 `docs/spikes/code/SPIKE-XX/`

目录结构：
```
SPIKE-XX/
├── Cargo.toml · Cargo.lock        # Cargo.lock 必须在（版本冻结 · 保证可复现）
├── src/                           # 所有实测源码
├── benches/                       # Criterion bench（如有）
└── README.md                      # 必须补齐（模板见 SPIKE-03/04 README）
```

README 必须包含字段：
- **来源**：交付 agent + 产出时间 + review accept 人
- **原始归档**：对应 `spike-tmp/archive/` 路径
- **复现命令**：`cd ... && cargo build --release && ./target/release/...`
- **关键结论溯源**：report 里每个大数字在本目录代码的哪个函数 / raw 的哪个字段

### 3. Raw 数据 `docs/spikes/raw/SPIKE-XX/`

- 原始 benchmark 输出（JSON / log / txt）
- 对应 `README.md` 做字段索引 · 让 report 的数字可溯源

### 4. 冷备 `spike-tmp/archive/SPIKE-XX/`

- 完整 tarball 解压副本（含 target/ · 大 DB · 历史 build 产物）
- gitignored · 不进 repo
- 用途：若 git 归档被误删 / 复现需要 byte-level 一致的 build artifact · 从这里恢复

## Review accept 的原子性

反模式：
> "先 accept · 明天再归档代码"

**正确做法**：review accept 必须在同一个主 agent 动作内完成以下全部步骤 · 不能拆分到下一个 session：

```
1. 对照 spec 判定 Pass/Fail
2. 决策文档入库       → git add docs/spikes/SPIKE-XX-report.md
3. 源码归档          → cp ... docs/spikes/code/SPIKE-XX/
4. Raw 数据归档       → cp ... docs/spikes/raw/SPIKE-XX/
5. 冷备               → cp -R ... spike-tmp/archive/SPIKE-XX/
6. ADR 翻转          → 同 PR 或独立 PR
7. Spec done 翻转    → 独立评审后

任一步骤中断 · session 结束前必须补全 · 不允许跨 session 遗留
```

## Spike PR Test Plan 必填项

所有 Spike 相关 PR（含 report + code + raw 归档的 PR）· body 的 Test Plan 必须显式包含：

```markdown
- [ ] 决策文档 docs/spikes/SPIKE-XX-report.md 已入库
- [ ] 源码归档 docs/spikes/code/SPIKE-XX/ 已入库（含 Cargo.lock）
- [ ] Raw 数据 docs/spikes/raw/SPIKE-XX/ 已入库
- [ ] 冷备 spike-tmp/archive/SPIKE-XX/ 本地保留（gitignored）
- [ ] Report 引用的每个数字都能在 raw 文件溯源
- [ ] clone 本 repo 后 · 在归档目录 cargo build 能复现 benchmark
```

独立评审者（Arbiter 或另一 agent）必须亲自验证上述每一项 · 不勾即 block merge。

## Session 结束前的 "Spike 物料出厂检查"

任何 session 结束前（包括交付 `/save-session`）· 若本轮有 Spike accept · 强制过一遍：

- [ ] 所有 accept 的 Spike 代码都在 `docs/spikes/code/` 下？
- [ ] 所有 raw 数据都在 `docs/spikes/raw/` 下？
- [ ] `/tmp/spike-*/` 里还有没有未归档的产出？
- [ ] 下一个 session 新 agent 只看 repo 能接手吗？

任一答 NO · 本 session 不得结束 · 立即补归档。

## 反模式（避免）

| 反模式 | 正确做法 |
|--------|---------|
| "report 就是交付物 · 代码本来就是丢弃的" | Decision-grade Spike 的代码是**证据** · 地位等同 report |
| "交付物在 /tmp 也能读 · 暂时不归档" | /tmp 3 天清 · 即使能读也必须立即归档 |
| 把"归档代码"作为独立 task 放下个 sprint | accept 和归档是同一个原子动作 · 不能拆 |
| Cargo.lock 进不进 git 纠结 | 归档目录的 Cargo.lock **必须**进 git（版本冻结是归档诉求）· `.gitignore` 已有白名单 |
| 只归档"成功版本"（v2）· 失败版本（v1）彻底删 | v1 留在 `spike-tmp/archive/` 作证据链 · report 里引用 v1 失败原因 · 但不进 repo |
| 未来 agent 看不到代码就靠 "/tmp 里还在的话" | /tmp 不可依赖 · 未来 agent 看到的只有 repo |

## 相关规则

- [全局] `~/.claude/rules/13-cross-agent-delivery.md`：跨 agent 协作的交付物持久化通用原则
- [全局] `~/.claude/rules/09-task-workflow.md`：任务完成后验收流程
- [项目] `CLAUDE.md` 📝 自审四问：本 checklist 也适用于自身（递归完备性）
- [项目] `docs/tasks/_template.md` §Deliverables：Spike spec 模板已内嵌本 checklist

## 适用范围

- ✅ SPIKE-04.5 / SPIKE-05 / SPIKE-06 / 未来所有 Spike
- ✅ 任何类型 Spike（benchmark / safety / protocol reverse / UI prototype / ...）
- ✅ 任何交付来源（主 agent 自跑 / opencode / codex / cursor / 其他）
- ⚠️ 非 Spike 类任务（MVP / BUG / FEAT）另有归档流程 · 但核心原则（代码即证据 · /tmp 不持久）仍适用
