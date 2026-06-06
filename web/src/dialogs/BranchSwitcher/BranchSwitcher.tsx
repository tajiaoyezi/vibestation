import {
  createEffect,
  createMemo,
  createSignal,
  For,
  on,
  onCleanup,
  onMount,
  Show,
  type Component,
} from "solid-js";
import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { t, normalizeLanguage } from "../../i18n";
import type {
  BranchCheckoutRequest,
  BranchError,
  BranchInfo,
  BranchListRequest,
  BranchListResponse,
  BranchSwitchResult,
  WorkspaceMetadata,
} from "../../bindings";
import {
  buildSwitcherItems,
  type BranchSwitcherGroup,
  type BranchSwitcherItem,
} from "./branchSwitcherLogic";
import {
  loadRecentBranches,
  recordRecentBranch,
  type BranchRecentEntry,
} from "./recentHistory";
import { useSettings } from "../../stores/settings";
import "./branchSwitcher.css";

const BRANCH_CHANGED_EVENT = "git:branch-changed";

interface BranchChangedPayload {
  workspaceId: string;
  branches: BranchInfo[];
  head: string | null;
}

interface BranchSwitcherProps {
  activeWorkspace: () => WorkspaceMetadata | null;
  open: () => boolean;
  onClose: () => void;
}

interface BranchWorkspaceState {
  branches: BranchInfo[];
  headName: string | null;
  loading: boolean;
  loaded: boolean;
  error: string | null;
}

const emptyState: BranchWorkspaceState = {
  branches: [],
  headName: null,
  loading: false,
  loaded: false,
  error: null,
};

const groupLabelKeys: Record<BranchSwitcherGroup, string> = {
  current: "dialogs.branchSwitcher.groups.current",
  recent: "dialogs.branchSwitcher.groups.recent",
  local: "dialogs.branchSwitcher.groups.local",
  remote: "dialogs.branchSwitcher.groups.remote",
};

