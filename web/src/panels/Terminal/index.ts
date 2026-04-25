import "@xterm/xterm/css/xterm.css";
import "./styles.css";

export { Terminal } from "./Terminal";

// MVP-05 Phase C · scaffolding · 集成留 PR #145
export { PaneTerminal, type PaneTerminalApi } from "./PaneTerminal";
export { PaneSplitView } from "./PaneSplitView";
export { PaneSplitter } from "./PaneSplitter";
export { usePaneShortcuts } from "./usePaneShortcuts";
