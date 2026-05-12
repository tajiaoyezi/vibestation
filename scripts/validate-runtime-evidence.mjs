#!/usr/bin/env node
// Runtime evidence 目录校验（ADR-011 / .claude/rules/runtime-evidence-location.md · R1-R4）
// R5（PR body 引用）为 PR-time 人工/CI 检查 · 本脚本不实现。
//
// 使用：
//   node scripts/validate-runtime-evidence.mjs
//   node scripts/validate-runtime-evidence.mjs --mvp mvp-02
//   node scripts/validate-runtime-evidence.mjs --report docs/runtime-evidence/_VALIDATION-REPORT.md
//   node scripts/validate-runtime-evidence.mjs --strict
//   node scripts/validate-runtime-evidence.mjs --exceptions .validator-exceptions.json
//
// Exception 配置（spec-mandated 命名 / 体积豁免 · 见 .validator-exceptions.json）：
//   - allow_r3_naming: 列表 · 命中即跳过 R3 命名违规
//   - r4_dir_tolerance_bytes: 数字 · 目录总和 < (10MB + tolerance) 时 R4 上限 ERROR 降级为 WARNING
// 默认读 <repoRoot>/.validator-exceptions.json（若存在）· 也可 --exceptions <path> 显式指定。
//
// 测试/自动化可设 RUNTIME_EVIDENCE_VALIDATOR_ROOT 指向临时 git 仓库根目录（覆盖脚本所在仓库推断）。

import { readdir, readFile, stat, writeFile } from "node:fs/promises";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const SCRIPT_DIR = fileURLToPath(new URL(".", import.meta.url));
const DEFAULT_REPO_ROOT = join(SCRIPT_DIR, "..");

const MEDIA_EXT = new Set(["jpg", "png", "mp4", "webm", "gif"]);
const MEDIA_NAME_RE = /^\d{2}-[a-z0-9]+(?:-[a-z0-9]+)*\.(jpg|png|mp4|webm|gif)$/;

const MAX_SINGLE_BYTES = 10 * 1024 * 1024;
const MAX_DIR_TOTAL_BYTES = 10 * 1024 * 1024;
const REC_JPG_PNG_BYTES = 3 * 1024 * 1024;
const REC_VIDEO_BYTES = 5 * 1024 * 1024;

/** @typedef {{ level: 'ERROR' | 'WARNING', code: string, message: string }} Issue */

function parseArgs(argv) {
  const out = { mvp: null, report: null, strict: false, exceptions: null };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--strict") out.strict = true;
    else if (a === "--mvp") out.mvp = argv[++i] ?? null;
    else if (a === "--report") out.report = argv[++i] ?? null;
    else if (a === "--exceptions") out.exceptions = argv[++i] ?? null;
    else if (a === "--help" || a === "-h") {
      console.log(`Usage: node scripts/validate-runtime-evidence.mjs [--mvp <id>] [--report <path>] [--strict] [--exceptions <path>]`);
      process.exit(0);
    }
  }
  return out;
}

/**
 * 读 exception JSON · 默认 <repoRoot>/.validator-exceptions.json · 不存在返回 null
 * @param {string} repoRoot
 * @param {string|null} customPath
 * @returns {Promise<{ version: number, mvps: Record<string, { reason?: string, spec_ref?: string, allow_r3_naming?: string[], r4_dir_tolerance_bytes?: number }> } | null>}
 */
async function loadExceptions(repoRoot, customPath) {
  const path = customPath
    ? customPath.startsWith("/")
      ? customPath
      : join(repoRoot, customPath)
    : join(repoRoot, ".validator-exceptions.json");
  if (!(await pathExists(path))) {
    if (customPath) {
      console.error(`exception 文件未找到：${path}`);
      process.exit(1);
    }
    return null;
  }
  try {
    const raw = await readFile(path, "utf8");
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object" && parsed.mvps) return parsed;
    console.error(`exception 文件格式异常：${path}（缺少 mvps 字段）`);
    process.exit(1);
  } catch (e) {
    console.error(`读 exception 文件失败：${path} · ${e.message}`);
    process.exit(1);
  }
  return null;
}