export const BranchSwitcher: Component<BranchSwitcherProps> = (props) => {
  const { settings } = useSettings();
  const [states, setStates] = createStore<Record<string, BranchWorkspaceState>>(
    {},
  );
  const [query, setQuery] = createSignal("");
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  const [recent, setRecent] = createSignal<BranchRecentEntry[]>([]);
  const [message, setMessage] = createSignal<string | null>(null);
  const [checkingOut, setCheckingOut] = createSignal(false);
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  let inputRef: HTMLInputElement | undefined;
  let unlistenBranchChanged: UnlistenFn | undefined;

  const workspace = () => props.activeWorkspace();
  const workspaceId = () => workspace()?.workspaceId ?? "";
  const hasGitWorkspace = () => Boolean(workspace()?.hasGit);
  const currentState = () => states[workspaceId()] ?? emptyState;
  const recentNames = () => recent().map((entry) => entry.name);

  const items = createMemo(() =>
    buildSwitcherItems(
      currentState().branches,
      currentState().headName,
      query(),
      recentNames(),
    ),
  );

  onMount(() => {
    void listen<BranchChangedPayload>(BRANCH_CHANGED_EVENT, (event) => {
      const payload = event.payload;
      const previousHead = states[payload.workspaceId]?.headName ?? null;
      setStates(payload.workspaceId, (prev = emptyState) => ({
        ...prev,
        branches: payload.branches,
        headName: payload.head,
        loaded: true,
        loading: false,
        error: null,
      }));
      if (
        payload.workspaceId === workspaceId() &&
        payload.head &&
        previousHead &&
        previousHead !== payload.head
      ) {
        setRecent(recordRecentBranch(payload.workspaceId, payload.head));
      }
    }).then((unlisten) => {
      unlistenBranchChanged = unlisten;
    });
  });

  onCleanup(() => {
    unlistenBranchChanged?.();
  });

  createEffect(
    on(
      () => [props.open(), workspaceId()] as const,
      ([isOpen, id]) => {
        if (!isOpen || !id) return;
        setQuery("");
        setMessage(null);
        setSelectedIndex(0);
        setRecent(loadRecentBranches(id));
        if (hasGitWorkspace()) {
          void loadBranches(id);
        }
        queueMicrotask(() => inputRef?.focus());
      },
    ),
  );

  createEffect(
    on(
      () => [query(), workspaceId()] as const,
      () => setSelectedIndex(0),
      { defer: true },
    ),
  );

  createEffect(() => {
    const count = items().length;
    if (count === 0) {
      setSelectedIndex(0);
      return;
    }
    if (selectedIndex() >= count) {
      setSelectedIndex(count - 1);
    }
  });

  const loadBranches = async (targetWorkspaceId: string) => {
    const req: BranchListRequest = { workspaceId: targetWorkspaceId };
    setStates(targetWorkspaceId, (prev = emptyState) => ({
      ...prev,
      loading: true,
      error: null,
    }));

    try {
      const response = await invoke<BranchListResponse>("branch_list", { req });
      setStates(targetWorkspaceId, {
        branches: response.branches,
        headName: response.headName,
        loading: false,
        loaded: true,
        error: null,
      });
    } catch (error) {
      setStates(targetWorkspaceId, (prev = emptyState) => ({
        ...prev,
        loading: false,
        loaded: true,
        error: branchErrorMessage(parseBranchError(error), error),
      }));
    }
  };

  const close = () => {
    if (checkingOut()) return;
    props.onClose();
  };

  const moveSelection = (delta: number) => {
    const count = items().length;
    if (count === 0) return;
    setSelectedIndex((current) => (current + delta + count) % count);
  };

  const checkoutSelected = async () => {
    const item = items()[selectedIndex()];
    if (!item) return;
    await checkoutBranch(item.branch);
  };

  const checkoutBranch = async (branch: BranchInfo) => {
    const id = workspaceId();
    if (!id || checkingOut()) return;
    if (branch.kind === "local" && branch.name === currentState().headName) {
      props.onClose();
      return;
    }

    setCheckingOut(true);
    setMessage(null);
    const req: BranchCheckoutRequest = {
      workspaceId: id,
      name: branch.name,
      force: false,
    };

    try {
      const result = await invoke<BranchSwitchResult>("branch_checkout", {
        req,
      });
      setRecent(recordRecentBranch(id, result.newHead));
      props.onClose();
    } catch (error) {
      setMessage(branchErrorMessage(parseBranchError(error), error));
    } finally {
      setCheckingOut(false);
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    event.stopPropagation();
    switch (event.key) {
      case "Escape":
        event.preventDefault();
        close();
        break;
      case "ArrowDown":
        event.preventDefault();
        moveSelection(1);
        break;
      case "ArrowUp":
        event.preventDefault();
        moveSelection(-1);
        break;
      case "Enter":
        event.preventDefault();
        void checkoutSelected();
        break;
    }
  };

  const renderBranchName = (item: BranchSwitcherItem) => {
    const marked = new Set(item.matchIndices);
    return (
      <For each={Array.from(item.branch.name)}>
        {(char, index) => (
          <Show when={marked.has(index())} fallback={char}>
            <mark>{char}</mark>
          </Show>
        )}
      </For>
    );
  };

  return (
    <Show when={props.open()}>
      <div
        class="vs-branch-switcher-overlay"
        role="dialog"
        aria-modal="true"
        aria-labelledby="vs-branch-switcher-title"
        onMouseDown={(event) => {
          if (event.target === event.currentTarget) {
            close();
          }
        }}
        onKeyDown={handleKeyDown}
      >
        <div
          class="vs-branch-switcher"
          data-no-drag
          onMouseDown={(event) => event.stopPropagation()}
        >
          <h3 id="vs-branch-switcher-title" class="vs-branch-switcher-title">
            {label("dialogs.branchSwitcher.title")}
          </h3>

          <input
            ref={inputRef}
            type="text"
            class="vs-branch-switcher-input"
            value={query()}
            placeholder={label("dialogs.branchSwitcher.typeBranchName")}
            onInput={(event) => setQuery(event.currentTarget.value)}
            spellcheck={false}
            autocomplete="off"
          />

          <Show
            when={hasGitWorkspace()}
            fallback={
              <p class="vs-branch-switcher-empty">
                {label("dialogs.branchSwitcher.notGitWorkspace")}
              </p>
            }
          >
            <Show when={!currentState().error} fallback={errorState()}>
              <Show
                when={!currentState().loading || currentState().loaded}
                fallback={
                  <p class="vs-branch-switcher-empty">
                    {label("dialogs.branchSwitcher.loadingBranches")}
                  </p>
                }
              >
                <Show
                  when={items().length > 0}
                  fallback={
                    <p class="vs-branch-switcher-empty">
                      {label("dialogs.branchSwitcher.noBranchMatched")}
                    </p>
                  }
                >
                  <div class="vs-branch-switcher-list" role="listbox">
                    <For each={items()}>
                      {(item, index) => (
                        <>
                          <Show
                            when={
                              index() === 0 ||
                              items()[index() - 1]?.group !== item.group
                            }
                          >
                            <div class="vs-branch-switcher-group">
                              {label(groupLabelKeys[item.group])}
                            </div>
                          </Show>
                          <button
                            type="button"
                            role="option"
                            aria-selected={index() === selectedIndex()}
                            classList={{
                              "vs-branch-switcher-row": true,
                              selected: index() === selectedIndex(),
                              current: item.group === "current",
                            }}
                            onMouseEnter={() => setSelectedIndex(index())}
                            onClick={() => void checkoutBranch(item.branch)}
                          >
                            <span class={`vs-ref-dot ${item.branch.kind}`} />
                            <span class="vs-branch-switcher-name">
                              {renderBranchName(item)}
                            </span>
                            <Show when={item.group === "current"}>
                              <span class="vs-branch-switcher-badge">
                                {label("dialogs.branchSwitcher.badges.current")}
                              </span>
                            </Show>
                            <Show when={item.group === "recent"}>
                              <span class="vs-branch-switcher-badge">
                                {label("dialogs.branchSwitcher.badges.recent")}
                              </span>
                            </Show>
                            <Show when={item.branch.kind === "remote"}>
                              <span class="vs-branch-switcher-badge">
                                {label("dialogs.branchSwitcher.badges.remote")}
                              </span>
                            </Show>
                          </button>
                        </>
                      )}
                    </For>
                  </div>
                </Show>
              </Show>
            </Show>
          </Show>

          <Show when={message()}>
            <p class="vs-branch-switcher-message" role="alert">
              {message()}
            </p>
          </Show>
        </div>
      </div>
    </Show>
  );

  function errorState() {
    return (
      <div class="vs-branch-switcher-empty">
        <p>{currentState().error}</p>
        <button type="button" onClick={() => void loadBranches(workspaceId())}>
          {label("dialogs.branchSwitcher.retry")}
        </button>
      </div>
    );
  }
};

function parseBranchError(error: unknown): BranchError | null {
  if (isBranchError(error)) {
    return error;
  }

  const raw = error instanceof Error ? error.message : stringifyUnknown(error);
  try {
    const parsed = JSON.parse(raw) as unknown;
    return isBranchError(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function isBranchError(error: unknown): error is BranchError {
  return (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    typeof (error as { kind: unknown }).kind === "string"
  );
}

function branchErrorMessage(
  error: BranchError | null,
  fallback: unknown,
): string {
  if (!error) {
    return stringifyUnknown(fallback);
  }

  switch (error.kind) {
    case "invalidName":
      return `分支名非法：${error.reason}`;
    case "notFound":
      return `分支 ${error.name} 不存在`;
    case "alreadyExists":
      return `分支 ${error.name} 已存在`;
    case "unmerged":
      return `分支 ${error.name} 含 ${error.missingCommits} 个未合并 commit`;
    case "protectedBranch":
      return `受保护分支 ${error.name} 不允许执行此操作`;
    case "detachedHead":
      return "当前处于 detached HEAD";
    case "unbornBranch":
      return "还没有任何 commit · 请先在 Git Status 面板创建首次提交";
    case "dirtyWorkingTree":
      return "工作区存在未提交修改 · 请先提交、清理或从分支树使用 Discard & Switch";
    case "indexLocked":
      return "Git index 被其他进程锁定 · 请稍后重试";
    case "git2Error":
      return `${error.message} (code ${error.code}, class ${error.class})`;
  }
}

function stringifyUnknown(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}
