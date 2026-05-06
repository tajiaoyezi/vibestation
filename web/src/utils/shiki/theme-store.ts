// MVP-15 Phase B · 主题 reactive store
//
// 设计：
// - 模块顶层 createSignal · 全局单例
// - DOM `data-shiki-theme` attribute 是 source of truth（CSS 也读它）
// - setShikiTheme 同时写 DOM + signal · 让 SolidJS 组件能 track theme 变化
// - 不监听外部 DOM 改动（避免 MutationObserver 复杂度 · 项目内统一走 setShikiTheme）

import { createSignal } from "solid-js";

export type ShikiTheme = "light" | "dark";

function readInitialTheme(): ShikiTheme {
  if (typeof document === "undefined") return "light";
  const attr = document.documentElement.getAttribute("data-shiki-theme");
  return attr === "dark" ? "dark" : "light";
}

const [theme, setThemeSignal] = createSignal<ShikiTheme>(readInitialTheme());

/**
 * SolidJS signal getter · 在 createEffect / createMemo 内调用会自动 track
 *
 * @example
 * createEffect(() => {
 *   const t = useShikiTheme();  // theme 变化时 effect 重跑
 *   shikiAdapter.highlight(code, lang, t).then(setHtml);
 * });
 */
export function useShikiTheme(): ShikiTheme {
  return theme();
}

/**
 * 设置 shiki theme · 同步写入 DOM attribute 和 SolidJS signal
 * - DOM attribute 让 CSS variable 切换
 * - signal 让消费 useShikiTheme() 的组件触发重渲
 */
export function setShikiTheme(next: ShikiTheme): void {
  if (typeof document !== "undefined") {
    document.documentElement.setAttribute("data-shiki-theme", next);
  }
  setThemeSignal(next);
}
