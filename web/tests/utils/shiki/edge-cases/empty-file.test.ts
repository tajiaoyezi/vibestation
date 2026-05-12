import { describe, it, expect } from "vitest";
import { createShikiAdapter } from "../../../../src/utils/shiki";

describe("G.3 空文件不崩溃", () => {
  it("空字符串不应 throw", async () => {
    const adapter = createShikiAdapter();
    const result = await adapter.highlight("", "typescript", "light");
    expect(result).toBeDefined();
  });

  it("空字符串应返回空或合理 HTML", async () => {
    const adapter = createShikiAdapter();
    const result = await adapter.highlight("", "typescript", "light");
    // shiki 对空字符串返回含空 pre/code 的 HTML（含 <span class="line"></span>）
    expect(result).toBeTruthy();
    expect(result).toContain("<pre");
  });

  it("仅空白字符不应 throw", async () => {
    const adapter = createShikiAdapter();
    const result = await adapter.highlight("   ", "typescript", "light");
    expect(result).toBeDefined();
  });

  it("仅换行符不应 throw", async () => {
    const adapter = createShikiAdapter();
    const result = await adapter.highlight("\n\n\n", "typescript", "light");
    expect(result).toBeDefined();
  });

  it("空字符串缓存应正常", async () => {
    const adapter = createShikiAdapter();
    const result1 = await adapter.highlight("", "typescript", "light");
    const result2 = await adapter.highlight("", "typescript", "light");
    expect(result1).toBe(result2);
    expect(adapter.getCacheStats().fileCount).toBe(1);
  });
});
