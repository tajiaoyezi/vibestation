# SPIKE-08 报告 · IPC contract 生成 + E2E runtime 双层防御

## 背景

MVP-02 的 H2 事故不是单点实现错误，而是两层质量门都缺位：

1. Rust `serde(rename_all = "camelCase")` 和前端 interface 分离维护，`tsc` 无法跨边界发现 drift。
2. 现有 CI 只跑 Rust/unit/build smoke，不走真实 `create/list/delete` IPC 链路，产品仍可能 broken。

本 spike 用一个**独立**的 mini Tauri 2 + SolidJS app 验证双层防御是否能在 v0.1 GA 前落地，不改动 MVP-02 生产代码。

## 结论

当前建议是：

- **§A Contract layer：选 `ts-rs`，在 v0.1 GA 前强制覆盖所有新增 IPC contract。**
- **§B Runtime layer：`Playwright + Vite` 可以作为 v0.1 的自动化 runtime 补层；真实 Tauri IPC E2E（B.1/B.3）本轮都没有收敛，不应在 v0.1 GA 前作为 required gate。**
- **§C CI：所有平台 required 跑 contract + browser smoke；真实 native runtime 继续保留 manual runtime evidence，Linux `tauri-driver` workflow 暂列 informational / follow-up。**

换句话说：**本 spike 的最终判定是“§A PASS；§B true-native-runtime FAIL；v0.1 采用 hybrid gate，而不是等待 native E2E 完全成熟”。**

## §A Contract Layer

### 候选对比

| 候选 | 版本 / 维护态 | 依赖规模（最小 Tauri 2 sample） | Tauri 2 集成成本 | codegen trigger | 结论 |
|---|---|---:|---|---|---|
| `ts-rs` | `12.0.1`；`ts-rs` repo `pushed_at=2026-04-09`；stars `1765` | `656` 行树 | 低。只管 Rust type → TS type；不接管 command wrapper | `build.rs` 可行；再配合 `beforeDev/BuildCommand` 保证前端构建前已生成 | **选用** |
| `tauri-specta` + `specta` | `2.0.0-rc.24`；repo `pushed_at=2026-04-20`；stars `702` | `675` 行树 | 中高。要接入 `Builder`/command collection，且当前主版本仍是 `rc` | 更偏向专用 export step / builder 收集命令 | 暂不用于 v0.1 GA |

数据来源：

- crate 元数据：`raw/SPIKE-08/ts-rs-cargo-info.txt`、`raw/SPIKE-08/tauri-specta-cargo-info.txt`
- 仓库维护态：`raw/SPIKE-08/ts-rs-repo.json`、`raw/SPIKE-08/tauri-specta-repo.json`、`raw/SPIKE-08/specta-repo.json`
- 依赖规模：`raw/SPIKE-08/ts-rs-cargo-tree.lines`、`raw/SPIKE-08/tauri-specta-cargo-tree.lines`

### POC 实施

选型落地在 [`docs/spikes/code/SPIKE-08`](./code/SPIKE-08/):

- Rust source of truth：`src-tauri/src/contract.rs`
- build trigger：`src-tauri/build.rs`
- 生成产物：`src/bindings/*.ts`
- 前端消费：`src/backend.ts`、`src/App.tsx`

关键实现点：

1. `contract.rs` 里的 5 个 struct 同时 derive `serde` 与 `TS`，保持 `camelCase` source of truth。
2. `build.rs` 在 `cargo build` 时写出 `src/bindings/*.ts` 和 `src/bindings/index.ts`。
3. `package.json` 的 `contract:generate`/`build`/`dev:tauri` 先跑 Rust build，再交给 Vite/Tauri，避免前端拿到 stale bindings。

### H2 compile-time 回归

真实回归做法：

1. 临时把 `WorkspaceRecord.id` 改成 `workspace_id`
2. 同步修 Rust store，让 `cargo build` 仍然成功并重新生成 bindings
3. 保持前端代码不动，直接跑 `pnpm typecheck`

结果：**必然 FAIL**，且失败点落在所有仍访问 `workspace.id` 的位置。

证据：

- `raw/SPIKE-08/h2-contract-regression-build.log`
- `raw/SPIKE-08/h2-contract-regression.log`

这说明 contract layer 能把 H2 类 drift 从 runtime 前移到 compile-time。

## §B Runtime Layer

### 3 选型 smoke 结果

| 路线 | 环境 | 结果 | 证据 | 判定 |
|---|---|---|---|---|
| B.1 `tauri-driver` | GitHub Ubuntu 24.04 runner | 第 1 次缺 `WebKitWebDriver`；第 2 次补装 `webkit2gtk-driver` 后，`tauri-driver` 仍报 `Connection refused`，未能进入 DOM 交互 | `linux-gh-run.log`、`linux-gh-artifact/`、`linux-gh-run-rerun.log`、`linux-gh-artifact-rerun/` | **FAIL** |
| B.2 `Playwright + Vite dev server` | 本机 macOS | `create/list/delete` golden path PASS；单次 `2.70s`；10 连跑 `10/10` 成功 | `playwright-browser-smoke.log`、`playwright-browser-timing.log`、`playwright-browser-10x.log`、trace/png | **PASS** |
| B.3 `tauri-playwright` 社区 recipe | 本机 macOS | app 与 socket 能启动，但第一条 `eval` 命令 30s timeout | `tauri-playwright-dev.log`、`tauri-playwright-smoke.log`、`tauri-playwright-results/` | **SMOKE FAIL** |

### B.2 结论

`Playwright + Vite` 无法覆盖真实 Tauri IPC，但非常适合补上以下盲区：

- 列表渲染/删除 modal/按钮 wiring
- DOM 回归和视觉快照
- 快速 flake 检测

