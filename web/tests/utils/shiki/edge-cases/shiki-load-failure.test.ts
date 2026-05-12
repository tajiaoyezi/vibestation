import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { fallbackToPlainText } from "../../../../src/utils/shiki";

describe("G.1 shiki 加载失败降级", () => {
  let consoleWarnSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    consoleWarnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    vi.resetModules();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("createHighlighterCore reject 时应降级到 fallbackToPlainText", async () => {
    vi.doMock("shiki/core", () => ({
      createHighlighterCore: vi.fn().mockRejectedValue(new Error("network error")),
    }));

    // 动态重新导入以使用 mock
    const { createShikiAdapter: createMockedAdapter } = await import(
      "../../../../src/utils/shiki"
    );

    const adapter = createMockedAdapter();
    const code = "const x = 1;";
    const result = await adapter.highlight(code, "typescript", "light");

    // fallbackToPlainText 不生成 HTML 标签，只做 escape
    expect(result).not.toContain("<span");
    expect(result).toContain("const x = 1;");
    expect(result).toBe(fallbackToPlainText(code));
  });

  it("多次调用时 highlighterPromise 只创建一次", async () => {
    const mockCreateHighlighter = vi
      .fn()
      .mockRejectedValue(new Error("CDN unreachable"));
    vi.doMock("shiki/core", () => ({
      createHighlighterCore: mockCreateHighlighter,
    }));

    const { createShikiAdapter: createMockedAdapter } = await import(
      "../../../../src/utils/shiki"
    );

    const adapter = createMockedAdapter();
    const code = "let y = 2;";

    await adapter.highlight(code, "typescript", "light");
    await adapter.highlight(code, "typescript", "light");

    // highlighterPromise 是单例，createHighlighterCore 只应被调一次
    expect(mockCreateHighlighter).toHaveBeenCalledTimes(1);
    // 每次 highlight 失败都会 console.warn，所以是 2 次
    expect(consoleWarnSpy).toHaveBeenCalledTimes(2);
  });

  it("错误路径不缓存，每次调用都走 fallback", async () => {
    vi.doMock("shiki/core", () => ({
      createHighlighterCore: vi.fn().mockRejectedValue(new Error("timeout")),
    }));

    const { createShikiAdapter: createMockedAdapter } = await import(
      "../../../../src/utils/shiki"
    );

    const adapter = createMockedAdapter();
    const code = "function test() {}";
    const theme = "light";

    const result1 = await adapter.highlight(code, "typescript", theme);
    const result2 = await adapter.highlight(code, "typescript", theme);

    // 两次结果相同（都是 fallback），但 cache stats 应为 0
    expect(result1).toBe(result2);
    expect(adapter.getCacheStats().fileCount).toBe(0);
    expect(consoleWarnSpy).toHaveBeenCalledTimes(2);
  });

  it("不同代码/语言组合应各自独立降级", async () => {
    vi.doMock("shiki/core", () => ({
      createHighlighterCore: vi
        .fn()
        .mockRejectedValue(new Error("module not found")),
    }));

    const { createShikiAdapter: createMockedAdapter } = await import(
      "../../../../src/utils/shiki"
    );

    const adapter = createMockedAdapter();
    const code1 = "const a = 1;";
    const code2 = "let b = 2;";

    const result1 = await adapter.highlight(code1, "typescript", "light");
    const result2 = await adapter.highlight(code2, "typescript", "light");

    // 各自 fallback，但内容不同
    expect(result1).toBe(fallbackToPlainText(code1));
    expect(result2).toBe(fallbackToPlainText(code2));
    expect(result1).not.toBe(result2);
  });
});
