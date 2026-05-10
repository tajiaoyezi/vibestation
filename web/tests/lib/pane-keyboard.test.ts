// MVP-14 Phase C · 几何相邻算法 + modifier matcher 单测
//
// 覆盖 spec §D.2 / §D.3 / §D.4 / §D.6：
// - 2x2 grid 4 方向
// - H(A, V(B, C)) A↓ no-op · A→ 选 vertical-center 更近者
// - 5 层 nested 边缘 case
// - cross-axis overlap = 0 必须 no-op（不跨非相邻 splitter）
// - macOS / Linux modifier 区分

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  findGeometricNeighbor,
  matchesPaneNavigateModifier,
  arrowKeyToDirection,
  type PaneRect,
} from "../../src/lib/pane-keyboard";

const r = (
  paneId: string,
  left: number,
  top: number,
  right: number,
  bottom: number,
): PaneRect => ({ paneId, rect: { left, top, right, bottom } });

describe("findGeometricNeighbor · 2x2 grid", () => {
  // H(V(A, B), V(C, D)) · 2x2 · 1000x1000 viewport
  // A: top-left  · B: bottom-left
  // C: top-right · D: bottom-right
  const grid: PaneRect[] = [
    r("A", 0, 0, 500, 500),
    r("B", 0, 500, 500, 1000),
    r("C", 500, 0, 1000, 500),
    r("D", 500, 500, 1000, 1000),
  ];

  it("A → right → C", () => {
    expect(findGeometricNeighbor(grid, "A", "right")).toBe("C");
  });

  it("A → down → B", () => {
    expect(findGeometricNeighbor(grid, "A", "down")).toBe("B");
  });

  it("D → left → B (not A · 不跳到对角)", () => {
    // §D.2 测试：从右下按 ← 到左下 · 不跨越 V(A,B) splitter 跳到 A
    expect(findGeometricNeighbor(grid, "D", "left")).toBe("B");
  });

  it("D → up → C", () => {
    expect(findGeometricNeighbor(grid, "D", "up")).toBe("C");
  });

  it("A → left → null (no candidate)", () => {
    expect(findGeometricNeighbor(grid, "A", "left")).toBeNull();
  });

  it("A → up → null (no candidate)", () => {
    expect(findGeometricNeighbor(grid, "A", "up")).toBeNull();
  });
});

describe("findGeometricNeighbor · H(A, V(B, C)) §D.3", () => {
  // A: 左占满 · B: 右上 · C: 右下 · 非对称 V ratio (B 0..400 · C 400..1000)
  const layout: PaneRect[] = [
    r("A", 0, 0, 500, 1000),
    r("B", 500, 0, 1000, 400),
    r("C", 500, 400, 1000, 1000),
  ];

  it("A → down → null (无目标 · A 已经占满左侧到 bottom · 没有更下方的 pane)", () => {
    expect(findGeometricNeighbor(layout, "A", "down")).toBeNull();
  });

  it("A → right · 选 overlap 更大者 · C (overlap 600) > B (overlap 400) → C", () => {
    // §D.2 重叠投影最大优先 · 不是中心距离
    // A overlap B vertical = 400 - 0 = 400
    // A overlap C vertical = 1000 - 400 = 600
    expect(findGeometricNeighbor(layout, "A", "right")).toBe("C");
  });

  it("对称 V (B 0..500 · C 500..1000) · A → right → tie · paneId asc → B", () => {
    // 对称 split · overlap B = 500 = overlap C · centerDist B = 250 = centerDist C
    // 完全 tie · 用 paneId 字典序确定性 tie-break (B < C)
    const symmetric: PaneRect[] = [
      r("A", 0, 0, 500, 1000),
      r("B", 500, 0, 1000, 500),
      r("C", 500, 500, 1000, 1000),
    ];
    expect(findGeometricNeighbor(symmetric, "A", "right")).toBe("B");
  });
});

describe("findGeometricNeighbor · cross-axis no overlap → null", () => {
  it("两个 pane 在 right 方向但 vertical 完全不重叠 → null (D.3 不跨非相邻 splitter)", () => {
    // A 占左上 · B 在右下完全错位 · A → right 不应该跳到 B
    const layout: PaneRect[] = [
      r("A", 0, 0, 400, 400),
      r("B", 500, 500, 1000, 1000),
    ];
    expect(findGeometricNeighbor(layout, "A", "right")).toBeNull();
  });
});

