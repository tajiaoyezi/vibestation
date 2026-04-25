/* @refresh reload */
import { render } from "solid-js/web";
import { App } from "./App";
import "./styles.css";
import "./styles/typography.css";

const rootEl = document.getElementById("root");
if (!rootEl) {
  throw new Error("Root element #root not found");
}
render(() => <App />, rootEl);
