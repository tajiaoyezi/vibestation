# PR Review: #410 — test(mvp-18): 补 §F.3 fixture 契约 smoke + 增强 stale_child 软删覆盖

**Reviewed**: 2026-05-21
**Author**: tajiaoyezi (Leafiel Lune)  ·  Implemented by: Claude Code
**Branch**: `chore/mvp-18-fixture-contract-smoke` → `main`
**HEAD**: `7f100542f13a19677a55fd7f0703a6bd0fe794df`
**Reviewer**: Claude Code · self-review（v2-D.2 单人项目模式）
**Decision**: **APPROVE with comments**

## Summary

测试 polish PR · 0 行为变更 · 0 前端 diff · +237/−11 行 · 4 文件。

实质是 MVP-18 v1.0 vision 代码侧 ship 后的 audit cleanup：删 3 处过时 `TODO(MVP-18 A3)`（unlink DAO 已实现 + fixture corpus 已 land）· 把 `pane_link_stale_child` 集成测试从"仅断言行存在"升级到完整 §G.1 软删 + §E B.7 二次 unlink 幂等覆盖 · 新增 6 个 fixture-consuming smoke 测试给 §F.3 `pane_failure_*.txt` 文件契约首次自动化覆盖（消费方按 path 读取的契约）。

无 CRITICAL/HIGH · clippy/test/fmt/build 4/4 PASS · MEDIUM 都是契约完备性 trade-off · LOW 是注释/耦合 nit。Approve safe to merge。

## Findings

### CRITICAL
None.

### HIGH
None.

### MEDIUM

**M1 — `redaction_count >= 6` 阈值是设计 trade-off · 应在测试中注释清楚**
- 位置：`crates/core/tests/pane_failure_fixture_sanitize.rs:106-111`
- 现状：`pane_failure_secret.txt` 实际触发 7 处 redaction（Bearer / OPENAI_API_KEY / AWS_SECRET / GITHUB_TOKEN / URL-creds × 2 / 末尾 bearer dup）· 阈值 `≥ 6` 保留 1 余量
- 风险：未来如果 sanitize 规则微调让其中 1 处 redaction 不再触发（例如某 token 形态被废）· 测试不会立即 fail · silent regression 风险存在
- 建议（非阻塞 · 可后续 PR）：要么 `assert_eq!(redaction_count, 7)` 严格契约 + 注释每条 redaction 来源 · 要么保留 ≥ 6 但在测试 docstring 显式写"fixture has 7 patterns · ≥ 6 allows sanitize rule micro-evolution"
- 当前 PR：阈值选择合理 · 不阻塞 merge

**M2 — fixture contract smoke 只覆盖 `ParserUnavailable` 一个 variant · 名义上"全契约"但实际单 path**
- 位置：`crates/core/tests/pane_failure_fixture_sanitize.rs:139-145`（`parser_bridge_raw_fallback_consumes_all_fixtures`）
- 现状：`UntrustedParserOutput` 共 5 个 variant（`Structured` / `ParserUnavailable` / `ParserTimeout` / `UnsupportedCliKind` / `ParserCrash`）· 后 4 个内部都走 `raw_fallback` · 测试只验了 `ParserUnavailable` 一个
- Trade-off：内部代码路径相同 · 多测变体是冗余 surface area（YAGNI）· 但"fixture contract smoke"命名隐含全变体契约
- 建议（非阻塞）：要么把测试名改成 `parser_bridge_unavailable_fallback_consumes_all_fixtures` 名实相符 · 要么后续切片用 `rstest` 参数化覆盖 4 个 fallback variant
- 当前 PR：本意是 polish 而非完整契约 · APPROVE

### LOW

**L1 — `fixture_failure_struct_smoke`（旧 §F.1 typed fixture）vs 新 §F.3 raw text fixture 没有 cross-link 注释**
- 位置：`crates/core/tests/pane_link_integration.rs:283-292`（旧测试）vs `pane_failure_fixture_sanitize.rs:1-9`（新文件 module doc）
- 现状：两组测试覆盖 spec 不同 §（F.1 typed vs F.3 raw）· 未来 reviewer 进文件不容易立即理解关系
- 建议：在 `pane_failure_fixture_sanitize.rs` module doc 加一句"§F.1 typed Rust 函数另见 `pane_link_integration.rs`"

**L2 — `fixture_dir()` 用 `env!("CARGO_MANIFEST_DIR")` 是正确做法但无 inline 注释**
- 位置：`pane_failure_fixture_sanitize.rs:19-24`
- 现状：cargo 跑测试时 CWD 可能非 crate root · `CARGO_MANIFEST_DIR` 是 stable 方案 · 但读者首次见此 pattern 需要去查 cargo doc
- 建议：加单行注释 `// CARGO_MANIFEST_DIR is crate root regardless of CWD (cargo test 默认 CWD 是 crate root · 显式 env! 防自动化脚本 cd 后跑测)`

