import { invoke } from "@tauri-apps/api/core";
import {
  createEffect,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import type { TabState } from "../../bindings";

type TabBarProps = {
  tabs: readonly TabState[];
  activeTabId: string | null;
  onClose: (tabId: string) => void;
  onContextMenuTab?: (tabId: string) => void;
  onCreate: () => void;
  onRename: (tabId: string, name: string) => Promise<void>;
  onSelect: (tabId: string) => void;
  pendingRenameTabId?: string | null;
};

export const TabBar: Component<TabBarProps> = (props) => {
  const [editingTabId, setEditingTabId] = createSignal<string | null>(null);
  const [draftName, setDraftName] = createSignal("");
  let renameInput: HTMLInputElement | undefined;

  const startRename = (tab: TabState) => {
    setEditingTabId(tab.tabId);
    setDraftName(tab.name);
  };

  const stopRename = () => {
    setEditingTabId(null);
    setDraftName("");
  };

  const commitRename = async () => {
    const tabId = editingTabId();
    const name = draftName().trim();
    if (!tabId) {
      return;
    }

    if (!name) {
      stopRename();
      return;
    }

    await props.onRename(tabId, name);
    stopRename();
  };

  createEffect(() => {
    if (!editingTabId()) {
      return;
    }

    queueMicrotask(() => {
      renameInput?.focus();
      renameInput?.select();
    });
  });

  createEffect(() => {
    const pendingId = props.pendingRenameTabId;
    if (!pendingId) {
      return;
    }

    const tab = props.tabs.find((t) => t.tabId === pendingId);
    if (tab) {
      props.onSelect(tab.tabId);
      startRename(tab);
    }
  });

  return (
    <div class="vs-terminal-tabbar" role="tablist" aria-label="Terminal tabs">
      <div class="vs-terminal-tabbar-scroll">
        <For each={props.tabs}>
          {(tab) => {
            const editing = () => editingTabId() === tab.tabId;
            const active = () => props.activeTabId === tab.tabId;

            return (
              <div
                class={`vs-terminal-tab ${active() ? "is-active" : ""}`}
                role="presentation"
                onContextMenu={(e) => {
                  e.preventDefault();
                  // 把右键命中的 tab id 提前告诉 Terminal · 让 menu:action listener
                  // 用 contextTabId 而非 currentActiveTabId · 修右键非 active tab
                  // 操作（rename / close 等）误作用到 active tab 的 bug。
                  props.onContextMenuTab?.(tab.tabId);
                  void invoke("menu_show_tab", {
                    x: e.clientX,
                    y: e.clientY,
                  });
                }}
              >
                <Show
                  when={editing()}
                  fallback={
                    <button
                      type="button"
                      class="vs-terminal-tab-trigger"
                      role="tab"
                      aria-selected={active()}
                      aria-controls={`terminal-pane-${tab.tabId}`}
                      onClick={() => props.onSelect(tab.tabId)}
                    >
                      <span
                        class="vs-terminal-tab-label"
                        onDblClick={(event) => {
                          event.preventDefault();
                          event.stopPropagation();
                          startRename(tab);
                        }}
                      >
                        {tab.name}
                      </span>
                    </button>
                  }
                >
                  <input
                    ref={renameInput}
                    type="text"
                    class="vs-terminal-tab-input"
                    value={draftName()}
                    onInput={(event) => setDraftName(event.currentTarget.value)}
                    onBlur={() => void commitRename()}
                    onKeyDown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void commitRename();
                      }

                      if (event.key === "Escape") {
                        event.preventDefault();
                        stopRename();
                      }
                    }}
                  />
                </Show>

                <button
                  type="button"
                  class="vs-terminal-tab-close"
                  aria-label={`关闭 ${tab.name}`}
                  onClick={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    props.onClose(tab.tabId);
                  }}
                >
                  ×
                </button>
              </div>
            );
          }}
        </For>
      </div>

      <button
        type="button"
        class="vs-terminal-new-tab"
        onClick={props.onCreate}
        aria-label="新建 Terminal Tab"
      >
        +
      </button>
    </div>
  );
};