/**
 * 对单 MVP issue 列表应用 exception · 返回保留的 issues 和已应用的豁免列表
 * @param {Issue[]} issues
 * @param {{ allow_r3_naming?: string[], r4_dir_tolerance_bytes?: number } | null} exception
 * @param {number} totalBytes
 * @returns {{ kept: Issue[], applied: { code: string, reason: string }[] }}
 */
function applyExceptions(issues, exception, totalBytes) {
  if (!exception) return { kept: issues, applied: [] };
  /** @type {Issue[]} */
  const kept = [];
  /** @type {{ code: string, reason: string }[]} */
  const applied = [];
  for (const issue of issues) {
    if (issue.code === "R3" && exception.allow_r3_naming && exception.allow_r3_naming.length > 0) {
      const hit = exception.allow_r3_naming.some((name) => issue.message.includes(name));
      if (hit) {
        applied.push({ code: "R3", reason: `命名豁免：${issue.message}` });
        continue;
      }
    }
    if (
      issue.code === "R4" &&
      issue.level === "ERROR" &&
      issue.message.includes("总体积") &&
      issue.message.includes("超过上限") &&
      exception.r4_dir_tolerance_bytes &&
      totalBytes <= MAX_DIR_TOTAL_BYTES + exception.r4_dir_tolerance_bytes
    ) {
      applied.push({ code: "R4", reason: `dir 总体积 tolerance：${issue.message}` });
      continue;
    }
    kept.push(issue);
  }
  return { kept, applied };
}

function posixRel(p) {
  return p.split(sep).join("/");
}

function getRepoRoot() {
  const env = process.env.RUNTIME_EVIDENCE_VALIDATOR_ROOT?.trim();
  if (env) return env;
  return DEFAULT_REPO_ROOT;
}

function gitSpawn(repoRoot, args, input) {
  const r = spawnSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
    input: input ?? undefined,
    maxBuffer: 50 * 1024 * 1024,
  });
  if (r.error) throw r.error;
  return r;
}

function gitHead(repoRoot) {
  const r = gitSpawn(repoRoot, ["rev-parse", "HEAD"]);
  if (r.status !== 0) return "(no git HEAD)";
  return r.stdout.trim();
}

/** @returns {Set<string>} posix paths relative to repo root */
function gitLsFilesTracked(repoRoot, underPosix) {
  const r = gitSpawn(repoRoot, ["ls-files", "-z", "--", underPosix]);
  if (r.status !== 0) {
    console.error(`git ls-files failed: ${r.stderr || r.stdout}`);
    return new Set();
  }
  const set = new Set();
  if (!r.stdout) return set;
  for (const p of r.stdout.split("\0")) {
    if (p) set.add(p);
  }
  return set;
}

async function pathExists(p) {
  try {
    await stat(p);
    return true;
  } catch {
    return false;
  }
}

async function walkFiles(dir) {
  /** @type {string[]} */
  const out = [];
  const entries = await readdir(dir, { withFileTypes: true });
  for (const e of entries) {
    const full = join(dir, e.name);
    if (e.isDirectory()) {
      out.push(...(await walkFiles(full)));
    } else if (e.isFile()) {
      out.push(full);
    }
  }
  return out;
}

async function dirHasAnyFile(dir) {
  const entries = await readdir(dir, { withFileTypes: true });
  for (const e of entries) {
    const full = join(dir, e.name);
    if (e.isFile()) return true;
    if (e.isDirectory() && (await dirHasAnyFile(full))) return true;
  }
  return false;
}

/**
 * @param {string} repoRoot
 * @returns {Promise<Issue[]>}
 */
async function checkSpikeTmpImg(repoRoot) {
  /** @type {Issue[]} */
  const issues = [];
  const base = join(repoRoot, "spike-tmp", "img");
  if (!(await pathExists(base))) return issues;
  let children;
  try {
    children = await readdir(base, { withFileTypes: true });
  } catch {
    return issues;
  }
  for (const c of children) {
    if (!c.isDirectory()) continue;
    const sub = join(base, c.name);
    if (await dirHasAnyFile(sub)) {
      issues.push({
        level: "ERROR",
        code: "R1",
        message: `禁止在 deprecated 路径存在内容：\`spike-tmp/img/${c.name}/\`（应迁移到 docs/runtime-evidence/）`,
      });
    }
  }
  return issues;
}

