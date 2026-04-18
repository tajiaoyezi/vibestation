#!/usr/bin/env node
// ─────────────────────────────────────────────────────────────────────
// Task spec frontmatter validator（Phase 4 落地）
// 用途：校验 docs/tasks/*.md 的 YAML frontmatter 字段合法性
// 参考：docs/tasks/README.md §字段说明 + §状态流转
// 触发：.github/workflows/task-spec-validator.yml（PR / push main）
//
// 校验规则：
//   1. 必填字段齐全（id / type / title / status / phase / depends_on / blocks / estimate / plan_ref）
//   2. status ∈ {draft, ready, in-progress, blocked, done}
//   3. type ∈ {spike, mvp, bug, feat}
//   4. status: blocked → 必填 blocked_by / blocked_from
//   5. blocked_from ∈ {ready, in-progress}
//   6. done 状态 → 必填 reviewer（≠ owner）
//   7. depends_on / blocks 形式必须是 list
//
// 使用：
//   node scripts/validate-task-spec.mjs [<file1.md> <file2.md> ...]
//   （无参数 → 扫 docs/tasks/ 下所有 *.md，排除 README 和 _template）
//
// 退出码：
//   0 · 全通过
//   1 · 任一校验失败
// ─────────────────────────────────────────────────────────────────────

import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, basename } from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = new URL("..", import.meta.url).pathname;
const TASKS_DIR = join(REPO_ROOT, "docs/tasks");

const VALID_STATUS = ["draft", "ready", "in-progress", "blocked", "done"];
const VALID_TYPE = ["spike", "mvp", "bug", "feat"];
const VALID_BLOCKED_FROM = ["ready", "in-progress"];

const REQUIRED_FIELDS = [
  "id",
  "type",
  "title",
  "status",
  "phase",
  "depends_on",
  "blocks",
  "estimate",
  "plan_ref",
];

/**
 * 解析 YAML frontmatter（简单实现，无外部依赖）
 * 只处理 scalar / list / 空值，不支持嵌套 map
 */
function parseFrontmatter(source) {
  const match = source.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return null;

  const body = match[1];
  const result = {};

  for (const rawLine of body.split(/\r?\n/)) {
    const line = rawLine.replace(/#.*$/, ""); // 去行内注释
    const m = line.match(/^([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*(.*)$/);
    if (!m) continue;

    const key = m[1];
    let value = m[2].trim();

    // list [a, b, c]
    if (/^\[.*\]$/.test(value)) {
      const inner = value.slice(1, -1).trim();
      result[key] = inner
        ? inner.split(",").map((s) => s.trim().replace(/^["']|["']$/g, ""))
        : [];
      continue;
    }

    // 去引号
    value = value.replace(/^["']|["']$/g, "");

    // 空值 → null
    result[key] = value === "" ? null : value;
  }

  return result;
}

/**
 * 校验单个 spec 文件
 * 返回 [] 表示通过，否则返回 error 消息列表
 */
function validateSpec(filepath) {
  const errors = [];
  const content = readFileSync(filepath, "utf8");
  const fm = parseFrontmatter(content);

  if (!fm) {
    return [`frontmatter missing or malformed`];
  }

  // 1. 必填字段
  for (const field of REQUIRED_FIELDS) {
    if (!(field in fm) || fm[field] === null) {
      errors.push(`required field missing or empty: ${field}`);
    }
  }

  // 2. enum 校验
  if (fm.status && !VALID_STATUS.includes(fm.status)) {
    errors.push(`status must be one of ${VALID_STATUS.join(" / ")}, got: ${fm.status}`);
  }
  if (fm.type && !VALID_TYPE.includes(fm.type)) {
    errors.push(`type must be one of ${VALID_TYPE.join(" / ")}, got: ${fm.type}`);
  }

  // 3. depends_on / blocks 必须是 list（即使空）
  for (const listField of ["depends_on", "blocks"]) {
    if (listField in fm && !Array.isArray(fm[listField])) {
      errors.push(`${listField} must be a list (use [] for empty), got: ${JSON.stringify(fm[listField])}`);
    }
  }

  // 4. status: blocked → 必填 blocked_by / blocked_from
  if (fm.status === "blocked") {
    if (!Array.isArray(fm.blocked_by) || fm.blocked_by.length === 0) {
      errors.push(`status: blocked requires non-empty blocked_by list`);
    }
    if (!fm.blocked_from) {
      errors.push(`status: blocked requires blocked_from (ready | in-progress)`);
    } else if (!VALID_BLOCKED_FROM.includes(fm.blocked_from)) {
      errors.push(
        `blocked_from must be one of ${VALID_BLOCKED_FROM.join(" / ")}, got: ${fm.blocked_from}`
      );
    }
  }

  // 5. 非 blocked 状态 → blocked_by / blocked_from / blocked_note 应为空
  if (fm.status !== "blocked") {
    if (Array.isArray(fm.blocked_by) && fm.blocked_by.length > 0) {
      errors.push(`non-blocked status should have empty blocked_by (got ${JSON.stringify(fm.blocked_by)})`);
    }
    if (fm.blocked_from) {
      errors.push(`non-blocked status should not have blocked_from (got ${fm.blocked_from})`);
    }
  }

  // 6. status: done → 必填 reviewer（且 ≠ owner）
  if (fm.status === "done") {
    if (!fm.reviewer) {
      errors.push(`status: done requires reviewer field`);
    } else if (fm.owner && fm.reviewer === fm.owner) {
      errors.push(`reviewer must differ from owner (both are: ${fm.owner})`);
    }
  }

  // 7. status: in-progress → 必填 owner
  if (fm.status === "in-progress" && !fm.owner) {
    errors.push(`status: in-progress requires owner field`);
  }

  // 8. id 必须与文件名前缀一致
  if (fm.id) {
    const expectedPrefix = fm.id.toLowerCase();
    const name = basename(filepath, ".md").toLowerCase();
    if (!name.startsWith(expectedPrefix)) {
      errors.push(`file name "${basename(filepath)}" does not start with id "${fm.id}"`);
    }
  }

  return errors;
}

/**
 * 收集要校验的文件列表
 * 默认扫 docs/tasks/*.md（排除 README / _template）
 */
function collectFiles(args) {
  if (args.length > 0) return args;

  const entries = readdirSync(TASKS_DIR);
  return entries
    .filter((e) => e.endsWith(".md"))
    .filter((e) => e !== "README.md" && e !== "_template.md")
    .map((e) => join(TASKS_DIR, e));
}

// ─────────── Main ───────────
const args = process.argv.slice(2);
const files = collectFiles(args);

if (files.length === 0) {
  console.log("No task spec files to validate.");
  process.exit(0);
}

let totalErrors = 0;
console.log(`\nValidating ${files.length} task spec file(s)...\n`);

for (const file of files) {
  try {
    const stat = statSync(file);
    if (!stat.isFile()) continue;
  } catch {
    console.error(`✗ ${file}: file not found`);
    totalErrors++;
    continue;
  }

  const errors = validateSpec(file);
  if (errors.length === 0) {
    console.log(`✓ ${basename(file)}`);
  } else {
    console.error(`✗ ${basename(file)}:`);
    for (const err of errors) console.error(`    - ${err}`);
    totalErrors += errors.length;
  }
}

console.log("");
if (totalErrors > 0) {
  console.error(`FAIL · ${totalErrors} error(s) across ${files.length} file(s)`);
  process.exit(1);
}
console.log(`PASS · all ${files.length} file(s) valid`);
process.exit(0);