describe("findGeometricNeighbor · 5 层 nested fixture", () => {
  // 5 层嵌套 · 16 leaf · spec §H.1 性能 fixture
  // 简化 representation：4x4 grid · 每个 cell 250x250
  const leaves: PaneRect[] = [];
  const ids = "ABCDEFGHIJKLMNOP".split("");
  let idx = 0;
  for (let row = 0; row < 4; row++) {
    for (let col = 0; col < 4; col++) {
      leaves.push(
        r(ids[idx++], col * 250, row * 250, (col + 1) * 250, (row + 1) * 250),
      );
    }
  }

  it("16 leaf · 任意中心 leaf 4 方向都有相邻", () => {
    // F 是 (1,1) cell · 上 B · 下 J · 左 E · 右 G
    expect(findGeometricNeighbor(leaves, "F", "up")).toBe("B");
    expect(findGeometricNeighbor(leaves, "F", "down")).toBe("J");
    expect(findGeometricNeighbor(leaves, "F", "left")).toBe("E");
    expect(findGeometricNeighbor(leaves, "F", "right")).toBe("G");
  });

  it("16 leaf · 边角 leaf 边界方向 null", () => {
    // A (0,0) ← null · ↑ null · → B · ↓ E
    expect(findGeometricNeighbor(leaves, "A", "left")).toBeNull();
    expect(findGeometricNeighbor(leaves, "A", "up")).toBeNull();
    expect(findGeometricNeighbor(leaves, "A", "right")).toBe("B");
    expect(findGeometricNeighbor(leaves, "A", "down")).toBe("E");
  });
});

describe("findGeometricNeighbor · edge cases", () => {
  it("focusedPaneId 不在列表中 → null", () => {
    const layout: PaneRect[] = [r("A", 0, 0, 500, 500)];
    expect(findGeometricNeighbor(layout, "missing", "right")).toBeNull();
  });

  it("空列表 → null", () => {
    expect(findGeometricNeighbor([], "A", "right")).toBeNull();
  });

  it("只有 focused 自己一个 pane → null", () => {
    const layout: PaneRect[] = [r("A", 0, 0, 500, 500)];
    expect(findGeometricNeighbor(layout, "A", "right")).toBeNull();
  });

  it("EPS 容差 · 完全贴边的 splitter 不漏判", () => {
    // 两个 pane 在 x=500 处完全贴边（无 splitter px 间隔）· A → right 应找到 B
    const layout: PaneRect[] = [
      r("A", 0, 0, 500, 500),
      r("B", 500, 0, 1000, 500),
    ];
    expect(findGeometricNeighbor(layout, "A", "right")).toBe("B");
  });
});

describe("matchesPaneNavigateModifier", () => {
  let originalPlatform: PropertyDescriptor | undefined;

  beforeEach(() => {
    originalPlatform = Object.getOwnPropertyDescriptor(
      window.navigator,
      "platform",
    );
  });

  afterEach(() => {
    if (originalPlatform) {
      Object.defineProperty(window.navigator, "platform", originalPlatform);
    }
  });

  const setPlatform = (platform: string) => {
    Object.defineProperty(window.navigator, "platform", {
      value: platform,
      configurable: true,
    });
  };

  it("macOS · ⌘⌥ matches", () => {
    setPlatform("MacIntel");
    const ev = new KeyboardEvent("keydown", {
      key: "ArrowRight",
      metaKey: true,
      altKey: true,
    });
    expect(matchesPaneNavigateModifier(ev)).toBe(true);
  });

  it("macOS · 只 ⌘ no match", () => {
    setPlatform("MacIntel");
    const ev = new KeyboardEvent("keydown", {
      key: "ArrowRight",
      metaKey: true,
      altKey: false,
    });
    expect(matchesPaneNavigateModifier(ev)).toBe(false);
  });

  it("macOS · ⌘⌥⇧ no match (Shift 加进来不算)", () => {
    setPlatform("MacIntel");
    const ev = new KeyboardEvent("keydown", {
      key: "ArrowRight",
      metaKey: true,
      altKey: true,
      shiftKey: true,
    });
    expect(matchesPaneNavigateModifier(ev)).toBe(false);
  });

  it("Linux · Ctrl+Alt matches", () => {
    setPlatform("Linux x86_64");
    const ev = new KeyboardEvent("keydown", {
      key: "ArrowRight",
      ctrlKey: true,
      altKey: true,
    });
    expect(matchesPaneNavigateModifier(ev)).toBe(true);
  });

  it("Linux · 只 Ctrl no match", () => {
    setPlatform("Linux x86_64");
    const ev = new KeyboardEvent("keydown", {
      key: "ArrowRight",
      ctrlKey: true,
      altKey: false,
    });
    expect(matchesPaneNavigateModifier(ev)).toBe(false);
  });

  it("Linux · 误用 ⌘⌥ (实际平台没 Cmd) no match", () => {
    setPlatform("Linux x86_64");
    const ev = new KeyboardEvent("keydown", {
      key: "ArrowRight",
      metaKey: true,
      altKey: true,
    });
    expect(matchesPaneNavigateModifier(ev)).toBe(false);
  });
});

describe("arrowKeyToDirection", () => {
  it("ArrowLeft → left", () => {
    expect(arrowKeyToDirection("ArrowLeft")).toBe("left");
  });
  it("ArrowRight → right", () => {
    expect(arrowKeyToDirection("ArrowRight")).toBe("right");
  });
  it("ArrowUp → up", () => {
    expect(arrowKeyToDirection("ArrowUp")).toBe("up");
  });
  it("ArrowDown → down", () => {
    expect(arrowKeyToDirection("ArrowDown")).toBe("down");
  });
  it("非箭头键 → null", () => {
    expect(arrowKeyToDirection("a")).toBeNull();
    expect(arrowKeyToDirection("Enter")).toBeNull();
    expect(arrowKeyToDirection(" ")).toBeNull();
  });
});