/**
 * @param {string} absFile
 * @param {string} extLower
 */
function sizeRecommendViolation(extLower, size) {
  if (["jpg", "png", "gif"].includes(extLower) && size > REC_JPG_PNG_BYTES) {
    return { code: "R4", message: `图片/GIF 体积超过推荐 3MB（当前 ${fmtBytes(size)}）` };
  }
  if (["mp4", "webm"].includes(extLower) && size > REC_VIDEO_BYTES) {
    return { code: "R4", message: `视频体积超过推荐 5MB（当前 ${fmtBytes(size)}）` };
  }
  return null;
}

function fmtBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(2)} KB`;
  return `${(n / (1024 * 1024)).toFixed(2)} MB`;
}

/**
 * @param {string} repoRoot
 * @param {string} mvpId
 * @param {Set<string>} tracked
 */
async function validateMvpDir(repoRoot, mvpId, tracked) {
  const relBase = posixRel(relative(repoRoot, join(repoRoot, "docs", "runtime-evidence", mvpId)));
  const absBase = join(repoRoot, "docs", "runtime-evidence", mvpId);
  /** @type {Issue[]} */
  const issues = [];

  if (mvpId !== mvpId.toLowerCase()) {
    issues.push({
      level: "ERROR",
      code: "R1",
      message: `task 目录名必须为小写：\`${mvpId}\``,
    });
  }

  const spikeResidual = join(repoRoot, "spike-tmp", "img", mvpId);
  if ((await pathExists(spikeResidual)) && (await dirHasAnyFile(spikeResidual))) {
    issues.push({
      level: "ERROR",
      code: "R1",
      message: `同 ID 在 \`spike-tmp/img/${mvpId}/\` 仍有残留文件`,
    });
  }

  if (!(await pathExists(absBase))) {
    issues.push({ level: "ERROR", code: "R1", message: `目录不存在：docs/runtime-evidence/${mvpId}/` });
    return { issues, files: [], totalBytes: 0, rows: [] };
  }

  const allAbs = await walkFiles(absBase);
  let totalBytes = 0;

  /** @type {{ name: string, size: number, nameOk: boolean }[]} */
  const rows = [];

  for (const abs of allAbs) {
    const rel = posixRel(relative(repoRoot, abs));
    const st = await stat(abs);
    const size = st.size;
    totalBytes += size;

    if (!tracked.has(rel)) {
      issues.push({
        level: "ERROR",
        code: "R2",
        message: `未纳入 git 跟踪（或已被 ignore）：\`${rel}\``,
      });
    }

    const baseName = rel.split("/").pop() ?? "";
    const dot = baseName.lastIndexOf(".");
    const extLower = dot >= 0 ? baseName.slice(dot + 1).toLowerCase() : "";

    let nameOk = true;
    if (MEDIA_EXT.has(extLower)) {
      if (!MEDIA_NAME_RE.test(baseName)) {
        nameOk = false;
        issues.push({
          level: "WARNING",
          code: "R3",
          message: `媒体文件命名不符合 \`NN-kebab-name.ext\`：\`${rel}\``,
        });
      }
      if (size > MAX_SINGLE_BYTES) {
        issues.push({
          level: "ERROR",
          code: "R4",
          message: `单文件超过上限 10MB：\`${rel}\`（${fmtBytes(size)}）`,
        });
      }
      const rec = sizeRecommendViolation(extLower, size);
      if (rec) {
        issues.push({ level: "WARNING", code: rec.code, message: `${rec.message} · \`${rel}\`` });
      }
    }

    rows.push({ name: baseName, size, nameOk: !MEDIA_EXT.has(extLower) || nameOk });
  }

  if (totalBytes > MAX_DIR_TOTAL_BYTES) {
    issues.push({
      level: "ERROR",
      code: "R4",
      message: `目录 \`${relBase}/\` 总体积 ${fmtBytes(totalBytes)} 超过上限 10MB`,
    });
  } else if (totalBytes > 3 * 1024 * 1024) {
    issues.push({
      level: "WARNING",
      code: "R4",
      message: `目录 \`${relBase}/\` 总体积 ${fmtBytes(totalBytes)} 超过推荐 3MB`,
    });
  }

  return { issues, files: allAbs, totalBytes, rows, relBase };
}

