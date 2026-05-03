import { createSignal, For, Show, type Component } from "solid-js";
import type { RemoteInfo } from "../../bindings";
import "./remoteSelector.css";

type RemoteOperation = "push" | "pull" | "fetch";

interface RemoteSelectorProps {
  operation: RemoteOperation;
  branch: string;
  remotes: RemoteInfo[];
  initialRemote?: string;
  initialPrune?: boolean;
  onConfirm: (remote: string, prune: boolean) => void;
  onCancel: () => void;
}

export const RemoteSelector: Component<RemoteSelectorProps> = (props) => {
  const [remote, setRemote] = createSignal(
    props.initialRemote || props.remotes[0]?.name || "origin",
  );
  const [prune, setPrune] = createSignal(Boolean(props.initialPrune));

  const title = () => {
    switch (props.operation) {
      case "push":
        return "选择 push remote";
      case "pull":
        return "选择 pull remote";
      case "fetch":
        return "Fetch remote";
    }
  };

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-remote-selector-title"
    >
      <div class="vs-dialog vs-remote-selector-dialog">
        <h3 id="vs-remote-selector-title" class="vs-dialog-title">
          {title()}
        </h3>
        <div class="vs-dialog-body">
          <p class="vs-remote-selector-target">
            {props.operation === "fetch" ? "remote refs" : props.branch}
          </p>
          <div class="vs-remote-selector-list">
            <For each={props.remotes}>
              {(item) => (
                <button
                  type="button"
                  classList={{
                    "vs-remote-selector-row": true,
                    "is-selected": remote() === item.name,
                  }}
                  onClick={() => setRemote(item.name)}
                >
                  <span>{item.name}</span>
                  <Show when={item.url}>
                    <small>{item.url}</small>
                  </Show>
                </button>
              )}
            </For>
          </div>

          <Show when={props.operation === "fetch"}>
            <label class="vs-dialog-checkbox vs-remote-selector-prune">
              <input
                type="checkbox"
                checked={prune()}
                onChange={(event) => setPrune(event.currentTarget.checked)}
              />
              <span>Prune deleted refs</span>
            </label>
          </Show>
        </div>
        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={props.onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            class="vs-dialog-btn-primary"
            disabled={!remote()}
            onClick={() => props.onConfirm(remote(), prune())}
          >
            Continue
          </button>
        </div>
      </div>
    </div>
  );
};
