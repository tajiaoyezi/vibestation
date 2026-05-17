# MVP-19 Phase E · Capture Playbook · Arbiter ~30-45 min 收口

> **目标**：按本 playbook 一次跑完 · 产出 **8-12 张截图（01-08 GitLog 徽章各态 + 详情视图 + 解绑 modal + 改绑 superseded + 脱敏示例 + a11y 焦点）+ 30s 录屏（键盘 + 模态流）+ filled metrics（§F.4 性能数字）** · 主 agent 据此把 MVP-19 spec Phase E.5 收口并为后续 done gate 准备证据。
>
> **触发条件**：MVP-19 Phase C/D 实施基本可用后 · Arbiter 执行窗口 · 按 §I.5.5 "runtime evidence 打包归档" 要求 · 覆盖 §D/§E/§M GUI + 安全 + a11y + 性能。
>
> **预计时间**：30-45 min（5 min 前置 + 20-30 min 实测 + 5 min commit + prettier）。
>
> **关联**：
>
> - spec：[`docs/tasks/MVP-19-session-commit-binding.md`](../../tasks/MVP-19-session-commit-binding.md)（§D UI wireframe / §E Acceptance / §I.5 Phase E / §M 脱敏 / §F.4 性能）
> - metrics：[`metrics-mvp-19.md`](./metrics-mvp-19.md)
> - ADR-011 + `.claude/rules/runtime-evidence-location.md`（R1-R5 硬规则）

---

## 🛡 7 Invariant（playbook design intent · spec 安全 + provenance 完整性）

未来改 playbook 任何段前必先对照本 7 条 · 防 redaction 泄露 / 状态遗漏 / a11y 假 PASS。

### I1 · precondition 必须显式验证 · 不能假设

- 应用：每段 capture 前必须有 "确认 workspace 有 ≥3 commits + 至少 1 个已绑定 session + 1 个 pending/low-conf" 的步骤 · 否则 01 徽章截图可能抓到空状态。

### I2 · spec acceptance 不能在 playbook 标"可选"

- 应用：§E4.4 "stale / pending / low-confidence 状态均有可见标记" · §E7.4 "脱敏失败时显示受限提示" · 任何一项 playbook 必须列为 mandatory · 不能跳过。

### I3 · evidence 必须直接演示 spec 描述的真实行为

- 应用：截 "点击徽章进入详情并定位 commit"（§D.2 + E4.2）时必须同时截详情页里该 commit 被高亮/anchor · 不能只截 GitLog 徽章。

### I4 · redaction / 安全路径必须 fail-closed（§M + E7）

- 应用：所有含敏感输入的 fixture（Bearer sk- / /Users/xxx）在脱敏后截图必须同时截 "原文不可见 + [REDACTED_*] 可见" + 错误路径下受限提示（E7.4）。

### I5 · a11y 必须可执行验证（§D.4 + E8）

- 应用：键盘 Tab 顺序、focus trap（modal）、Esc 关闭、aria-label 读屏模拟 · 必须有可重复步骤 · 颜色不是唯一状态（徽章 pending/stale 用 icon+text+色）。

### I6 · stale / pending / superseded 状态必须有独立证据（§D.2 + E4.4 + E5.5）

- 应用：§D.2 列出的 6 种徽章态 + §E5.5 改绑后 superseded 必须逐个有截图/录屏 · 不能只测 happy path confirmed。

### I7 · 每项 capture 步骤必须有 spec 锚点 + 可量化 pass 判据

- 应用：写完后自检 grep 必须能定位到 §D.x / §E.x / §M.x · pass 条件必须可执行（"徽章可见 + hover tooltip 含 title + 点击后详情 commit 列表首项匹配"）。

**自检**（写完 playbook 后立即跑）：

```bash
# I7 自检（每步必须有 spec 锚点）
grep -nE "§D\.|§E\.|§M\." docs/runtime-evidence/mvp-19/CAPTURE-PLAYBOOK.md | wc -l
# 应 ≥ 15 处（覆盖徽章/详情/ modal/脱敏/a11y）
```

---

## 0 · 前置准备（5 min）

