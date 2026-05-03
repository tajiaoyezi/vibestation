import {
  type Component,
  createSignal,
  createEffect,
  createMemo,
  onMount,
  onCleanup,
  For,
  Show,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AuthMethod,
  AuthRequest,
  BranchListRequest,
  BranchListResponse,
  ConflictFile,
  FetchProgressEvent,
  FetchRequest,
  FetchResult,
  GitLogEntry,
  GitLogQueryRequest,
  GitLogQueryResponse,
  GitStatusResponse,
  NetworkOpError,
  OperationDoneEvent,
  PullRequest,
  PullResult,
  PullStrategy,
  PushProgressEvent,
  PushRequest,
  PushResult,
  RemoteInfo,
  RemoteListRequest,
  RemoteListResponse,
  CommitDetail,
  WorkspaceMetadata,
} from "../../bindings";
import { queryLog, fetchDetail, clearCache } from "./gitLogApi";
import type { DiffTarget } from "../../components/MainContent";
import {
  GitSyncProgressDialog,
  type GitSyncKind,
  type GitSyncProgressValue,
  type GitSyncStage,
} from "../../dialogs/GitSyncProgress/GitSyncProgressDialog";
import { AuthDialog } from "../../dialogs/AuthDialog/AuthDialog";
import {
  ForcePushDialog,
  type ForcePushCommit,
} from "../../dialogs/ForcePushDialog/ForcePushDialog";
import { PullConflictDialog } from "../../dialogs/PullConflictDialog/PullConflictDialog";
import { RemoteSelector } from "../../dialogs/RemoteSelector/RemoteSelector";
import {
  useRemoteSyncStatus,
  type RemoteSyncHighlightRequest,
} from "../../stores/remote-sync-status";

export interface GitLogPanelProps {
  activeWorkspace: () => WorkspaceMetadata | null;
  onOpenDiff?: (target: DiffTarget) => void;
  onOpenGitStatus?: () => void;
}

type ToastKind = "success" | "error" | "warning" | "info";

interface ToastState {
  message: string;
  kind: ToastKind;
  actionLabel?: string;
  onAction?: () => void;
  timeoutMs?: number;
}

interface ActiveOperation {
  kind: GitSyncKind | null;
  taskId: string | null;
  remote: string;
  branch: string;
  stage: GitSyncStage;
  progress: GitSyncProgressValue;
  abortable: boolean;
  pullStrategy: PullStrategy;
  prune: boolean;
  lastBytes: number;
  lastAt: number;
}

type RemoteOperation = "push" | "pull" | "fetch";

interface PendingRemoteSelection {
  operation: RemoteOperation;
  branch: string;
  remotes: RemoteInfo[];
  initialRemote: string;
}

interface OperationRetry {
  operation: RemoteOperation;
  remote: string;
  branch: string;
  prune?: boolean;
  strategy?: PullStrategy;
  force?: boolean;
  expectedRemoteOid?: string | null;
  authMethod?: AuthMethod | null;
}

interface PendingAuth {
  remoteUrl: string;
  retry: OperationRetry;
  error: string | null;
}

interface PendingForcePush {
  remote: string;
  branch: string;
  localAhead: number;
  remoteAhead: number;
  expectedRemoteOid: string | null;
  commits: ForcePushCommit[];
}

interface PendingConflict {
  remote: string;
  branch: string;
  files: ConflictFile[];
}

interface ActiveLogHighlight extends RemoteSyncHighlightRequest {
  targetSha: string | null;
}

const GIT_PUSH_PROGRESS_EVENT = "git:push-progress";
const GIT_FETCH_PROGRESS_EVENT = "git:fetch-progress";
const GIT_OPERATION_DONE_EVENT = "git:operation-done";
const PROTECTED_BRANCHES = new Set(["main", "master", "trunk"]);

const emptyProgress: GitSyncProgressValue = {
  current: 0,
  total: 0,
  bytesDone: 0,
  bytesTotal: 0,
  bytesPerSec: 0,
};

const emptyOperation: ActiveOperation = {
  kind: null,
  taskId: null,
  remote: "",
  branch: "",
  stage: "fetch",
  progress: emptyProgress,
  abortable: false,
  pullStrategy: "merge",
  prune: false,
  lastBytes: 0,
  lastAt: 0,
};

