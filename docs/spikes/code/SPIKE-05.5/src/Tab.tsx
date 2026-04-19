import { createEffect, onMount } from "solid-js";
import { XTermWrapper } from "./xterm-wrapper";

export interface TabMetricsView {
  queueDepth: number;
  queuedBytes: number;
  avgQueueDepth: number;
  maxQueueDepth: number;
  totalReadBytes: number;
  totalDrainedBytes: number;
  droppedChunks: number;
  droppedBytes: number;
  renderDelayMs: number;
  readCalls: number;
  avgReadBytes: number;
  avgReadSyscallUs: number;
  avgEnqueueUs: number;
  invokeP50Ms: number;
  invokeP99Ms: number;
  exitStatus?: string | null;
}

interface TabProps {
  id: string;
  label: string;
  command: string;
  active: boolean;
  wrapper: XTermWrapper;
  metrics: TabMetricsView;
}

const formatBytes = (value: number): string => {
  if (value >= 1024 * 1024) return `${(value / (1024 * 1024)).toFixed(2)} MB`;
  if (value >= 1024) return `${(value / 1024).toFixed(2)} KB`;
  return `${value} B`;
};

export default function Tab(props: TabProps) {
  let hostRef: HTMLDivElement | undefined;

  onMount(() => {
    if (hostRef) props.wrapper.attach(hostRef);
  });

  createEffect(() => {
    if (props.active) props.wrapper.fit();
  });

  return (
    <section classList={{ "tab-panel": true, active: props.active }} aria-hidden={!props.active}>
      <header class="tab-metrics">
        <div>
          <strong>{props.label}</strong>
          <span class="tab-command">{props.command}</span>
        </div>
        <div class="pill-row">
          <span class="pill">queue {props.metrics.queueDepth}</span>
          <span class="pill">avg {props.metrics.avgQueueDepth.toFixed(1)}</span>
          <span class="pill">max {props.metrics.maxQueueDepth}</span>
          <span class="pill">reads {props.metrics.readCalls}</span>
          <span class="pill danger">drop {props.metrics.droppedChunks}</span>
          <span class="pill">invoke p99 {props.metrics.invokeP99Ms.toFixed(2)}ms</span>
        </div>
      </header>

      <div ref={hostRef} class="terminal-host" data-session-id={props.id} />

      <footer class="tab-footer">
        <dl>
          <div><dt>read</dt><dd>{formatBytes(props.metrics.totalReadBytes)}</dd></div>
          <div><dt>drained</dt><dd>{formatBytes(props.metrics.totalDrainedBytes)}</dd></div>
          <div><dt>drop</dt><dd>{formatBytes(props.metrics.droppedBytes)}</dd></div>
          <div><dt>avg read bytes</dt><dd>{props.metrics.avgReadBytes.toFixed(1)}</dd></div>
          <div><dt>read syscall</dt><dd>{props.metrics.avgReadSyscallUs.toFixed(2)} µs</dd></div>
          <div><dt>enqueue</dt><dd>{props.metrics.avgEnqueueUs.toFixed(2)} µs</dd></div>
          <div><dt>render delay</dt><dd>{props.metrics.renderDelayMs} ms</dd></div>
          <div><dt>invoke p50/p99</dt><dd>{props.metrics.invokeP50Ms.toFixed(2)} / {props.metrics.invokeP99Ms.toFixed(2)} ms</dd></div>
          <div><dt>exit</dt><dd>{props.metrics.exitStatus ?? "running"}</dd></div>
        </dl>
      </footer>
    </section>
  );
}
