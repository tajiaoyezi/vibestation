import type { Component, JSX } from "solid-js";

const iconProps: JSX.SvgSVGAttributes<SVGSVGElement> = {
  width: 14,
  height: 14,
  viewBox: "0 0 16 16",
  fill: "none",
  stroke: "currentColor",
  "stroke-width": "1.5",
  "stroke-linecap": "round",
  "stroke-linejoin": "round",
};

export const SunIcon: Component = () => (
  <svg {...iconProps} aria-hidden="true">
    <circle cx="8" cy="8" r="3" />
    <path d="M8 1v1.5M8 14v-1.5M2.5 3l1 1M12.5 12l1 1M1 8h1.5M13.5 8H15M3 13l1-1M12 3l1-1" />
  </svg>
);

export const MoonIcon: Component = () => (
  <svg {...iconProps} aria-hidden="true">
    <path d="M5.5 2.5a6.5 6.5 0 0 0 7 9.5 5.5 5.5 0 1 1-7-9.5Z" />
  </svg>
);

export const AutoIcon: Component = () => (
  <svg {...iconProps} aria-hidden="true">
    <circle cx="8" cy="8" r="6.5" />
    <path
      d="M8 1.5a6.5 6.5 0 0 1 0 13"
      fill="currentColor"
      stroke="none"
      opacity=".35"
    />
  </svg>
);

export const GearIcon: Component = () => (
  <svg {...iconProps} aria-hidden="true">
    <circle
      cx="8"
      cy="8"
      r="4.2"
      stroke-width="2.8"
      stroke-dasharray="2.2 3.3"
      stroke-linecap="butt"
    />
    <circle cx="8" cy="8" r="2.8" />
    <circle cx="8" cy="8" r="1" fill="currentColor" stroke="none" />
  </svg>
);

/** Toggle primary sidebar 图标 · Cursor / VSCode 标准样式：
 *  外框矩形 + 左侧实心条表示 sidebar · 与 ⌘B 行为一致 */
export const SidebarLeftIcon: Component = () => (
  <svg {...iconProps} aria-hidden="true">
    <rect x="2" y="3" width="12" height="10" rx="1.5" />
    <line x1="6" y1="3" x2="6" y2="13" />
    <rect
      x="2.5"
      y="3.5"
      width="3"
      height="9"
      fill="currentColor"
      stroke="none"
      rx="1"
    />
  </svg>
);
