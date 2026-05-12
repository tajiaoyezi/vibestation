import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { bench, describe } from "vitest";
import { createShikiAdapter } from "../../../../src/utils/shiki";

const BENCH_OPTIONS = {
  iterations: 5,
  time: 1,
  warmupIterations: 1,
  warmupTime: 1,
};

async function readFixture(fileName: string): Promise<string> {
  return readFile(
    join(process.cwd(), "tests/fixtures/syntax-highlight", fileName),
    "utf8",
  );
}

function withRunSuffix(code: string, run: number): string {
  return `${code}\n// vitest-bench-run-${run}`;
}

const fixtures = [
  ["typescript", "1mb-typescript.ts"],
  ["rust", "1mb-rust.rs"],
  ["python", "1mb-python.py"],
  ["go", "1mb-go.go"],
  ["java", "1mb-java.java"],
] as const;

const fixtureCode = new Map<string, string>();

for (const [, fileName] of fixtures) {
  fixtureCode.set(fileName, await readFixture(fileName));
}

describe("MVP-15 Phase D §F.1 shiki parse 1MB fixtures", () => {
  for (const [lang, fileName] of fixtures) {
    let run = 0;

    bench(
      `parse ${fileName} as ${lang} (cache miss, sync ShikiAdapter)`,
      async () => {
        const adapter = createShikiAdapter();
        const code = withRunSuffix(fixtureCode.get(fileName)!, run++);
        const html = await adapter.highlight(code, lang, "light");

        if (!html.includes('class="shiki')) {
          throw new Error(`expected shiki HTML for ${fileName}`);
        }
      },
      BENCH_OPTIONS,
    );
  }
});
