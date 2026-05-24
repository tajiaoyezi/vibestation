import type { ImportFieldType } from "@/bindings";

/**
 * Issue #205 Finding 2 (a) 修复 · ANSI color v0.1 实务方案：
 *
 * 后端 apply 函数明示「ANSI color 由前端处理 · v0.1 不进 app_settings」·
 * 即 ANSI palette 在 v0.1 不会被持久化。前端如果让用户勾选 ANSI checkbox 后
 * apply · UI 显示 success（0 errors）· 但实际颜色没改 · 用户体验错乱。
 *
 * 实务方案：v0.1 默认 disable ANSI color checkbox + 显式 tooltip 告知 v0.2 支持。
 * v0.2 实施 ANSI palette 持久化时 · 移除此 helper 或 disabled 标志。
 */

const ANSI_COLOR_DISABLED_REASON =
  "ANSI palette 持久化将在 v0.2 支持 · v0.1 勾选无效不会被 apply";

/** 字段在 v0.1 是否被 disabled（不可勾选 / 不可被默认勾选） */
export function isFieldDisabled(field: ImportFieldType): boolean {
  return field.kind === "ansiColor";
}

/** disabled 字段的 tooltip 文案 · 用于 checkbox title attribute */
export function getDisabledReason(field: ImportFieldType): string | undefined {
  if (field.kind === "ansiColor") {
    return ANSI_COLOR_DISABLED_REASON;
  }
  return undefined;
}
