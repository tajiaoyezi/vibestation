// ShikiAdapter · shiki v3+ 语法高亮适配层
// MVP-15 Phase A · 仅 TypeScript + light/dark 主题

import { createHighlighterCore, type HighlighterCore } from "shiki/core";
import { createOnigurumaEngine } from "shiki/engine/oniguruma";
import githubLight from "shiki/themes/github-light.mjs";
import githubDark from "shiki/themes/github-dark.mjs";
import javascript from "shiki/langs/javascript.mjs";
import typescript from "shiki/langs/typescript.mjs";
import rust from "shiki/langs/rust.mjs";
import python from "shiki/langs/python.mjs";
import go from "shiki/langs/go.mjs";
import java from "shiki/langs/java.mjs";
import markdown from "shiki/langs/markdown.mjs";
import json from "shiki/langs/json.mjs";
import yaml from "shiki/langs/yaml.mjs";
import shell from "shiki/langs/shell.mjs";
import { setShikiTheme } from "./theme-store";

export interface ShikiAdapter {
  highlight(
    code: string,
    lang: string,
    theme: "light" | "dark",
  ): Promise<string>;
  setTheme(theme: "light" | "dark"): void;
  clearCache(): void;
  getCacheStats(): { fileCount: number; sizeBytes: number };
}

export const SHIKI_CACHE_LIMITS = {
  maxFiles: 100,
  maxSizeBytes: 50 * 1024 * 1024, // 50MB
};

interface CacheEntry {
  html: string;
  sizeBytes: number;
  lastAccessTime: number;
}

class LRUCache {
  private map = new Map<string, CacheEntry>();
  private currentSizeBytes = 0;

  get(key: string): CacheEntry | undefined {
    const entry = this.map.get(key);
    if (entry) {
      entry.lastAccessTime = Date.now();
      this.map.delete(key);
      this.map.set(key, entry);
    }
    return entry;
  }

  set(key: string, html: string): void {
    const sizeBytes = new TextEncoder().encode(html).length;

    // 如果单个条目超过 maxSizeBytes，拒绝缓存
    if (sizeBytes > SHIKI_CACHE_LIMITS.maxSizeBytes) {
      return;
    }

    // 删除已存在的同名 key
    const existing = this.map.get(key);
    if (existing) {
      this.currentSizeBytes -= existing.sizeBytes;
      this.map.delete(key);
    }

    // 淘汰最久未用，直到能放下新条目
    while (
      this.map.size >= SHIKI_CACHE_LIMITS.maxFiles ||
      this.currentSizeBytes + sizeBytes > SHIKI_CACHE_LIMITS.maxSizeBytes
    ) {
      const oldest = this.map.keys().next().value;
      if (oldest === undefined) break;
      const oldEntry = this.map.get(oldest)!;
      this.currentSizeBytes -= oldEntry.sizeBytes;
      this.map.delete(oldest);
    }

    this.map.set(key, { html, sizeBytes, lastAccessTime: Date.now() });
    this.currentSizeBytes += sizeBytes;
  }

  clear(): void {
    this.map.clear();
    this.currentSizeBytes = 0;
  }

  stats(): { fileCount: number; sizeBytes: number } {
    return { fileCount: this.map.size, sizeBytes: this.currentSizeBytes };
  }
}

let highlighterPromise: Promise<HighlighterCore> | null = null;

/**
 * Tier 1 语言（spec §B.5）· 一次性 preload 避免首屏 race
 * bundle 增量 ~1MB textmate grammar · 在 spec R1 监控范围内
 */
export const TIER1_LANGS = [
  "javascript",
  "typescript",
  "rust",
  "python",
  "go",
  "java",
  "markdown",
  "json",
  "yaml",
  "shell",
] as const;

export type Tier1Lang = (typeof TIER1_LANGS)[number];

function getHighlighter(): Promise<HighlighterCore> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighterCore({
      themes: [githubLight, githubDark],
      langs: [
        javascript,
        typescript,
        rust,
        python,
        go,
        java,
        markdown,
        json,
        yaml,
        shell,
      ],
      engine: createOnigurumaEngine(import("shiki/wasm")),
    });
  }
  return highlighterPromise;
}

function buildCacheKey(code: string, lang: string, theme: string): string {
  // 简单 hash：前 100 字符 + 长度 + lang + theme
  const prefix = code.slice(0, 100);
  return `${lang}:${theme}:${code.length}:${prefix}`;
}

export function fallbackToPlainText(code: string): string {
  // 纯文本 · 无 HTML 标签 · 不 crash
  return code
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

export function guessLanguageFromPath(path: string): string | null {
  const ext = path.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "ts":
    case "tsx":
      return "typescript";
    case "js":
    case "jsx":
      return "javascript";
    case "py":
      return "python";
    case "rs":
      return "rust";
    case "go":
      return "go";
    case "java":
      return "java";
    case "md":
    case "markdown":
      return "markdown";
    case "json":
      return "json";
    case "yml":
    case "yaml":
      return "yaml";
    case "sh":
    case "bash":
    case "zsh":
      return "shell";
    default:
      return null;
  }
}

export function createShikiAdapter(): ShikiAdapter {
  const cache = new LRUCache();

  return {
    async highlight(code, lang, theme) {
      const cacheKey = buildCacheKey(code, lang, theme);
      const cached = cache.get(cacheKey);
      if (cached) {
        return cached.html;
      }

      try {
        const highlighter = await getHighlighter();
        const html = highlighter.codeToHtml(code, {
          lang,
          theme: theme === "light" ? "github-light" : "github-dark",
        });
        cache.set(cacheKey, html);
        return html;
      } catch (err) {
        console.warn(
          `[shiki] highlight failed for lang=${lang}:`,
          err instanceof Error ? err.message : String(err),
        );
        return fallbackToPlainText(code);
      }
    },

    setTheme(theme) {
      // 同步到 reactive theme store · 触发消费 useShikiTheme() 的组件重渲
      setShikiTheme(theme);
    },

    clearCache() {
      cache.clear();
    },

    getCacheStats() {
      return cache.stats();
    },
  };
}

// 默认导出：方便直接 import { shikiAdapter } from "~/utils/shiki"
export const shikiAdapter = createShikiAdapter();
