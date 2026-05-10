import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import type { RailGraphInputCommit, RailLaneAssignment } from "./types";
import type { RailPathHighlight } from "./types-canvas";
import { DEFAULT_RAIL_VIEW_OPTIONS } from "./types-canvas";
import {
  clampRailDpr,
  configureCanvasBitmapForDpr,
  configureCanvasForDpr,
  copyRailBackBufferToCanvas,
  paintRailGraphFallback,
  paintRailGraphFrame,
  paintRailGraphOverlay,
} from "./canvas-paint";
import { paintDebugGrid } from "./debug-grid";
import { computeRailGeometry } from "./geometry";
import {
  hitTestRailGeometry,
  reduceRailPointerHighlight,
} from "./interactions";
import {
  createRailFrameScheduler,
  createRailPerformanceSampler,
} from "./raf-scheduler";
import {
  buildRailRowMetrics,
  computeVisibleRangeFromMetrics,
  filterRailGeometryToVisibleRange,
} from "./RailGraphVirtualizer";
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
  viewportHeight?: number;
  overscanRows?: number;
}

function clampRailWidth(width: number): number {
  if (!Number.isFinite(width)) return 120;
  return Math.max(120, Math.min(180, Math.round(width)));
}

function deviceDpr(fallback: number): number {
  if (typeof window === "undefined") return clampRailDpr(fallback);
  return clampRailDpr(fallback || window.devicePixelRatio || 1);
}

type RailBackBuffer = HTMLCanvasElement | OffscreenCanvas;

function createBackBuffer(): RailBackBuffer | null {
  if (typeof OffscreenCanvas !== "undefined") {
    return new OffscreenCanvas(1, 1);
  }
  if (typeof document !== "undefined") {
    return document.createElement("canvas");
  }
  return null;
}

export function RailGraphCanvas(props: RailGraphCanvasProps) {
  let rootEl: HTMLDivElement | undefined;
  let mainCanvas: HTMLCanvasElement | undefined;
  let overlayCanvas: HTMLCanvasElement | undefined;
  let backBuffer: RailBackBuffer | null = null;
  const scheduler = createRailFrameScheduler();
  const performanceSampler = createRailPerformanceSampler({
    sampleEvery: 100,
  });
  const [observedWidth, setObservedWidth] = createSignal<number | null>(null);
  const [themeRevision, setThemeRevision] = createSignal(0);
  const [activeHighlight, setActiveHighlight] =
    createSignal<RailPathHighlight | null>(null);

  const cssWidth = createMemo(() =>
    clampRailWidth(observedWidth() ?? props.width),
  );
  const layout = createMemo(() =>
    computeRailGeometry(props.input, props.assignments, props.rowHeights, {
      width: cssWidth(),
      tipStartX: Math.min(92, Math.max(68, cssWidth() - 88)),
    }),
  );
  const rowMetrics = createMemo(() =>
    buildRailRowMetrics(
      props.rowHeights,
      props.input.length,
      DEFAULT_RAIL_VIEW_OPTIONS.rowFallbackHeight,
    ),
  );
  const viewportHeight = createMemo(() => {
    const measured = props.viewportHeight;
    if (Number.isFinite(measured) && measured != null && measured > 0) {
      return measured;
    }
    return rootEl?.parentElement?.clientHeight ?? layout().height;
  });
  const visibleRange = createMemo(() =>
    computeVisibleRangeFromMetrics(
      props.scrollTop,
      viewportHeight(),
      rowMetrics(),
      props.overscanRows ?? 100,
    ),
  );
  const visibleLayout = createMemo(() =>
    filterRailGeometryToVisibleRange(layout(), visibleRange()),
  );
  const cssHeight = createMemo(() => Math.max(1, visibleLayout().height));

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

  onCleanup(() => scheduler.dispose());

  createEffect(() => {
    const nextLayout = visibleLayout();
    const nextWidth = cssWidth();
    const nextHeight = cssHeight();
    const nextDpr = deviceDpr(props.dpr);
    const selectedRowIndex = props.selectedRowIndex;
    const theme = props.theme;
    const scrollTop = props.scrollTop;
    const highlight = activeHighlight();
    const revision = themeRevision();
    void scrollTop;
    void revision;

    scheduler.invalidate(() => {
      if (!mainCanvas || !overlayCanvas) return;

      const finishSample = performanceSampler.startFrame();
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

      if (!main || !overlay) {
        finishSample();
        return;
      }

      backBuffer ??= createBackBuffer();
      const back = backBuffer
        ? configureCanvasBitmapForDpr(
            backBuffer,
            nextWidth,
            nextHeight,
            nextDpr,
          )
        : null;
      const paintCtx = back?.ctx ?? main.ctx;

      if (nextLayout.nodes.length === 0) {
        paintRailGraphFallback(paintCtx, {
          theme,
          width: nextWidth,
          height: nextHeight,
          root: rootEl,
        });
      } else {
        paintRailGraphFrame(paintCtx, nextLayout, {
          theme,
          width: nextWidth,
          height: nextHeight,
          root: rootEl,
        });
        paintDebugGrid(paintCtx, nextLayout);
      }

      if (back && backBuffer) {
        copyRailBackBufferToCanvas(
          main.ctx,
          backBuffer as CanvasImageSource & { width: number; height: number },
          nextWidth,
          nextHeight,
        );
      }

      paintRailGraphOverlay(overlay.ctx, nextLayout, {
        theme,
        width: nextWidth,
        height: nextHeight,
        selectedRowIndex,
        highlight,
        root: rootEl,
      });
      finishSample();
    });
  });

  const pointerTarget = (event: PointerEvent) => {
    if (!rootEl) return null;
    const rect = rootEl.getBoundingClientRect();
    return hitTestRailGeometry(
      event.clientX - rect.left,
      event.clientY - rect.top,
      visibleLayout(),
    );
  };

  const handlePointerMove = (event: PointerEvent) => {
    if (event.pointerType === "touch") return;
    const target = pointerTarget(event);
    setActiveHighlight((current) =>
      reduceRailPointerHighlight(current, {
        type: "hover",
        target,
        layout: visibleLayout(),
      }),
    );
  };

  const handlePointerLeave = () => {
    setActiveHighlight((current) =>
      reduceRailPointerHighlight(current, { type: "leave" }),
    );
  };

  const handlePointerDown = (event: PointerEvent) => {
    if (event.pointerType !== "touch") return;
    event.preventDefault();
    const target = pointerTarget(event);
    setActiveHighlight((current) =>
      reduceRailPointerHighlight(current, {
        type: "tap",
        target,
        layout: visibleLayout(),
      }),
    );
  };

  return (
    <div
      ref={rootEl}
      class={styles.railGraph}
      style={{
        width: `${cssWidth()}px`,
        height: `${cssHeight()}px`,
        transform: `translateY(${visibleRange().startY}px)`,
      }}
      data-rail-theme={props.theme}
      data-rail-interactive="true"
      aria-hidden="true"
      onPointerMove={handlePointerMove}
      onPointerLeave={handlePointerLeave}
      onPointerDown={handlePointerDown}
    >
      <canvas ref={mainCanvas} class={styles.canvas} />
      <canvas
        ref={overlayCanvas}
        class={`${styles.canvas} ${styles.overlay}`}
      />
    </div>
  );
}
