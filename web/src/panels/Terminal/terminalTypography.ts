export const TERMINAL_LINE_HEIGHT = 1.2;
export const TERMINAL_FONT_WEIGHT = 450;
export const TERMINAL_FONT_WEIGHT_BOLD = 700;

const TERMINAL_FONT_FALLBACKS = [
  "JetBrains Mono",
  "JetBrains Mono Variable",
  "JetBrainsMono NF",
  "Cascadia Code",
  "DejaVu Sans Mono",
  "Ubuntu Mono",
  "ui-monospace",
  "Liberation Mono",
  "Sarasa Term SC",
  "PingFang SC",
  "Hiragino Sans GB",
  "Microsoft YaHei",
  "Noto Sans CJK SC",
  "WenQuanYi Micro Hei",
  "monospace",
];

function quoteFontFamily(font: string): string {
  if (font === "ui-monospace" || font === "monospace") return font;
  return `"${font}"`;
}

export function buildTerminalFontStack(preferred: string): string {
  const normalized = preferred.trim();
  const shouldPreferUserFont =
    normalized !== "" && normalized !== "JetBrainsMono NF";
  const stack = shouldPreferUserFont
    ? [normalized, ...TERMINAL_FONT_FALLBACKS]
    : TERMINAL_FONT_FALLBACKS;

  return Array.from(new Set(stack)).map(quoteFontFamily).join(", ");
}
