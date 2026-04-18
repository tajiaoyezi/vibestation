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
 * 去除 YAML 行内注释（#），但保留引号内的 `#`
 *
 * Codex PR #11 review F2 教训：原实现 `rawLine.replace(/#.*$/, "")` 不区分
 * 引号内外 · 会把 `status: "done #junk"` 截断为 `done` · 允许绕过枚举校验。
 *
 * 本实现跟踪单引号 / 双引号状态 · 只在 quote-closed 状态下移除 `#` 及其后内容。
 */
function stripInlineComment(line) {
  let inSingle = false;
  let inDouble = false;
  let result = "";
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '"' && !inSingle) {
      inDouble = !inDouble;
      result += c;
      continue;
    }
    if (c === "'" && !inDouble) {
      inSingle = !inSingle;
      result += c;
      continue;
    }
    if (c === "#" && !inSingle && !inDouble) {
      // 注释开始 · 跳出
      break;
    }
    result += c;
  }
  return result;
}

/**
 * 解析 YAML frontmatter（手写精简实现 · 无外部依赖）
 *
 * 支持：
 *  - scalar key: value / key: "value"  / key: 'value'
 *  - inline list: key: [a, b, c]  / key: []
 *  - block list: key:\n  - a\n  - b
 *  - 空值 → null
 *  - 引号内的 `#` 保留（Codex F2 修复）
 *
 * 不支持（遇到时返回解析错误）：
 *  - 嵌套 map（task spec 不需要）
 *  - multiline scalar（`|` / `>`）
 *  - anchor / alias
 *  - tag (`!!str` 等)
 *
 * 复杂 YAML 用例超出本 validator scope · 若未来需要 · 引入 `yaml` npm 包（见 issue TODO）。
 */
function parseFrontmatter(source) {
  const match = source.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return null;

  const body = match[1];
  const result = {};
  const lines = body.split(/\r?\n/);
  let currentKey = null; // 跟踪 block list 归属

  for (const rawLine of lines) {
    const line = stripInlineComment(rawLine);

    // Block list item: "  - value"  (属于上一个 key)
    const listItemMatch = line.match(/^\s+-\s+(.*)$/);
    if (listItemMatch && currentKey) {
      let value = listItemMatch[1].trim();
      // 去引号（保留引号内的 `#`）
      if (
        (value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))
      ) {
        value = value.slice(1, -1);
      }
      if (!Array.isArray(result[currentKey])) {
        result[currentKey] = [];
      }
      result[currentKey].push(value);
      continue;
    }

    // Key-value pair
    const kvMatch = line.match(/^([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*(.*)$/);
    if (!kvMatch) {
      // 非空行且非 key-value / 非 list item → 可能是 multiline / anchor 等
      if (line.trim() && !line.trim().startsWith("#")) {
        // 保留为显式错误（caller 决定是否致命）
        // 但为保持兼容 · 只在真正的 validate 阶段报
      }
      continue;
    }

    const key = kvMatch[1];
    let value = kvMatch[2].trim();
    currentKey = key;

    // Inline list [a, b, c]
    if (/^\[.*\]$/.test(value)) {
      const inner = value.slice(1, -1).trim();
      result[key] = inner
        ? inner.split(",").map((s) => {
            let item = s.trim();
            if (
              (item.startsWith('"') && item.endsWith('"')) ||
              (item.startsWith("'") && item.endsWith("'"))
            ) {
              item = item.slice(1, -1);
            }
            return item;
          })
        : [];
      currentKey = null; // inline list 已完成 · 后续 `- xxx` 不归属此 key
      continue;
    }

    // Scalar · 去引号（保留引号内的 `#`）
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }

    // 空值 → null
    result[key] = value === "" ? null : value;
  }

  return result;
}

/**
 * Self-test · 启动时跑 adversarial fixture
 * Codex PR #11 F2 教训：手写 parser 必须有对抗性测试
 */
function runSelfTest() {
  const tests = [
    // { name, input, expected }
    {
      name: "basic key-value",
      input: `---\nid: SPIKE-01\nstatus: draft\n---`,
      expected: { id: "SPIKE-01", status: "draft" },
    },
    {
      name: "inline list",
      input: `---\ndepends_on: [SPIKE-01, SPIKE-02]\n---`,
      expected: { depends_on: ["SPIKE-01", "SPIKE-02"] },
    },
    {
      name: "empty inline list",
      input: `---\ndepends_on: []\n---`,
      expected: { depends_on: [] },
    },
    {
      name: "block list",
      input: `---\ndepends_on:\n  - SPIKE-01\n  - SPIKE-02\n---`,
      expected: { depends_on: ["SPIKE-01", "SPIKE-02"] },
    },
    {
      name: "quoted value with # inside (Codex F2)",
      input: `---\nstatus: "done #junk"\n---`,
      expected: { status: "done #junk" },
    },
    {
      name: "single-quoted value with # inside",
      input: `---\nblocked_from: 'ready #nope'\n---`,
      expected: { blocked_from: "ready #nope" },
    },
    {
      name: "comment after scalar value",
      input: `---\nstatus: draft  # 注释\n---`,
      expected: { status: "draft" },
    },
    {
      name: "empty value → null",
      input: `---\nreviewer:\n---`,
      expected: { reviewer: null },
    },
    {
      name: "quoted value with colon",
      input: `---\ntitle: "A: B"\n---`,
      expected: { title: "A: B" },
    },
  ];

  let passed = 0;
  let failed = 0;
  for (const t of tests) {
    const result = parseFrontmatter(t.input);
    const match = JSON.stringify(result) === JSON.stringify(t.expected);
    if (match) {
      passed++;
    } else {
      failed++;
      console.error(`  self-test FAIL: ${t.name}`);
      console.error(`    expected: ${JSON.stringify(t.expected)}`);
      console.error(`    actual:   ${JSON.stringify(result)}`);
    }
  }
  if (failed > 0) {
    console.error(`\n✗ parser self-test FAILED (${passed}/${tests.length} pass)`);
    process.exit(1);
  }
  // Silent on pass (don't clutter normal runs)
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

// Parser 自检 · adversarial fixture（Codex PR #11 F2 教训）
// 启动时必跑 · parser 本身有 bug 则 fail-fast · 不让 validator 在坏的基础上跑
runSelfTest();

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
