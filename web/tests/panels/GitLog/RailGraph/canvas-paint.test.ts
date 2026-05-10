import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  RailGeometryLayout,
  RailNodeGeo,
  RailTipGeo,
} from "../../../../src/panels/GitLog/RailGraph/types-canvas";
import {
  copyRailBackBufferToCanvas,
  configureCanvasBitmapForDpr,
  configureCanvasForDpr,
  paintRailGraphFrame,
  paintRailGraphOverlay,
} from "../../../../src/panels/GitLog/RailGraph/canvas-paint";

type Call = [name: string, ...args: unknown[]];

class FakeCanvasContext {
  calls: Call[] = [];
  fillStyle = "";
  strokeStyle = "";
  lineWidth = 1;
  globalAlpha = 1;
  font = "";
  textBaseline = "";
  textAlign = "";

  save() {
    this.calls.push(["save"]);
  }

  restore() {
    this.calls.push(["restore"]);
  }

  scale(x: number, y: number) {
    this.calls.push(["scale", x, y]);
  }

  clearRect(x: number, y: number, width: number, height: number) {
    this.calls.push(["clearRect", x, y, width, height]);
  }

  beginPath() {
    this.calls.push(["beginPath"]);
  }

  moveTo(x: number, y: number) {
    this.calls.push(["moveTo", x, y]);
  }

  lineTo(x: number, y: number) {
    this.calls.push(["lineTo", x, y]);
  }

  bezierCurveTo(
    cp1x: number,
    cp1y: number,
    cp2x: number,
    cp2y: number,
    x: number,
    y: number,
  ) {
    this.calls.push(["bezierCurveTo", cp1x, cp1y, cp2x, cp2y, x, y]);
  }

  arc(x: number, y: number, radius: number) {
    this.calls.push(["arc", x, y, radius]);
  }

  rect(x: number, y: number, width: number, height: number) {
    this.calls.push(["rect", x, y, width, height]);
  }

  clip() {
    this.calls.push(["clip"]);
  }

  roundRect(
    x: number,
    y: number,
    width: number,
    height: number,
    radius: number,
  ) {
    this.calls.push(["roundRect", x, y, width, height, radius]);
  }

  translate(x: number, y: number) {
    this.calls.push(["translate", x, y]);
  }

  rotate(angle: number) {
    this.calls.push(["rotate", angle]);
  }

  fill() {
    this.calls.push(["fill", this.fillStyle, this.globalAlpha]);
  }

  stroke() {
    this.calls.push(["stroke", this.strokeStyle, this.lineWidth]);
  }

  fillText(text: string, x: number, y: number) {
    this.calls.push(["fillText", text, x, y, this.fillStyle]);
  }

  drawImage(
    source: unknown,
    sx: number,
    sy: number,
    sw: number,
    sh: number,
    dx: number,
    dy: number,
    dw: number,
    dh: number,
  ) {
    this.calls.push(["drawImage", source, sx, sy, sw, sh, dx, dy, dw, dh]);
  }

  setTransform(a: number, b: number, c: number, d: number, e: number, f: number) {
    this.calls.push(["setTransform", a, b, c, d, e, f]);
  }
}

function installComputedStyleStub() {
  return vi.spyOn(globalThis, "getComputedStyle").mockReturnValue({
    getPropertyValue(name: string) {
      if (name === "--vs-rail-color-3") return "oklch(0.52 0.17 56)";
      if (name === "--text-1") return "oklch(0.22 0.02 255)";
      if (name === "--text-2") return "oklch(0.42 0.02 255)";
      if (name === "--bg-1") return "oklch(0.965 0.004 255)";
      if (name === "--bg-2") return "oklch(0.935 0.006 255)";
      if (name === "--line-soft") return "oklch(0.92 0.006 255)";
      return "";
    },
  } as CSSStyleDeclaration);
}

function makeNode(partial: Partial<RailNodeGeo> = {}): RailNodeGeo {
  return {
    oid: "node",
    rowIndex: 0,
    laneIndex: 0,
    colorKey: "color-3",
    x: 16,
    y: 12,
    kind: "normal",
    radius: 6,
    ringWidth: 0,
    parentCount: 1,
    childCount: 1,
    ...partial,
  };
}

function makeTip(partial: Partial<RailTipGeo> = {}): RailTipGeo {
  return {
    oid: "node",
    rowIndex: 0,
    colorKey: "color-3",
    kind: "local",
    label: "main",
    x: 72,
    y: 12,
    width: 44,
    height: 16,
    radius: 4,
    ...partial,
  };
}

