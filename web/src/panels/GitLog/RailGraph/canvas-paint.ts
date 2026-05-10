import type {
  ConfiguredCanvas,
  RailEdgeGeo,
  RailGeometryLayout,
  RailNodeGeo,
  RailPaintOptions,
  RailTipGeo,
} from "./types-canvas";

const FALLBACK_RAIL_COLOR = "oklch(0.62 0.14 230)";
const FALLBACK_TEXT_1 = "oklch(0.96 0.005 255)";
const FALLBACK_TEXT_2 = "oklch(0.74 0.012 255)";
const FALLBACK_BG_2 = "oklch(0.22 0.012 255)";
const FALLBACK_LINE = "oklch(0.28 0.012 255)";

type RailCanvasBitmap = HTMLCanvasElement | OffscreenCanvas;

export function clampRailDpr(dpr: number): number {
  if (!Number.isFinite(dpr) || dpr <= 0) return 1;
  return Math.min(Math.max(dpr, 1), 2);
}

export function configureCanvasBitmapForDpr(
  canvas: RailCanvasBitmap,
  cssWidth: number,
  cssHeight: number,
  dpr: number,
): ConfiguredCanvas | null {
  const ctx = canvas.getContext("2d") as CanvasRenderingContext2D | null;
  if (!ctx) return null;
  const resolvedDpr = clampRailDpr(dpr);
  const width = Math.max(1, Math.floor(cssWidth * resolvedDpr));
  const height = Math.max(1, Math.floor(cssHeight * resolvedDpr));

  canvas.width = width;
  canvas.height = height;
  ctx.setTransform?.(1, 0, 0, 1, 0, 0);
  ctx.scale(resolvedDpr, resolvedDpr);

  return { ctx, dpr: resolvedDpr, width, height };
}

export function configureCanvasForDpr(
  canvas: HTMLCanvasElement,
  cssWidth: number,
  cssHeight: number,
  dpr: number,
): ConfiguredCanvas | null {
  canvas.style.width = `${cssWidth}px`;
  canvas.style.height = `${cssHeight}px`;
  return configureCanvasBitmapForDpr(canvas, cssWidth, cssHeight, dpr);
}

export function copyRailBackBufferToCanvas(
  ctx: CanvasRenderingContext2D,
  backBuffer: CanvasImageSource & { width: number; height: number },
  cssWidth: number,
  cssHeight: number,
): void {
  ctx.clearRect(0, 0, cssWidth, cssHeight);
  ctx.drawImage(
    backBuffer,
    0,
    0,
    backBuffer.width,
    backBuffer.height,
    0,
    0,
    cssWidth,
    cssHeight,
  );
}

function computedStyle(root?: Element): CSSStyleDeclaration | null {
  if (typeof getComputedStyle !== "function") return null;
  if (root) return getComputedStyle(root);
  if (typeof document === "undefined") return null;
  return getComputedStyle(document.documentElement);
}

function cssVar(
  style: CSSStyleDeclaration | null,
  name: string,
  fallback: string,
): string {
  const value = style?.getPropertyValue(name).trim();
  return value && value.length > 0 ? value : fallback;
}

function colorForKey(
  style: CSSStyleDeclaration | null,
  colorKey: string,
): string {
  const index = colorKey.replace("color-", "");
  return cssVar(style, `--vs-rail-color-${index}`, FALLBACK_RAIL_COLOR);
}

function withRoundedRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
): void {
  if (typeof ctx.roundRect === "function") {
    ctx.roundRect(x, y, width, height, radius);
    return;
  }

  const r = Math.min(radius, width / 2, height / 2);
  ctx.moveTo(x + r, y);
  ctx.lineTo(x + width - r, y);
  ctx.quadraticCurveTo(x + width, y, x + width, y + r);
  ctx.lineTo(x + width, y + height - r);
  ctx.quadraticCurveTo(x + width, y + height, x + width - r, y + height);
  ctx.lineTo(x + r, y + height);
  ctx.quadraticCurveTo(x, y + height, x, y + height - r);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
}

function paintEdge(
  ctx: CanvasRenderingContext2D,
  edge: RailEdgeGeo,
  color: string,
): void {
  ctx.beginPath();
  ctx.strokeStyle = color;
  ctx.lineWidth = 1.5;
  ctx.lineCap = "round";
  ctx.moveTo(edge.fromX, edge.fromY);
  if (edge.pathKind === "bezier") {
    ctx.bezierCurveTo(
      edge.fromX,
      edge.fromY + edge.controlOffsetY,
      edge.toX,
      edge.toY - edge.controlOffsetY,
      edge.toX,
      edge.toY,
    );
  } else {
    ctx.lineTo(edge.toX, edge.toY);
  }
  ctx.stroke();
}

