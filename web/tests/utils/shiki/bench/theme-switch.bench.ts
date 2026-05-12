import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { bench, describe } from "vitest";
import { createShikiAdapter } from "../../../../src/utils/shiki";

const BENCH_OPTIONS = {
  iterations: 10,
  time: 5,
  warmupIterations: 2,
  warmupTime: 1,
};

async function readFixture(fileName: string): Promise<string> {
  return readFile(
    join(process.cwd(), "tests/fixtures/syntax-highlight", fileName),
    "utf8",
  );
}

const adapter = createShikiAdapter();
const code = await readFixture("1mb-typescript.ts");

await adapter.highlight(code, "typescript", "light");
await adapter.highlight(code, "typescript", "dark");

describe("MVP-15 Phase D §F.3 theme switch signal to cache hit", () => {
  let nextTheme: "light" | "dark" = "light";

  bench(
    "setShikiTheme + cached highlight 1MB TypeScript",
    async () => {
      nextTheme = nextTheme === "light" ? "dark" : "light";
      adapter.setTheme(nextTheme);
      const html = await adapter.highlight(code, "typescript", nextTheme);

      if (!html.includes('class="shiki')) {
        throw new Error("expected cached shiki HTML after theme switch");
      }
    },
    BENCH_OPTIONS,
  );
});
