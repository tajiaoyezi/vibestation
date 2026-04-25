/* @refresh reload */
import { render } from "solid-js/web";
import { App } from "./App";
import "./styles.css";

const isLinux = navigator.platform.toLowerCase().includes("linux");
if (isLinux) {
  document.documentElement.classList.add("platform-linux");
}

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
