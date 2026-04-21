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

## "3 样必交 + 1 样推荐" 归档位置（v2 简化 · 2026-04-21 · ADR-013）

每个 Spike accept 前必须确认以下位置齐全：

| # | 物料 | 位置 | 是否进 git | 级别 | 备注 |
|---|---|---|:---:|:---:|---|
| 1 | **决策文档** | `docs/spikes/SPIKE-XX-report.md` | ✅ | 🔴 必须 | 结论 / 数据 / v1→v2 追溯 / 瑕疵归属 |
| 2 | **实测源码** | `docs/spikes/code/SPIKE-XX/` | ✅ | 🔴 必须 | 含 `src/` + `Cargo.toml` + `Cargo.lock`（已 gitignore 白名单）+ `README.md` |
| 3 | **Raw 数据** | `docs/spikes/raw/SPIKE-XX/` | ✅ | 🔴 必须 | JSON / log / benchmark 输出 + `README.md` 索引 |
| 4 | **冷备**（含 build 产物） | `spike-tmp/archive/SPIKE-XX/` | ❌ | 🟡 推荐 | gitignored · 含 `target/` / 大 DB 测试文件 · 本地保留 · **Cargo workspace + Cargo.lock 进 git 的前提下可选** |

**3 样必须齐全 · accept 不成立若缺任一**。冷备（#4）是"推荐"而非"必须"· 原因见本文件 "冷备降级" 段。

## 冷备降级（v1 → v2 · session 13 audit M-1）

**2026-04-21 · session 13 audit M-1 · ADR-013**：

- v1 规则（session 7-12）· 冷备 `spike-tmp/archive/SPIKE-XX/` 为"必须"· 实测 9 个 Spike（SPIKE-01/02/03/04/04.5/05/05.5/06/08）只有 **2 个做到**（SPIKE-05 + SPIKE-06-pr2）· 合规率 22%
- v2 规则（本次修订）· 冷备降为"推荐"· 原因：
  - `Cargo.lock` + `src/` + `benches/` 已进 git · 任何机器 `cargo build --release` 可 byte-level 复现
  - 冷备的 `target/` build 产物仅省几分钟 build 时间 · 不增加信息量
  - 大 DB 测试文件若确需保留 · 可以独立归档到 `docs/spikes/raw/SPIKE-XX/` 进 git（用 git LFS 若 > 50MB）
  - 22% 合规率证明规则和现实不匹配 · 规则贬值

**什么时候还是要做冷备**（推荐场景）：

1. Spike 有 **> 100MB 的随机生成测试数据**（如 SPIKE-04.5 的 sqlite 测试 DB · 从 seed 生成需数分钟）· 冷备可省 re-generate 时间
2. Spike 涉及**外部工具二进制**（如 pre-built binary 不在 Cargo.toml）· 冷备保留二进制快照
3. Spike 有**非 Cargo 构建**（shell script / Makefile 自定义 build）· 冷备保留完整 build 环境

MVP 阶段冷备仍 **推荐做**（不强制）· 未来若出现"code + raw 进 git 但无法复现"的案例 · 重新评估升级回 v1 强制。

## 历史冷备欠账处理（v1 时期 7 个 Spike · session 13 audit）

**不追溯补齐**（audit M-1 决定）：

- SPIKE-01 / SPIKE-02 / SPIKE-03 / SPIKE-04 / SPIKE-04.5 / SPIKE-05.5 / SPIKE-08 无冷备
- 这 7 个的 `code/` + `raw/` 均已进 git · `Cargo.lock` 冻结 · 可用 `cargo build` 复现 benchmark
- 补做冷备成本高（需重新 build + tar + 本地保留）· 收益低（信息已在 git）
- 接受为已存在的技术债 · 不追溯补齐

**新 Spike**（未来）按 v2 规则做：3 样必须 + 冷备推荐（按上述 3 个场景判断是否做）。

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

### 4. 冷备 `spike-tmp/archive/SPIKE-XX/` · 🟡 推荐（v2 · 非必须）

- 完整 tarball 解压副本（含 target/ · 大 DB · 历史 build 产物）
- gitignored · 不进 repo
- 用途：若 git 归档被误删 / 复现需要 byte-level 一致的 build artifact · 从这里恢复
- **何时强烈推荐**：> 100MB 随机测试数据 · 外部二进制工具 · 非 Cargo 构建（见"冷备降级"段 3 场景）
- **何时可省略**：纯 Cargo 项目 · Cargo.lock 进 git · code + raw 已归档 · `cargo build` 可复现

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
- [ ] 决策文档 docs/spikes/SPIKE-XX-report.md 已入库（🔴 必须）
- [ ] 源码归档 docs/spikes/code/SPIKE-XX/ 已入库（含 Cargo.lock · 🔴 必须）
- [ ] Raw 数据 docs/spikes/raw/SPIKE-XX/ 已入库（🔴 必须）
- [ ] 冷备 spike-tmp/archive/SPIKE-XX/ 本地保留（🟡 推荐 · 若命中 3 场景则必做 · 见"冷备降级"段）
- [ ] Report 引用的每个数字都能在 raw 文件溯源（🔴 必须）
- [ ] clone 本 repo 后 · 在归档目录 cargo build 能复现 benchmark（🔴 必须）
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
