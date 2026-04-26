# MVP-09 Linux Runtime Evidence

**平台**：Ubuntu 24.04.4 LTS (Linux 6.17.0-22-generic · x86_64)
**CPU**：AMD Ryzen 7 9800X3D 8-Core Processor
**Rust**：rustc 1.95.0 (2026-04-14)
**测量时间**：2026-04-26
**实施 agent**：Claude Code（Ubuntu 独立项目目录 · session 20 · B4 task）
**对应 spec**：[MVP-09 §D + §E](../../../tasks/MVP-09-stage-unstage-commit.md)

## §D 性能 P99（Linux 基线 · Criterion bench）

| ID | bench | mean (µs) | spec 上限 | PASS/FAIL |
|----|------|-----------|-----------|-----------|
| D.1 | stage_single_file | 263 µs (0.26 ms) | < 100ms | ✅ PASS |
| D.2 | commit_typical | 352 µs (0.35 ms) | < 500ms | ✅ PASS |
| D.3 | stage_all_1000_files | 31,514 µs (31.5 ms) | < 2s | ✅ PASS |

> Criterion 100 samples per bench · `iter_with_setup` 每迭代创建 fresh tempdir · setup 时间不计入测量。
> 注：macOS 数字由主 agent 后续在主开发机补 · Linux 数字仅作平台对照基线 · 不代表 spec 收尾的最终标准。

## §A.4.3 git2 stage 操作 < 50ms

| ID | bench | mean | spec 上限 | PASS/FAIL |
|----|------|------|-----------|-----------|
| A.4.3 | stage_single_file | 0.26 ms | < 50ms | ✅ PASS |

> Linux 平台 mean 0.26ms · 远低于 spec < 50ms（190× 余量）· §D.1 同源数据。

## §E 集成测试覆盖

14 tests · 0 failed · 0 ignored

| Test | 状态 |
|------|------|
| test_commit_single_file | ✅ PASS |
| test_commit_multiple_files | ✅ PASS |
| test_commit_rejected_when_no_staged | ✅ PASS |
| test_amend_changes_sha_and_message | ✅ PASS |
| test_chinese_message_and_filename | ✅ PASS |
| test_chinese_message_body_multiline | ✅ PASS |
| test_untracked_file_can_stage | ✅ PASS |
| test_unstage_untracked_after_stage | ✅ PASS |
| test_modified_after_stage_double_status | ✅ PASS |
| test_commit_succeeds_when_hook_script_present | ✅ PASS |
| test_hook_script_present_commit_creates_sha | ✅ PASS |
| test_stage_nonexistent_file_failed_list | ✅ PASS (extra coverage) |
| test_multiple_commits_in_sequence | ✅ PASS (extra coverage) |
| test_stage_result_failed_item_fields | ✅ PASS (extra coverage) |

> 注：pre-commit hook 测试文档化当前行为（libgit2 不执行 hooks · commit 成功）。
> Phase C 实施 hook 执行后 · 测试应按 spec §C.4 更新为 assert!(is_err()) + match HookFailed。

## 备注

- A.4.1 / A.4.2 / A.4.4 / A.4.5 前端 timing 由主 agent 后续在 macOS 实测补 · 本目录不含
- §A.1-§A.3 / §B / §C 前端 UI acceptance · PR #118 已实施 · 主 agent macOS runtime 接 fix-up
- 本目录 raw 文件：
  - `01-bench-output.txt` — cargo bench 完整输出
  - `02-integration-tests-output.txt` — cargo test verbose 输出
  - `03-full-workspace-test-summary.txt` — cargo test --workspace 汇总
