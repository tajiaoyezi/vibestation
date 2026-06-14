import {
  buildFontStack,
  primaryFontFamily,
  quoteFontFamily,
} from "../../utils/font-stack";

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

export function buildTerminalFontStack(preferred: string): string {
  const normalized = primaryFontFamily(preferred);
  const shouldPreferUserFont =
    normalized !== "" && normalized !== "JetBrainsMono NF";
  if (!shouldPreferUserFont) {
    return Array.from(new Set(TERMINAL_FONT_FALLBACKS))
      .map(quoteFontFamily)
      .join(", ");
  }
  return buildFontStack(normalized, TERMINAL_FONT_FALLBACKS);
}