**L3 — `secret_fixture_redacts_all_token_shapes` 的 `banned` 数组硬编码 fixture token 字符串 · 耦合 fixture 内容**
- 位置：`pane_failure_fixture_sanitize.rs:113-121`
- 现状：未来如果 fixture token 改字面值（即使语义不变）· 测试要同步改
- Reviewer 角度：fixture README 已明确 fixture 是 §F.3 契约 · token 不应随意改 · 耦合可接受 · 但应在测试注释里 explicitly say "tightly coupled to fixture content by design"

## Validation Results

| Check | Result | Command |
|---|---|---|
| Type check | ✅ Pass | `cargo build --workspace` exit 0 |
| Lint | ✅ Pass | `cargo clippy --workspace --tests -- -D warnings` exit 0 · 0 warning |
| Tests | ✅ Pass | `cargo test --workspace` exit 0 · 1004+ tests · 0 failed · 18 ignored (Linux-only pre-existing) |
| Format | ✅ Pass | `cargo fmt --all -- --check` exit 0 |
| Build | ✅ Pass | `cargo build --workspace` exit 0 |
| TODO sweep | ✅ Pass | `grep -rn 'TODO(MVP-18' crates/` → 0 hits |

新增/修改的 6 个 fixture smoke 测试全过：
- `all_failure_fixtures_present_and_non_empty`
- `sanitize_consumes_all_fixtures_without_error`
- `osc52_fixture_strips_control_sequence`
- `secret_fixture_redacts_all_token_shapes`
- `parser_bridge_raw_fallback_consumes_all_fixtures`
- `parser_bridge_raw_fallback_redacts_secret_fixture`

增强的 `pane_link_stale_child` 测试在 `pane_link_integration` 10/10 内全过。

## Files Reviewed

| File | Change | Lines |
|---|---|---|
| `crates/core/src/parser_bridge.rs` | Modified | −2（删过时 TODO 注释） |
| `crates/core/src/sanitize.rs` | Modified | −2（删过时 TODO 注释） |
| `crates/core/tests/pane_link_integration.rs` | Modified | +50/−11（增强 stale_child + import PaneUnlinkRequest） |
| `crates/core/tests/pane_failure_fixture_sanitize.rs` | Added | +189（6 个 smoke 测试） |

## 7-Category Checklist Coverage

| Category | Pass/Concern |
|---|---|
| **Correctness** | ✅ stale_child 测试覆盖 create → unlink first → unlink idempotent 全状态机 · OSC52 检查 raw precondition + sanitized invariant 双向 · secret fixture 7 形态全断言不残留 |
| **Type Safety** | ✅ Rust 强类型 · 0 `unsafe` · 0 unwrap-on-Option-without-justification（`unwrap()` on `pool.get()` 在 test 里 acceptable） |
| **Pattern Compliance** | ✅ snake_case test 命名 · spec §段引用 · use 语句分组 · 符合 `pane_link_integration.rs` 既有风格 |
| **Security** | ✅ 0 真凭据 · fixture 全占位符 · `banned` 数组用 fake token shape · 安全防线 fail-loud |
| **Performance** | ✅ 测试 < 100ms · 不在 hot path |
| **Completeness** | 🟡 M2（单 variant 覆盖）· L1（cross-link 注释缺）· L3（耦合无注释） |
| **Maintainability** | 🟡 M1（阈值 trade-off 应注释）· L2（`env!` pattern 无说明） |

## 自审四问（reviewer 视角验 implementer 是否做了）

1. **递归完备性** ✅ — fixture path 契约 + 测试 contract 双向锁定（fixture README 锁文件名 · 测试硬编码消费 path）
2. **反向场景** ✅ — sanitize 漏 secret / 改 fixture 名字 / 删 fixture 文件 / OSC52 不再 strip → 测试立即 fail
3. **边界适用性** ✅ — 6 fixture（rustc / vitest / pytest / ansi_json / secret / osc52）全覆盖 · 不只 happy path
4. **YAGNI** 🟡 — 边界判断主观：fixture-consuming smoke 是否"真需要"vs 已有 inline test 覆盖单元层 · 我判断需要（spec §F.3 + §I.1 契约需 automated 验证）· 但 M2 提示当前覆盖度其实可以更窄

## Recommendation

**APPROVE · safe to merge**.

3 LOW + 2 MEDIUM 都是注释/契约-完备性级 nit · 不阻塞 v2-D.2 self-review approval。建议 Arbiter 后续在 PR comment 给 approval trailer 后即可 merge。M1/M2 可记为 follow-up（不必本 PR 处理）· L1/L2/L3 同 follow-up（或下一个相关 PR 顺便补）。

待 Arbiter approval trailer 落 PR body 第 3 行（`Arbiter approval: tajiaoyezi · YYYY-MM-DD HH:MM · "..."`）后 · 可 merge。
