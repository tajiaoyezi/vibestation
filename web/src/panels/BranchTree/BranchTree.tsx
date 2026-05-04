import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type Component,
} from "solid-js";
import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BranchCheckoutRequest,
  BranchCreateRequest,
  BranchDeleteRequest,
  BranchError,
  BranchInfo,
  BranchListRequest,
  BranchListResponse,
  BranchSwitchResult,
  GitStatusResponse,
  WorkspaceMetadata,
} from "../../bindings";
import { CreateBranchDialog } from "../../dialogs/CreateBranchDialog/CreateBranchDialog";
import {
  DirtyTreeDialog,
  type DirtyFiles,
} from "../../dialogs/DirtyTreeDialog/DirtyTreeDialog";
import { ForceDeleteDialog } from "../../dialogs/ForceDeleteDialog/ForceDeleteDialog";
import { BranchTreeRow } from "./BranchTreeRow";
import "./branchTree.css";

const BRANCH_CHANGED_EVENT = "git:branch-changed";
const PROTECTED_BRANCHES = new Set(["main", "master", "trunk"]);

interface BranchChangedPayload {
  workspaceId: string;
  branches: BranchInfo[];
  head: string | null;
}

interface BranchWorkspaceState {
  branches: BranchInfo[];
  headName: string | null;
  detached: boolean;
  loading: boolean;
  loaded: boolean;
  error: string | null;
}

interface ToastState {
  message: string;
  kind: "success" | "error" | "warning";
  actionLabel?: string;
  onAction?: () => void;
}

interface PendingDirtyCheckout {
  branch: BranchInfo;
  dirty: DirtyFiles;
}

interface PendingForceDelete {
  branch: BranchInfo;
  missingCommits: number;
}

interface BranchTreeProps {
  activeWorkspace: () => WorkspaceMetadata | null;
}

const emptyState: BranchWorkspaceState = {
  branches: [],
  headName: null,
  detached: false,
  loading: false,
  loaded: false,
  error: null,
};

