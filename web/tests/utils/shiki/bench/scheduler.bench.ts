import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { bench, describe, vi } from "vitest";
import {
  scheduleHighlight,
  STREAMING_THRESHOLDS,
} from "../../../../src/utils/shiki/scheduler";

vi.mock("../../../../src/utils/shiki/worker-client", () => ({
  highlightInWorker: vi
    .fn()
    .mockRejectedValue(new Error("vitest bench worker unsupported")),
}));

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

const oneMbTypeScript = await readFixture("1mb-typescript.ts");
const tenMbTypeScript = await readFixture("10mb-typescript.ts");
const underOneMbTypeScript = oneMbTypeScript.slice(
  0,
  STREAMING_THRESHOLDS.SIZE_1MB - 16 * 1024,
);

async function schedulerProbe(
  code: string,
  lang: string,
  theme: "light" | "dark",
): Promise<string> {
  return `<pre data-lang="${lang}" data-theme="${theme}" data-bytes="${code.length}"></pre>`;
}

describe("MVP-15 Phase D §F.2 scheduler three-tier throughput", () => {
  bench(
    "< 1MB sync scheduler path",
    async () => {
      await scheduleHighlight(
        underOneMbTypeScript,
        "typescript",
        "light",
        underOneMbTypeScript.length,
        schedulerProbe,
      );
    },
    BENCH_OPTIONS,
  );

  bench(
    "1MB-10MB requestIdleCallback scheduler path",
    async () => {
      await scheduleHighlight(
        oneMbTypeScript,
        "typescript",
        "light",
        oneMbTypeScript.length,
        schedulerProbe,
      );
    },
    BENCH_OPTIONS,
  );

  bench(
    ">=10MB worker fail fallback requestIdleCallback scheduler path",
    async () => {
      await scheduleHighlight(
        tenMbTypeScript,
        "typescript",
        "dark",
        tenMbTypeScript.length,
        schedulerProbe,
      );
    },
    BENCH_OPTIONS,
  );
});
