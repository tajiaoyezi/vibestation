import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { bench, describe } from "vitest";
import { createShikiAdapter } from "../../../../src/utils/shiki";

const BENCH_OPTIONS = {
  iterations: 1,
  time: 1,
  warmupIterations: 0,
  warmupTime: 0,
};

async function readFixture(fileName: string): Promise<string> {
  return readFile(
    join(process.cwd(), "tests/fixtures/syntax-highlight", fileName),
    "utf8",
  );
}

function heapUsedMb(): number {
  return process.memoryUsage().heapUsed / 1024 / 1024;
}

function collectGarbageIfAvailable(): boolean {
  const candidate = globalThis as typeof globalThis & { gc?: () => void };
  if (typeof candidate.gc !== "function") {
    return false;
  }

  for (let index = 0; index < 3; index += 1) {
    candidate.gc();
  }

  return true;
}

const code = await readFixture("1mb-typescript.ts");

describe("MVP-15 Phase D §F.5 10x1MB heap estimate", () => {
  bench(
    "highlight 10 distinct 1MB TypeScript variants and report heap delta",
    async () => {
      const adapter = createShikiAdapter();
      const hasGc = collectGarbageIfAvailable();
      const beforeMb = heapUsedMb();

      for (let index = 0; index < 10; index += 1) {
        const variant = [
          code,
          `// memory-bench-variant-${index}`,
          "// cache-key-length-pad ".repeat(index + 1),
        ].join("\n");
        await adapter.highlight(variant, "typescript", "light");
      }

      collectGarbageIfAvailable();
      const afterMb = heapUsedMb();
      const deltaMb = afterMb - beforeMb;
      const stats = adapter.getCacheStats();

      console.log(
        [
          "[mvp-15-memory]",
          `heapBeforeMb=${beforeMb.toFixed(2)}`,
          `heapAfterMb=${afterMb.toFixed(2)}`,
          `heapDeltaMb=${deltaMb.toFixed(2)}`,
          `cacheFiles=${stats.fileCount}`,
          `cacheSizeMb=${(stats.sizeBytes / 1024 / 1024).toFixed(2)}`,
          `gc=${hasGc ? "available" : "unavailable"}`,
          "note=jsdom process.memoryUsage heap estimate; Chrome DevTools Memory snapshot deferred",
        ].join(" "),
      );

      if (deltaMb > 100) {
        throw new Error(`heap delta exceeded 100MB: ${deltaMb.toFixed(2)}MB`);
      }
    },
    BENCH_OPTIONS,
  );
});
