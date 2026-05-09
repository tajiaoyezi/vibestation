import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import type { RailGraphInputCommit, RailLaneAssignment } from "./types";
import {
  clampRailDpr,
  configureCanvasForDpr,
  paintRailGraphFallback,
  paintRailGraphFrame,
  paintRailGraphOverlay,
} from "./canvas-paint";
import { paintDebugGrid } from "./debug-grid";
import { computeRailGeometry } from "./geometry";
import styles from "./RailGraphCanvas.module.css";

export interface RailGraphCanvasProps {
  input: RailGraphInputCommit[];
  assignments: RailLaneAssignment[];
  rowHeights: number[];
  scrollTop: number;
  selectedRowIndex: number | null;
  theme: "light" | "dark";
  dpr: number;
  width: number;
}

function clampRailWidth(width: number): number {
  if (!Number.isFinite(width)) return 120;
  return Math.max(120, Math.min(180, Math.round(width)));
}

function deviceDpr(fallback: number): number {
  if (typeof window === "undefined") return clampRailDpr(fallback);
  return clampRailDpr(fallback || window.devicePixelRatio || 1);
}

export function RailGraphCanvas(props: RailGraphCanvasProps) {
  let rootEl: HTMLDivElement | undefined;
  let mainCanvas: HTMLCanvasElement | undefined;
  let overlayCanvas: HTMLCanvasElement | undefined;
  const [observedWidth, setObservedWidth] = createSignal<number | null>(null);
  const [themeRevision, setThemeRevision] = createSignal(0);

  const cssWidth = createMemo(() =>
    clampRailWidth(observedWidth() ?? props.width),
  );
  const layout = createMemo(() =>
    computeRailGeometry(props.input, props.assignments, props.rowHeights, {
      width: cssWidth(),
      tipStartX: Math.min(92, Math.max(68, cssWidth() - 88)),
    }),
  );
  const cssHeight = createMemo(() => Math.max(1, layout().height));

  onMount(() => {
    if (typeof ResizeObserver !== "undefined" && rootEl) {
      const observer = new ResizeObserver((entries) => {
        const width = entries[0]?.contentRect.width;
        if (width && Number.isFinite(width)) {
          setObservedWidth(width);
        }
      });
      observer.observe(rootEl);
      onCleanup(() => observer.disconnect());
    }

    if (
      typeof MutationObserver !== "undefined" &&
      typeof document !== "undefined"
    ) {
      const observer = new MutationObserver((records) => {
        if (records.some((record) => record.attributeName === "data-theme")) {
          setThemeRevision((value) => value + 1);
        }
      });
      observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["data-theme"],
      });
      onCleanup(() => observer.disconnect());
    }
  });

  createEffect(() => {
    const nextLayout = layout();
    const nextWidth = cssWidth();
    const nextHeight = cssHeight();
    const nextDpr = deviceDpr(props.dpr);
    const selectedRowIndex = props.selectedRowIndex;
    const theme = props.theme;
    const scrollTop = props.scrollTop;
    const revision = themeRevision();
    void scrollTop;
    void revision;

    if (!mainCanvas || !overlayCanvas) return;

    const main = configureCanvasForDpr(
      mainCanvas,
      nextWidth,
      nextHeight,
      nextDpr,
    );
    const overlay = configureCanvasForDpr(
      overlayCanvas,
      nextWidth,
      nextHeight,
      nextDpr,
    );

    if (!main || !overlay) return;

    if (nextLayout.nodes.length === 0) {
      paintRailGraphFallback(main.ctx, {
        theme,
        width: nextWidth,
        height: nextHeight,
        root: rootEl,
      });
      paintRailGraphOverlay(overlay.ctx, nextLayout, {
        theme,
        width: nextWidth,
        height: nextHeight,
        selectedRowIndex,
        root: rootEl,
      });
      return;
    }

    paintRailGraphFrame(main.ctx, nextLayout, {
      theme,
      width: nextWidth,
      height: nextHeight,
      root: rootEl,
    });
    paintDebugGrid(main.ctx, nextLayout);
    paintRailGraphOverlay(overlay.ctx, nextLayout, {
      theme,
      width: nextWidth,
      height: nextHeight,
      selectedRowIndex,
      root: rootEl,
    });
  });

  return (
    <div
      ref={rootEl}
      class={styles.railGraph}
      style={{
        width: `${cssWidth()}px`,
        height: `${cssHeight()}px`,
      }}
      data-rail-theme={props.theme}
      aria-hidden="true"
    >
      <canvas ref={mainCanvas} class={styles.canvas} />
      <canvas
        ref={overlayCanvas}
        class={`${styles.canvas} ${styles.overlay}`}
      />
    </div>
  );
}
