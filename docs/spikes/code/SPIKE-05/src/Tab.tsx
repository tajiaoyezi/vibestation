import { createEffect, onCleanup, onMount } from "solid-js";
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
  eventLoopLagMs: number;
  freezes: number;
  preview: string;
  pollEveryMs: number;
  hiddenPollEveryMs: number;
  renderDelayMs: number;
  renderWhenHidden: boolean;
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

  onCleanup(() => {
    // wrapper 生命周期由 App 管理
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
          <span class="pill danger">drop {props.metrics.droppedChunks}</span>
          <span class="pill">lag {props.metrics.eventLoopLagMs.toFixed(1)}ms</span>
          <span class="pill">freeze {props.metrics.freezes}</span>
        </div>
      </header>

      <div ref={hostRef} class="terminal-host" data-session-id={props.id} />

      <footer class="tab-footer">
        <dl>
          <div>
            <dt>read</dt>
            <dd>{formatBytes(props.metrics.totalReadBytes)}</dd>
          </div>
          <div>
            <dt>drained</dt>
            <dd>{formatBytes(props.metrics.totalDrainedBytes)}</dd>
          </div>
          <div>
            <dt>queued</dt>
            <dd>{formatBytes(props.metrics.queuedBytes)}</dd>
          </div>
          <div>
            <dt>drop</dt>
            <dd>{formatBytes(props.metrics.droppedBytes)}</dd>
          </div>
          <div>
            <dt>poll(active/hidden)</dt>
            <dd>{props.metrics.pollEveryMs} / {props.metrics.hiddenPollEveryMs} ms</dd>
          </div>
          <div>
            <dt>render delay</dt>
            <dd>{props.metrics.renderDelayMs} ms</dd>
          </div>
          <div>
            <dt>hidden strategy</dt>
            <dd>{props.metrics.renderWhenHidden ? "render" : "drop"}</dd>
          </div>
          <div>
            <dt>exit</dt>
            <dd>{props.metrics.exitStatus ?? "running"}</dd>
          </div>
        </dl>
        <pre class="preview">{props.metrics.preview || "(waiting for output...)"}</pre>
      </footer>
    </section>
  );
}