```bash
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation

# 0.1 确认 main 最新（不在 feature 分支）
git checkout main && git pull --ff-only origin main

# 0.2 创新 branch 收 evidence（不污染主工作树）
git checkout -b docs/mvp-19-phase-e-capture

# 0.3 确认依赖
node --version  # ≥ 20
pnpm --version  # 9.x
cargo --version

# 0.4 清理旧 evidence（保留 playbook + metrics · 删 PNG/MOV）
rm -f docs/runtime-evidence/mvp-19/*.png docs/runtime-evidence/mvp-19/*.mov 2>/dev/null || true
mkdir -p docs/runtime-evidence/mvp-19

# 0.5 准备 dev 模式（Phase E 需要真实交互 + DevTools 看 timing）
pnpm install
```

---

## 1 · 启动 dev mode + 验证 MVP-19 起点（5 min · I1）

```bash
pnpm tauri:dev
```

等窗口就绪（约 5-15s）。

**1.1 验证起点 fixture（必须有可绑定数据）**：

- 当前 workspace 至少有 3 个 commit（本地或测试 repo）
- 已存在至少 1 个 active/ended session（通过 W1/W2 实施或手动触发）
- Git Log 面板可见（MVP-18 已就位）

若无足够数据：用测试 CLI 快速产生 2-3 个 commit + 触发 1 个 session（或等 Phase C/D 提供 fixture）。

**❌ 如果 Git Log 无 session 徽章区域** · 说明 Phase C 未就绪 · 本 playbook 仍可按 §D 规格描述应呈现内容 · 报告主 agent。

---

## 2 · 截图 + 录屏清单（20-25 min · 覆盖 §D/§E/§M 关键面）

**推荐完全手动**（避免自动化在 badge 状态 / modal 焦点上的 timing 问题）。

```bash
OUT=docs/runtime-evidence/mvp-19
WID=$(osascript -e 'tell application "System Events" to id of front window of (first process whose name contains "Vibestation")')
```

### 2.1 Git Log 徽章 6 态（§D.2 + E4.1/E4.4 · I6）

- **01-gitlog-confirmed.png**  
  截图：commit 行右侧显示主徽章（confirmed 态）  
  Pass 判据：徽章可见 + 颜色/图标区分 + hover tooltip 含 session title + 时间 + 置信度（§D.2）  
  Spec 锚点：§D.2 "每条 commit 右侧最多显示 1 个主徽章"

- **02-gitlog-pending.png**  
  截图：低置信 commit 显示 `pending` 小点或弱化徽章  
  Pass 判据：pending 状态有可见标记（非 silent）（§E4.4）

- **03-gitlog-stale.png**  
  截图：session 损坏时徽章 `stale` 态  
  Pass 判据：stale 可见 + 点击进入故障说明（§D.2 异常行为）

- **04-gitlog-plusN.png**  
  截图：多候选时显示 `+N` 次级标记  
  Pass 判据：+N 存在且不遮挡主徽章

- **05-gitlog-weak-confidence.png**  
  截图：置信度低于阈值时的弱化样式 + tooltip  
  Pass 判据：弱化样式 + tooltip 解释（§D.2）

- **06-gitlog-no-link.png**（或 pending 初始）  
  截图：commit 尚未判定时的小点

### 2.2 反查跳转 + 详情视图（§D.1 + §D.2 + E4.2/E4.3）

- **07-detail-from-badge.png**  
  录屏/截图序列：GitLog 徽章 click → Session 详情打开 + 自动定位到该 commit  
  Pass 判据：详情 Header 显示 session 标题/起止/状态（active/ended/idle-cutoff/manual-ended）· Commit panel 里该 commit 被 anchor/highlight（§D.1）

- **08-detail-full.png**  
  截图：完整详情视图（Header + Summary strip + Timeline + Commit panel + Actions）  
  Pass 判据：5 种状态说明可见 · Summary 含 commit/文件/置信均值 · Timeline 展示输入/绑定/解绑事件（§D.1）

### 2.3 解绑 / 改绑 Modal（§D.3 + E5.1/E5.4/E5.5）

- **09-unbind-modal.png**  
  截图：从徽章右键或详情 "解绑" 触发 modal  
  Pass 判据：modal 含 commit sha + session 标题 + reason 输入 + Cancel / Unbind / Unbind and recalc 三按钮（§D.3）

- **10-rebind-superseded.png**  
  截图序列：改绑到另一 session 后 · 旧 link 在详情中状态变为 `superseded`  
  Pass 判据：superseded 可见 + 审计记录存在（§E5.5）

### 2.4 脱敏 + 红线（§M.5 + E7.1/E7.4 · I4）

- **11-redaction-token.png**  
  Fixture：含 `Bearer sk-live-...` 的摘要  
  截图：详情页显示 `[REDACTED_TOKEN]`（或等价）· 原文不可见  
  Pass 判据：与 §M.5 示例一致

