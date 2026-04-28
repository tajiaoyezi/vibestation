import type { Component, JSX } from "solid-js";

const svgProps: JSX.SvgSVGAttributes<SVGSVGElement> = {
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
  <svg {...svgProps} aria-hidden="true">
    <circle cx="8" cy="8" r="3.5" />
    <path d="M8 1v1.5M8 13.5v1.5M3 3l1 1M12 12l1 1M1 8h1.5M13.5 8H15M3 13l1-1M12 4l1-1" />
  </svg>
);

export const MoonIcon: Component = () => (
  <svg {...svgProps} aria-hidden="true">
    <path d="M13 10.5A5.5 5.5 0 0 1 5.5 3 5 5 0 1 0 13 10.5Z" />
  </svg>
);

export const AutoIcon: Component = () => (
  <svg {...svgProps} aria-hidden="true">
    <circle cx="8" cy="8" r="6" />
    <path d="M8 2a6 6 0 0 1 0 12" />
    <circle cx="8" cy="8" r="1.5" fill="currentColor" stroke="none" />
  </svg>
);

export const GearIcon: Component = () => (
  <svg {...svgProps} aria-hidden="true">
    <circle cx="8" cy="8" r="3" />
    <path d="M8 1.5v1M8 13.5v1M2.5 4l.8.5M12.7 11.5l.8.5M1.5 8h1M13.5 8h1M2.5 12l.8-.5M12.7 4.5l.8-.5" />
    <circle cx="8" cy="8" r="1" fill="currentColor" stroke="none" />
  </svg>
);
