import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createSignal } from "solid-js";
import type { PaneDetachOpenRequest } from "../bindings/PaneDetachOpenRequest";
import type { PaneDetachOpenResult } from "../bindings/PaneDetachOpenResult";
import type { PaneDetachCloseRequest } from "../bindings/PaneDetachCloseRequest";
import type { PaneDetachCloseResult } from "../bindings/PaneDetachCloseResult";
import type { PaneDetachListEntry } from "../bindings/PaneDetachListEntry";
import type { PaneDetachStateEvent } from "../bindings/PaneDetachStateEvent";

/**
 * MVP-17 Phase B · Pane Detach IPC wrapper + runtime state signal
 */

// runtime-only Solid signal · paneId → windowLabel · App.tsx 监听
export const [detachedPanes, setDetachedPanes] = createSignal<
  Map<string, string>
>(new Map());

export async function detachPane(
  request: PaneDetachOpenRequest,
): Promise<PaneDetachOpenResult> {
  return invoke<PaneDetachOpenResult>("pane_detach_open", { request });
}

export async function closeDetachedPane(
  request: PaneDetachCloseRequest,
): Promise<PaneDetachCloseResult> {
  return invoke<PaneDetachCloseResult>("pane_detach_close", { request });
}

export async function listDetachedPanes(): Promise<PaneDetachListEntry[]> {
  return invoke<PaneDetachListEntry[]>("pane_detach_list");
}

// 启动时调一次 · 订阅 backend 事件
export async function initPaneDetachStateListener(): Promise<() => void> {
  const unlisten = await listen<PaneDetachStateEvent>(
    "pane_detach_state_changed",
    (event) => {
      const { paneId, action, windowLabel } = event.payload;
      setDetachedPanes((prev) => {
        const next = new Map(prev);
        if (action === "detached" && windowLabel) {
          next.set(paneId, windowLabel);
        } else {
          next.delete(paneId);
        }
        return next;
      });
    },
  );

  return unlisten;
}
