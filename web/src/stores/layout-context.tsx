import { createContext, useContext, type ParentComponent } from "solid-js";
import { createSignal } from "solid-js";
import {
  type LayoutState,
  type LayoutAction,
  DEFAULT_LAYOUT,
  layoutReducer,
} from "./layout";
import { invoke } from "@tauri-apps/api/core";

interface LayoutContextValue {
  layout: () => LayoutState;
  dispatch: (action: LayoutAction) => void;
  loadForWorkspace: (workspaceId: string) => Promise<void>;
}

const LayoutContext = createContext<LayoutContextValue>();

export const LayoutProvider: ParentComponent<{
  activeWorkspaceId: () => string | null;
}> = (props) => {
  const [layout, setLayout] = createSignal<LayoutState>({ ...DEFAULT_LAYOUT });
  let persistTimer: ReturnType<typeof setTimeout> | null = null;

  const schedulePersist = () => {
    if (persistTimer) clearTimeout(persistTimer);
    persistTimer = setTimeout(() => {
      const id = props.activeWorkspaceId();
      if (id) {
        invoke("layout_save", { workspaceId: id, layoutState: layout() }).catch(
          () => {},
        );
      }
    }, 250);
  };

  const dispatch = (action: LayoutAction) => {
    setLayout((prev) => layoutReducer(prev, action));
    if (action.kind !== "restore") {
      schedulePersist();
    }
  };

  const loadForWorkspace = async (id: string) => {
    try {
      const state = await invoke<LayoutState>("layout_load", {
        workspaceId: id,
      });
      dispatch({ kind: "restore", state });
    } catch {
      dispatch({ kind: "restore", state: { ...DEFAULT_LAYOUT } });
    }
  };

  return (
    <LayoutContext.Provider value={{ layout, dispatch, loadForWorkspace }}>
      {props.children}
    </LayoutContext.Provider>
  );
};

export function useLayout() {
  const ctx = useContext(LayoutContext);
  if (!ctx) throw new Error("useLayout must be used within LayoutProvider");
  return [ctx.layout, ctx.dispatch] as const;
}
