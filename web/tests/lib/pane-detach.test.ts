import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  detachedPanes,
  setDetachedPanes,
  initPaneDetachStateListener,
} from "../../../src/lib/pane-detach";

describe("pane-detach.ts · detachedPanes signal", () => {
  beforeEach(() => {
    setDetachedPanes(new Map());
  });

  it("初始状态为空 Map", () => {
    expect(detachedPanes().size).toBe(0);
  });

  it("setDetachedPanes 添加 entry", () => {
    setDetachedPanes(new Map([["pane-1", "win-1"]]));
    expect(detachedPanes().get("pane-1")).toBe("win-1");
    expect(detachedPanes().size).toBe(1);
  });

  it("setDetachedPanes 删除 entry", () => {
    setDetachedPanes(new Map([["pane-1", "win-1"]]));
    setDetachedPanes(new Map());
    expect(detachedPanes().has("pane-1")).toBe(false);
    expect(detachedPanes().size).toBe(0);
  });

  it("多个 pane 同时 detached", () => {
    setDetachedPanes(
      new Map([
        ["pane-1", "win-1"],
        ["pane-2", "win-2"],
      ]),
    );
    expect(detachedPanes().size).toBe(2);
    expect(detachedPanes().get("pane-2")).toBe("win-2");
  });

  it("重复 set 同一 pane 覆盖 windowLabel", () => {
    setDetachedPanes(new Map([["pane-1", "win-1"]]));
    setDetachedPanes(new Map([["pane-1", "win-2"]]));
    expect(detachedPanes().get("pane-1")).toBe("win-2");
    expect(detachedPanes().size).toBe(1);
  });
});

describe("pane-detach.ts · initPaneDetachStateListener", () => {
  beforeEach(() => {
    setDetachedPanes(new Map());
    vi.restoreAllMocks();
  });

  it("返回 unlisten cleanup 函数", async () => {
    vi.stubGlobal("__TAURI_INTERNALS__", {
      transformCallback: vi.fn(() => 42),
      invoke: vi.fn(),
    });

    const unlisten = await initPaneDetachStateListener();
    expect(typeof unlisten).toBe("function");
    unlisten();
  });
});