export const BranchTree: Component<BranchTreeProps> = (props) => {
  const [states, setStates] = createStore<Record<string, BranchWorkspaceState>>(
    {},
  );
  const [createOpen, setCreateOpen] = createSignal(false);
  const [createFromRef, setCreateFromRef] = createSignal<string | null>(null);
  const [dirtyCheckout, setDirtyCheckout] =
    createSignal<PendingDirtyCheckout | null>(null);
  const [forceDelete, setForceDelete] = createSignal<PendingForceDelete | null>(
    null,
  );
  const [forceConfirmation, setForceConfirmation] = createSignal("");
  const [forceDeleting, setForceDeleting] = createSignal(false);
  const [toast, setToast] = createSignal<ToastState | null>(null);

  // MVP-13 polish (2026-05-04 · session 23 testing)：BRANCHES section 折叠 + 上方拉伸 ·
  // 仿 IntelliJ / VSCode sidebar section pattern · 默认展开 · 高度 220px · 拖动 [80, 600] clamp · 双击 reset 220。
  const [collapsed, setCollapsed] = createSignal(false);
  const [height, setHeight] = createSignal(220);
  // 拖拽期间 panel CSS height transition 必须临时 disable · 否则 220ms transition 让
  // height 跟手延迟（mouse 已到 250 · CSS 还在动到 230）· 视觉错位。class is-resizing
  // 在 transition: none · 释放后恢复（user 折叠/展开时仍走 transition）。
  const [isResizing, setIsResizing] = createSignal(false);
  // Collapsed 时 panel 视觉高度 · 含 border-top 1 + sub-head padding 14 + content 14 + panel padding-bottom var(--space-2) ≈ 38。
  const COLLAPSED_PANEL_HEIGHT = 38;

  const startResize = (event: MouseEvent) => {
    event.preventDefault();
    setIsResizing(true);
    const startY = event.clientY;
    const startHeight = height();
    const onMove = (ev: MouseEvent) => {
      // 上拉 ev.clientY < startY → delta > 0 → height 增大 · row-resize 直觉一致。
      const delta = startY - ev.clientY;
      const next = Math.max(80, Math.min(600, startHeight + delta));
      setHeight(next);
    };
    const onUp = () => {
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
      setIsResizing(false);
    };
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  };

  let unlistenBranchChanged: UnlistenFn | undefined;
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  const workspace = () => props.activeWorkspace();
  const workspaceId = () => workspace()?.workspaceId ?? "";
  const hasGit = () => workspace()?.hasGit ?? false;
  const currentState = () => states[workspaceId()] ?? emptyState;

  const branchRows = createMemo(() =>
    currentState().branches.filter(
      (branch) => branch.kind !== "remote" || !branch.name.endsWith("/HEAD"),
    ),
  );

  // Unborn HEAD：repo 已 `git init` 但还没有任何 commit · `refs/heads/main` ref 不存在 ·
  // 此时不能 create branch（git 约束 · 因 branch 是指向 commit 的 ref）。判定条件四合一：
  // (1) loaded · 非 loading 状态 · (2) 无 error · (3) 非 detached · (4) headName 缺失 + 无 local branch。
  // 触发 UX：BRANCHES list 显示 "还没有 commit · 请先在 Git Status 面板创建首次提交" + "+" disabled。
  const isUnborn = createMemo(() => {
    const state = currentState();
    return (
      state.loaded &&
      state.error === null &&
      !state.detached &&
      state.headName === null &&
      branchRows().length === 0
    );
  });

  onMount(() => {
    void listen<BranchChangedPayload>(BRANCH_CHANGED_EVENT, (event) => {
      const payload = event.payload;
      setStates(payload.workspaceId, (prev = emptyState) => ({
        ...prev,
        branches: payload.branches,
        headName: payload.head,
        detached: payload.head === null,
        loaded: true,
        loading: false,
        error: null,
      }));
    }).then((unlisten) => {
      unlistenBranchChanged = unlisten;
    });
  });

  onCleanup(() => {
    unlistenBranchChanged?.();
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
  });

  createEffect(() => {
    const id = workspaceId();
    if (!id || !hasGit()) {
      return;
    }
    void loadBranches(id);
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
      if (workspaceId() !== targetWorkspaceId) {
        setStates(targetWorkspaceId, {
          branches: response.branches,
          headName: response.headName,
          detached: response.detached,
          loading: false,
          loaded: true,
          error: null,
        });
        return;
      }
      setStates(targetWorkspaceId, {
        branches: response.branches,
        headName: response.headName,
        detached: response.detached,
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

  const showToast = (
    nextToast: ToastState,
    timeoutMs = nextToast.actionLabel ? 5000 : 3200,
  ) => {
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
    setToast(nextToast);
    toastTimer = setTimeout(() => setToast(null), timeoutMs);
  };

  const closeToast = () => {
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
    setToast(null);
  };

  const openCreateDialog = (fromRef: string | null = null) => {
    setCreateFromRef(fromRef);
    setCreateOpen(true);
  };

  const handleCreate = async (
    payload: Omit<BranchCreateRequest, "workspaceId">,
  ): Promise<boolean> => {
    const id = workspaceId();
    if (!id) {
      return false;
    }

    const req: BranchCreateRequest = { workspaceId: id, ...payload };

    try {
      await invoke("branch_create", { req });
      showToast({
        kind: "success",
        message: `分支 ${payload.name} 已创建`,
      });
      return true;
    } catch (error) {
      const parsed = parseBranchError(error);
      if (payload.checkout && parsed?.kind === "dirtyWorkingTree") {
        showToast({
          kind: "warning",
          message: `分支已创建 · 但切换失败：${branchErrorMessage(parsed, error)}`,
        });
        return true;
      }

      showToast({
        kind: "error",
        message: branchErrorMessage(parsed, error),
      });
      return false;
    }
  };

  const queryDirtyFiles = async (
    targetWorkspaceId: string,
  ): Promise<DirtyFiles> => {
    const status = await invoke<GitStatusResponse>("git_status_query", {
      req: { workspaceId: targetWorkspaceId },
    });
    return {
      modified: status.unstaged.map((file) => file.path),
      staged: status.staged.map((file) => file.path),
      untracked: status.untracked.map((file) => file.path),
    };
  };

  const isDirty = (dirty: DirtyFiles) =>
    dirty.modified.length + dirty.staged.length + dirty.untracked.length > 0;

  const checkoutBranch = async (
    branch: BranchInfo,
    options: { force?: boolean; skipDirtyCheck?: boolean } = {},
  ) => {
    const id = workspaceId();
    if (!id || branch.kind === "tag") {
      return;
    }

    if (
      branch.kind === "local" &&
      branch.name === currentState().headName &&
      !options.force
    ) {
      return;
    }

    if (!options.force && !options.skipDirtyCheck) {
      try {
        const dirty = await queryDirtyFiles(id);
        if (isDirty(dirty)) {
          setDirtyCheckout({ branch, dirty });
          return;
        }
      } catch (error) {
        showToast({
          kind: "error",
          message: `读取 Git status 失败：${stringifyUnknown(error)}`,
        });
        return;
      }
    }

    const req: BranchCheckoutRequest = {
      workspaceId: id,
      name: branch.name,
      force: Boolean(options.force),
    };

    try {
      const result = await invoke<BranchSwitchResult>("branch_checkout", {
        req,
      });
      if (branch.kind === "remote" && result.newHead !== branch.name) {
        showToast({
          kind: "success",
          message: `已基于 ${branch.name} 创建本地分支 ${result.newHead} 并切换`,
        });
      } else if (result.dirtyFilesDropped > 0) {
        showToast({
          kind: "warning",
          message: `已切换到 ${result.newHead} · 丢弃 ${result.dirtyFilesDropped} 个文件修改`,
        });
      } else {
        showToast({
          kind: "success",
          message: `已切换到 ${result.newHead}`,
        });
      }
      setDirtyCheckout(null);
    } catch (error) {
      const parsed = parseBranchError(error);
      if (parsed?.kind === "dirtyWorkingTree") {
        setDirtyCheckout({
          branch,
          dirty: {
            modified: parsed.modified,
            staged: parsed.staged,
            untracked: parsed.untracked,
          },
        });
        return;
      }
      showToast({
        kind: "error",
        message: branchErrorMessage(parsed, error),
      });
    }
  };

  const deleteBranch = async (branch: BranchInfo, force = false) => {
    const id = workspaceId();
    if (!id) {
      return;
    }

    const req: BranchDeleteRequest = {
      workspaceId: id,
      name: branch.name,
      force,
    };

    try {
      await invoke("branch_delete", { req });
      setForceDelete(null);
      setForceConfirmation("");
      showDeletedToast(branch, force);
    } catch (error) {
      const parsed = parseBranchError(error);
      if (!force && parsed?.kind === "unmerged") {
        setForceDelete({
          branch,
          missingCommits: parsed.missingCommits,
        });
        setForceConfirmation("");
        showToast({
          kind: "error",
          message: `分支 ${branch.name} 含未合并 commit · 强制删除？`,
          actionLabel: "Force Delete",
          onAction: () => {
            closeToast();
            setForceDelete({
              branch,
              missingCommits: parsed.missingCommits,
            });
          },
        });
        return;
      }

      showToast({
        kind: "error",
        message: branchErrorMessage(parsed, error),
      });
    }
  };

  const showDeletedToast = (branch: BranchInfo, forced: boolean) => {
    const headCommit = branch.headCommit;
    showToast({
      kind: forced ? "warning" : "success",
      message: forced
        ? `已强制删除 ${branch.name}`
        : `已删除分支 ${branch.name}`,
      actionLabel: headCommit ? "Undo" : undefined,
      onAction: headCommit
        ? () => {
            closeToast();
            void restoreDeletedBranch(branch.name, headCommit);
          }
        : undefined,
    });
  };

  const restoreDeletedBranch = async (name: string, headCommit: string) => {
    const id = workspaceId();
    if (!id) {
      return;
    }

    const req: BranchCreateRequest = {
      workspaceId: id,
      name,
      fromRef: headCommit,
      checkout: false,
    };

    try {
      await invoke("branch_create", { req });
      showToast({
        kind: "success",
        message: `已恢复分支 ${name}`,
      });
    } catch (error) {
      showToast({
        kind: "error",
        message: `恢复分支失败：${branchErrorMessage(parseBranchError(error), error)}`,
      });
    }
  };

  const confirmForceDelete = async () => {
    const pending = forceDelete();
    if (!pending) {
      return;
    }
    setForceDeleting(true);
    await deleteBranch(pending.branch, true);
    setForceDeleting(false);
  };

  const deleteDisabledReason = (branch: BranchInfo): string | undefined => {
    if (branch.kind !== "local") {
      return "仅支持删除本地分支";
    }
    if (PROTECTED_BRANCHES.has(branch.name)) {
      return "受保护分支 · 不允许删除";
    }
    if (branch.name === currentState().headName) {
      return "无法删除当前分支 · 请先切换";
    }
    return undefined;
  };

  const checkoutDisabledReason = (branch: BranchInfo): string | undefined => {
    if (branch.kind === "tag") {
      return "tag 不支持在 Phase B 分支树中切换";
    }
    if (branch.kind === "local" && branch.name === currentState().headName) {
      return "当前分支";
    }
    return undefined;
  };

  return (
    <Show when={hasGit()}>
      <section
        class="vs-branch-tree-panel"
        classList={{
          "is-collapsed": collapsed(),
          "is-resizing": isResizing(),
        }}
        style={{
          height: `${collapsed() ? COLLAPSED_PANEL_HEIGHT : height()}px`,
        }}
        aria-label="Branches"
      >
        <Show when={!collapsed()}>
          <div
            class="vs-branch-tree-vresize"
            role="separator"
            aria-orientation="horizontal"
            aria-label="Resize branches section"
            title="拖动调整高度 · 双击复位 220px"
            onMouseDown={startResize}
            onDblClick={() => setHeight(220)}
          />
        </Show>
        <header class="vs-sub-head">
          <button
            type="button"
            class="vs-sub-head-toggle"
            aria-expanded={!collapsed()}
            title={collapsed() ? "Expand" : "Collapse"}
            onClick={() => setCollapsed(!collapsed())}
          >
            <svg
              width="8"
              height="8"
              viewBox="0 0 12 12"
              fill="none"
              aria-hidden="true"
            >
              <path
                d="M3 4.5L6 7.5L9 4.5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            <span>Branches</span>
          </button>
          <button
            type="button"
            class="vs-sub-head-add"
            title={
              isUnborn()
                ? "还没有 commit · 请先在 Git Status 面板创建首次提交"
                : "New branch"
            }
            aria-label="New branch"
            disabled={isUnborn()}
            onClick={() => {
              if (collapsed()) {
                setCollapsed(false);
              }
              openCreateDialog();
            }}
          >
            <svg
              width="11"
              height="11"
              viewBox="0 0 12 12"
              fill="none"
              aria-hidden="true"
            >
              <path
                d="M6 2.5V9.5M2.5 6H9.5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </header>

        {/* Always render · collapse/expand 用 CSS opacity + max-height 实现真过渡（不 unmount）。
            collapsed=true 时 .is-collapsed parent 触发 opacity 0 + pointer-events none + overflow hidden 视觉收起 ·
            section.height transition 同时 220ms 收缩 · 避免 <Show> mount/unmount 瞬变。 */}
        <div class="vs-branch-tree-collapsible">
          <Show
            when={!currentState().error}
            fallback={
              <div class="vs-branch-empty">
                <p>Git repo unavailable · 请检查 .git 目录</p>
                <button
                  type="button"
                  class="vs-branch-retry"
                  onClick={() => void loadBranches(workspaceId())}
                >
                  Retry
                </button>
              </div>
            }
          >
            <div class="vs-branch-tree">
              <Show when={isUnborn()}>
                <div class="vs-branch-empty vs-branch-empty-unborn">
                  <p class="vs-branch-empty-title">还没有任何 commit</p>
                  <p class="vs-branch-empty-hint">
                    请打开 Git Status 面板创建首次提交 · 之后才能创建分支
                  </p>
                </div>
              </Show>
              <Show when={currentState().detached}>
                <div class="vs-branch-row is-head">
                  <span class="vs-branch-guide">┬ </span>
                  <span class="vs-branch-name">HEAD</span>
                  <span class="vs-branch-badge">detached</span>
                </div>
              </Show>
              <Show
                when={!currentState().loading || currentState().loaded}
                fallback={<p class="vs-branch-loading">Loading branches…</p>}
              >
                <For each={branchRows()}>
                  {(branch, index) => (
                    <BranchTreeRow
                      branch={branch}
                      guide={
                        index() === branchRows().length - 1 ? "└─ " : "├─ "
                      }
                      active={
                        branch.kind === "local" &&
                        branch.name === currentState().headName
                      }
                      deleteDisabledReason={deleteDisabledReason(branch)}
                      checkoutDisabledReason={checkoutDisabledReason(branch)}
                      onCheckout={() => void checkoutBranch(branch)}
                      onDelete={() => void deleteBranch(branch)}
                      onCreateFrom={() => openCreateDialog(branch.name)}
                    />
                  )}
                </For>
              </Show>
            </div>
          </Show>
        </div>
      </section>

      <Show when={createOpen()}>
        <CreateBranchDialog
          branches={currentState().branches}
          initialFromRef={createFromRef()}
          onCreate={handleCreate}
          onCancel={() => setCreateOpen(false)}
        />
      </Show>

      <Show when={dirtyCheckout()}>
        {(pending) => (
          <DirtyTreeDialog
            branchName={pending().branch.name}
            dirty={pending().dirty}
            onDiscard={() =>
              checkoutBranch(pending().branch, {
                force: true,
                skipDirtyCheck: true,
              })
            }
            onCancel={() => setDirtyCheckout(null)}
          />
        )}
      </Show>

      <Show when={forceDelete()}>
        {(pending) => (
          <ForceDeleteDialog
            branch={pending().branch}
            missingCommits={pending().missingCommits}
            confirmation={forceConfirmation()}
            onConfirmationChange={setForceConfirmation}
            onConfirm={confirmForceDelete}
            onCancel={() => {
              setForceDelete(null);
              setForceConfirmation("");
            }}
            deleting={forceDeleting()}
          />
        )}
      </Show>

      <Show when={toast()}>
        {(currentToast) => (
          <div
            class={`vs-branch-toast is-${currentToast().kind}`}
            role="status"
            onClick={(event) => event.stopPropagation()}
          >
            <span>{currentToast().message}</span>
            <Show when={currentToast().actionLabel && currentToast().onAction}>
              <button type="button" onClick={() => currentToast().onAction?.()}>
                {currentToast().actionLabel}
              </button>
            </Show>
          </div>
        )}
      </Show>
    </Show>
  );
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
      return "工作区存在未提交修改";
    case "indexLocked":
      return "Git index 被其他进程锁定 · 请稍后重试";
    case "git2Error":
      return `${error.message} (code ${error.code}, class ${error.class}) · 请检查仓库权限或 .git 目录`;
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
