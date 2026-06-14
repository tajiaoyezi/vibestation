const CSS_GENERIC_FAMILIES = new Set([
  "system-ui",
  "sans-serif",
  "serif",
  "monospace",
  "ui-sans-serif",
  "ui-serif",
  "ui-monospace",
]);

/** CSS 泛型族 / 厂商关键字不加引号，避免 `"system-ui"` 被当成自定义字体名。 */
export function quoteFontFamily(font: string): string {
  const trimmed = font.trim();
  if (!trimmed) return trimmed;
  if (CSS_GENERIC_FAMILIES.has(trimmed) || trimmed.startsWith("-")) {
    return trimmed;
  }
  return `"${trimmed}"`;
}

/** 逗号分隔栈只取首选族名（legacy DB / Rust 默认栈兼容）。 */
export function primaryFontFamily(value: string): string {
  const first = value.split(",")[0]?.trim() ?? "";
  return first || value.trim();
}

export function buildFontStack(
  primary: string,
  fallbacks: readonly string[],
): string {
  const normalized = primaryFontFamily(primary);
  const stack = normalized
    ? [normalized, ...fallbacks.filter((font) => font !== normalized)]
    : [...fallbacks];
  return Array.from(new Set(stack)).map(quoteFontFamily).join(", ");
}

export const UI_FONT_FALLBACKS = [
  "Inter Variable",
  "Inter",
  "-apple-system",
  "Segoe UI Variable Text",
  "Segoe UI",
  "system-ui",
  "sans-serif",
] as const;

export function buildUiFontStack(preferred: string): string {
  return buildFontStack(preferred, UI_FONT_FALLBACKS);
}
