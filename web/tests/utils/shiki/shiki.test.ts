import { describe, it, expect, beforeEach } from "vitest";
import {
  createShikiAdapter,
  guessLanguageFromPath,
  fallbackToPlainText,
  SHIKI_CACHE_LIMITS,
} from "../../../src/utils/shiki";

describe("LRU cache", () => {
  it("should cache and retrieve highlighted code", async () => {
    const adapter = createShikiAdapter();
    const code = "const x = 1;";
    const html = await adapter.highlight(code, "typescript", "light");
    expect(html).toContain("<span"); // shiki 输出含 HTML tags
    expect(html).not.toContain("const x = 1;"); // 被 token 包裹

    const stats = adapter.getCacheStats();
    expect(stats.fileCount).toBe(1);
    expect(stats.sizeBytes).toBeGreaterThan(0);
  });

  it("should return cached result on second call", async () => {
    const adapter = createShikiAdapter();
    const code = "let y = 2;";
    const html1 = await adapter.highlight(code, "typescript", "light");
    const html2 = await adapter.highlight(code, "typescript", "light");
    expect(html1).toBe(html2);
  });

  it("should evict oldest when max files exceeded", async () => {
    const adapter = createShikiAdapter();
    const originalMax = SHIKI_CACHE_LIMITS.maxFiles;
    SHIKI_CACHE_LIMITS.maxFiles = 2;

    await adapter.highlight("const a = 1;", "typescript", "light");
    await adapter.highlight("const b = 2;", "typescript", "light");
    await adapter.highlight("const c = 3;", "typescript", "light");

    const stats = adapter.getCacheStats();
    expect(stats.fileCount).toBeLessThanOrEqual(2);

    SHIKI_CACHE_LIMITS.maxFiles = originalMax;
  });

  it("should clear cache", async () => {
    const adapter = createShikiAdapter();
    await adapter.highlight("const x = 1;", "typescript", "light");
    adapter.clearCache();
    const stats = adapter.getCacheStats();
    expect(stats.fileCount).toBe(0);
    expect(stats.sizeBytes).toBe(0);
  });
});

describe("guessLanguageFromPath", () => {
  it("should detect TypeScript", () => {
    expect(guessLanguageFromPath("foo.ts")).toBe("typescript");
    expect(guessLanguageFromPath("bar.tsx")).toBe("typescript");
  });

  it("should detect JavaScript", () => {
    expect(guessLanguageFromPath("foo.js")).toBe("javascript");
    expect(guessLanguageFromPath("bar.jsx")).toBe("javascript");
  });

  it("should detect Python", () => {
    expect(guessLanguageFromPath("script.py")).toBe("python");
  });

  it("should return null for unknown extension", () => {
    expect(guessLanguageFromPath("file.unknown")).toBeNull();
  });

  it("should return null for no extension", () => {
    expect(guessLanguageFromPath("Makefile")).toBeNull();
  });
});

describe("fallbackToPlainText", () => {
  it("should escape HTML entities", () => {
    const code = "<div>hello & goodbye</div>";
    const result = fallbackToPlainText(code);
    expect(result).toBe(
      "&lt;div&gt;hello &amp; goodbye&lt;/div&gt;",
    );
    expect(result).not.toContain("<div>");
  });
});

describe("ShikiAdapter theme", () => {
  it("should set theme via data-attribute", () => {
    const adapter = createShikiAdapter();
    adapter.setTheme("dark");
    expect(document.documentElement.getAttribute("data-shiki-theme")).toBe(
      "dark",
    );
  });

  it("should fallback to plain text on unsupported language", async () => {
    const adapter = createShikiAdapter();
    const code = "some code";
    // 使用一个 shiki 不支持的语言
    const html = await adapter.highlight(code, "unsupported-lang", "light");
    // 应该 fallback 到纯文本（escaped）
    expect(html).toContain("some code");
  });
});

describe("ShikiAdapter failure path", () => {
  it("should not throw on highlight failure", async () => {
    const adapter = createShikiAdapter();
    // 空字符串不应该 crash
    const html = await adapter.highlight("", "typescript", "light");
    expect(html).toBeDefined();
  });
});