export function createGitLogStore() {
  const [entries, setEntries] = createSignal<GitLogEntry[]>([]);
  const [hasMore, setHasMore] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [selectedSha, setSelectedSha] = createSignal<string | null>(null);
  const [filterMessage, setFilterMessage] = createSignal("");
  const [filterAuthor, setFilterAuthor] = createSignal("");
  const [offset, setOffset] = createSignal(0);
  const [detail, setDetail] = createSignal<CommitDetail | null>(null);
  const [detailLoading, setDetailLoading] = createSignal(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const load = async (workspaceId: string, resetOffset = true) => {
    setLoading(true);
    setError(null);
    const newOffset = resetOffset ? 0 : offset();
    if (resetOffset) {
      setOffset(0);
      setEntries([]);
    }

    const req: GitLogQueryRequest = {
      workspaceId,
      offset: newOffset,
      limit: 100,
      filterMessage: filterMessage() || null,
      filterAuthor: filterAuthor() || null,
      filterAfter: null,
    };

    try {
      const resp: GitLogQueryResponse = await queryLog(req);
      if (resetOffset) {
        setEntries(resp.entries);
      } else {
        setEntries((prev) => [...prev, ...resp.entries]);
      }
      setHasMore(resp.hasMore);
      setOffset(newOffset + resp.entries.length);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const loadMore = (workspaceId: string) => {
    if (hasMore() && !loading()) {
      load(workspaceId, false);
    }
  };

  const loadDetail = async (workspaceId: string, sha: string) => {
    setDetailLoading(true);
    try {
      const result = await fetchDetail(workspaceId, sha);
      setDetail(result);
      setSelectedSha(sha);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setDetailLoading(false);
    }
  };

  const debouncedLoad = (workspaceId: string) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => load(workspaceId, true), 300);
  };

  const clearFilter = () => {
    setFilterMessage("");
    setFilterAuthor("");
  };

  return {
    entries,
    hasMore,
    loading,
    error,
    selectedSha,
    detail,
    detailLoading,
    filterMessage,
    filterAuthor,
    setFilterMessage: (v: string) => {
      setFilterMessage(v);
    },
    setFilterAuthor: (v: string) => {
      setFilterAuthor(v);
    },
    load,
    loadMore,
    loadDetail,
    debouncedLoad,
    clearCache,
    clearFilter,
    setError,
  };
}

export type GitLogStore = ReturnType<typeof createGitLogStore>;

export const GitLogPanel: Component<GitLogPanelProps> = (props) => {
  const store = createGitLogStore();
  const remoteSync = useRemoteSyncStatus();
  let scrollContainer: HTMLDivElement | undefined;
  let panelRoot: HTMLDivElement | undefined;
  const entryRefs = new Map<string, HTMLButtonElement>();
  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  let closeProgressTimer: ReturnType<typeof setTimeout> | undefined;
  let highlightTimer: ReturnType<typeof setTimeout> | undefined;
  let lastHighlightRequestId = 0;
  let lastScrolledHighlightId = 0;

  // detail 区高度 · 可拖动 · 记忆到组件实例上 · 关掉 detail 再打开仍保留
  const [detailHeight, setDetailHeight] = createSignal(280);
  const [isResizing, setIsResizing] = createSignal(false);
  const [operation, setOperation] =
    createSignal<ActiveOperation>(emptyOperation);
  const [toast, setToast] = createSignal<ToastState | null>(null);
  const [remoteSelection, setRemoteSelection] =
    createSignal<PendingRemoteSelection | null>(null);
  const [pendingAuth, setPendingAuth] = createSignal<PendingAuth | null>(null);
  const [submittingAuth, setSubmittingAuth] = createSignal(false);
  const [forcePush, setForcePush] = createSignal<PendingForcePush | null>(null);
  const [forceConfirmation, setForceConfirmation] = createSignal("");
  const [forceSubmitting, setForceSubmitting] = createSignal(false);
  const [pullConflict, setPullConflict] = createSignal<PendingConflict | null>(
    null,
  );
  const [logHighlight, setLogHighlight] =
    createSignal<ActiveLogHighlight | null>(null);

  const startResize = (e: PointerEvent) => {
    e.preventDefault();
    const panelHeight = panelRoot?.clientHeight ?? 800;
    const minH = 96;
    const maxH = Math.max(minH + 80, panelHeight * 0.85);
    const startY = e.clientY;
    const startHeight = detailHeight();
    setIsResizing(true);

    const onMove = (ev: PointerEvent) => {
      // 鼠标向上拖（clientY 减小）→ detail 增高
      const delta = startY - ev.clientY;
      const next = Math.max(minH, Math.min(maxH, startHeight + delta));
      setDetailHeight(next);
    };
    const onUp = () => {
      setIsResizing(false);
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  };

  const workspaceId = () => {
    const ws = props.activeWorkspace();
    return ws ? ws.workspaceId : "";
  };

  const hasGit = () => {
    const ws = props.activeWorkspace();
    return ws ? ws.hasGit : false;
  };

  const workspacePath = () =>
    props.activeWorkspace()?.repoRoot ?? props.activeWorkspace()?.path ?? "";
  const activeOperation = createMemo(() => operation());

  const showToast = (nextToast: ToastState) => {
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
    setToast(nextToast);
    toastTimer = setTimeout(
      () => setToast(null),
      nextToast.timeoutMs ?? (nextToast.actionLabel ? 30000 : 3600),
    );
  };

  const closeToast = () => {
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
    setToast(null);
  };

  const startOperation = (
    kind: GitSyncKind,
    remote: string,
    branch: string,
    taskId: string,
    options: {
      stage?: GitSyncStage;
      strategy?: PullStrategy;
      prune?: boolean;
    } = {},
  ) => {
    if (closeProgressTimer) {
      clearTimeout(closeProgressTimer);
    }
    setOperation({
      kind,
      taskId,
      remote,
      branch,
      stage: options.stage ?? (kind === "push" ? "writing" : "fetch"),
      progress: { ...emptyProgress },
      abortable: true,
      pullStrategy: options.strategy ?? "merge",
      prune: Boolean(options.prune),
      lastBytes: 0,
      lastAt: performance.now(),
    });
  };

  const finishOperation = (stage: GitSyncStage = "done") => {
    setOperation((prev) => ({ ...prev, stage, abortable: false }));
    closeProgressTimer = setTimeout(() => {
      setOperation(emptyOperation);
    }, 1000);
  };

  const stopOperation = () => {
    if (closeProgressTimer) {
      clearTimeout(closeProgressTimer);
    }
    setOperation(emptyOperation);
  };

  const updateProgress = (
    taskId: string,
    stage: GitSyncStage,
    current: number,
    total: number,
    bytesDone: number,
    bytesTotal: number,
  ) => {
    const currentOperation = operation();
    if (!currentOperation.taskId || currentOperation.taskId !== taskId) {
      return;
    }
    const now = performance.now();
    const elapsed = Math.max(1, now - currentOperation.lastAt) / 1000;
    const deltaBytes = Math.max(0, bytesDone - currentOperation.lastBytes);
    setOperation((prev) => ({
      ...prev,
      stage,
      progress: {
        current,
        total,
        bytesDone,
        bytesTotal,
        bytesPerSec: deltaBytes / elapsed,
      },
      lastBytes: bytesDone,
      lastAt: now,
    }));
  };

  const makeTaskId = (kind: RemoteOperation | "auth") =>
    `${kind}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

  const listRemotes = async (): Promise<RemoteInfo[] | null> => {
    const id = workspaceId();
    if (!id) return null;
    const req: RemoteListRequest = { workspaceId: id };
    try {
      const response = await invoke<RemoteListResponse>("git_remote_list", {
        req,
      });
      if (response.remotes.length === 0) {
        showToast({
          kind: "error",
          message: "未配置 remote · 请用终端 git remote add origin <url>",
        });
        return null;
      }
      return response.remotes;
    } catch (error) {
      showToast({
        kind: "error",
        message: networkErrorMessage(parseNetworkError(error), error),
      });
      return null;
    }
  };

  const loadBranchList = async (): Promise<BranchListResponse | null> => {
    const id = workspaceId();
    if (!id) return null;
    const req: BranchListRequest = { workspaceId: id };
    try {
      return await invoke<BranchListResponse>("branch_list", { req });
    } catch (error) {
      showToast({
        kind: "error",
        message: `读取当前分支失败：${stringifyUnknown(error)}`,
      });
      return null;
    }
  };

  const currentBranchName = async (): Promise<string | null> => {
    const list = await loadBranchList();
    if (!list?.headName) {
      showToast({
        kind: "error",
        message: "当前处于 detached HEAD · 远端同步需要先切到本地分支",
      });
      return null;
    }
    return list.headName;
  };

  const remoteUrl = (remoteName: string, remotes?: RemoteInfo[]) =>
    remotes?.find((item) => item.name === remoteName)?.url ?? remoteName;

  const remoteHeadOid = async (
    remote: string,
    branch: string,
  ): Promise<string | null> => {
    const list = await loadBranchList();
    return (
      list?.branches.find(
        (item) => item.kind === "remote" && item.name === `${remote}/${branch}`,
      )?.headCommit ?? null
    );
  };

  const beginRemoteOperation = async (nextOperation: RemoteOperation) => {
    if (!hasGit() || !workspaceId()) {
      return;
    }
    const branch =
      nextOperation === "fetch"
        ? ((await loadBranchList())?.headName ?? "HEAD")
        : await currentBranchName();
    if (!branch) {
      return;
    }
    const remotes = await listRemotes();
    if (!remotes) {
      return;
    }
    const initialRemote =
      remotes.find((item) => item.name === "origin")?.name ?? remotes[0]?.name;

    if (nextOperation === "fetch" || remotes.length > 1) {
      setRemoteSelection({
        operation: nextOperation,
        branch,
        remotes,
        initialRemote,
      });
      return;
    }

    await runRemoteOperation({
      operation: nextOperation,
      remote: initialRemote,
      branch,
      strategy: "merge",
      prune: false,
    });
  };

  const runRemoteOperation = async (retry: OperationRetry): Promise<void> => {
    switch (retry.operation) {
      case "push":
        await runPush(retry.remote, retry.branch, {
          force: Boolean(retry.force),
          expectedRemoteOid: retry.expectedRemoteOid ?? null,
          authMethod: retry.authMethod ?? null,
        });
        break;
      case "pull":
        await runPull(retry.remote, retry.branch, retry.strategy ?? "merge", {
          authMethod: retry.authMethod ?? null,
        });
        break;
      case "fetch":
        await runFetch(retry.remote, retry.branch, Boolean(retry.prune), {
          authMethod: retry.authMethod ?? null,
        });
        break;
    }
  };

  const runPush = async (
    remote: string,
    branch: string,
    options: {
      force?: boolean;
      expectedRemoteOid?: string | null;
      authMethod?: AuthMethod | null;
    } = {},
  ) => {
    const id = workspaceId();
    if (!id) return;
    if (options.force && PROTECTED_BRANCHES.has(branch)) {
      showToast({
        kind: "error",
        message: "受保护分支 · 不允许 force push · 请改名其他 branch",
      });
      return;
    }
    if (options.force && !options.expectedRemoteOid) {
      showToast({
        kind: "error",
        message: "无法确认远端 lease · 请先 fetch 后重试 force push",
      });
      return;
    }

    const taskId = makeTaskId("push");
    startOperation("push", remote, branch, taskId, { stage: "writing" });
    const req: PushRequest = {
      workspaceId: id,
      remote,
      branch,
      force: Boolean(options.force),
      expectedRemoteOid: options.expectedRemoteOid ?? null,
      authMethod: options.authMethod ?? null,
      taskId,
    };

    try {
      const result = await invoke<PushResult>("git_push", { req });
      finishOperation("done");
      showToast({
        kind: options.force ? "warning" : "success",
        message: `已推送 ${result.pushedCommits} 个 commit 到 ${remote}/${branch}`,
      });
      store.clearCache();
      await store.load(id);
    } catch (error) {
      stopOperation();
      await handleNetworkError(error, {
        operation: "push",
        remote,
        branch,
        force: Boolean(options.force),
        expectedRemoteOid: options.expectedRemoteOid ?? null,
        authMethod: options.authMethod ?? null,
      });
    }
  };

  const runPull = async (
    remote: string,
    branch: string,
    strategy: PullStrategy,
    options: { authMethod?: AuthMethod | null } = {},
  ) => {
    const id = workspaceId();
    if (!id) return;

    let statusSnapshot: GitStatusResponse;
    try {
      statusSnapshot = await invoke<GitStatusResponse>("git_status_query", {
        req: { workspaceId: id },
      });
    } catch (error) {
      showToast({
        kind: "error",
        message: `读取 Git status 失败：${stringifyUnknown(error)}`,
      });
      return;
    }

    if (isDirty(statusSnapshot)) {
      showToast({
        kind: "warning",
        message: "工作区有未提交修改 · 请先 commit / stash / discard",
        actionLabel: "Git Status",
        onAction: () => {
          closeToast();
          props.onOpenGitStatus?.();
        },
      });
      props.onOpenGitStatus?.();
      return;
    }

    const taskId = makeTaskId("pull");
    startOperation("pull", remote, branch, taskId, {
      stage: "fetch",
      strategy,
    });
    const req: PullRequest = {
      workspaceId: id,
      remote,
      branch,
      strategy,
      frontendStatusSnapshot: statusSnapshot,
      frontendStatusTakenAt: null,
      authMethod: options.authMethod ?? null,
      taskId,
    };

    try {
      const result = await invoke<PullResult>("git_pull", { req });
      finishOperation(result.stage === "rebase" ? "rebase" : "merge");
      showToast({
        kind: "success",
        message: pullSuccessMessage(remote, branch, result),
      });
      store.clearCache();
      await store.load(id);
    } catch (error) {
      stopOperation();
      await handleNetworkError(error, {
        operation: "pull",
        remote,
        branch,
        strategy,
        authMethod: options.authMethod ?? null,
      });
    }
  };

  const runFetch = async (
    remote: string,
    branch: string,
    prune: boolean,
    options: { authMethod?: AuthMethod | null } = {},
  ) => {
    const id = workspaceId();
    if (!id) return;
    const taskId = makeTaskId("fetch");
    startOperation("fetch", remote, branch, taskId, {
      stage: "fetch",
      prune,
    });
    const req: FetchRequest = {
      workspaceId: id,
      remote,
      prune,
      authMethod: options.authMethod ?? null,
      taskId,
    };

    try {
      const result = await invoke<FetchResult>("git_fetch", { req });
      finishOperation("done");
      const pruned =
        result.prunedRefs.length > 0
          ? ` · 已删除 ${result.prunedRefs.length} 个远端已不存在的 ref`
          : "";
      showToast({
        kind: "success",
        message: `已 fetch · 远端 ${result.fetchedRefs.length} refs${pruned}`,
      });
    } catch (error) {
      stopOperation();
      await handleNetworkError(error, {
        operation: "fetch",
        remote,
        branch,
        prune,
        authMethod: options.authMethod ?? null,
      });
    }
  };

  const handleNetworkError = async (
    error: unknown,
    retry: OperationRetry,
  ): Promise<void> => {
    const parsed = parseNetworkError(error);
    if (!parsed) {
      showToast({ kind: "error", message: stringifyUnknown(error) });
      return;
    }

    switch (parsed.kind) {
      case "authFailed":
        setPendingAuth({
          remoteUrl: remoteUrl(retry.remote),
          retry,
          error: parsed.detail,
        });
        break;
      case "nonFastForward": {
        const expectedRemoteOid = await remoteHeadOid(
          retry.remote,
          retry.branch,
        );
        const commits = expectedRemoteOid
          ? [
              {
                sha: expectedRemoteOid,
                message: "remote tip at confirmation time",
              },
            ]
          : [];
        showToast({
          kind: "error",
          message: `${retry.remote}/${retry.branch} 已有更新 · 请先 pull 或 force push`,
          actionLabel: "Force Push",
          onAction: () => {
            closeToast();
            setForcePush({
              remote: retry.remote,
              branch: retry.branch,
              localAhead: parsed.localAhead,
              remoteAhead: parsed.remoteAhead,
              expectedRemoteOid,
              commits,
            });
          },
        });
        break;
      }
      case "mergeConflict":
        setPullConflict({
          remote: retry.remote,
          branch: retry.branch,
          files: parsed.files,
        });
        break;
      case "dirtyWorkingTree":
        showToast({
          kind: "warning",
          message: "工作区有未提交修改 · 请先 commit / stash / discard",
          actionLabel: "Git Status",
          onAction: () => {
            closeToast();
            props.onOpenGitStatus?.();
          },
        });
        props.onOpenGitStatus?.();
        break;
      case "remoteNotFound":
        showToast({
          kind: "error",
          message: `未配置 remote ${parsed.remote} · 请用终端 git remote add ${parsed.remote} <url>`,
        });
        break;
      case "networkUnreachable":
        showToast({
          kind: "error",
          message: `网络不通 · 请检查代理 / DNS：${parsed.detail}`,
          actionLabel: "Retry",
          onAction: () => {
            closeToast();
            void runRemoteOperation(retry);
          },
        });
        break;
      case "sslError":
        showToast({
          kind: "error",
          message: `SSL 证书无效 · 请检查 url 或在终端临时配置 http.sslVerify：${parsed.detail}`,
        });
        break;
      case "staleLease":
        showToast({
          kind: "error",
          message: "远端在确认后发生变化 · 请 fetch 后重新确认 force push",
          actionLabel: "Fetch",
          onAction: () => {
            closeToast();
            void runFetch(retry.remote, retry.branch, false);
          },
        });
        break;
      case "aborted":
        showToast({ kind: "warning", message: `已取消：${parsed.reason}` });
        break;
      case "rejectedByRemote":
        showToast({
          kind: "error",
          message: `远端拒绝操作：${parsed.detail}`,
        });
        break;
      case "git2Error":
        showToast({
          kind: "error",
          message: `${parsed.message} (code ${parsed.code}, class ${parsed.class})`,
        });
        break;
    }
  };

  const submitAuth = async (method: AuthMethod) => {
    const pending = pendingAuth();
    const id = workspaceId();
    if (!pending || !id) return;
    setSubmittingAuth(true);
    const taskId = makeTaskId("auth");
    const req: AuthRequest = {
      workspaceId: id,
      authChallengeId: taskId,
      taskId,
      remoteUrl: pending.remoteUrl,
      allowedMethods: [],
      method,
      expiresAt: Math.floor(Date.now() / 1000) + 300,
    };
    try {
      await invoke("git_auth_provide", { req });
      setPendingAuth(null);
      await runRemoteOperation({ ...pending.retry, authMethod: method });
    } catch (error) {
      setPendingAuth({
        ...pending,
        error: networkErrorMessage(parseNetworkError(error), error),
      });
    } finally {
      setSubmittingAuth(false);
    }
  };

  const confirmForcePush = async () => {
    const pending = forcePush();
    if (!pending) return;
    setForceSubmitting(true);
    setForcePush(null);
    setForceConfirmation("");
    await runPush(pending.remote, pending.branch, {
      force: true,
      expectedRemoteOid: pending.expectedRemoteOid,
    });
    setForceSubmitting(false);
  };

  const cancelOperation = async () => {
    const current = operation();
    const id = workspaceId();
    stopOperation();
    if (id) {
      try {
        await invoke("git_merge_abort", { workspaceId: id });
      } catch {
        // best-effort cancel path; backend may have no merge/rebase in progress.
      }
    }
    showToast({
      kind: "warning",
      message: current.kind === "push" ? "已取消推送" : "已取消远端同步",
    });
  };

  onMount(() => {
    if (hasGit() && workspaceId()) {
      store.load(workspaceId());
    }

    const unlisteners: UnlistenFn[] = [];

    void listen<PushProgressEvent>(GIT_PUSH_PROGRESS_EVENT, (event) => {
      if (event.payload.workspaceId !== workspaceId()) return;
      updateProgress(
        event.payload.taskId,
        event.payload.stage === "writing" ? "writing" : "compressing",
        event.payload.objectsDone,
        event.payload.objectsTotal,
        event.payload.bytesDone,
        event.payload.bytesTotal,
      );
    }).then((unlisten) => unlisteners.push(unlisten));

    void listen<FetchProgressEvent>(GIT_FETCH_PROGRESS_EVENT, (event) => {
      if (event.payload.workspaceId !== workspaceId()) return;
      updateProgress(
        event.payload.taskId,
        "fetching",
        event.payload.receivedObjects,
        event.payload.totalObjects,
        event.payload.receivedBytes,
        event.payload.receivedBytes,
      );
    }).then((unlisten) => unlisteners.push(unlisten));

    void listen<OperationDoneEvent>(GIT_OPERATION_DONE_EVENT, (event) => {
      if (event.payload.workspaceId !== workspaceId()) return;
      if (event.payload.taskId !== operation().taskId) return;
      if (event.payload.outcome === "success") {
        setOperation((prev) => ({ ...prev, stage: "done", abortable: false }));
      }
    }).then((unlisten) => unlisteners.push(unlisten));

    onCleanup(() => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    });
  });

  createEffect(() => {
    const wid = workspaceId();
    if (wid && hasGit()) {
      store.load(wid);
    }
  });

  onCleanup(() => {
    store.clearCache();
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
    if (closeProgressTimer) {
      clearTimeout(closeProgressTimer);
    }
    if (highlightTimer) {
      clearTimeout(highlightTimer);
    }
  });

  createEffect(() => {
    const wid = workspaceId();
    if (!wid || !hasGit()) {
      return;
    }

    let disposed = false;
    let unlisten: UnlistenFn | undefined;

    void listen<{ workspaceId: string }>("git:branch-changed", (event) => {
      if (event.payload.workspaceId !== wid) {
        return;
      }
      store.clearCache();
      void store.load(wid);
    }).then((stop) => {
      if (disposed) {
        stop();
        return;
      }
      unlisten = stop;
    });

    onCleanup(() => {
      disposed = true;
      unlisten?.();
    });
  });

  createEffect(() => {
    const request = remoteSync.highlightRequest();
    const wid = workspaceId();
    if (
      !request ||
      request.id === lastHighlightRequestId ||
      request.workspaceId !== wid ||
      !hasGit()
    ) {
      return;
    }

    lastHighlightRequestId = request.id;
    setLogHighlight({ ...request, targetSha: null });
    store.clearCache();
    void store.load(wid, true);
  });

  createEffect(() => {
    const highlight = logHighlight();
    const entries = store.entries();
    if (
      !highlight ||
      entries.length === 0 ||
      lastScrolledHighlightId === highlight.id
    ) {
      return;
    }

    const targetSha = resolveHighlightTarget(highlight, entries);
    if (!targetSha) {
      return;
    }

    lastScrolledHighlightId = highlight.id;
    setLogHighlight({ ...highlight, targetSha });
    requestAnimationFrame(() => {
      entryRefs.get(targetSha)?.scrollIntoView({
        block: "center",
        behavior: "smooth",
      });
    });

    if (highlightTimer) {
      clearTimeout(highlightTimer);
    }
    highlightTimer = setTimeout(() => {
      setLogHighlight((current) =>
        current?.id === highlight.id ? null : current,
      );
    }, 4500);
  });

  const handleScroll = () => {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    if (scrollHeight - scrollTop - clientHeight < 50 && store.hasMore()) {
      store.loadMore(workspaceId());
    }
  };

  const handleFilterKeydown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && hasGit() && workspaceId()) {
      store.load(workspaceId());
    }
  };

  const formatTime = (timestamp: number): string => {
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  return (
    <Show
      when={hasGit()}
      fallback={
        <div class="vs-git-log-empty">
          <p class="vs-placeholder-text">No git repository found</p>
          <p class="vs-placeholder-text">
            Open a directory containing a .git folder
          </p>
        </div>
      }
    >
      <div class="vs-git-log" ref={panelRoot}>
        <div class="vs-panel-head vs-git-sync-panel-head">
          <span class="vs-panel-title">Git Log</span>
          <div class="vs-panel-actions vs-git-sync-actions">
            <span class="vs-kbd-tip">⌘2</span>
            <button
              type="button"
              class="vs-git-sync-icon-btn"
              title="Pull"
              aria-label="Pull"
              onClick={() => void beginRemoteOperation("pull")}
              disabled={operation().kind !== null}
            >
              <svg viewBox="0 0 16 16" aria-hidden="true">
                <path d="M8 11V3 M4 7l4 4 4-4 M3 13h10" />
              </svg>
            </button>
            <button
              type="button"
              class="vs-git-sync-icon-btn"
              title="Push"
              aria-label="Push"
              onClick={() => void beginRemoteOperation("push")}
              disabled={operation().kind !== null}
            >
              <svg viewBox="0 0 16 16" aria-hidden="true">
                <path d="M8 3v8 M4 7l4-4 4 4 M3 13h10" />
              </svg>
            </button>
            <button
              type="button"
              class="vs-git-sync-icon-btn"
              title="Fetch / Prune"
              aria-label="Fetch"
              onClick={() => void beginRemoteOperation("fetch")}
              disabled={operation().kind !== null}
            >
              <svg viewBox="0 0 16 16" aria-hidden="true">
                <path d="M3 8a5 5 0 0 1 8.2-3.8M13 4v4H9 M13 8a5 5 0 0 1-8.2 3.8M3 12V8h4" />
              </svg>
            </button>
          </div>
        </div>

        <div class="vs-git-log-search">
          <input
            type="text"
            class="vs-git-log-search-input"
            placeholder="Search messages..."
            value={store.filterMessage()}
            onInput={(e) => {
              store.setFilterMessage(e.currentTarget.value);
              store.debouncedLoad(workspaceId());
            }}
            onKeyPress={handleFilterKeydown}
          />
          <input
            type="text"
            class="vs-git-log-search-input"
            placeholder="author:name"
            value={store.filterAuthor()}
            onInput={(e) => {
              store.setFilterAuthor(e.currentTarget.value);
              store.debouncedLoad(workspaceId());
            }}
            onKeyPress={handleFilterKeydown}
          />
        </div>

        <Show when={store.error()}>
          <div class="vs-git-log-error">{store.error()}</div>
        </Show>

        <Show when={logHighlight()}>
          {(highlight) => (
            <div
              class={`vs-git-log-highlight-note is-${highlight().direction}`}
            >
              {logHighlightMessage(highlight())}
            </div>
          )}
        </Show>

        <div
          class="vs-git-log-list"
          ref={scrollContainer}
          onScroll={handleScroll}
        >
          <Show
            when={!store.loading() || store.entries().length > 0}
            fallback={<div class="vs-git-log-loading">Loading...</div>}
          >
            <For each={store.entries()}>
              {(entry, index) => (
                <button
                  ref={(el) => entryRefs.set(entry.shortSha, el)}
                  class="vs-git-log-entry"
                  classList={{
                    "vs-git-log-entry-selected":
                      store.selectedSha() === entry.shortSha,
                    "vs-git-log-entry-highlight-ahead": isEntryHighlighted(
                      logHighlight(),
                      entry,
                      index(),
                      "ahead",
                    ),
                    "vs-git-log-entry-highlight-behind": isEntryHighlighted(
                      logHighlight(),
                      entry,
                      index(),
                      "behind",
                    ),
                  }}
                  onClick={() =>
                    store.loadDetail(workspaceId(), entry.shortSha)
                  }
                >
                  <div class="vs-git-log-entry-header">
                    <span class="vs-git-log-sha">{entry.shortSha}</span>
                    <span class="vs-git-log-time">
                      {formatTime(entry.authoredDate)}
                    </span>
                  </div>
                  <div class="vs-git-log-message">{entry.message}</div>
                  <div class="vs-git-log-author">{entry.authorName}</div>
                  <div class="vs-git-log-labels">
                    <For each={entry.branchLabels}>
                      {(label) => (
                        <span class="vs-git-log-label vs-git-log-branch">
                          {label}
                        </span>
                      )}
                    </For>
                    <For each={entry.tagLabels}>
                      {(label) => (
                        <span class="vs-git-log-label vs-git-log-tag">
                          {label}
                        </span>
                      )}
                    </For>
                  </div>
                </button>
              )}
            </For>
          </Show>

          <Show when={store.hasMore()}>
            <button
              class="vs-git-log-load-more"
              onClick={() => store.loadMore(workspaceId())}
              disabled={store.loading()}
            >
              {store.loading() ? "Loading..." : "Load more"}
            </button>
          </Show>
        </div>

        <Show when={store.detail()}>
          <div
            class="vs-git-log-splitter"
            classList={{ "is-resizing": isResizing() }}
            role="separator"
            aria-orientation="horizontal"
            aria-label="Resize commit detail"
            onPointerDown={startResize}
          />
          <div
            class="vs-git-log-detail"
            style={{
              height: `${detailHeight()}px`,
              "max-height": "none",
              "flex-shrink": "0",
            }}
          >
            <h4 class="vs-git-log-detail-title">
              Commit: {store.detail()?.shortSha}
            </h4>
            <Show when={store.detailLoading()}>
              <div class="vs-git-log-loading">Loading detail...</div>
            </Show>
            <Show when={store.detail() && !store.detailLoading()}>
              <div class="vs-git-log-detail-content">
                <div class="vs-git-log-detail-meta">
                  <div>
                    <strong>Author:</strong> {store.detail()!.author.name} &lt;
                    {store.detail()!.author.email}&gt;
                  </div>
                  <div>
                    <strong>Date:</strong>{" "}
                    {new Date(
                      store.detail()!.author.timestamp * 1000,
                    ).toLocaleString()}
                  </div>
                  <div>
                    <strong>Committer:</strong> {store.detail()!.committer.name}
                  </div>
                  <div>
                    <strong>SHA:</strong> {store.detail()!.fullSha}
                  </div>
                  <Show when={store.detail()!.parents.length > 0}>
                    <div>
                      <strong>Parents:</strong>{" "}
                      <For each={store.detail()!.parents}>
                        {(p) => (
                          <span class="vs-git-log-parent">{p.shortSha}</span>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
                <div class="vs-git-log-detail-message">
                  {store.detail()!.message}
                </div>
                <div class="vs-git-log-detail-files">
                  <Show
                    when={store.detail()!.files.length <= 1000}
                    fallback={
                      <div>
                        <strong>Files ({store.detail()!.files.length}):</strong>{" "}
                        Showing first 1000
                      </div>
                    }
                  >
                    <div>
                      <strong>Files ({store.detail()!.files.length}):</strong>
                    </div>
                  </Show>
                  <For each={store.detail()!.files.slice(0, 200)}>
                    {(file) => (
                      <button
                        type="button"
                        class="vs-git-log-file"
                        onClick={() => {
                          if (!props.onOpenDiff) return;
                          props.onOpenDiff({
                            workspaceId: workspaceId(),
                            source: store.detail()!.fullSha,
                            filePath: file.path,
                          });
                        }}
                      >
                        <span
                          class={`vs-git-log-file-status vs-git-log-status-${file.status}`}
                        >
                          {file.status}
                        </span>
                        <span class="vs-git-log-file-path">{file.path}</span>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            </Show>
          </div>
        </Show>

        <Show when={activeOperation().kind}>
          <GitSyncProgressDialog
            kind={activeOperation().kind!}
            remote={activeOperation().remote}
            branch={activeOperation().branch}
            stage={activeOperation().stage}
            progress={activeOperation().progress}
            abortable={activeOperation().abortable}
            pullStrategy={activeOperation().pullStrategy}
            prune={activeOperation().prune}
            largeTransfer={
              activeOperation().progress.bytesDone > 100 * 1024 * 1024 ||
              activeOperation().progress.bytesTotal > 100 * 1024 * 1024
            }
            onPullStrategyChange={(strategy) =>
              setOperation((prev) => ({ ...prev, pullStrategy: strategy }))
            }
            onPruneChange={(prune) =>
              setOperation((prev) => ({ ...prev, prune }))
            }
            onCancel={() => void cancelOperation()}
          />
        </Show>

        <Show when={remoteSelection()}>
          {(selection) => (
            <RemoteSelector
              operation={selection().operation}
              branch={selection().branch}
              remotes={selection().remotes}
              initialRemote={selection().initialRemote}
              onCancel={() => setRemoteSelection(null)}
              onConfirm={(remote, prune) => {
                const current = selection();
                setRemoteSelection(null);
                void runRemoteOperation({
                  operation: current.operation,
                  remote,
                  branch: current.branch,
                  prune,
                  strategy: operation().pullStrategy,
                });
              }}
            />
          )}
        </Show>

        <Show when={pendingAuth()}>
          {(pending) => (
            <AuthDialog
              remoteUrl={pending().remoteUrl}
              submitting={submittingAuth()}
              error={pending().error}
              onSubmit={submitAuth}
              onCancel={() => {
                setPendingAuth(null);
                showToast({
                  kind: "warning",
                  message: "已取消 · 凭证未提供",
                });
              }}
            />
          )}
        </Show>

        <Show when={forcePush()}>
          {(pending) => (
            <ForcePushDialog
              remote={pending().remote}
              branch={pending().branch}
              remoteAhead={pending().remoteAhead}
              expectedRemoteOid={pending().expectedRemoteOid}
              commits={pending().commits}
              confirmation={forceConfirmation()}
              submitting={forceSubmitting()}
              onConfirmationChange={setForceConfirmation}
              onConfirm={confirmForcePush}
              onCancel={() => {
                setForcePush(null);
                setForceConfirmation("");
              }}
            />
          )}
        </Show>

        <Show when={pullConflict()}>
          {(conflict) => (
            <PullConflictDialog
              workspacePath={workspacePath()}
              remote={conflict().remote}
              branch={conflict().branch}
              files={conflict().files}
              onCopied={() =>
                showToast({
                  kind: "success",
                  message: "已复制 · 在终端粘贴执行",
                })
              }
              onClose={() => setPullConflict(null)}
            />
          )}
        </Show>

        <Show when={toast()}>
          {(currentToast) => (
            <div
              class={`vs-git-sync-toast is-${currentToast().kind}`}
              role="status"
              onClick={(event) => event.stopPropagation()}
            >
              <span>{currentToast().message}</span>
              <Show
                when={currentToast().actionLabel && currentToast().onAction}
              >
                <button
                  type="button"
                  onClick={() => currentToast().onAction?.()}
                >
                  {currentToast().actionLabel}
                </button>
              </Show>
            </div>
          )}
        </Show>
      </div>
    </Show>
  );
};

function resolveHighlightTarget(
  highlight: ActiveLogHighlight,
  entries: GitLogEntry[],
): string | null {
  if (entries.length === 0) {
    return null;
  }

  if (highlight.direction === "ahead") {
    return entries[0]?.shortSha ?? null;
  }

  if (highlight.upstream) {
    const upstreamBoundary = entries.find((entry) =>
      entry.branchLabels.includes(highlight.upstream!),
    );
    if (upstreamBoundary) {
      return upstreamBoundary.shortSha;
    }
  }

  return entries[0]?.shortSha ?? null;
}

function isEntryHighlighted(
  highlight: ActiveLogHighlight | null,
  entry: GitLogEntry,
  index: number,
  direction: "ahead" | "behind",
): boolean {
  if (!highlight || highlight.direction !== direction) {
    return false;
  }

  if (direction === "ahead") {
    return index < highlight.count;
  }

  return highlight.targetSha === entry.shortSha;
}

function logHighlightMessage(highlight: ActiveLogHighlight): string {
  if (highlight.direction === "ahead") {
    return `${highlight.branchName ?? "HEAD"} 领先 remote ${highlight.count} commits · 已高亮本地提交`;
  }
  if (highlight.upstream) {
    return `${highlight.upstream} 领先本地 ${highlight.count} commits · 已定位 upstream 边界`;
  }
  return `remote 领先本地 ${highlight.count} commits · 当前分支未配置 upstream`;
}

function isDirty(status: GitStatusResponse): boolean {
  return (
    status.staged.length + status.unstaged.length + status.untracked.length > 0
  );
}

function pullSuccessMessage(
  remote: string,
  branch: string,
  result: PullResult,
): string {
  switch (result.stage) {
    case "ff":
      return `已 fast-forward 到 ${remote}/${branch} · ${result.mergedCommits} commits`;
    case "merge":
      return `已合并 ${remote}/${branch} · 创建合并 commit ${result.newHead.slice(0, 8)}`;
    case "rebase":
      return `已 rebase ${result.mergedCommits} 个 commit 到 ${remote}/${branch}`;
    case "upToDate":
      return `${remote}/${branch} 已是最新`;
    default:
      return `已完成 pull ${remote}/${branch}`;
  }
}

function parseNetworkError(error: unknown): NetworkOpError | null {
  if (isNetworkError(error)) {
    return error;
  }

  const raw = error instanceof Error ? error.message : stringifyUnknown(error);
  try {
    const parsed = JSON.parse(raw) as unknown;
    return isNetworkError(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function isNetworkError(error: unknown): error is NetworkOpError {
  return (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    typeof (error as { kind: unknown }).kind === "string"
  );
}

function networkErrorMessage(
  error: NetworkOpError | null,
  fallback: unknown,
): string {
  if (!error) {
    return stringifyUnknown(fallback);
  }

  switch (error.kind) {
    case "authFailed":
      return `凭证错误 · 请检查 username / password / token：${error.detail}`;
    case "networkUnreachable":
      return `网络不通 · 请检查代理 / DNS：${error.detail}`;
    case "remoteNotFound":
      return `未配置 remote ${error.remote}`;
    case "nonFastForward":
      return `${error.remoteBranch} 已有更新 · 请先 pull 或 force push`;
    case "mergeConflict":
      return `合并冲突 · ${error.files.length} 个文件`;
    case "aborted":
      return `已取消：${error.reason}`;
    case "dirtyWorkingTree":
      return "工作区有未提交修改 · 请先 commit / stash / discard";
    case "rejectedByRemote":
      return `远端拒绝操作：${error.detail}`;
    case "staleLease":
      return "远端在确认后发生变化 · 请重新 fetch";
    case "sslError":
      return `SSL 证书无效：${error.detail}`;
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
