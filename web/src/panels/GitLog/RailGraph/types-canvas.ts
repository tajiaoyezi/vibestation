export type RailTheme = "light" | "dark";

export type RailNodeKind = "normal" | "merge" | "fork" | "head";

export type RailEdgePathKind = "line" | "bezier";

export type RailTipKind = "local" | "remote" | "tag";

export interface RailViewOptions {
  width?: number;
  lanePaddingX?: number;
  laneGap?: number;
  rowFallbackHeight?: number;
  tipStartX?: number;
  tipGap?: number;
  tipHeight?: number;
  tipPaddingX?: number;
  maxTipWidth?: number;
}

export interface RailNodeGeo {
  oid: string;
  rowIndex: number;
  laneIndex: number;
  colorKey: string;
  x: number;
  y: number;
  kind: RailNodeKind;
  radius: number;
  ringWidth: number;
  parentCount: number;
  childCount: number;
}

export interface RailEdgeGeo {
  fromOid: string;
  toOid: string;
  fromRowIndex: number;
  toRowIndex: number;
  fromLaneIndex: number;
  toLaneIndex: number;
  colorKey: string;
  fromX: number;
  fromY: number;
  toX: number;
  toY: number;
  pathKind: RailEdgePathKind;
  controlOffsetY: number;
}

export interface RailTipGeo {
  oid: string;
  rowIndex: number;
  colorKey: string;
  kind: RailTipKind;
  label: string;
  x: number;
  y: number;
  width: number;
  height: number;
  radius: number;
}

export interface RailGeometryLayout {
  width: number;
  height: number;
  laneCount: number;
  nodes: RailNodeGeo[];
  edges: RailEdgeGeo[];
  tips: RailTipGeo[];
}

export interface RailPaintOptions {
  theme: RailTheme;
  width: number;
  height: number;
  selectedRowIndex?: number | null;
  root?: Element;
}

export interface ConfiguredCanvas {
  ctx: CanvasRenderingContext2D;
  dpr: number;
  width: number;
  height: number;
}

export const DEFAULT_RAIL_VIEW_OPTIONS = {
  lanePaddingX: 16,
  laneGap: 16,
  rowFallbackHeight: 32,
  tipStartX: 72,
  tipGap: 6,
  tipHeight: 16,
  tipPaddingX: 8,
  maxTipWidth: 96,
} satisfies Required<Omit<RailViewOptions, "width">>;
