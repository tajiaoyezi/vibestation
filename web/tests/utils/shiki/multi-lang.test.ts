// MVP-15 Phase B · 多语言支持 RED
// 验证 Tier 1 10 种语言（spec §B.5）都能 highlight · 不退到 plain text fallback

import { describe, it, expect } from "vitest";
import {
  createShikiAdapter,
  guessLanguageFromPath,
} from "../../../src/utils/shiki";

const TIER1_SAMPLES: Array<[string, string]> = [
  ["javascript", "const x = 1;"],
  ["typescript", "const x: number = 1;"],
  ["rust", 'fn main() { println!("hi"); }'],
  ["python", "x = 1"],
  ["go", "var x = 1"],
  ["java", "int x = 1;"],
  ["markdown", "# Hello"],
  ["json", '{"x": 1}'],
  ["yaml", "x: 1"],
  ["shell", 'echo "hi"'],
];

describe("Tier 1 多语言 highlight", () => {
  it.each(TIER1_SAMPLES)(
    "lang=%s 应输出 shiki 格式 HTML（含 .shiki container）",
    async (lang, code) => {
      // 真实 highlight · 验证 lang 已 preload · 不走 fallback
      const adapter = createShikiAdapter();
      const html = await adapter.highlight(code, lang, "light");

      // shiki 输出特征：含 <pre class="shiki ..."> 或类似 token span
      // fallback 时只会 escape HTML · 不含 shiki container
      expect(html).toMatch(/class="shiki|<span/);
    },
  );

  it("dark 主题与 light 主题输出不同 HTML（颜色不同）", async () => {
    // 同 code 同 lang · 不同 theme 应缓存独立 · HTML 应不同
    const adapter = createShikiAdapter();
    const code = "const x = 1;";
    const lightHtml = await adapter.highlight(code, "typescript", "light");
    const darkHtml = await adapter.highlight(code, "typescript", "dark");

    expect(lightHtml).not.toBe(darkHtml);
    // 缓存应有 2 entries
    expect(adapter.getCacheStats().fileCount).toBe(2);
  });
});

describe("guessLanguageFromPath 扩展（Phase B 新增映射）", () => {
  it("应识别 .sh / .bash / .zsh 为 shell（与 shiki lang ID 对齐）", () => {
    // Phase B 与 shiki 的 lang ID 对齐 · 'shell' 而非 'bash'
    expect(guessLanguageFromPath("script.sh")).toBe("shell");
    expect(guessLanguageFromPath("setup.bash")).toBe("shell");
    expect(guessLanguageFromPath("dotfile.zsh")).toBe("shell");
  });

  it("应识别 .yaml / .yml 为 yaml", () => {
    // 验证 yaml 后缀映射稳定
    expect(guessLanguageFromPath("config.yaml")).toBe("yaml");
    expect(guessLanguageFromPath("docker-compose.yml")).toBe("yaml");
  });

  it("应识别 .md / .markdown 为 markdown", () => {
    // markdown 双后缀
    expect(guessLanguageFromPath("README.md")).toBe("markdown");
    expect(guessLanguageFromPath("notes.markdown")).toBe("markdown");
  });
});
