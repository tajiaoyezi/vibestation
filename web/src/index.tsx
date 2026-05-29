/* @refresh reload */
import { render } from "solid-js/web";
import { App } from "./App";
import { applyPlatformClass } from "@/lib/platform";
import "./styles.css";
import "./styles/typography.css";

// 平台 class + data-platform 属性（Task 4.1 · single source = @/lib/platform）
applyPlatformClass();

if (import.meta.env.PROD) {
  document.addEventListener("contextmenu", (e) => e.preventDefault());
  document.addEventListener("keydown", (e) => {
    const mac = navigator.platform.toLowerCase().includes("mac");
    const meta = mac ? e.metaKey : e.ctrlKey;
    if (meta && e.key === "r") e.preventDefault();
    if (meta && (e.key === "-" || e.key === "=" || e.key === "+"))
      e.preventDefault();
    if (
      meta &&
      e.key === "a" &&
      !(
        e.target instanceof Element &&
        e.target.closest(
          ".xterm, .diff-view, input, textarea, [data-selectable]",
        )
      )
    )
      e.preventDefault();
  });
}

const rootEl = document.getElementById("root");
if (!rootEl) {
  throw new Error("Root element #root not found");
}
render(() => <App />, rootEl);
