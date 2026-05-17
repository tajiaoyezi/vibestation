# MVP-20 Phase D/E · Capture Playbook · Arbiter ~30-45 min 收口

> **目标**：按本 playbook 一次跑完 · 产出 **14 张截图（01-14 覆盖 §E.1-6 + §M 危险操作 + §E.4 冲突复用 MVP-16）+ 30s 录屏（键盘 + 模态 + abort 流）+ filled metrics（§F.1 Criterion 5/20 commit revert P99 + §L 跨平台表）** · 主 agent 据此把 MVP-20 spec Phase E 收口并为后续 done gate 准备证据。
>
> **触发条件**：MVP-20 Phase C/D 实施基本可用后 · Arbiter 执行窗口 · 按 §I Phase E "runtime evidence + 性能量化" 要求 · 覆盖 §E/§F/§I/§L/§M GUI + 安全 + a11y + 性能 + 跨平台。
>
> **预计时间**：30-45 min（5 min 前置 + 20-30 min 实测 + 5 min commit + prettier）。
>
> **关联**：
>
> - spec：[`docs/tasks/MVP-20-ai-one-click-rollback.md`](../../tasks/MVP-20-ai-one-click-rollback.md)（§E Acceptance / §F 测试矩阵 / §I Phase D/E / §L 跨平台 / §M 危险操作 UX）
> - metrics：[`metrics-mvp-20.md`](./metrics-mvp-20.md)
> - ADR-011 + `.claude/rules/runtime-evidence-location.md`（R1-R5 硬规则）
> - 冲突 UI 复用：MVP-16 `ConflictBanner` + `Diff/3way/`

---

## 🛡 7 Invariant（playbook design intent · revert 安全 + 历史保留 + 冲突复用 MVP-16）

未来改 playbook 任何段前必先对照本 7 条 · 防危险操作误触 / 历史丢失 / 冲突 UI 假复用 / a11y 假 PASS。

### I1 · precondition 必须显式验证 · 不能假设

- 应用：每段 capture 前必须有 "确认当前 workspace 存在 ≥3 commit 的 AI session + 至少 1 个低置信候选 + 1 个会产生冲突的 revert 场景" 的步骤 · 否则 01 按钮截图可能抓到无数据空状态。

### I2 · spec acceptance 不能在 playbook 标"可选"

- 应用：§E.2 危险操作安全防护（E.2.1-E.2.4）· §E.3 中断与 Abort（E.3.1-E.3.4）· §E.4 冲突处理 · §E.5 历史保留验证 · 任何一项 playbook 必须列为 mandatory · 不能跳过。

### I3 · evidence 必须直接演示 spec 描述的真实行为

- 应用：截"已回滚徽章"（§E.1.9-10）时必须同时截 `git log --oneline` 输出展示原 session commit + N 个新 revert commit 都可见 · 不能只截 UI badge（§E.5.1 历史保留验证）。

### I4 · revert 保留历史 fail-closed（§E.5）

- 应用：每张回滚完成截图必伴随 `git log --oneline` + `git diff <pre-session-sha>` 验证为空的终端输出 · 证明是 `git revert` 而非 `reset --hard`（§E.2.3 严禁 reset + §E.5.2）。

### I5 · 危险操作 UX 必须可演示（§M + E.2）

- 应用：二次确认 dialog 必须有两张独立截图：session ID 输入错（按钮 disabled + 红色 border）+ 输入正确（按钮 enable + 红色实心）· 不能只截 happy path。

### I6 · 冲突复用 MVP-16（§E.4 + §H.3）

- 应用：截 ConflictBanner 时必须同时截 ThreeWayDiffView 确认是 `web/src/panels/Diff/3way/` 同款组件（不是自建 conflict UI）· 颜色/交互一致性验证。

### I7 · 每步必须有 spec 锚点 + 可量化 pass 判据

- 应用：写完后自检 grep 必须能定位到 §E.x / §F.x / §I.x / §L.x / §M.x · pass 条件必须可执行（"红色按钮 + disabled 状态 + session ID 校验失败文案"）。

**自检**（写完 playbook 后立即跑）：