它对 H2 runtime regression 也有效，只要前端通过 `any`/`Reflect.get` 等方式绕过 compile-time 检查。

### H2 runtime 回归

真实回归做法：

1. 前端保留 generated bindings，不改 Rust contract
2. 仅把删除路径改成按旧 key `workspace_id` 取值
3. `pnpm typecheck` 仍然通过
4. 重新跑 browser E2E

结果：**删除后列表不会消失，E2E 在等待 empty state 时 30s timeout FAIL。**

证据：

- `raw/SPIKE-08/h2-runtime-regression.log`
- `raw/SPIKE-08/h2-runtime-regression-trace.zip`
- `raw/SPIKE-08/h2-runtime-regression.png`

这证明 runtime layer 能拦住“compile-time 被绕过后”的交互破坏。

### B.1 结论

官方 Linux `tauri-driver` 路线在本 spike 里做了两轮真实 runner 验证：

1. **run `24653654459`**
   - 失败点：`can not find binary WebKitWebDriver in the PATH`
   - 结论：只装 `libwebkit2gtk-4.1-dev` 不够，runner recipe 还需要 `webkit2gtk-driver`
2. **run `24653923822`**
   - 在 workflow 中补装 `webkit2gtk-driver`
   - `tauri-driver` 已能启动，但 artifact `tauri-driver.log` 报：
     - `Error serving connection`
     - `Connection refused (os error 111)`
   - 同时 `Run Linux smoke` 仍在等待 `http://127.0.0.1:4444/status` 超时失败

基于现有证据，本 spike 内的判断是：

- **证据**：官方文档要求 Linux 额外准备 `webkit2gtk-driver` + `xvfb`；本次第二轮已经满足这两项。
- **推断**：即便满足官方列出的额外依赖，Ubuntu runner 上仍需进一步调通 native driver 启动/连接细节（可能是 `WebKitWebDriver` 启动方式、端口或 `--native-driver` 路径配置）。
- **结论**：这条 lane 在 2d spike 预算内**未收敛到可做 GA required**。

### B.3 结论

社区 `tauri-playwright` 的价值在于它理论上能把 macOS/Linux 的真实 Tauri window 暴露成 Playwright 风格 API；但在本次 POC 上：

- socket 启动成功：`tauri-plugin-playwright: listening on unix:/tmp/tauri-playwright.sock`
- 测试 runner 成功连上 native app
- 但第一条 `eval` 命令即超时 30s，未能进入黄金路径

因此它目前更像**值得继续观察的实验路线**，不适合在 v0.1 GA 前替代 Linux `tauri-driver` required lane。

## §C CI 推荐

### 推荐矩阵

| OS / lane | Contract | Runtime | 是否 required | 说明 |
|---|---|---|---|---|
| All OS | `pnpm contract:check` | compile-time drift 防线 | **required** | H2 类字段 mismatch 必须在 PR 前失败 |
| All OS | `Playwright + Vite` | browser DOM/视觉回归 | **required** | 单次 2.70s，10 连跑 0% flake，适合作为快反馈 |
| Linux (`ubuntu-24.04`) | `tauri-driver` workflow | true native IPC E2E | **informational / follow-up** | 两轮真实 runner 都未收敛，不能先挂 required |
| macOS | manual runtime evidence | destructive/native 行为复核 | **required for release sign-off** | `tauri-driver` 不支持；B.3 社区 recipe 当前 smoke fail |

### macOS 降级策略

因为 `tauri-driver` 不支持 macOS，v0.1 GA 前的最低可接受策略是：

1. **所有 PR required**：contract check + browser smoke。
2. **macOS release sign-off required**：保留 manual runtime evidence，至少覆盖 destructive IPC 路径。
3. **Linux native E2E**：继续保留 workflow 原型，但在 recipe 收敛前不升级为 required。

### Flaky 处理

- 默认 **不允许 silent skip**。
- 规则：
  - 单次失败先保留 trace/screenshot/log。
  - 可自动重试 1 次，但只限 runtime job，不限次重试会掩盖真实问题。
  - 连续 2 个 PR 或 24h 内重复 flake 的 case 直接进 quarantine 列表，并在 CI summary 明示，不得默默 `continue-on-error`。

## 风险与后续

- `ts-rs` 只解决 type drift，不生成 typesafe command wrapper；如果未来希望把 invoke 层也完全收敛成单源，可以在 v0.2 再评估 `tauri-specta`。
- B.3 社区 recipe 发布时间很新（`0.2.2` 发布于 2026-03-30），当前 smoke fail，不建议纳入 GA required。

## 上游参考

- Tauri WebDriver 文档：<https://v2.tauri.app/develop/tests/webdriver/>
- Tauri WebDriver CI 文档（Linux 额外依赖提到 `webkit2gtk-driver` 与 `xvfb`）：<https://v2.tauri.app/zh-cn/develop/tests/webdriver/ci/>
- `ts-rs`：<https://github.com/Aleph-Alpha/ts-rs>
- `tauri-specta`：<https://github.com/specta-rs/tauri-specta>
- `tauri-playwright`：<https://github.com/srsholmes/tauri-playwright>

## 当前建议

本 spike 后适合立刻落地的方案是：

1. **马上强制**：所有新增 IPC struct 必须走 generated bindings（`ts-rs`）。
2. **马上强制**：所有 PR 跑 browser smoke，兜住 DOM/交互回归。
3. **继续保留**：manual runtime evidence 作为 release sign-off 门槛，直到 true native E2E 收敛。
4. **不要现在开 ADR-012 锁死 native E2E 架构**；先把 Linux `tauri-driver` 连接问题单独收敛，再决定是否升级为 required。
