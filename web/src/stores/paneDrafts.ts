import { createStore } from "solid-js/store";

export type DraftMergeResult = {
  applied: boolean;
  needsPreview: boolean;
  previewText: string;
};

export interface PaneDraftsStore {
  getDraft: (paneId: string) => string;
  hasDraft: (paneId: string) => boolean;
  setDraft: (paneId: string, text: string) => void;
  clearDraft: (paneId: string) => void;
  insertFragment: (paneId: string, fragment: string) => DraftMergeResult;
  confirmMerge: (paneId: string, mergedText: string) => void;
}

/**
 * Internal extension with reactive pending-merge state.
 * Consumed only by PaneDraftComposer within the package boundary.
 * Not part of Seam Contract 2 — Wave 2 types against PaneDraftsStore.
 */
export interface PaneDraftsStoreWithMergeState extends PaneDraftsStore {
  getPendingMerge: (paneId: string) => DraftMergeResult | null;
  clearPendingMerge: (paneId: string) => void;
}

export function createPaneDraftsStore(): PaneDraftsStoreWithMergeState {
  const [drafts, setDrafts] = createStore<Record<string, string>>({});
  const [pending, setPending] = createStore<
    Record<string, DraftMergeResult | null>
  >({});

  const getDraft = (paneId: string): string => drafts[paneId] ?? "";

  const hasDraft = (paneId: string): boolean => (drafts[paneId] ?? "") !== "";

  const setDraft = (paneId: string, text: string): void => {
    setDrafts(paneId, text);
  };

  const clearDraft = (paneId: string): void => {
    setDrafts(paneId, "");
    setPending(paneId, null);
  };

  const insertFragment = (
    paneId: string,
    fragment: string,
  ): DraftMergeResult => {
    const current = getDraft(paneId);

    if (current === "") {
      setDrafts(paneId, fragment);
      const result: DraftMergeResult = {
        applied: true,
        needsPreview: false,
        previewText: fragment,
      };
      setPending(paneId, null);
      return result;
    }

    const merged = current + "\n\n" + fragment;
    const result: DraftMergeResult = {
      applied: false,
      needsPreview: true,
      previewText: merged,
    };
    setPending(paneId, result);
    return result;
  };

  const confirmMerge = (paneId: string, mergedText: string): void => {
    setDrafts(paneId, mergedText);
    setPending(paneId, null);
  };

  const getPendingMerge = (paneId: string): DraftMergeResult | null =>
    pending[paneId] ?? null;

  const clearPendingMerge = (paneId: string): void => {
    setPending(paneId, null);
  };

  return {
    getDraft,
    hasDraft,
    setDraft,
    clearDraft,
    insertFragment,
    confirmMerge,
    getPendingMerge,
    clearPendingMerge,
  };
}
