import { execFileSync, spawnSync } from "node:child_process";
import { mkdir, mkdtemp, readFile, rm, writeFile, open } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

// Vitest 可能改写 import.meta.url（非 file:）· 从 web/ 工作区向上解析仓库根
const REPO_ROOT = join(process.cwd(), "..");
const SCRIPT = join(REPO_ROOT, "scripts", "validate-runtime-evidence.mjs");

function runValidator(tmpRoot: string, args: string[] = []) {
  const r = spawnSync(process.execPath, [SCRIPT, ...args], {
    cwd: tmpRoot,
    env: { ...process.env, RUNTIME_EVIDENCE_VALIDATOR_ROOT: tmpRoot },
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  const code = r.status === 0 ? (0 as const) : (1 as const);
  return { code, stdout: r.stdout ?? "", stderr: r.stderr ?? "" };
}

async function gitInit(tmpRoot: string) {
  execFileSync("git", ["init", "-b", "main"], { cwd: tmpRoot, stdio: "pipe" });
  execFileSync("git", ["config", "user.email", "validator-test@local"], { cwd: tmpRoot, stdio: "pipe" });
  execFileSync("git", ["config", "user.name", "validator-test"], { cwd: tmpRoot, stdio: "pipe" });
}

async function gitCommitAll(tmpRoot: string, message: string) {
  execFileSync("git", ["add", "-A"], { cwd: tmpRoot, stdio: "pipe" });
  execFileSync("git", ["commit", "-m", message], { cwd: tmpRoot, stdio: "pipe" });
}

describe("validate-runtime-evidence.mjs", () => {
  let tmpRoot: string;

  beforeEach(async () => {
    tmpRoot = await mkdtemp(join(tmpdir(), "rev-validator-"));
    await gitInit(tmpRoot);
  });

  afterEach(async () => {
    await rm(tmpRoot, { recursive: true, force: true });
  });

  it("PASS · 标准 mvp-02 样例布局", async () => {
    const base = join(tmpRoot, "docs", "runtime-evidence", "mvp-02");
    await mkdir(base, { recursive: true });
    await writeFile(join(base, "01-welcome-page.jpg"), Buffer.alloc(100));
    await writeFile(join(base, "02-multi-workspace.jpg"), Buffer.alloc(100));
    await writeFile(join(base, "03-delete-confirm-modal.jpg"), Buffer.alloc(100));
    await writeFile(join(base, "README.md"), "# ok\n");
    await gitCommitAll(tmpRoot, "init evidence");

    const r = runValidator(tmpRoot);
    expect(r.code).toBe(0);
    expect(r.stdout).toContain("PASS 1");
    const reportPath = join(tmpRoot, "report.md");
    runValidator(tmpRoot, ["--report", reportPath]);
    const report = await readFile(reportPath, "utf8");
    expect(report).toContain("MVP-02");
    expect(report).toContain("✅ PASS");
  });

  it("ERROR · 单文件超过 10MB", async () => {
    const base = join(tmpRoot, "docs", "runtime-evidence", "mvp-x");
    await mkdir(base, { recursive: true });
    const huge = join(base, "01-huge.jpg");
    const fh = await open(huge, "w");
    await fh.truncate(11 * 1024 * 1024);
    await fh.close();
    await gitCommitAll(tmpRoot, "huge");

    expect(runValidator(tmpRoot).code).toBe(1);
    const rep = join(tmpRoot, "huge-report.md");
    runValidator(tmpRoot, ["--report", rep]);
    expect(await readFile(rep, "utf8")).toMatch(/10 ?MB|R4/);
  });

  it("WARNING · 单张 jpg 超过推荐 3MB（非 strict 仍 exit 0）", async () => {
    const base = join(tmpRoot, "docs", "runtime-evidence", "mvp-fat");
    await mkdir(base, { recursive: true });
    const p = join(base, "01-large.jpg");
    const fh = await open(p, "w");
    await fh.truncate(4 * 1024 * 1024);
    await fh.close();
    await gitCommitAll(tmpRoot, "fat");

    expect(runValidator(tmpRoot).code).toBe(0);
    expect(runValidator(tmpRoot, ["--strict"]).code).toBe(1);
  });

  it("WARNING · 媒体文件命名缺少顺序前缀", async () => {
    const base = join(tmpRoot, "docs", "runtime-evidence", "mvp-name");
    await mkdir(base, { recursive: true });
    await writeFile(join(base, "welcome.jpg"), Buffer.alloc(50));
    await gitCommitAll(tmpRoot, "bad name");

    expect(runValidator(tmpRoot).code).toBe(0);
    runValidator(tmpRoot, ["--report", join(tmpRoot, "r.md")]);
    const report = await readFile(join(tmpRoot, "r.md"), "utf8");
    expect(report).toMatch(/R3|命名/);
    expect(runValidator(tmpRoot, ["--strict"]).code).toBe(1);
  });

  it("ERROR · 文件未 git 跟踪", async () => {
    const base = join(tmpRoot, "docs", "runtime-evidence", "mvp-untracked");
    await mkdir(base, { recursive: true });
    await writeFile(join(base, "01-a.jpg"), Buffer.alloc(20));
    await gitCommitAll(tmpRoot, "empty");
    await writeFile(join(base, "02-b.jpg"), Buffer.alloc(20));

    expect(runValidator(tmpRoot).code).toBe(1);
    const rep = join(tmpRoot, "untracked-report.md");
    runValidator(tmpRoot, ["--report", rep]);
    const body = await readFile(rep, "utf8");
    expect(body).toMatch(/R2|跟踪/);
  });

  it("ERROR · spike-tmp/img 下仍有残留", async () => {
    const base = join(tmpRoot, "docs", "runtime-evidence", "mvp-clean");
    await mkdir(base, { recursive: true });
    await writeFile(join(base, "01-a.jpg"), Buffer.alloc(20));
    const bad = join(tmpRoot, "spike-tmp", "img", "mvp-clean");
    await mkdir(bad, { recursive: true });
    await writeFile(join(bad, "legacy.jpg"), Buffer.alloc(10));
    await gitCommitAll(tmpRoot, "spike leak");

    expect(runValidator(tmpRoot).code).toBe(1);
    const rep = join(tmpRoot, "spike-report.md");
    runValidator(tmpRoot, ["--report", rep]);
    const body = await readFile(rep, "utf8");
    expect(body).toMatch(/spike-tmp|deprecated|R1/);
  });

  it("ERROR · 目录总体积超过 10MB", async () => {
    const base = join(tmpRoot, "docs", "runtime-evidence", "mvp-sum");
    await mkdir(base, { recursive: true });
    for (const n of ["01-a.jpg", "02-b.jpg"]) {
      const p = join(base, n);
      const fh = await open(p, "w");
      await fh.truncate(6 * 1024 * 1024);
      await fh.close();
    }
    await gitCommitAll(tmpRoot, "sum");

    const r = runValidator(tmpRoot);
    expect(r.code).toBe(1);
    const reportPath = join(tmpRoot, "rep.md");
    runValidator(tmpRoot, ["--report", reportPath]);
    const report = await readFile(reportPath, "utf8");
    expect(report).toMatch(/10 ?MB|总体积/);
  });

  it("--mvp 仅扫描指定目录", async () => {
    const good = join(tmpRoot, "docs", "runtime-evidence", "mvp-ok");
    const bad = join(tmpRoot, "docs", "runtime-evidence", "mvp-bad");
    await mkdir(good, { recursive: true });
    await mkdir(bad, { recursive: true });
    await writeFile(join(good, "01-a.jpg"), Buffer.alloc(20));
    const fh = await open(join(bad, "01-b.jpg"), "w");
    await fh.truncate(11 * 1024 * 1024);
    await fh.close();
    await gitCommitAll(tmpRoot, "two");

    expect(runValidator(tmpRoot, ["--mvp", "mvp-ok"]).code).toBe(0);
    expect(runValidator(tmpRoot).code).toBe(1);
  });

  it("ERROR · task 目录名含大写字母", async () => {
    const base = join(tmpRoot, "docs", "runtime-evidence", "MVP-bad");
    await mkdir(base, { recursive: true });
    await writeFile(join(base, "01-a.jpg"), Buffer.alloc(20));
    await gitCommitAll(tmpRoot, "upper");

    expect(runValidator(tmpRoot).code).toBe(1);
    const rep = join(tmpRoot, "case-report.md");
    runValidator(tmpRoot, ["--report", rep]);
    const body = await readFile(rep, "utf8");
    expect(body).toMatch(/小写|R1/);
  });
});