function dedupeIssues(issues) {
  const seen = new Set();
  return issues.filter((i) => {
    const k = `${i.level}|${i.code}|${i.message}`;
    if (seen.has(k)) return false;
    seen.add(k);
    return true;
  });
}

function mvpStatus(issues) {
  const hasE = issues.some((i) => i.level === "ERROR");
  const hasW = issues.some((i) => i.level === "WARNING");
  if (hasE) return "ERROR";
  if (hasW) return "WARNING";
  return "PASS";
}

function buildMarkdownReport(repoRoot, globalIssues, mvpResults, isoDate, head) {
  let pass = 0,
    warn = 0,
    err = 0;
  let warnItems = 0,
    errItems = 0;
  for (const r of mvpResults) {
    const s = mvpStatus(r.issues);
    if (s === "PASS") pass++;
    else if (s === "WARNING") warn++;
    else err++;
    warnItems += r.issues.filter((i) => i.level === "WARNING").length;
    errItems += r.issues.filter((i) => i.level === "ERROR").length;
  }
  for (const i of globalIssues) {
    if (i.level === "ERROR") errItems++;
    else warnItems++;
  }

  const lines = [];
  lines.push(`# Runtime Evidence Validation Report`);
  lines.push(``);
  lines.push(`> 生成时间：${isoDate} · 由 \`scripts/validate-runtime-evidence.mjs\` 自动生成`);
  lines.push(`> Repo HEAD：${head}`);
  lines.push(``);
  lines.push(`## 总览`);
  lines.push(``);
  lines.push(`- 扫描 ${mvpResults.length} 个 MVP 目录`);
  lines.push(`- ✅ 全 PASS：${pass} 个`);
  lines.push(`- 🟡 WARNING：${warn} 个（${warnItems} 项推荐违反）`);
  lines.push(`- 🔴 ERROR：${err} 个（${errItems} 项硬规则违反）`);
  if (globalIssues.length) {
    lines.push(`- ⚠️ 全局检查：${globalIssues.length} 项（含 deprecated 路径等）`);
    lines.push(``);
    lines.push(`### 全局`);
    lines.push(``);
    for (const i of globalIssues) {
      lines.push(`- ${i.level === "ERROR" ? "🔴" : "🟡"} **${i.code}** · ${i.message}`);
    }
  }
  lines.push(``);
  lines.push(`## 详情`);
  lines.push(``);

  for (const r of mvpResults) {
    const st = mvpStatus(r.issues);
    const icon = st === "PASS" ? "✅ PASS" : st === "WARNING" ? "🟡 WARNING" : "🔴 ERROR";
    const mvpId = r.mvpId;
    lines.push(`### ${mvpId.toUpperCase()} ${icon}`);
    lines.push(``);
    lines.push(`- 路径：\`docs/runtime-evidence/${mvpId}/\``);
    lines.push(`- 文件数：${r.rows.length}`);
    lines.push(`- 总体积：${fmtBytes(r.totalBytes)}`);
    if (r.issues.length) {
      lines.push(`- 问题：`);
      for (const i of dedupeIssues(r.issues)) {
        lines.push(`  - ${i.level === "ERROR" ? "🔴" : "🟡"} **${i.code}** · ${i.message}`);
      }
    }
    lines.push(`- 文件清单：`);
    lines.push(``);
    lines.push(`| 文件 | 体积 | 命名合规 |`);
    lines.push(`|---|---|---|`);
    for (const row of r.rows.sort((a, b) => a.name.localeCompare(b.name))) {
      const mark = row.nameOk ? "✅" : "🟡";
      lines.push(`| ${row.name} | ${fmtBytes(row.size)} | ${mark} |`);
    }
    lines.push(``);
  }

  lines.push(`## 修复建议`);
  lines.push(``);
  lines.push(`- 🟡 推荐违反：手动重命名 / 压缩媒体 · 控制目录总体积在 3MB 内更佳`);
  lines.push(`- 🔴 硬违反：阻塞 release · 修复后重跑 validator`);
  lines.push(`- **R5**：PR body 须引用 \`docs/runtime-evidence/<task-id>/\` · 本工具不校验 PR`);
  lines.push(``);

  return lines.join("\n");
}

