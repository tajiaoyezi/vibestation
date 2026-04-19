import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import {
  closeAllSessions,
  drainSession,
  exitApp,
  readArtifact,
  resizeSession,
  runtimeConfig,
  sampleProcessStats,
  sessionSnapshot,
  spawnSession,
  writeArtifact,
  type DrainResponse,
  type ProcessStats,
  type RuntimeConfig,
  type SessionSummary,
} from "./backend";
import Tab, { type TabMetricsView } from "./Tab";
import { XTermWrapper } from "./xterm-wrapper";

type ScenarioName =
  | "single-yes"
  | "interactive-top"
  | "four-yes"
  | "soak-10min"
  | "hidden-5min"
  | "hol-frontend-slow"
  | "hol-ipc-saturated"
  | "hol-hidden-throttle"
  | "correctness";

interface SessionPlan {
  label: string;
  command: string;
  pollEveryMs: number;
  hiddenPollEveryMs: number;
  renderDelayMs: number;
  renderWhenHidden: boolean;
}

interface ScenarioPlan {
  name: ScenarioName;
  title: string;
  durationMs: number;
  queueCapacity: number;
  sampleEveryMs: number;
  sessions: SessionPlan[];
  initialActiveIndex: number;
  autoSwitchEveryMs?: number;
  notes: string[];
}

interface UiSession {
  id: string;
  label: string;
  command: string;
  wrapper: XTermWrapper;
  active: boolean;
  pollEveryMs: number;
  hiddenPollEveryMs: number;
  renderDelayMs: number;
  renderWhenHidden: boolean;
  queueDepth: number;
  queuedBytes: number;
  avgQueueDepth: number;
  maxQueueDepth: number;
  totalReadBytes: number;
  totalDrainedBytes: number;
  droppedChunks: number;
  droppedBytes: number;
  preview: string;
  eventLoopLagMs: number;
  freezes: number;
  exitStatus?: string | null;
}

interface QueueSample {
  elapsedMs: number;
  sessionId: string;
  queueDepth: number;
  queuedBytes: number;
  avgQueueDepth: number;
  maxQueueDepth: number;
  totalReadBytes: number;
  totalDrainedBytes: number;
  droppedChunks: number;
  droppedBytes: number;
}

interface ProcessSample extends ProcessStats {
  elapsedMs: number;
}

interface EventLoopSnapshot {
  maxLagMs: number;
  freezeCount: number;
  samples: number[];
}

const MAX_DRAIN_CHUNKS = 128;
const MAX_DRAIN_BYTES = 1024 * 1024;
const DEFAULT_COLS = 120;
const DEFAULT_ROWS = 34;

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const formatBytes = (value: number): string => {
  if (value >= 1024 * 1024) return `${(value / (1024 * 1024)).toFixed(2)} MB`;
  if (value >= 1024) return `${(value / 1024).toFixed(2)} KB`;
  return `${value} B`;
};

const bytesPerSecond = (bytes: number, durationMs: number): number =>
  durationMs <= 0 ? 0 : bytes / (durationMs / 1000);

const formatThroughput = (bytes: number, durationMs: number): string =>
  `${(bytesPerSecond(bytes, durationMs) / (1024 * 1024)).toFixed(2)} MB/s`;