```bash
# I7 自检（每步必须有 spec 锚点）
grep -nE "§E\.|§F\.|§I\.|§L\.|§M\." docs/runtime-evidence/mvp-20/CAPTURE-PLAYBOOK.md | wc -l
# 应 ≥ 15 处（覆盖按钮/预览/二次确认/冲突/历史/a11y/性能/跨平台）
```

---

## 0 · 前置准备（5 min）

```bash
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation

# 0.1 确认 main 最新（不在 feature 分支）
git checkout main && git pull --ff-only origin main

# 0.2 创新 branch 收 evidence
git checkout -b docs/mvp-20-phase-de-capture

# 0.3 确认依赖
node --version  # ≥ 20
pnpm --version  # 9.x
cargo --version

# 0.4 清理旧 evidence（保留 playbook + metrics · 删 PNG/MOV）
rm -f docs/runtime-evidence/mvp-20/*.png docs/runtime-evidence/mvp-20/*.mov 2>/dev/null || true
mkdir -p docs/runtime-evidence/mvp-20

# 0.5 准备 fixture（构造 ≥ 3 commit AI session + 1 个会冲突的 revert 场景）
# 参考 spec 附录 C `create_5commit_session_fixture` 伪代码（用测试 CLI 或脚本快速生成）
# 确保至少 1 个 commit revert 会产生冲突（故意在目标文件制造并行修改）

# 0.6 准备 dev 模式 + release build（Criterion bench 需要）
pnpm install
cargo build --release
```

---

## 1 · GUI capture（20-25 min · 覆盖 §E + §M + §E.4 MVP-16 复用）

**推荐完全手动**（避免自动化在 modal 焦点 / 二次确认输入 / 冲突解决上的 timing 问题）。

```bash
OUT=docs/runtime-evidence/mvp-20
WID=$(osascript -e 'tell application "System Events" to id of front window of (first process whose name contains "Vibestation")')
```

### 1.1 一键回滚按钮初态（§E.1.1 + §M.2）

- 截图：Session 详情顶部红色 "一键回滚" 按钮（`--color-status-error`）
- Pass 判据：按钮可见 + 红色系 + `aria-label="回滚 Session #N 的所有 AI commit"`（§E.6.1）

### 1.2 预览 modal 展开（§E.1.2-3）

- 截图：点击后弹出的 commit 列表 + 置信度分级（≥0.9 / <0.9 视觉区分）
- Pass 判据：列表完整 + 低置信 commit 可见高亮

### 1.3 低置信度警告（§E.1.4）

- 截图：`< 0.9` commit 默认未勾选状态
- Pass 判据：默认不选 + 用户可手动勾选

### 1.4 二次确认 dialog - session ID 输入错（§E.1.5 + §E.2.2 + §M.2）

- 截图：输入错误 session ID 后 "执行回滚" 按钮 disabled + 红色 border
- Pass 判据：按钮 disabled + 明确错误提示

### 1.5 二次确认 dialog - session ID 输入正（§E.1.5 + §E.2.1 + §M.2）

- 截图：输入正确后按钮 enable + 红色实心填充（危险操作视觉）
- Pass 判据：红色实心 + 可点击

### 1.6 progress banner 执行中（§E.1.7）

- 截图/短录屏：`{done}/{total}` + 百分比进度条
- Pass 判据：banner 可见 + `role="status"` + `aria-live="polite"`

### 1.7 完成态 - 已回滚徽章（§E.1.9-10）

- 截图：Session 详情 "已回滚" 灰色徽章 + tooltip
- Pass 判据：灰色 + tooltip 内容 `"此 session 已回滚"`

### 1.8 完成态 - Git Log 刷新 + 历史保留（§E.1.11 + §E.5.1 + §E.5.3）

- 截图 + 终端输出：`git log --oneline` 显示原 session commit + N 个新 revert commit（revert message 含 `[AI session rollback: {id}]` 后缀）
- Pass 判据：历史完整保留（非 reset）

### 1.9 冲突场景 - ConflictBanner（§E.4.1）

- 截图：revert 过程中出现 ConflictBanner（红色 · operation="rollback" · MVP-16 同款）
- Pass 判据：与 MVP-16 ConflictBanner 视觉/文案一致

### 1.10 冲突场景 - ThreeWayDiffView（§E.4.2）

