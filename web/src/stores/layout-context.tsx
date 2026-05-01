import { createContext, useContext, type ParentComponent } from "solid-js";
import { createEffect, createSignal } from "solid-js";
import {
  type LayoutState,
  type LayoutAction,
  DEFAULT_LAYOUT,
  layoutReducer,
} from "./layout";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, SettingsUpdateRequest } from "../bindings";

interface LayoutContextValue {
  layout: () => LayoutState;
  dispatch: (action: LayoutAction) => void;
  loadForWorkspace: (workspaceId: string) => Promise<void>;
}

const LayoutContext = createContext<LayoutContextValue>();

// 哪些 LayoutState 字段是全局（跨 workspace 共享 · 持久化到 app_settings）·
// 哪些是 per-workspace（持久化到 workspace_layout 表）。
const GLOBAL_KEYS = ["primaryWidth", "secondaryWidth", "bottomHeight"] as const;

export const LayoutProvider: ParentComponent<{
  activeWorkspaceId: () => string | null;
  dbReady: () => boolean;
}> = (props) => {
  const [layout, setLayout] = createSignal<LayoutState>({ ...DEFAULT_LAYOUT });
  let perWsPersistTimer: ReturnType<typeof setTimeout> | null = null;
  let globalPersistTimer: ReturnType<typeof setTimeout> | null = null;
  let globalHydrated = false;

  // 启动时一次性拉全局 layout（width / height）· 之后 workspace 切换不再覆盖
  // 等 dbReady（workspace_init resolve · pool 已初始化）· 否则 settings_get 会
  // 在 module-load race 期失败 · 保留 defaults · 用户重启永远看不到上次的宽度
  const initGlobalLayout = async () => {
    if (globalHydrated) return;
    try {
      const s = await invoke<AppSettings>("settings_get");
      setLayout((prev) => ({
        ...prev,
        primaryWidth: s.primaryWidth || prev.primaryWidth,
        secondaryWidth: s.secondaryWidth || prev.secondaryWidth,
        bottomHeight: s.bottomHeight || prev.bottomHeight,
      }));
      globalHydrated = true;
    } catch {
      // 失败保留 DEFAULT_LAYOUT · 不阻塞 · dbReady 下次变 true 时再试
    }
  };

  createEffect(() => {
    if (props.dbReady()) {
      void initGlobalLayout();
    }
  });

  const schedulePerWorkspacePersist = () => {
    if (perWsPersistTimer) clearTimeout(perWsPersistTimer);
    perWsPersistTimer = setTimeout(() => {
      const id = props.activeWorkspaceId();
      if (id) {
        // 仅持久化 open/close · width 类全局字段已走 settings · 这里写也不影响
        // （后端 layout_save 仍接收完整 LayoutState · 不破兼容）
        invoke("layout_save", { workspaceId: id, layoutState: layout() }).catch(
          () => {},
        );
      }
    }, 250);
  };

  const scheduleGlobalPersist = () => {
    if (globalPersistTimer) clearTimeout(globalPersistTimer);
    globalPersistTimer = setTimeout(() => {
      const cur = layout();
      const req: SettingsUpdateRequest = {
        theme: null,
        fontFamily: null,
        fontSize: null,
        defaultShell: null,
        pasteProtection: null,
        telemetryOptIn: null,
        gitUserName: null,
        gitUserEmail: null,
        bgOpacity: null,
        bgBlur: null,
        windowPaddingX: null,
        windowPaddingY: null,
        cursorStyle: null,
        cursorBlink: null,
        unfocusedPaneOpacity: null,
        ptyPoolEnabled: null,
        ptyPoolSize: null,
        primaryWidth: cur.primaryWidth,
        secondaryWidth: cur.secondaryWidth,
        bottomHeight: cur.bottomHeight,
      };
      invoke("settings_update", { req }).catch(() => {});
    }, 250);
  };

  const isGlobalAction = (action: LayoutAction): boolean => {
    return (
      action.kind === "resize-primary" ||
      action.kind === "resize-secondary" ||
      action.kind === "resize-bottom" ||
      action.kind === "reset-primary" ||
      action.kind === "reset-secondary" ||
      action.kind === "reset-bottom"
    );
  };

  const dispatch = (action: LayoutAction) => {
    setLayout((prev) => layoutReducer(prev, action));
    if (action.kind === "restore") return;
    if (isGlobalAction(action)) {
      scheduleGlobalPersist();
    } else {
      schedulePerWorkspacePersist();
    }
  };

  const loadForWorkspace = async (id: string) => {
    try {
      const state = await invoke<LayoutState>("layout_load", {
        workspaceId: id,
      });
      // 切 workspace 只接受 open/close · width 类用当前全局值（不被 workspace 覆盖）
      const cur = layout();
      const merged: LayoutState = {
        ...state,
        primaryWidth: cur.primaryWidth,
        secondaryWidth: cur.secondaryWidth,
        bottomHeight: cur.bottomHeight,
      };
      dispatch({ kind: "restore", state: merged });
    } catch {
      const cur = layout();
      dispatch({
        kind: "restore",
        state: {
          ...DEFAULT_LAYOUT,
          primaryWidth: cur.primaryWidth,
          secondaryWidth: cur.secondaryWidth,
          bottomHeight: cur.bottomHeight,
        },
      });
    }
  };

  return (
    <LayoutContext.Provider value={{ layout, dispatch, loadForWorkspace }}>
      {props.children}
    </LayoutContext.Provider>
  );
};

// 标记 GLOBAL_KEYS 用于将来类型检查 · 当前 isGlobalAction 直接 switch action.kind
void GLOBAL_KEYS;

export function useLayout() {
  const ctx = useContext(LayoutContext);
  if (!ctx) throw new Error("useLayout must be used within LayoutProvider");
  return ctx;
}