const csvEscape = (value: string | number | boolean): string => {
  const text = String(value);
  return /[",\n]/.test(text) ? `"${text.replace(/"/g, '""')}"` : text;
};

const createEventLoopMonitor = (intervalMs = 16) => {
  const samples: number[] = [];
  let timer = 0;
  let freezeCount = 0;
  let maxLagMs = 0;

  const start = () => {
    const startedAt = performance.now();
    timer = window.setInterval(() => {
      const now = performance.now();
      const ticks = Math.max(1, Math.round((now - startedAt) / intervalMs));
      const expected = startedAt + ticks * intervalMs;
      const lag = Math.max(0, now - expected);
      samples.push(lag);
      maxLagMs = Math.max(maxLagMs, lag);
      if (lag > 100) freezeCount += 1;
    }, intervalMs);
  };

  const stop = (): EventLoopSnapshot => {
    window.clearInterval(timer);
    return {
      maxLagMs,
      freezeCount,
      samples,
    };
  };

  return { start, stop };
};

const baseYes = (label: string): SessionPlan => ({
  label,
  command: `yes ${label}`,
  pollEveryMs: 4,
  hiddenPollEveryMs: 4,
  renderDelayMs: 0,
  renderWhenHidden: true,
});

const scenarioCatalog: Record<Exclude<ScenarioName, "correctness">, ScenarioPlan> = {
  "single-yes": {
    name: "single-yes",
    title: "A · 单 Tab yes 10s",
    durationMs: 10_000,
    queueCapacity: 256,
    sampleEveryMs: 1_000,
    sessions: [baseYes("tab-1")],
    initialActiveIndex: 0,
    notes: ["单 Tab 高吞吐基线。"],
  },
  "interactive-top": {
    name: "interactive-top",
    title: "A · 交互 TUI（top）10s",
    durationMs: 10_000,
    queueCapacity: 128,
    sampleEveryMs: 1_000,
    sessions: [
      {
        label: "top",
        command: "while true; do clear; date '+tick %H:%M:%S'; ps -A -o pid,pcpu,pmem,comm | head -n 12; sleep 0.2; done",
        pollEveryMs: 100,
        hiddenPollEveryMs: 100,
        renderDelayMs: 0,
        renderWhenHidden: true,
      },
    ],
    initialActiveIndex: 0,
    notes: [
      "宿主机未安装 htop，使用 macOS 原生 top -s 0.2 等价验证连续刷新。",
    ],
  },
  "four-yes": {
    name: "four-yes",
    title: "A · 4 Tab yes 10s",
    durationMs: 10_000,
    queueCapacity: 256,
    sampleEveryMs: 1_000,
    sessions: [baseYes("tab-1"), baseYes("tab-2"), baseYes("tab-3"), baseYes("tab-4")],
    initialActiveIndex: 0,
    autoSwitchEveryMs: 2_500,
    notes: ["自动切 Tab，验证切换不冻结、scrollback 不串流。"],
  },
  "soak-10min": {
    name: "soak-10min",
    title: "B.1 · 4 Tab 慢消费者 soak 10 min",
    durationMs: 600_000,
    queueCapacity: 256,
    sampleEveryMs: 10_000,
    sessions: [baseYes("tab-1"), baseYes("tab-2"), baseYes("tab-3"), baseYes("tab-4")].map(
      (session) => ({
        ...session,
        renderDelayMs: 50,
      }),
    ),
    initialActiveIndex: 0,
    notes: ["所有 tab 渲染前人工延迟 50ms，验证 bounded queue + drop-oldest。"],
  },
  "hidden-5min": {
    name: "hidden-5min",
    title: "B.2 · 隐藏 tab 5 min",
    durationMs: 300_000,
    queueCapacity: 256,
    sampleEveryMs: 10_000,
    sessions: [
      baseYes("tab-1"),
      { ...baseYes("tab-2"), hiddenPollEveryMs: 1_000, renderWhenHidden: false },
      { ...baseYes("tab-3"), hiddenPollEveryMs: 1_000, renderWhenHidden: false },
      { ...baseYes("tab-4"), hiddenPollEveryMs: 1_000, renderWhenHidden: false },
    ],
    initialActiveIndex: 0,
    notes: ["隐藏 tab 策略 = bounded queue + 1Hz drain + drop-oldest，不允许无界积压。"],
  },
  "hol-frontend-slow": {
    name: "hol-frontend-slow",
    title: "B.4.1 · 前端 render 慢 3 min",
    durationMs: 180_000,
    queueCapacity: 256,
    sampleEveryMs: 5_000,
    sessions: [
      { ...baseYes("tab-1"), renderDelayMs: 500 },
      baseYes("tab-2"),
      baseYes("tab-3"),
      baseYes("tab-4"),
    ],
    initialActiveIndex: 1,
    notes: ["Tab 1 人工延迟 500ms，验证共享读线程不 HOL 阻塞其他 tab。"],
  },
  "hol-ipc-saturated": {
    name: "hol-ipc-saturated",
    title: "B.4.2 · IPC queue 满 3 min",
    durationMs: 180_000,
    queueCapacity: 256,
    sampleEveryMs: 5_000,
    sessions: [
      { ...baseYes("tab-1"), pollEveryMs: 0, hiddenPollEveryMs: 0, renderWhenHidden: false },
      baseYes("tab-2"),
      baseYes("tab-3"),
      baseYes("tab-4"),
    ],
    initialActiveIndex: 1,
    notes: ["Tab 1 完全停止 drain，逼满 bounded queue；其余 tab 必须持续推进。"],
  },
  "hol-hidden-throttle": {
    name: "hol-hidden-throttle",
    title: "B.4.3 · hidden-tab throttle 3 min",
    durationMs: 180_000,
    queueCapacity: 256,
    sampleEveryMs: 5_000,
    sessions: [
      { ...baseYes("tab-1"), hiddenPollEveryMs: 1_000, renderWhenHidden: false },
      baseYes("tab-2"),
      baseYes("tab-3"),
      baseYes("tab-4"),
    ],
    initialActiveIndex: 1,
    notes: ["Tab 1 模拟 hidden → 1Hz drain，其余 tab 走正常 60fps cadence。"],
  },
};

const toMarkdown = (
  plan: ScenarioPlan | { title: string; notes: string[] },
  durationMs: number,
  sessionViews: UiSession[],
  processSamples: ProcessSample[],
  eventLoop: EventLoopSnapshot,
  extraLines: string[] = [],
): string => {
  const totalRead = sessionViews.reduce((sum, session) => sum + session.totalReadBytes, 0);
  const totalDrained = sessionViews.reduce((sum, session) => sum + session.totalDrainedBytes, 0);
  const totalDropped = sessionViews.reduce((sum, session) => sum + session.droppedBytes, 0);
  const readThroughput = formatThroughput(totalRead, durationMs);
  const drainedThroughput = formatThroughput(totalDrained, durationMs);
  const peakRss = processSamples.reduce((peak, sample) => Math.max(peak, sample.rssKb), 0);

  return [
    `# ${plan.title}`,
    "",
    `- 持续时间：${(durationMs / 1000).toFixed(1)}s`,
    `- 总吞吐(read path)：${readThroughput}`,
    `- UI 吞吐(drain path)：${drainedThroughput}`,
    `- 总读取：${formatBytes(totalRead)}`,
    `- 总 drain：${formatBytes(totalDrained)}`,
    `- 总 drop：${formatBytes(totalDropped)}`,
    `- 主线程最大 lag：${eventLoop.maxLagMs.toFixed(2)}ms`,
    `- freeze (>100ms)：${eventLoop.freezeCount}`,
    `- 峰值 RSS：${peakRss} KB`,
    "",
    "## Sessions",
    "",
    ...sessionViews.flatMap((session) => [
      `### ${session.label}`,
      `- command: \`${session.command}\``,
      `- throughput(read): ${formatThroughput(session.totalReadBytes, durationMs)}`,
      `- throughput(drain): ${formatThroughput(session.totalDrainedBytes, durationMs)}`,
      `- queue: depth=${session.queueDepth}, avg=${session.avgQueueDepth.toFixed(2)}, max=${session.maxQueueDepth}`,
      `- drop: ${session.droppedChunks} chunks / ${formatBytes(session.droppedBytes)}`,
      `- preview:`,
      "```text",
      session.preview || "(empty)",
      "```",
      "",
    ]),
    "## Notes",
    "",
    ...plan.notes.map((note) => `- ${note}`),
    ...extraLines.map((line) => `- ${line}`),
    "",
  ].join("\n");
};

const queueSamplesToCsv = (samples: QueueSample[]): string => {
  const rows = [
    [
      "elapsed_ms",
      "session_id",
      "queue_depth",
      "queued_bytes",
      "avg_queue_depth",
      "max_queue_depth",
      "total_read_bytes",
      "total_drained_bytes",
      "dropped_chunks",
      "dropped_bytes",
    ].join(","),
  ];

  for (const sample of samples) {
    rows.push(
      [
        sample.elapsedMs,
        sample.sessionId,
        sample.queueDepth,
        sample.queuedBytes,
        sample.avgQueueDepth.toFixed(4),
        sample.maxQueueDepth,
        sample.totalReadBytes,
        sample.totalDrainedBytes,
        sample.droppedChunks,
        sample.droppedBytes,
      ]
        .map((value) => csvEscape(value))
        .join(","),
    );
  }

  return rows.join("\n");
};

const serializableSession = (session: UiSession) => ({
  id: session.id,
  label: session.label,
  command: session.command,
  active: session.active,
  pollEveryMs: session.pollEveryMs,
  hiddenPollEveryMs: session.hiddenPollEveryMs,
  renderDelayMs: session.renderDelayMs,
  renderWhenHidden: session.renderWhenHidden,
  queueDepth: session.queueDepth,
  queuedBytes: session.queuedBytes,
  avgQueueDepth: session.avgQueueDepth,
  maxQueueDepth: session.maxQueueDepth,
  totalReadBytes: session.totalReadBytes,
  totalDrainedBytes: session.totalDrainedBytes,
  droppedChunks: session.droppedChunks,
  droppedBytes: session.droppedBytes,
  preview: session.preview,
  eventLoopLagMs: session.eventLoopLagMs,
  freezes: session.freezes,
  exitStatus: session.exitStatus,
});

const processSamplesToCsv = (samples: ProcessSample[]): string => {
  const rows = ["elapsed_ms,pid,rss_kb,fd_count,session_count,reader_thread_alive"];
  for (const sample of samples) {
    rows.push(
      [
        sample.elapsedMs,
        sample.pid,
        sample.rssKb,
        sample.fdCount,
        sample.sessionCount,
        sample.readerThreadAlive,
      ]
        .map((value) => csvEscape(value))
        .join(","),
    );
  }
  return rows.join("\n");
};

export default function App() {
  const [runtime, setRuntime] = createSignal<RuntimeConfig | null>(null);
  const [sessions, setSessions] = createSignal<UiSession[]>([]);
  const [logs, setLogs] = createSignal<string[]>([]);
  const [summary, setSummary] = createSignal<string>("尚未运行 benchmark。");
  const [currentScenario, setCurrentScenario] = createSignal<string>("idle");
  const [running, setRunning] = createSignal(false);
  const [activeIndex, setActiveIndex] = createSignal(0);

  let pumpStops: Array<() => void> = [];
  let timerStops: Array<() => void> = [];

  const sessionTabs = createMemo(() => sessions());

  const appendLog = (message: string) => {
    const line = `[${new Date().toLocaleTimeString()}] ${message}`;
    setLogs((previous) => [...previous.slice(-199), line]);
    console.info(line);
  };

  const updateSession = (sessionId: string, patch: Partial<UiSession>) => {
    setSessions((previous) =>
      previous.map((session) =>
        session.id === sessionId
          ? {
              ...session,
              ...patch,
            }
          : session,
      ),
    );
  };

  const setActiveSessionIndex = (index: number) => {
    setActiveIndex(index);
    setSessions((previous) =>
      previous.map((session, sessionIndex) => ({
        ...session,
        active: sessionIndex === index,
      })),
    );
    const next = sessions()[index];
    queueMicrotask(() => next?.wrapper.fit());
  };

  const stopTimers = () => {
    for (const stop of pumpStops) stop();
    for (const stop of timerStops) stop();
    pumpStops = [];
    timerStops = [];
  };

  const disposeUiSessions = () => {
    stopTimers();
    for (const session of sessions()) {
      session.wrapper.dispose();
    }
    setSessions([]);
    setActiveIndex(0);
  };

  const hardReset = async () => {
    stopTimers();
    try {
      await closeAllSessions();
    } catch (error) {
      appendLog(`close_all_sessions failed: ${String(error)}`);
    }
    disposeUiSessions();
  };

  const startPump = (sessionId: string) => {
    let cancelled = false;
    let busy = false;

    const loop = async () => {
      while (!cancelled) {
        const current = sessions().find((session) => session.id === sessionId);
        if (!current) {
          await sleep(50);
          continue;
        }

        const pollEveryMs = current.active ? current.pollEveryMs : current.hiddenPollEveryMs;
        if (pollEveryMs <= 0) {
          await sleep(50);
          continue;
        }

        if (!busy) {
          busy = true;
          try {
            const response: DrainResponse = await drainSession({
              sessionId,
              maxChunks: MAX_DRAIN_CHUNKS,
              maxBytes: MAX_DRAIN_BYTES,
            });
            const payload = response.chunks.join("");
            const shouldRender = current.active || current.renderWhenHidden;
            if (payload && shouldRender) {
              current.wrapper.write(payload, current.renderDelayMs);
            }

            updateSession(sessionId, {
              queueDepth: response.queueDepth,
              queuedBytes: response.queuedBytes,
              avgQueueDepth: response.avgQueueDepth,
              maxQueueDepth: response.maxQueueDepth,
              totalReadBytes: response.totalReadBytes,
              totalDrainedBytes: response.totalDrainedBytes,
              droppedChunks: response.droppedChunks,
              droppedBytes: response.droppedBytes,
              exitStatus: response.exitStatus,
              preview: current.wrapper.getPreview(3),
            });
          } catch (error) {
            appendLog(`drain_session(${sessionId}) failed: ${String(error)}`);
          } finally {
            busy = false;
          }
        }

        await sleep(Math.max(4, pollEveryMs));
      }
    };

    void loop();

    return () => {
      cancelled = true;
    };
  };

  const sampleCurrentState = async (
    startedAt: number,
    queueSamples: QueueSample[],
    processSamples: ProcessSample[],
  ) => {
    const elapsedMs = Math.round(performance.now() - startedAt);
    for (const session of sessions()) {
      queueSamples.push({
        elapsedMs,
        sessionId: session.id,
        queueDepth: session.queueDepth,
        queuedBytes: session.queuedBytes,
        avgQueueDepth: session.avgQueueDepth,
        maxQueueDepth: session.maxQueueDepth,
        totalReadBytes: session.totalReadBytes,
        totalDrainedBytes: session.totalDrainedBytes,
        droppedChunks: session.droppedChunks,
        droppedBytes: session.droppedBytes,
      });
    }

    try {
      const stats = await sampleProcessStats();
      processSamples.push({
        ...stats,
        elapsedMs,
      });
    } catch (error) {
      appendLog(`sample_process_stats failed: ${String(error)}`);
    }
  };

  const writeRunArtifacts = async (
    outputDir: string | null | undefined,
    files: Array<{ name: string; contents: string }>,
  ) => {
    if (!outputDir) return;
    for (const file of files) {
      await writeArtifact({ path: `${outputDir}/${file.name}`, contents: file.contents });
    }
  };

  const collectSessionSnapshots = async (): Promise<UiSession[]> => {
    const snapshotMap = new Map<string, SessionSummary>();
    for (const session of sessions()) {
      try {
        const snapshot = await sessionSnapshot(session.id);
        snapshotMap.set(session.id, snapshot);
      } catch (error) {
        appendLog(`session_snapshot(${session.id}) failed: ${String(error)}`);
      }
    }

    return sessions().map((session) => {
      const snapshot = snapshotMap.get(session.id);
      return snapshot
        ? {
            ...session,
            queueDepth: snapshot.queueDepth,
            queuedBytes: snapshot.queuedBytes,
            avgQueueDepth: snapshot.avgQueueDepth,
            maxQueueDepth: snapshot.maxQueueDepth,
            totalReadBytes: snapshot.totalReadBytes,
            totalDrainedBytes: snapshot.totalDrainedBytes,
            droppedChunks: snapshot.droppedChunks,
            droppedBytes: snapshot.droppedBytes,
            exitStatus: snapshot.exitStatus,
            preview: session.wrapper.getPreview(4),
          }
        : session;
    });
  };

  const runTerminalScenario = async (plan: ScenarioPlan) => {
    await hardReset();
    setRunning(true);
    setCurrentScenario(plan.title);
    setSummary(`准备运行：${plan.title}`);
    appendLog(`scenario start → ${plan.title}`);

    const createdSessions: UiSession[] = [];
    for (let index = 0; index < plan.sessions.length; index += 1) {
      const sessionPlan = plan.sessions[index];
      const created = await spawnSession({
        label: sessionPlan.label,
        command: sessionPlan.command,
        cols: DEFAULT_COLS,
        rows: DEFAULT_ROWS,
        queueCapacity: plan.queueCapacity,
      });

      createdSessions.push({
        id: created.id,
        label: created.label,
        command: created.command,
        wrapper: new XTermWrapper(),
        active: index === plan.initialActiveIndex,
        pollEveryMs: sessionPlan.pollEveryMs,
        hiddenPollEveryMs: sessionPlan.hiddenPollEveryMs,
        renderDelayMs: sessionPlan.renderDelayMs,
        renderWhenHidden: sessionPlan.renderWhenHidden,
        queueDepth: created.queueDepth,
        queuedBytes: created.queuedBytes,
        avgQueueDepth: created.avgQueueDepth,
        maxQueueDepth: created.maxQueueDepth,
        totalReadBytes: created.totalReadBytes,
        totalDrainedBytes: created.totalDrainedBytes,
        droppedChunks: created.droppedChunks,
        droppedBytes: created.droppedBytes,
        preview: "",
        eventLoopLagMs: 0,
        freezes: 0,
        exitStatus: created.exitStatus,
      });
    }

    setSessions(createdSessions);
    setActiveSessionIndex(plan.initialActiveIndex);
    pumpStops = createdSessions.map((session) => startPump(session.id));

    const eventLoop = createEventLoopMonitor();
    eventLoop.start();

    const queueSamples: QueueSample[] = [];
    const processSamples: ProcessSample[] = [];
    const startedAt = performance.now();

    await sampleCurrentState(startedAt, queueSamples, processSamples);

    const samplerId = window.setInterval(() => {
      void sampleCurrentState(startedAt, queueSamples, processSamples);
    }, plan.sampleEveryMs);
    timerStops.push(() => window.clearInterval(samplerId));

    if (plan.autoSwitchEveryMs) {
      const switchId = window.setInterval(() => {
        const sessionCount = sessions().length;
        if (sessionCount === 0) return;
        const nextIndex = (activeIndex() + 1) % sessionCount;
        setActiveSessionIndex(nextIndex);
      }, plan.autoSwitchEveryMs);
      timerStops.push(() => window.clearInterval(switchId));
    }

    await sleep(plan.durationMs);
    appendLog(`duration elapsed → ${plan.title}`);
    const eventLoopSnapshot = eventLoop.stop();
    stopTimers();

    appendLog("collecting snapshots");
    const sessionViews = await collectSessionSnapshots();
    setSessions(
      sessionViews.map((session) => ({
        ...session,
        eventLoopLagMs: eventLoopSnapshot.maxLagMs,
        freezes: eventLoopSnapshot.freezeCount,
      })),
    );

    const markdown = toMarkdown(plan, plan.durationMs, sessionViews, processSamples, eventLoopSnapshot);
    const json = JSON.stringify(
      {
        plan,
        durationMs: plan.durationMs,
        eventLoop: eventLoopSnapshot,
        sessions: sessionViews.map(serializableSession),
        processSamples,
      },
      null,
      2,
    );

    const outputDir = runtime()?.outputDir ?? null;
    appendLog(`writing artifacts → ${outputDir ?? "<skip>"}`);
    await writeRunArtifacts(outputDir, [
      { name: "summary.md", contents: markdown },
      { name: "summary.json", contents: json },
      { name: "queue-depth.csv", contents: queueSamplesToCsv(queueSamples) },
      { name: "rss-over-time.csv", contents: processSamplesToCsv(processSamples) },
      {
        name: "session-previews.txt",
        contents: sessionViews
          .map((session) => `## ${session.label}\n${session.preview || "(empty)"}`)
          .join("\n\n"),
      },
    ]);

    setSummary(markdown);
    appendLog(`scenario done → ${plan.title}`);

    appendLog("closing sessions");
    await closeAllSessions();
    setRunning(false);
  };

  const runCorrectnessScenario = async () => {
    await hardReset();
    setRunning(true);
    setCurrentScenario("C · correctness");
    appendLog("scenario start → correctness");

    const outputDir = runtime()?.outputDir ?? "/tmp";
    const resizeFile = `${outputDir}/resize-winch.txt`;
    const beforeStats = await sampleProcessStats();

    const resizeSessionView = await spawnSession({
      label: "resize-check",
      command: `rm -f ${resizeFile}; trap 'stty size > ${resizeFile}' WINCH; printf 'READY\\n'; while true; do sleep 1; done`,
      cols: 80,
      rows: 24,
      queueCapacity: 64,
    });

    const wrapper = new XTermWrapper();
    setSessions([
      {
        id: resizeSessionView.id,
        label: resizeSessionView.label,
        command: resizeSessionView.command,
        wrapper,
        active: true,
        pollEveryMs: 100,
        hiddenPollEveryMs: 100,
        renderDelayMs: 0,
        renderWhenHidden: true,
        queueDepth: resizeSessionView.queueDepth,
        queuedBytes: resizeSessionView.queuedBytes,
        avgQueueDepth: resizeSessionView.avgQueueDepth,
        maxQueueDepth: resizeSessionView.maxQueueDepth,
        totalReadBytes: resizeSessionView.totalReadBytes,
        totalDrainedBytes: resizeSessionView.totalDrainedBytes,
        droppedChunks: resizeSessionView.droppedChunks,
        droppedBytes: resizeSessionView.droppedBytes,
        preview: "",
        eventLoopLagMs: 0,
        freezes: 0,
        exitStatus: resizeSessionView.exitStatus,
      },
    ]);
    pumpStops = [startPump(resizeSessionView.id)];

    await sleep(1200);
    await resizeSession({ sessionId: resizeSessionView.id, cols: 100, rows: 40 });
    await sleep(1500);

    let resizeResult = "<missing>";
    try {
      resizeResult = await readArtifact({ path: resizeFile });
    } catch (error) {
      appendLog(`read_artifact(${resizeFile}) failed: ${String(error)}`);
    }

    await closeAllSessions();
    stopTimers();
    const afterStats = await sampleProcessStats();

    const markdown = [
      "# C · correctness",
      "",
      `- resize trap output: \`${resizeResult.trim() || "<missing>"}\``,
      `- fd before: ${beforeStats.fdCount}`,
      `- fd after: ${afterStats.fdCount}`,
      `- rss before: ${beforeStats.rssKb} KB`,
      `- rss after: ${afterStats.rssKb} KB`,
      "",
      "## Checks",
      "",
      `- SIGWINCH: ${resizeResult.trim() === "40 100" ? "PASS" : "CHECK"}`,
      `- cleanup(fd delta): ${afterStats.fdCount - beforeStats.fdCount}`,
      `- reader alive: ${afterStats.readerThreadAlive}`,
      "",
    ].join("\n");

    await writeRunArtifacts(outputDir, [
      {
        name: "correctness-summary.md",
        contents: markdown,
      },
      {
        name: "correctness-summary.json",
        contents: JSON.stringify(
          {
            resizeResult,
            beforeStats,
            afterStats,
          },
          null,
          2,
        ),
      },
    ]);

    setSummary(markdown);
    appendLog("scenario done → correctness");
    setRunning(false);
  };

  const runScenario = async (scenario: ScenarioName) => {
    if (running()) return;
    setLogs([]);
    try {
      if (scenario === "correctness") {
        await runCorrectnessScenario();
      } else {
        await runTerminalScenario(scenarioCatalog[scenario]);
      }
      if (runtime()?.closeOnComplete) {
        appendLog("closeOnComplete=1 → exit_app");
        await sleep(350);
        await exitApp();
      }
    } catch (error) {
      appendLog(`scenario failed: ${String(error)}`);
      setSummary(`执行失败：${String(error)}`);
      setRunning(false);
    }
  };

  onMount(async () => {
    const config = await runtimeConfig();
    setRuntime(config);
    appendLog(`runtime config loaded → scenario=${config.scenario ?? "<none>"}`);
    if (config.scenario) {
      const scenario = config.scenario as ScenarioName;
      if (scenario === "correctness" || scenarioCatalog[scenario as Exclude<ScenarioName, "correctness">]) {
        void runScenario(scenario);
      }
    }
  });

  onCleanup(() => {
    void hardReset();
  });

  const aggregate = createMemo(() => {
    const allSessions = sessions();
    const totalRead = allSessions.reduce((sum, session) => sum + session.totalReadBytes, 0);
    const totalDrained = allSessions.reduce((sum, session) => sum + session.totalDrainedBytes, 0);
    const totalDropped = allSessions.reduce((sum, session) => sum + session.droppedBytes, 0);
    return {
      sessionCount: allSessions.length,
      totalRead,
      totalDrained,
      totalDropped,
    };
  });

  return (
    <div class="app-shell">
      <aside class="sidebar">
        <div>
          <h1>SPIKE-05 · portable-pty + xterm 多 Tab 压测</h1>
          <p class="muted">
            共享读线程 + bounded queue + drop-oldest（禁止 block-producer）。
          </p>
          <small>
            当前 runtime：<code>{runtime()?.scenario ?? "manual"}</code>
          </small>
        </div>

        <section class="card">
          <h2>Quick Actions</h2>
          <div class="button-grid">
            <button class="primary" disabled={running()} onClick={() => void runScenario("single-yes")}>
              A · 单 Tab yes
            </button>
            <button disabled={running()} onClick={() => void runScenario("interactive-top")}>
              A · top 10s
            </button>
            <button disabled={running()} onClick={() => void runScenario("four-yes")}>
              A · 4 Tab yes
            </button>
            <button disabled={running()} onClick={() => void runScenario("correctness")}>
              C · correctness
            </button>
            <button disabled={running()} onClick={() => void runScenario("soak-10min")}>
              B.1 · soak 10m
            </button>
            <button disabled={running()} onClick={() => void runScenario("hidden-5min")}>
              B.2 · hidden 5m
            </button>
            <button disabled={running()} onClick={() => void runScenario("hol-frontend-slow")}>
              B.4.1 · render slow
            </button>
            <button disabled={running()} onClick={() => void runScenario("hol-ipc-saturated")}>
              B.4.2 · queue full
            </button>
            <button disabled={running()} onClick={() => void runScenario("hol-hidden-throttle")}>
              B.4.3 · hidden 1Hz
            </button>
            <button class="danger" disabled={running()} onClick={() => void hardReset()}>
              Reset
            </button>
          </div>
        </section>

        <section class="card kv">
          <h3>Aggregate</h3>
          <div>
            <span>sessions</span>
            <strong>{aggregate().sessionCount}</strong>
          </div>
          <div>
            <span>read</span>
            <strong>{formatBytes(aggregate().totalRead)}</strong>
          </div>
          <div>
            <span>drained</span>
            <strong>{formatBytes(aggregate().totalDrained)}</strong>
          </div>
          <div>
            <span>dropped</span>
            <strong>{formatBytes(aggregate().totalDropped)}</strong>
          </div>
        </section>

        <section class="card">
          <h3>Logs</h3>
          <div class="log-box">{logs().join("\n") || "(no logs yet)"}</div>
        </section>

        <section class="card">
          <h3>Latest Summary</h3>
          <div class="summary-box">{summary()}</div>
        </section>
      </aside>

      <main class="main-pane">
        <header class="toolbar">
          <div>
            <strong>{currentScenario()}</strong>
            <div class="muted">{running() ? "running…" : "idle"}</div>
          </div>
          <div class="status">
            <span class="pill">active {activeIndex() + 1}</span>
            <span class="pill">read {formatBytes(aggregate().totalRead)}</span>
            <span class="pill">drain {formatBytes(aggregate().totalDrained)}</span>
            <span class="pill danger">drop {formatBytes(aggregate().totalDropped)}</span>
          </div>
        </header>

        <section class="workspace">
          <div class="tabs">
            <For each={sessionTabs()}>
              {(session, index) => (
                <button
                  classList={{ active: session.active }}
                  onClick={() => setActiveSessionIndex(index())}
                >
                  {session.label}
                </button>
              )}
            </For>
          </div>

          <For each={sessionTabs()}>
            {(session) => (
              <Tab
                id={session.id}
                label={session.label}
                command={session.command}
                active={session.active}
                wrapper={session.wrapper}
                metrics={{
                  queueDepth: session.queueDepth,
                  queuedBytes: session.queuedBytes,
                  avgQueueDepth: session.avgQueueDepth,
                  maxQueueDepth: session.maxQueueDepth,
                  totalReadBytes: session.totalReadBytes,
                  totalDrainedBytes: session.totalDrainedBytes,
                  droppedChunks: session.droppedChunks,
                  droppedBytes: session.droppedBytes,
                  eventLoopLagMs: session.eventLoopLagMs,
                  freezes: session.freezes,
                  preview: session.preview,
                  pollEveryMs: session.pollEveryMs,
                  hiddenPollEveryMs: session.hiddenPollEveryMs,
                  renderDelayMs: session.renderDelayMs,
                  renderWhenHidden: session.renderWhenHidden,
                  exitStatus: session.exitStatus,
                } satisfies TabMetricsView}
              />
            )}
          </For>

          <Show when={sessionTabs().length === 0}>
            <div class="card" style={{ margin: "60px 20px 20px" }}>
              <h3>等待任务</h3>
              <p class="muted">
                运行左侧任一 scenario。若通过 shell 自动启动，会在完成后把 summary.json / queue-depth.csv / rss-over-time.csv
                写入指定 output dir。
              </p>
            </div>
          </Show>
        </section>
      </main>
    </div>
  );
}