- 截图：3-way Diff 视图（复用 `web/src/panels/Diff/3way/`）
- Pass 判据：确认是 MVP-16 同款组件（非自建）

### 1.11 解决 → Continue 续跑（§E.4.3）

- 截图：解决冲突后点 Continue → progress 继续

### 1.12 Abort flow（§E.3.1-4）

- 截图序列 + `git log` 验证：执行中点"取消回滚" → HEAD 干净回到起点 · 0 残留 revert commit
- Pass 判据：`Repository::cleanup_state()` 生效 + 原 commit 仍在

### 1.13 DirtyWorkingTree 跳转（§E.2.4）

- 截图：有未提交改动时触发 → 明确提示 + 自动跳转 Status 面板
- Pass 判据：dirty 文件可见 + 无 `reset --hard`

### 1.14 a11y 焦点流（§E.6）

- 短录屏/截图：Tab 顺序 · Esc 关 modal · Enter 提交 · 屏幕阅读器 aria-live 模拟
- Pass 判据：focus trap + aria 正确

### 1.15 reduced-motion（§E.6）

- 截图：系统 reduced-motion 偏好开启后动画降级
- Pass 判据：无过度动画

**命名与体积**：严格 `01-` 顺序前缀 · 单图 ≤ 500KB · 总目录 ≤ 10MB（R3/R4）。

---

## 2 · 录屏（30s）

```bash
# 推荐 QuickTime 或 ffmpeg
ffmpeg -f avfoundation -i "1" -r 30 -t 30 docs/runtime-evidence/mvp-20/rollback-flow.mov
```

**录屏内容**（键盘 + 模态 + abort）：

- Tab 到"一键回滚" → Enter 触发预览 → Esc 关闭 → 重新 Tab + Enter → 输入 session ID → 提交 → 看 progress banner → 看完成徽章
- 另录一段 Abort 流程（执行中点取消 → 干净回滚）

---

## 3 · 性能 metrics（spec §F.1 Criterion bench）

见 `metrics-mvp-20.md` 模板 · Arbiter 运行：

```bash
cargo bench -p vibestation-core --bench rollback -- 5-commit 20-commit
```

或等效 instrumentation（若 Phase E 已暴露）。

---

## 4 · 跨平台 smoke（spec §L.1）

- **macOS**：本机直接跑 §1 全套 + Criterion bench
- **Linux（Ubuntu 24）**：GitHub Actions runner 或 Arbiter VM · 验证 §L.1 表格 6 行行为（git revert 顺序 / file lock / cleanup_state / 路径大小写 / SQLite WAL / 冲突解决）

---

## 5 · 收尾 · PR + R1-R5 落地

1. `npx prettier --write docs/runtime-evidence/mvp-20/*.md`
2. `git add docs/runtime-evidence/mvp-20/`
3. commit + trailer
4. PR body 必须包含：
   - [x] Runtime 证据已提交到 `docs/runtime-evidence/mvp-20/` · 含 14 张截图 + 30s 录屏 + filled metrics（R5）
5. 验证 R1-R5：
   - R1 位置：`docs/runtime-evidence/mvp-20/`
   - R2 进 git：`git ls-files docs/runtime-evidence/mvp-20/`
   - R3 命名：`01-` 顺序前缀
   - R4 体积：`du -sh ...`
   - R5 PR body 已引

**本 playbook 仅描述 "按 §E/§F/§I/§L/§M 规格该呈现什么" · 不依赖 Phase C/D 具体像素实现**。

---

**关联规则**：`.claude/rules/runtime-evidence-location.md` + MVP-19 PR #376 先例 + MVP-20 spec §E/§F/§I/§L/§M

**自审四问**（本 playbook 对自己）：

1. 递归完备：每步都有 spec 锚点 · 7 条 invariant 自己也受 §7 自检保护 ✅
2. 反向场景：危险操作（E.2）、历史保留（E.5）、abort 干净（E.3）、冲突复用（E.4）已内建 ✅
3. 边界：macOS + Linux smoke 留 §4 · 不跨项目 ✅
4. YAGNI：只覆盖 Phase D/E 必须的 GUI/安全/性能/跨平台 · 不加语义分类等超前项 ✅

GO（Arbiter 按此 playbook 实跑后翻 Phase E 证据门）。