async function main() {
  const args = parseArgs(process.argv);
  const repoRoot = getRepoRoot();
  const evidenceDir = join(repoRoot, "docs", "runtime-evidence");

  if (!(await pathExists(evidenceDir))) {
    console.error(`docs/runtime-evidence 不存在：${evidenceDir}`);
    process.exit(1);
  }

  const globalIssues = await checkSpikeTmpImg(repoRoot);

  let mvpIds = [];
  const entries = await readdir(evidenceDir, { withFileTypes: true });
  for (const e of entries) {
    if (e.isDirectory()) mvpIds.push(e.name);
  }
  mvpIds.sort();

  if (args.mvp) {
    if (!mvpIds.includes(args.mvp)) {
      console.error(`未找到 MVP 目录：docs/runtime-evidence/${args.mvp}/`);
      process.exit(1);
    }
    mvpIds = [args.mvp];
  }

  const trackedRoot = gitLsFilesTracked(repoRoot, "docs/runtime-evidence");
  const exceptions = await loadExceptions(repoRoot, args.exceptions);

  /** @type {{ mvpId: string, issues: Issue[], rows: any[], totalBytes: number, appliedExceptions: { code: string, reason: string }[] }[]} */
  const mvpResults = [];

  for (const mvpId of mvpIds) {
    const { issues, rows, totalBytes } = await validateMvpDir(repoRoot, mvpId, trackedRoot);
    const mvpException = exceptions?.mvps?.[mvpId] ?? null;
    const { kept, applied } = applyExceptions(issues, mvpException, totalBytes);
    mvpResults.push({ mvpId, issues: kept, rows, totalBytes, appliedExceptions: applied });
  }

  const isoDate = new Date().toISOString().replace(/\.\d{3}Z$/, "");
  const head = gitHead(repoRoot);
  const md = buildMarkdownReport(repoRoot, globalIssues, mvpResults, isoDate, head);

  if (args.report) {
    const reportPath = args.report.startsWith("/") ? args.report : join(repoRoot, args.report);
    await writeFile(reportPath, md, "utf8");
  }

  const hasError =
    globalIssues.some((i) => i.level === "ERROR") ||
    mvpResults.some((r) => r.issues.some((i) => i.level === "ERROR"));
  const hasWarn =
    globalIssues.some((i) => i.level === "WARNING") ||
    mvpResults.some((r) => r.issues.some((i) => i.level === "WARNING"));

  let pass = 0,
    wMvp = 0,
    eMvp = 0;
  for (const r of mvpResults) {
    const s = mvpStatus(r.issues);
    if (s === "PASS") pass++;
    else if (s === "WARNING") wMvp++;
    else eMvp++;
  }
  const globalErr = globalIssues.filter((i) => i.level === "ERROR").length;

  console.log(
    `Runtime evidence：扫描 ${mvpResults.length} 个目录 · ✅ PASS ${pass} · 🟡 WARNING ${wMvp} · 🔴 ERROR ${eMvp}${globalErr ? ` · 全局 🔴 ${globalErr}` : ""}${args.report ? ` · 报告已写入 ${args.report}` : ""}`,
  );
  if (hasError) {
    console.error("存在 🔴 ERROR · 见报告或上方详情");
  } else if (hasWarn && args.strict) {
    console.error("存在 🟡 WARNING（--strict 模式下失败）");
  }

  const exitCode = hasError || (args.strict && hasWarn) ? 1 : 0;
  process.exit(exitCode);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
