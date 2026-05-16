import {
  createContext,
  onCleanup,
  onMount,
  useContext,
  type Accessor,
  type ParentComponent,
} from "solid-js";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  PaneBuildFailedEvent,
  PaneFailurePreviewRequest,
  PaneFailurePreviewResult,
  PaneLinkErrorEvent,
  PaneLinkedEvent,
  PaneLinkRequest,
  PaneLinkResult,
  PaneLinkSetEnabledRequest,
  PaneLinksListRequest,
  PaneLinksListResult,
  PaneTriggerEvent,
  PaneUnlinkRequest,
  PaneUnlinkResult,
} from "../bindings";
import {
  linkPanes,
  listPaneLinks,
  previewFailurePrompt,
  setPaneLinkEnabled,
  unlinkPane,
} from "../panels/Terminal/paneLinkApi";
import { createPaneLinksStore, type PaneLinksStore } from "./paneLinks";

export type PaneLinksContextValue = {
  store: PaneLinksStore;
  selectorsFor: (
    workspaceId: Accessor<string | null | undefined>,
  ) => ReturnType<PaneLinksStore["createWorkspaceScopedSelectors"]>;
  createLink: (req: PaneLinkRequest) => Promise<PaneLinkResult>;
  unlink: (req: PaneUnlinkRequest) => Promise<PaneUnlinkResult>;
  setEnabled: (req: PaneLinkSetEnabledRequest) => Promise<PaneLinkResult>;
  listLinks: (req: PaneLinksListRequest) => Promise<PaneLinksListResult>;
  previewPrompt: (
    req: PaneFailurePreviewRequest,
  ) => Promise<PaneFailurePreviewResult>;
};

const PaneLinksContext = createContext<PaneLinksContextValue>();

export const PaneLinksProvider: ParentComponent = (props) => {
  const store = createPaneLinksStore();
  let unlisteners: UnlistenFn[] = [];
  let disposed = false;

  onMount(() => {
    void Promise.all([
      listen<PaneLinkedEvent>("pane:linked", (event) =>
        store.applyLinkedEvent(event.payload),
      ),
      listen<PaneBuildFailedEvent>("pane:build-failed", (event) =>
        store.applyBuildFailedEvent(event.payload),
      ),
      listen<PaneLinkErrorEvent>("pane:link-error", (event) =>
        store.applyErrorEvent(event.payload),
      ),
      listen<PaneTriggerEvent>("pane:trigger", () => {
        // MVP-18 Wave 1-A only wires the subscription seam; failure display is
        // driven by the richer pane:build-failed event.
      }),
    ])
      .then((nextUnlisteners) => {
        if (disposed) {
          nextUnlisteners.forEach((unlisten) => unlisten());
          return;
        }
        unlisteners = nextUnlisteners;
      })
      .catch((error) => {
        console.warn("[mvp-18] pane link event subscription failed:", error);
      });
  });

  onCleanup(() => {
    disposed = true;
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners = [];
  });

  const value: PaneLinksContextValue = {
    store,
    selectorsFor: (workspaceId) =>
      store.createWorkspaceScopedSelectors(workspaceId),
    createLink: linkPanes,
    unlink: unlinkPane,
    setEnabled: setPaneLinkEnabled,
    listLinks: listPaneLinks,
    previewPrompt: previewFailurePrompt,
  };

  return (
    <PaneLinksContext.Provider value={value}>
      {props.children}
    </PaneLinksContext.Provider>
  );
};

export function usePaneLinks(): PaneLinksContextValue {
  const ctx = useContext(PaneLinksContext);
  if (!ctx) {
    throw new Error("usePaneLinks must be used within PaneLinksProvider");
  }
  return ctx;
}