function makeLayout(): RailGeometryLayout {
  return {
    width: 140,
    height: 48,
    laneCount: 1,
    nodes: [makeNode()],
    edges: [
      {
        fromOid: "node",
        toOid: "parent",
        fromRowIndex: 0,
        toRowIndex: 1,
        fromLaneIndex: 0,
        toLaneIndex: 0,
        colorKey: "color-3",
        fromX: 16,
        fromY: 12,
        toX: 16,
        toY: 36,
        pathKind: "line",
        controlOffsetY: 12,
      },
    ],
    tips: [],
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("configureCanvasForDpr", () => {
  it("clamps DPR to 2 and scales the backing store once", () => {
    const ctx = new FakeCanvasContext();
    const canvas = {
      width: 0,
      height: 0,
      style: { width: "", height: "" },
      getContext: () => ctx,
    };

    const result = configureCanvasForDpr(
      canvas as unknown as HTMLCanvasElement,
      120,
      40,
      3,
    );

    expect(result?.dpr).toBe(2);
    expect(canvas.width).toBe(240);
    expect(canvas.height).toBe(80);
    expect(canvas.style).toEqual({ width: "120px", height: "40px" });
    expect(ctx.calls).toContainEqual(["scale", 2, 2]);
  });
});

describe("configureCanvasBitmapForDpr", () => {
  it("sizes a back-buffer canvas without touching CSS style", () => {
    const ctx = new FakeCanvasContext();
    const canvas = {
      width: 0,
      height: 0,
      getContext: () => ctx,
    };

    const result = configureCanvasBitmapForDpr(
      canvas as unknown as HTMLCanvasElement,
      90,
      30,
      2.5,
    );

    expect(result?.dpr).toBe(2);
    expect(canvas.width).toBe(180);
    expect(canvas.height).toBe(60);
    expect(ctx.calls).toContainEqual(["scale", 2, 2]);
  });

  it("copies the rendered back buffer to the visible canvas with one drawImage", () => {
    const ctx = new FakeCanvasContext();
    const backBuffer = { width: 180, height: 60 };

    copyRailBackBufferToCanvas(
      ctx as unknown as CanvasRenderingContext2D,
      backBuffer as unknown as HTMLCanvasElement,
      90,
      30,
    );

    expect(ctx.calls).toContainEqual(["clearRect", 0, 0, 90, 30]);
    expect(ctx.calls.filter((call) => call[0] === "drawImage")).toHaveLength(1);
  });
});

describe("paintRailGraphFrame", () => {
  it("reads rail color tokens before drawing edges and nodes", () => {
    installComputedStyleStub();
    const ctx = new FakeCanvasContext();

    paintRailGraphFrame(
      ctx as unknown as CanvasRenderingContext2D,
      makeLayout(),
      { theme: "light", width: 140, height: 48 },
    );

    expect(ctx.calls[0]).toEqual(["save"]);
    expect(ctx.calls).toContainEqual(["stroke", "oklch(0.52 0.17 56)", 1.5]);
    expect(ctx.calls).toContainEqual(["fill", "oklch(0.52 0.17 56)", 1]);
  });

  it("uses bezier drawing for diagonal merge and fork edges", () => {
    installComputedStyleStub();
    const ctx = new FakeCanvasContext();
    const layout = makeLayout();
    layout.edges[0] = {
      ...layout.edges[0],
      toLaneIndex: 1,
      toX: 32,
      pathKind: "bezier",
    };

    paintRailGraphFrame(ctx as unknown as CanvasRenderingContext2D, layout, {
      theme: "dark",
      width: 140,
      height: 48,
    });

    expect(ctx.calls.some((call) => call[0] === "bezierCurveTo")).toBe(true);
  });

  it("draws local, remote, and tag tips with distinct paint operations", () => {
    installComputedStyleStub();
    const ctx = new FakeCanvasContext();
    const layout = {
      ...makeLayout(),
      edges: [],
      tips: [
        makeTip({ kind: "local", label: "main" }),
        makeTip({ kind: "remote", label: "origin/main", x: 120, width: 76 }),
        makeTip({ kind: "tag", label: "v0.1.0", x: 204, width: 58 }),
      ],
    };

    paintRailGraphFrame(ctx as unknown as CanvasRenderingContext2D, layout, {
      theme: "light",
      width: 280,
      height: 48,
    });

    expect(ctx.calls).toContainEqual([
      "fillText",
      "main",
      80,
      12,
      "oklch(0.22 0.02 255)",
    ]);
    expect(ctx.calls).toContainEqual([
      "fillText",
      "origin/main",
      128,
      12,
      "oklch(0.42 0.02 255)",
    ]);
    expect(ctx.calls).toContainEqual([
      "fillText",
      "v0.1.0",
      212,
      12,
      "oklch(0.22 0.02 255)",
    ]);
    expect(ctx.calls.some((call) => call[0] === "roundRect")).toBe(true);
  });
});

describe("paintRailGraphOverlay", () => {
  it("draws a selected-row halo without repainting unrelated rows", () => {
    installComputedStyleStub();
    const ctx = new FakeCanvasContext();

    paintRailGraphOverlay(
      ctx as unknown as CanvasRenderingContext2D,
      {
        ...makeLayout(),
        nodes: [makeNode({ rowIndex: 0 }), makeNode({ rowIndex: 1, y: 36 })],
      },
      { theme: "light", width: 140, height: 48, selectedRowIndex: 1 },
    );

    expect(ctx.calls).toContainEqual(["clearRect", 0, 0, 140, 48]);
    expect(ctx.calls).toContainEqual(["arc", 16, 36, 11]);
    expect(ctx.calls).not.toContainEqual(["arc", 16, 12, 11]);
  });
});
