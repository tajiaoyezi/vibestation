import type { RailGeometryLayout } from "./types-canvas";

export function paintDebugGrid(
  ctx: CanvasRenderingContext2D,
  layout: RailGeometryLayout,
): void {
  if (!import.meta.env.DEV) return;

  ctx.save();
  ctx.strokeStyle = "oklch(0.78 0.13 230 / 0.18)";
  ctx.lineWidth = 1;

  for (const node of layout.nodes) {
    ctx.beginPath();
    ctx.moveTo(0, node.y);
    ctx.lineTo(layout.width, node.y);
    ctx.stroke();
  }

  ctx.restore();
}