function paintNode(
  ctx: CanvasRenderingContext2D,
  node: RailNodeGeo,
  color: string,
): void {
  ctx.save();
  ctx.fillStyle = color;
  ctx.strokeStyle = color;
  ctx.lineWidth = 1.5;
  ctx.beginPath();

  if (node.kind === "merge") {
    ctx.translate(node.x, node.y);
    ctx.rotate(Math.PI / 4);
    ctx.rect(-node.radius / 2, -node.radius / 2, node.radius, node.radius);
  } else if (node.kind === "fork") {
    ctx.rect(
      node.x - node.radius / 2,
      node.y - node.radius / 2,
      node.radius,
      node.radius,
    );
  } else {
    ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
  }

  ctx.fill();
  if (node.kind === "head") {
    ctx.beginPath();
    ctx.lineWidth = node.ringWidth;
    ctx.arc(node.x, node.y, node.radius + 4, 0, Math.PI * 2);
    ctx.stroke();
  }
  ctx.restore();
}

function paintTip(
  ctx: CanvasRenderingContext2D,
  tip: RailTipGeo,
  style: CSSStyleDeclaration | null,
  color: string,
): void {
  const x = tip.x;
  const y = tip.y - tip.height / 2;
  const textY = tip.y;
  const textX = tip.x + 8;
  const text1 = cssVar(style, "--text-1", FALLBACK_TEXT_1);
  const text2 = cssVar(style, "--text-2", FALLBACK_TEXT_2);
  const bg2 = cssVar(style, "--bg-2", FALLBACK_BG_2);
  const fontMono = cssVar(style, "--font-mono", "ui-monospace, monospace");

  ctx.save();
  ctx.beginPath();
  withRoundedRect(ctx, x, y, tip.width, tip.height, tip.radius);

  if (tip.kind === "remote") {
    ctx.fillStyle = color;
    ctx.globalAlpha = 0.18;
    ctx.fill();
    ctx.globalAlpha = 1;
    ctx.strokeStyle = color;
    ctx.lineWidth = 1;
    ctx.stroke();
    ctx.fillStyle = text2;
  } else if (tip.kind === "tag") {
    ctx.fillStyle = bg2;
    ctx.fill();
    ctx.strokeStyle = color;
    ctx.lineWidth = 1;
    ctx.stroke();
    ctx.fillStyle = text1;
  } else {
    ctx.fillStyle = color;
    ctx.fill();
    ctx.fillStyle = text1;
  }

  ctx.font = `10px ${fontMono}`;
  ctx.textBaseline = "middle";
  ctx.textAlign = "left";
  ctx.fillText(tip.label, textX, textY);
  ctx.restore();
}

export function paintRailGraphFrame(
  ctx: CanvasRenderingContext2D,
  layout: RailGeometryLayout,
  options: RailPaintOptions,
): void {
  const style = computedStyle(options.root);
  ctx.save();
  ctx.clearRect(0, 0, options.width, options.height);
  ctx.beginPath();
  ctx.rect(0, 0, options.width, options.height);
  ctx.clip();

  for (const edge of layout.edges) {
    paintEdge(ctx, edge, colorForKey(style, edge.colorKey));
  }

  for (const node of layout.nodes) {
    paintNode(ctx, node, colorForKey(style, node.colorKey));
  }

  for (const tip of layout.tips) {
    paintTip(ctx, tip, style, colorForKey(style, tip.colorKey));
  }

  ctx.restore();
}

export function paintRailGraphOverlay(
  ctx: CanvasRenderingContext2D,
  layout: RailGeometryLayout,
  options: RailPaintOptions,
): void {
  const style = computedStyle(options.root);
  ctx.save();
  ctx.clearRect(0, 0, options.width, options.height);

  if (options.selectedRowIndex == null) {
    ctx.restore();
    return;
  }

  const selectedNode = layout.nodes.find(
    (node) => node.rowIndex === options.selectedRowIndex,
  );
  if (!selectedNode) {
    ctx.restore();
    return;
  }

  ctx.beginPath();
  ctx.strokeStyle = colorForKey(style, selectedNode.colorKey);
  ctx.lineWidth = 2;
  ctx.globalAlpha = 0.55;
  ctx.arc(
    selectedNode.x,
    selectedNode.y,
    selectedNode.radius + 5,
    0,
    Math.PI * 2,
  );
  ctx.stroke();
  ctx.globalAlpha = 1;
  ctx.restore();
}

export function paintRailGraphFallback(
  ctx: CanvasRenderingContext2D,
  options: RailPaintOptions,
): void {
  const style = computedStyle(options.root);
  ctx.save();
  ctx.clearRect(0, 0, options.width, options.height);
  ctx.fillStyle = cssVar(style, "--line-soft", FALLBACK_LINE);
  ctx.fillRect(0, 0, Math.max(1, options.width), 1);
  ctx.restore();
}
