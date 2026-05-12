# 仓库脚本说明

本目录存放与主应用解耦的 Node / Shell 工具脚本。

## Runtime Evidence Validator

按 [`.claude/rules/runtime-evidence-location.md`](../.claude/rules/runtime-evidence-location.md) **R1–R4** 扫描 `docs/runtime-evidence/<task-id>/`：**位置**、**git 跟踪**、**媒体文件命名**、**体积（单文件 / 目录总和）**。

**R5**（PR body 必须引用证据路径）为 PR 阶段人工检查项，本脚本不解析 GitHub PR。

### 用法

```bash
# 扫描全部 task 目录
node scripts/validate-runtime-evidence.mjs

# 仅扫描某一 task
node scripts/validate-runtime-evidence.mjs --mvp mvp-02

# 写出 Markdown 报告（相对路径相对于仓库根）
node scripts/validate-runtime-evidence.mjs --report docs/runtime-evidence/_VALIDATION-REPORT.md

# 任一 WARNING 也令进程退出码为 1（适合 CI / pre-push）
node scripts/validate-runtime-evidence.mjs --strict
```

### 退出码

- `0`：无 ERROR，且在非 `--strict` 模式下无「需当失败处理」的 WARNING。
- `1`：存在 ERROR，或 `--strict` 且存在 WARNING。

### 自动化测试仓库根目录

Vitest 等场景可在子进程环境中设置：

```bash
RUNTIME_EVIDENCE_VALIDATOR_ROOT=/path/to/tmp-repo node scripts/validate-runtime-evidence.mjs
```

未设置时默认使用「本脚本所在目录的上一级」，即仓库根。

### CI / 本地 hook 集成建议

- **GitHub Actions**（计费恢复后）：在 PR workflow 中增加一步，于仓库根执行  
  `node scripts/validate-runtime-evidence.mjs --strict`  
  与现有 `pnpm lint` / `pnpm typecheck` 并列。
- **pre-push**（可选）：与 [`.githooks/pre-push`](../.githooks/pre-push) 相同模式追加调用；默认不强制，避免本地未归档证据时分支无法推送。

首版全量扫描报告见 [`docs/runtime-evidence/_VALIDATION-REPORT.md`](../docs/runtime-evidence/_VALIDATION-REPORT.md)；批量补 capture 后由维护者重跑上述命令刷新。

## 其他脚本

- `validate-task-spec.mjs`：校验 `docs/tasks/*.md` frontmatter（见仓库 CI task-spec-validator）。
