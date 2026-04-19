import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import {
  closeAllSessions,
  drainSession,
  exitApp,
  readArtifact,
  resizeSession,
  runtimeConfig,
  sampleProcessStats,
  spawnSession,
  writeArtifact,
  type DrainResponse,
  type ProcessStats,
  type RuntimeConfig,
} from "./backend";
import Tab, { type TabMetricsView } from "./Tab";
import { XTermWrapper } from "./xterm-wrapper";

type ScenarioName = "single-yes" | "four-yes" | "interactive-tui" | "correctness";

interface SessionPlan {
  label: string;
  command: string;
  pollEveryMs: number;
  hiddenPollEveryMs: number;
  renderDelayMs: number;
  renderWhenHidden: boolean;
}

interface ScenarioPlan {
  name: Exclude<ScenarioName, "correctness">;
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
  exitStatus?: string | null;
  readerStrategy: string;
  readCalls: number;
  avgReadBytes: number;
  avgReadSyscallUs: number;
  avgEnqueueUs: number;
  invokeP50Ms: number;
  invokeP99Ms: number;
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

interface DrainSample {
  elapsedMs: number;
  sessionId: string;
  invokeMs: number;
  loopGapMs: number;
  drainedBytes: number;
  shouldRender: boolean;
  queueDepth: number;
  queuedBytes: number;
}

interface SeriesStats {
  count: number;
  mean: number;
  p50: number;
  p99: number;
  max: number;
}

interface FrontendProfileState {
  lastLoopStartedAt: number;
  lastTotalDrainedBytes: number;
  invokeMs: number[];
  loopGapMs: number[];
  drainedBytes: number[];
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

const throughputMbps = (bytes: number, durationMs: number): number =>
  durationMs <= 0 ? 0 : bytes / (durationMs / 1000) / (1024 * 1024);

const formatThroughput = (bytes: number, durationMs: number): string =>
  `${throughputMbps(bytes, durationMs).toFixed(2)} MB/s`;

const csvEscape = (value: string | number | boolean): string => {
  const text = String(value);
  return /[",\n]/.test(text) ? `"${text.replace(/"/g, '""')}"` : text;
};

const percentile = (values: number[], ratio: number): number => {
  if (values.length === 0) return 0;
  const ordered = [...values].sort((a, b) => a - b);
  const index = Math.min(ordered.length - 1, Math.max(0, Math.ceil(ordered.length * ratio) - 1));
  return ordered[index];
};

const summarizeSeries = (values: number[]): SeriesStats => {
  if (values.length === 0) return { count: 0, mean: 0, p50: 0, p99: 0, max: 0 };
  const sum = values.reduce((total, value) => total + value, 0);
  return {
    count: values.length,
    mean: sum / values.length,
    p50: percentile(values, 0.5),
    p99: percentile(values, 0.99),
    max: Math.max(...values),
  };
};

const queueSamplesToCsv = (samples: QueueSample[]): string => {
  const rows = [
    "elapsed_ms,session_id,queue_depth,queued_bytes,avg_queue_depth,max_queue_depth,total_read_bytes,total_drained_bytes,dropped_chunks,dropped_bytes",
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
        .map(csvEscape)
        .join(","),
    );
  }
  return rows.join("\n");
};

const processSamplesToCsv = (samples: ProcessSample[]): string => {
  const rows = ["elapsed_ms,pid,rss_kb,fd_count,session_count,reader_thread_alive,reader_threads,reader_strategy"];
  for (const sample of samples) {
    rows.push(
      [
        sample.elapsedMs,
        sample.pid,
        sample.rssKb,
        sample.fdCount,
        sample.sessionCount,
        sample.readerThreadAlive,
        sample.readerThreads,
        sample.readerStrategy,
      ]
        .map(csvEscape)
        .join(","),
    );
  }
  return rows.join("\n");
};

const drainSamplesToCsv = (samples: DrainSample[]): string => {
  const rows = ["elapsed_ms,session_id,invoke_ms,loop_gap_ms,drained_bytes,should_render,queue_depth,queued_bytes"];
  for (const sample of samples) {
    rows.push(
      [
        sample.elapsedMs,
        sample.sessionId,
        sample.invokeMs.toFixed(4),
        sample.loopGapMs.toFixed(4),
        sample.drainedBytes,
        sample.shouldRender,
        sample.queueDepth,
        sample.queuedBytes,
      ]
        .map(csvEscape)
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
  exitStatus: session.exitStatus,
  readerStrategy: session.readerStrategy,
  readCalls: session.readCalls,
  avgReadBytes: session.avgReadBytes,
  avgReadSyscallUs: session.avgReadSyscallUs,
  avgEnqueueUs: session.avgEnqueueUs,
  invokeP50Ms: session.invokeP50Ms,
  invokeP99Ms: session.invokeP99Ms,
});

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
    notes: ["shared-reader vs per-session-reader 对照基线。"],
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
    notes: ["自动切 Tab，观察 reader 策略变化是否真的影响 UI drain。"],
  },
  "interactive-tui": {
    name: "interactive-tui",
    title: "A · synthetic TUI 10s",
    durationMs: 10_000,
    queueCapacity: 128,
    sampleEveryMs: 1_000,
    sessions: [
      {
        label: "tui",
        command: "while true; do clear; date '+tick %H:%M:%S'; ps -A -o pid,pcpu,pmem,comm | head -n 12; sleep 0.2; done",
        pollEveryMs: 100,
        hiddenPollEveryMs: 100,
        renderDelayMs: 0,
        renderWhenHidden: true,
      },
    ],
    initialActiveIndex: 0,
    notes: ["宿主机无 htop，使用 5Hz synthetic TUI 替代。"],
  },
};

export default function App() {
  const [runtime, setRuntime] = createSignal<RuntimeConfig | null>(null);
  const [sessions, setSessions] = createSignal<UiSession[]>([]);
  const [logs, setLogs] = createSignal<string[]>([]);
  const [summary, setSummary] = createSignal<string>("等待运行 SPIKE-05.5 对照场景。");
  const [currentScenario, setCurrentScenario] = createSignal<string>("idle");
  const [running, setRunning] = createSignal(false);
  const [activeIndex, setActiveIndex] = createSignal(0);

  let pumpStops: Array<() => void> = [];
  let timerStops: Array<() => void> = [];

  const appendLog = (message: string) => {
    const line = `[${new Date().toLocaleTimeString()}] ${message}`;
    setLogs((previous) => [...previous.slice(-199), line]);
    console.info(line);
  };

  const updateSession = (sessionId: string, patch: Partial<UiSession>) => {
    setSessions((previous) => previous.map((session) => (session.id === sessionId ? { ...session, ...patch } : session)));
  };

  const stopTimers = () => {
    for (const stop of pumpStops) stop();
    for (const stop of timerStops) stop();
    pumpStops = [];
    timerStops = [];
  };

  const hardReset = async () => {
    stopTimers();
    try {
      await closeAllSessions();
    } catch (error) {
      appendLog(`closeAllSessions failed: ${String(error)}`);
    }
    for (const session of sessions()) session.wrapper.dispose();
    setSessions([]);
    setActiveIndex(0);
  };

  const setActiveSessionIndex = (index: number) => {
    setActiveIndex(index);
    setSessions((previous) => previous.map((session, sessionIndex) => ({ ...session, active: sessionIndex === index })));
    const target = sessions()[index];
    queueMicrotask(() => target?.wrapper.fit());
  };

  const writeRunArtifacts = async (outputDir: string | null | undefined, files: Array<{ name: string; contents: string }>) => {
    if (!outputDir) return;
    for (const file of files) {
      await writeArtifact({ path: `${outputDir}/${file.name}`, contents: file.contents });
    }
  };

  const startPump = (
    sessionId: string,
    startedAt: number,
    profiles: Map<string, FrontendProfileState>,
    drainSamples: DrainSample[],
  ) => {
    let cancelled = false;
    let busy = false;

    const loop = async () => {
      while (!cancelled) {
        const current = sessions().find((session) => session.id === sessionId);
        if (!current) {
          await sleep(25);
          continue;
        }

        const pollEveryMs = current.active ? current.pollEveryMs : current.hiddenPollEveryMs;
        if (pollEveryMs <= 0) {
          await sleep(25);
          continue;
        }

        if (!busy) {
          busy = true;
          const profile = profiles.get(sessionId)!;
          const invokeStarted = performance.now();
          const loopGapMs = profile.lastLoopStartedAt === 0 ? 0 : invokeStarted - profile.lastLoopStartedAt;
          profile.lastLoopStartedAt = invokeStarted;

          try {
            const response: DrainResponse = await drainSession({
              sessionId,
              maxChunks: MAX_DRAIN_CHUNKS,
              maxBytes: MAX_DRAIN_BYTES,
            });
            const invokeMs = performance.now() - invokeStarted;
            const drainedBytes = Math.max(0, response.totalDrainedBytes - profile.lastTotalDrainedBytes);
            profile.lastTotalDrainedBytes = response.totalDrainedBytes;
            profile.invokeMs.push(invokeMs);
            profile.loopGapMs.push(loopGapMs);
            profile.drainedBytes.push(drainedBytes);
            const payload = response.chunks.join("");
            const shouldRender = current.active || current.renderWhenHidden;
            if (payload && shouldRender) {
              current.wrapper.write(payload, current.renderDelayMs);
            }

            drainSamples.push({
              elapsedMs: Math.round(performance.now() - startedAt),
              sessionId,
              invokeMs,
              loopGapMs,
              drainedBytes,
              shouldRender,
              queueDepth: response.queueDepth,
              queuedBytes: response.queuedBytes,
            });

            const invokeSummary = summarizeSeries(profile.invokeMs);
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
              readerStrategy: response.readerStrategy,
              readCalls: response.readCalls,
              avgReadBytes: response.avgReadBytes,
              avgReadSyscallUs: response.avgReadSyscallUs,
              avgEnqueueUs: response.avgEnqueueUs,
              preview: runtime()?.closeOnComplete ? "<headless>" : current.wrapper.getPreview(4),
              invokeP50Ms: invokeSummary.p50,
              invokeP99Ms: invokeSummary.p99,
            });
          } catch (error) {
            appendLog(`drainSession(${sessionId}) failed: ${String(error)}`);
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

  const sampleCurrentState = async (startedAt: number, queueSamples: QueueSample[], processSamples: ProcessSample[]) => {
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
      processSamples.push({ ...stats, elapsedMs });
    } catch (error) {
      appendLog(`sampleProcessStats failed: ${String(error)}`);
    }
  };

  const runTerminalScenario = async (plan: ScenarioPlan) => {
    const progressLines: string[] = [];
    const markProgress = async (message: string) => {
      progressLines.push(message);
      const outputDir = runtime()?.outputDir;
      if (outputDir) {
        await writeArtifact({ path: `${outputDir}/progress.log`, contents: progressLines.join("\n") });
      }
    };

    await hardReset();
    await markProgress(`start ${plan.name}`);
    setRunning(true);
    setCurrentScenario(`${plan.title} · ${runtime()?.strategy ?? "unknown"}`);
    setSummary(`准备运行：${plan.title}`);
    appendLog(`scenario start → ${plan.title} (${runtime()?.strategy})`);

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
        exitStatus: created.exitStatus,
        readerStrategy: created.readerStrategy,
        readCalls: created.readCalls,
        avgReadBytes: created.avgReadBytes,
        avgReadSyscallUs: created.avgReadSyscallUs,
        avgEnqueueUs: created.avgEnqueueUs,
        invokeP50Ms: 0,
        invokeP99Ms: 0,
      });
    }

    setSessions(createdSessions);
    await markProgress(`spawned ${createdSessions.length}`);
    setActiveSessionIndex(plan.initialActiveIndex);

    const profiles = new Map<string, FrontendProfileState>();
    for (const session of createdSessions) {
      profiles.set(session.id, {
        lastLoopStartedAt: 0,
        lastTotalDrainedBytes: 0,
        invokeMs: [],
        loopGapMs: [],
        drainedBytes: [],
      });
    }

    const queueSamples: QueueSample[] = [];
    const processSamples: ProcessSample[] = [];
    const drainSamples: DrainSample[] = [];
    const startedAt = performance.now();

    pumpStops = createdSessions.map((session) => startPump(session.id, startedAt, profiles, drainSamples));
    await sampleCurrentState(startedAt, queueSamples, processSamples);
    await markProgress("sampling started");

    const samplerId = window.setInterval(() => {
      void sampleCurrentState(startedAt, queueSamples, processSamples);
    }, plan.sampleEveryMs);
    timerStops.push(() => window.clearInterval(samplerId));

    if (plan.autoSwitchEveryMs) {
      const switchId = window.setInterval(() => {
        const count = sessions().length;
        if (count === 0) return;
        setActiveSessionIndex((activeIndex() + 1) % count);
      }, plan.autoSwitchEveryMs);
      timerStops.push(() => window.clearInterval(switchId));
    }

    await markProgress("sleep begin");
    await sleep(plan.durationMs);
    await markProgress("sleep end");
    stopTimers();
    await sleep(400);

    await markProgress("finalize session views");
    const sessionViews = sessions().map((session) => ({
      ...session,
      invokeP50Ms: summarizeSeries(profiles.get(session.id)?.invokeMs ?? []).p50,
      invokeP99Ms: summarizeSeries(profiles.get(session.id)?.invokeMs ?? []).p99,
      preview: runtime()?.closeOnComplete ? "<headless>" : session.wrapper.getPreview(4),
    }));

    const totalRead = sessionViews.reduce((sum, session) => sum + session.totalReadBytes, 0);
    const totalDrained = sessionViews.reduce((sum, session) => sum + session.totalDrainedBytes, 0);
    const totalDropped = sessionViews.reduce((sum, session) => sum + session.droppedBytes, 0);
    const strategy = runtime()?.strategy ?? sessionViews[0]?.readerStrategy ?? "unknown";
    const peakRss = processSamples.reduce((peak, sample) => Math.max(peak, sample.rssKb), 0);
    const lastProcess = processSamples[processSamples.length - 1];

    const markdown = [
      `# ${plan.title}`,
      "",
      `- strategy: **${strategy}**`,
      `- duration: ${(plan.durationMs / 1000).toFixed(1)}s`,
      `- read-path throughput: **${formatThroughput(totalRead, plan.durationMs)}**`,
      `- UI drain throughput: **${formatThroughput(totalDrained, plan.durationMs)}**`,
      `- total drop: **${formatBytes(totalDropped)}**`,
      `- peak RSS: **${peakRss} KB**`,
      `- reader threads: **${lastProcess?.readerThreads ?? 0}**`,
      "",
      "## Sessions",
      "",
      ...sessionViews.flatMap((session) => [
        `### ${session.label}`,
        `- throughput(read): ${formatThroughput(session.totalReadBytes, plan.durationMs)}`,
        `- throughput(drain): ${formatThroughput(session.totalDrainedBytes, plan.durationMs)}`,
        `- queue depth avg/max: ${session.avgQueueDepth.toFixed(2)} / ${session.maxQueueDepth}`,
        `- drop: ${session.droppedChunks} chunks / ${formatBytes(session.droppedBytes)}`,
        `- read calls: ${session.readCalls} · avgReadBytes=${session.avgReadBytes.toFixed(2)} · readSyscall=${session.avgReadSyscallUs.toFixed(2)}µs · enqueue=${session.avgEnqueueUs.toFixed(2)}µs`,
        `- invoke latency p50/p99: ${session.invokeP50Ms.toFixed(2)} / ${session.invokeP99Ms.toFixed(2)} ms`,
        `- preview:`,
        "```text",
        session.preview || "(empty)",
        "```",
        "",
      ]),
      "## Notes",
      "",
      ...plan.notes.map((note) => `- ${note}`),
      "",
    ].join("\n");

    const json = JSON.stringify(
      {
        strategy,
        plan,
        durationMs: plan.durationMs,
        totals: {
          totalReadBytes: totalRead,
          totalDrainedBytes: totalDrained,
          totalDroppedBytes: totalDropped,
          peakRssKb: peakRss,
        },
        sessions: sessionViews.map(serializableSession),
        processSamples,
      },
      null,
      2,
    );

    const outputDir = runtime()?.outputDir ?? null;
    await markProgress("writing artifacts");
    await writeRunArtifacts(outputDir, [
      { name: "summary.md", contents: markdown },
      { name: "summary.json", contents: json },
      { name: "queue-depth.csv", contents: queueSamplesToCsv(queueSamples) },
      { name: "rss-over-time.csv", contents: processSamplesToCsv(processSamples) },
      { name: "frontend-drain.csv", contents: drainSamplesToCsv(drainSamples) },
    ]);

    setSummary(markdown);
    await markProgress("summary ready");
    appendLog(`scenario done → ${plan.title}`);
    await markProgress("close sessions");
    await closeAllSessions();
    setRunning(false);
    await markProgress("done");
  };

  const runCorrectnessScenario = async () => {
    await hardReset();
    setRunning(true);
    setCurrentScenario(`C · correctness · ${runtime()?.strategy ?? "unknown"}`);

    const outputDir = runtime()?.outputDir ?? "/tmp";
    const resizeFile = `${outputDir}/resize-winch.txt`;
    const beforeStats = await sampleProcessStats();
    const resizeSessionView = await spawnSession({
      label: "resize-check",
      command: `rm -f ${resizeFile}; printf 'READY\\n'; while true; do stty size > ${resizeFile}; sleep 0.2; done`,
      cols: 80,
      rows: 24,
      queueCapacity: 64,
    });

    setSessions([
      {
        id: resizeSessionView.id,
        label: resizeSessionView.label,
        command: resizeSessionView.command,
        wrapper: new XTermWrapper(),
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
        exitStatus: resizeSessionView.exitStatus,
        readerStrategy: resizeSessionView.readerStrategy,
        readCalls: resizeSessionView.readCalls,
        avgReadBytes: resizeSessionView.avgReadBytes,
        avgReadSyscallUs: resizeSessionView.avgReadSyscallUs,
        avgEnqueueUs: resizeSessionView.avgEnqueueUs,
        invokeP50Ms: 0,
        invokeP99Ms: 0,
      },
    ]);

    await sleep(1200);
    await resizeSession({ sessionId: resizeSessionView.id, cols: 100, rows: 40 });
    await sleep(1500);

    let resizeResult = "<missing>";
    try {
      resizeResult = await readArtifact({ path: resizeFile });
    } catch (error) {
      appendLog(`readArtifact(${resizeFile}) failed: ${String(error)}`);
    }

    await closeAllSessions();
    const afterStats = await sampleProcessStats();

    const markdown = [
      "# C · correctness",
      "",
      `- strategy: **${runtime()?.strategy ?? "unknown"}**`,
      `- resize result: \`${resizeResult.trim() || "<missing>"}\``,
      `- fd delta: ${afterStats.fdCount - beforeStats.fdCount}`,
      `- rss before/after: ${beforeStats.rssKb} / ${afterStats.rssKb} KB`,
      "",
      "## Checks",
      "",
      `- SIGWINCH: ${resizeResult.trim() === "40 100" ? "PASS" : "CHECK"}`,
      `- cleanup(fd delta): ${afterStats.fdCount - beforeStats.fdCount}`,
      "",
    ].join("\n");

    await writeRunArtifacts(outputDir, [
      { name: "correctness-summary.md", contents: markdown },
      { name: "correctness-summary.json", contents: JSON.stringify({ resizeResult, beforeStats, afterStats }, null, 2) },
    ]);

    setSummary(markdown);
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
        await sleep(300);
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
    appendLog(`runtime config → scenario=${config.scenario ?? "<none>"} strategy=${config.strategy}`);
    if (config.scenario) {
      void runScenario(config.scenario as ScenarioName);
    }
  });

  onCleanup(() => {
    void hardReset();
  });

  const aggregate = createMemo(() => {
    const items = sessions();
    return {
      sessionCount: items.length,
      totalRead: items.reduce((sum, session) => sum + session.totalReadBytes, 0),
      totalDrained: items.reduce((sum, session) => sum + session.totalDrainedBytes, 0),
      totalDropped: items.reduce((sum, session) => sum + session.droppedBytes, 0),
    };
  });

  return (
    <div class="app-shell">
      <aside class="sidebar">
        <div>
          <h1>SPIKE-05.5 · visible throughput 对照</h1>
          <p class="muted">当前策略：<strong>{runtime()?.strategy ?? "<loading>"}</strong></p>
          <small>scenario=<code>{runtime()?.scenario ?? "manual"}</code></small>
        </div>

        <section class="card">
          <h2>Quick Actions</h2>
          <div class="button-grid">
            <button class="primary" disabled={running()} onClick={() => void runScenario("single-yes")}>A · 单 Tab yes</button>
            <button disabled={running()} onClick={() => void runScenario("four-yes")}>A · 4 Tab yes</button>
            <button disabled={running()} onClick={() => void runScenario("interactive-tui")}>A · synthetic TUI</button>
            <button disabled={running()} onClick={() => void runScenario("correctness")}>C · correctness</button>
            <button class="danger" disabled={running()} onClick={() => void hardReset()}>Reset</button>
          </div>
        </section>

        <section class="card kv">
          <h3>Aggregate</h3>
          <div><span>sessions</span><strong>{aggregate().sessionCount}</strong></div>
          <div><span>read</span><strong>{formatBytes(aggregate().totalRead)}</strong></div>
          <div><span>drained</span><strong>{formatBytes(aggregate().totalDrained)}</strong></div>
          <div><span>dropped</span><strong>{formatBytes(aggregate().totalDropped)}</strong></div>
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
            <span class="pill">read {formatBytes(aggregate().totalRead)}</span>
            <span class="pill">drain {formatBytes(aggregate().totalDrained)}</span>
            <span class="pill danger">drop {formatBytes(aggregate().totalDropped)}</span>
          </div>
        </header>

        <section class="workspace">
          <div class="tabs">
            <For each={sessions()}>
              {(session, index) => (
                <button classList={{ active: session.active }} onClick={() => setActiveSessionIndex(index())}>
                  {session.label}
                </button>
              )}
            </For>
          </div>

          <For each={sessions()}>
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
                  renderDelayMs: session.renderDelayMs,
                  readCalls: session.readCalls,
                  avgReadBytes: session.avgReadBytes,
                  avgReadSyscallUs: session.avgReadSyscallUs,
                  avgEnqueueUs: session.avgEnqueueUs,
                  invokeP50Ms: session.invokeP50Ms,
                  invokeP99Ms: session.invokeP99Ms,
                  exitStatus: session.exitStatus,
                } satisfies TabMetricsView}
              />
            )}
          </For>

          <Show when={sessions().length === 0}>
            <div class="card" style={{ margin: "60px 20px 20px" }}>
              <h3>等待对照运行</h3>
              <p class="muted">可通过 env 自动执行，也可手动点击上方按钮。</p>
            </div>
          </Show>
        </section>
      </main>
    </div>
  );
}
