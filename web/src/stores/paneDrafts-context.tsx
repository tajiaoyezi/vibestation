import { createContext, useContext, type ParentComponent } from "solid-js";
import {
  createPaneDraftsStore,
  type PaneDraftsStoreWithMergeState,
} from "./paneDrafts";

/**
 * App-level provider for the pane draft buffer (MVP-18 Wave 2).
 *
 * One store instance is shared across the whole app, keyed by paneId. This is
 * required by the failure → Insert flow: a FailureCallout rendered for a child
 * pane writes the sanitized fragment into the *parent* AI pane's draft, so the
 * draft store must be reachable from any pane, not instantiated per-pane.
 *
 * Mirrors the `paneLinks-context` provider pattern. No event subscription —
 * drafts are pure frontend memory; "send" is flushed to the PTY by the
 * consumer (Wave 2 wires PaneDraftComposer.onSend).
 */
const PaneDraftsContext = createContext<PaneDraftsStoreWithMergeState>();

export const PaneDraftsProvider: ParentComponent = (props) => {
  const store = createPaneDraftsStore();
  return (
    <PaneDraftsContext.Provider value={store}>
      {props.children}
    </PaneDraftsContext.Provider>
  );
};

export function usePaneDrafts(): PaneDraftsStoreWithMergeState {
  const ctx = useContext(PaneDraftsContext);
  if (!ctx) {
    throw new Error("usePaneDrafts must be used within PaneDraftsProvider");
  }
  return ctx;
}
