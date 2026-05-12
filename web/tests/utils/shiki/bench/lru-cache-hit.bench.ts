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
const warmedHtml = await adapter.highlight(code, "typescript", "light");

describe("MVP-15 Phase D §F.4 LRU cache hit", () => {
  bench(
    "ShikiAdapter second highlight same key 1MB TypeScript",
    async () => {
      const html = await adapter.highlight(code, "typescript", "light");

      if (html !== warmedHtml) {
        throw new Error("expected exact cached HTML");
      }
    },
    BENCH_OPTIONS,
  );
});
