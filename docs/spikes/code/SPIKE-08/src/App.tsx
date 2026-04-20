import { For, Show, createSignal, onMount } from "solid-js";
import {
  createWorkspace,
  deleteWorkspace,
  listWorkspaces,
  type WorkspaceRecord,
} from "./backend";

function legacyWorkspaceId(workspace: WorkspaceRecord) {
  return Reflect.get(workspace as Record<string, unknown>, "workspace_id") as
    | string
    | undefined;
}

function App() {
  const [name, setName] = createSignal("Calm Studio");
  const [rootPath, setRootPath] = createSignal("/tmp/spike-08/calm-studio");
  const [note, setNote] = createSignal("Golden path sample");
  const [workspaces, setWorkspaces] = createSignal<WorkspaceRecord[]>([]);
  const [pendingDelete, setPendingDelete] = createSignal<WorkspaceRecord | null>(null);
  const [status, setStatus] = createSignal<string>("等待 contract + runtime smoke");
  const [error, setError] = createSignal<string | null>(null);
  const [submitting, setSubmitting] = createSignal(false);

  async function refreshWorkspaces() {
    try {
      const response = await listWorkspaces();
      setWorkspaces(response.items);
      setStatus(`已同步 ${response.total} 个 mock workspace`);
      setError(null);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
    }
  }

  async function handleCreate(event: SubmitEvent) {
    event.preventDefault();
    setSubmitting(true);

    try {
      const created = await createWorkspace({
        name: name().trim(),
        rootPath: rootPath().trim(),
        note: note().trim(),
      });
      setWorkspaces((current) => [created, ...current]);
      setStatus(`已创建 ${created.name} · id=${created.id}`);
      setError(null);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
    } finally {
      setSubmitting(false);
    }
  }

  async function confirmDelete() {
    const workspace = pendingDelete();
    if (!workspace) {
      return;
    }

    try {
      const response = await deleteWorkspace({ workspaceId: workspace.id });
      setWorkspaces((current) =>
        current.filter((item) => item.id !== response.deletedWorkspaceId),
      );
      setStatus(
        `已删除 ${workspace.name} · deleted=${response.deletedWorkspaceId} · remaining=${response.remaining}`,
      );
      setPendingDelete(null);
      setError(null);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
    }
  }

  onMount(() => {
    void refreshWorkspaces();
  });

  return (
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">SPIKE-08</p>
        <h1>IPC contract + runtime 双层防御</h1>
        <p class="lede">
          Rust contract 是唯一事实源；前端直接 import 生成 bindings；runtime 再用 golden
          path 验证 create/list/delete 没被静态检查漏掉。
        </p>
      </section>

      <section class="panel">
        <div class="panel-header">
          <div>
            <h2>Mock Workspace Harness</h2>
            <p class="panel-copy">3 commands · 5 structs · enough to reproduce H2-style drift.</p>
          </div>
          <button
            class="ghost-button"
            data-testid="refresh-button"
            onClick={() => void refreshWorkspaces()}
            type="button"
          >
            Refresh
          </button>
        </div>

        <form class="workspace-form" data-testid="create-form" onSubmit={handleCreate}>
          <label class="field">
            <span>Name</span>
            <input
              data-testid="workspace-name-input"
              onInput={(event) => setName(event.currentTarget.value)}
              value={name()}
            />
          </label>

          <label class="field">
            <span>Root Path</span>
            <input
              data-testid="workspace-root-path-input"
              onInput={(event) => setRootPath(event.currentTarget.value)}
              value={rootPath()}
            />
          </label>

          <label class="field">
            <span>Note</span>
            <textarea
              data-testid="workspace-note-input"
              onInput={(event) => setNote(event.currentTarget.value)}
              rows={3}
              value={note()}
            />
          </label>

          <button
            class="primary-button"
            data-testid="create-workspace-button"
            disabled={submitting()}
            type="submit"
          >
            {submitting() ? "Creating..." : "Create mock workspace"}
          </button>
        </form>
      </section>

      <section class="panel">
        <div class="panel-header">
          <div>
            <h2>Workspace List</h2>
            <p class="panel-copy">Golden path 断言点：列表出现、confirm modal、删除后消失。</p>
          </div>
          <span class="count-pill" data-testid="workspace-count">
            {workspaces().length} items
          </span>
        </div>

        <Show
          fallback={
            <div class="empty-state" data-testid="empty-state">
              <strong>Welcome</strong>
              <p>No mock workspace yet. Create one to start the runtime smoke.</p>
            </div>
          }
          when={workspaces().length > 0}
        >
          <ul class="workspace-list" data-testid="workspace-list">
            <For each={workspaces()}>
              {(workspace) => (
                <li class="workspace-card" data-testid={`workspace-card-${workspace.id}`}>
                  <div>
                    <p class="workspace-title">{workspace.name}</p>
                    <p class="workspace-meta">{workspace.rootPath}</p>
                    <p class="workspace-note">{workspace.note}</p>
                  </div>
                  <button
                    class="danger-button"
                    data-testid={`delete-workspace-button-${workspace.id}`}
                    onClick={() => setPendingDelete(workspace)}
                    type="button"
                  >
                    Delete
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </section>

      <div class="status-row">
        <p class="status-pill" data-testid="status-message">
          {status()}
        </p>
        <Show when={error()}>
          {(message) => (
            <p class="error-pill" data-testid="error-message">
              {message()}
            </p>
          )}
        </Show>
      </div>

      <Show when={pendingDelete()}>
        {(workspace) => (
          <div class="modal-backdrop" data-testid="delete-modal">
            <div class="modal-card" role="dialog" aria-modal="true">
              <h3>Delete {workspace().name}?</h3>
              <p>
                This mirrors the MVP-02 destructive path: modal appears, confirm, list entry
                disappears.
              </p>
              <div class="modal-actions">
                <button
                  class="ghost-button"
                  data-testid="cancel-delete-button"
                  onClick={() => setPendingDelete(null)}
                  type="button"
                >
                  Cancel
                </button>
                <button
                  class="danger-button"
                  data-testid="confirm-delete-button"
                  onClick={() => void confirmDelete()}
                  type="button"
                >
                  Confirm delete
                </button>
              </div>
            </div>
          </div>
        )}
      </Show>
    </main>
  );
}

export default App;