- **12-redaction-path.png**  
  Fixture：含 `/Users/alice/...` 的路径  
  截图：显示 `/Users/[REDACTED_USER]/...`  
  Pass 判据：PII 路径脱敏

- **13-redaction-fail-closed.png**（§E7.4）  
  录屏/截图：故意触发脱敏引擎异常或高危模式  
  Pass 判据：显示受限提示（"内容因安全策略受限" 类文案）· 绝不回退明文（§E7.4 红线）

### 2.5 a11y（§D.4 + E8）

- **14-a11y-keyboard-modal.png** + 短录屏  
  操作：Tab 进入徽章 → Enter 打开 modal → Tab 导航按钮 → Esc 关闭  
  Pass 判据：focus trap 生效 · aria-label 可被读屏器模拟 · 颜色不是唯一状态表达（§D.4 / E8.3）

- **15-reduced-motion.png**（可选 · reduced-motion 偏好）  
  Pass 判据：动画降级（§E8.4）

**命名与体积**：严格 `01-` 顺序前缀 · 单图 ≤ 500KB（或压缩后）· 总目录 ≤ 10MB（R3/R4）。

---

## 3 · 性能测量（§F.4 · 填 metrics-mvp-19.md）

见 `metrics-mvp-19.md` 模板 · Arbiter 用 DevTools Performance + 自定义 instrumentation（若 Phase E 仪表化已就位）实测：

- 绑定计算 < 20ms（500 commit 样本）
- Git Log 徽章渲染不触发全表重绘（观察 React 渲染次数或 console timing）
- Session 详情首次打开 < 200ms（缓存命中 < 80ms）

---

## 4 · 自检 grep（I7 + 跨段一致性）

```bash
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation

# 每步必须有 spec 锚点
grep -nE "§D\.|§E\.|§M\." docs/runtime-evidence/mvp-19/CAPTURE-PLAYBOOK.md

# 6 种徽章态 + superseded 必须全覆盖
grep -nE "confirmed|pending|stale|weak|plusN|superseded" docs/runtime-evidence/mvp-19/CAPTURE-PLAYBOOK.md

# 脱敏示例必须引用 M.5
grep -n "REDACTED\|Bearer\|/Users/" docs/runtime-evidence/mvp-19/CAPTURE-PLAYBOOK.md

# placeholder 只能留在 metrics（未填时）
grep -nE "<TBD>|<PASS_or_FAIL>" docs/runtime-evidence/mvp-19/metrics-mvp-19.md
```

任一异常即 BLOCK · 修完再 commit。

---

## 5 · 收尾 · PR + R1-R5 落地

1. `npx prettier --write docs/runtime-evidence/mvp-19/*.md`
2. `git add docs/runtime-evidence/mvp-19/`
3. commit（见交付要求）
4. PR body 必须包含：
   - [x] Runtime 证据已提交到 `docs/runtime-evidence/mvp-19/` · 含 N 张截图 / 录屏 + filled metrics（R5）
   - 列出本 playbook 产出的 01-15 证据文件（或实际数量）
5. 验证 R1-R5：
   - R1 位置：`docs/runtime-evidence/mvp-19/`
   - R2 进 git：`git ls-files | grep mvp-19`
   - R3 命名：`01-` 顺序前缀
   - R4 体积：`du -sh docs/runtime-evidence/mvp-19`
   - R5 PR body 已引

**本 playbook 仅描述 "按 §D/§E/§M 规格该呈现什么" · 不依赖 Phase C/D/E 具体像素实现**（HC-1 / HC-5）。

---

**关联规则**：`.claude/rules/runtime-evidence-location.md` + MVP-05 先例 + MVP-19 spec §I.5.5 / §D / §E / §M / §F.4

**自审四问**（本 playbook 对自己）：

1. 递归完备：每步都有 spec 锚点 · 7 条 invariant 自己也受 §7 自检保护 ✅
2. 反向场景：红线（E7.4）+ fail-closed（I4）+ 状态全覆盖（I6）已内建 ✅
3. 边界：仅 workspace 内 · macOS/Linux smoke 留 §4 · 不跨项目 ✅
4. YAGNI：只覆盖 Phase E 必须的 GUI/安全/a11y/perf · 不加语义分类等超前项 ✅

GO（Arbiter 按此 playbook 实跑后翻 Phase E 证据门）。
