// MVP-15 Phase B · DiffLineContent IntersectionObserver lazy load + theme reactive
//
// 验证目标（spec §B.1 / §D.1）：
// - lazy load：viewport 外不触发 highlight（innerHTML 仅 escape · 不含 shiki span）
// - theme reactive：useShikiTheme() 切换主题 · DiffLine 自动重新 highlight

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";
import { DiffLineContent } from "../../../src/panels/Diff/DiffLine";
import { setShikiTheme } from "../../../src/utils/shiki/theme-store";
import { shikiAdapter } from "../../../src/utils/shiki";

// 可控 IntersectionObserver mock
type IOCallback = (entries: IntersectionObserverEntry[]) => void;
const observers: Array<{ cb: IOCallback; targets: Element[] }> = [];

class MockIntersectionObserver {
  callback: IOCallback;
  targets: Element[] = [];
  constructor(cb: IOCallback) {
    this.callback = cb;
    observers.push({ cb, targets: this.targets });
  }
  observe(target: Element) {
    this.targets.push(target);
  }
  unobserve(target: Element) {
    this.targets = this.targets.filter((t) => t !== target);
  }
  disconnect() {
    this.targets = [];
  }
}

function flushVisible() {
  // 模拟所有 observed target 进入 viewport
  for (const obs of observers) {
    const entries = obs.targets.map(
      (t) =>
        ({
          target: t,
          isIntersecting: true,
        }) as IntersectionObserverEntry,
    );
    obs.cb(entries);
  }
}

beforeEach(() => {
  observers.length = 0;
  vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);
  setShikiTheme("light");
  cleanup();
});

afterEach(() => {
  vi.restoreAllMocks();
});

async function waitForHighlight(timeoutMs = 1000): Promise<void> {
  // shiki highlight 是 async · 给 microtask + macrotask 时间
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    await new Promise((r) => setTimeout(r, 30));
  }
}

describe("DiffLineContent · IntersectionObserver lazy load", () => {
  it("viewport 外的行不触发 shiki highlight（innerHTML 仅 escape）", async () => {
    // 不调 flushVisible · 模拟行始终在 viewport 外
    const { container } = render(() => (
      <DiffLineContent
        content="const x = 1;"
        filePath="test.ts"
        lineType="context"
      />
    ));

    await waitForHighlight(200);

    const span = container.querySelector(".vs-diff-line-content")!;
    expect(span.innerHTML).not.toMatch(/class="shiki/);
  });

  it("行进入 viewport 后 · 异步 highlight 完成 · innerHTML 含 shiki 输出", async () => {
    const { container } = render(() => (
      <DiffLineContent
        content="const x = 1;"
        filePath="test.ts"
        lineType="context"
      />
    ));

    flushVisible();
    await waitForHighlight();

    const span = container.querySelector(".vs-diff-line-content")!;
    // 触发后必须含 shiki HTML 特征
    expect(span.innerHTML).toMatch(/class="shiki|<span/);
  });
});

describe("DiffLineContent · theme reactive", () => {
  it("theme 切换后 · innerHTML 重新计算（light vs dark 输出不同）", async () => {
    setShikiTheme("light");
    const { container } = render(() => (
      <DiffLineContent
        content="const x = 1;"
        filePath="test.ts"
        lineType="context"
      />
    ));

    flushVisible();
    await waitForHighlight();
    const lightHtml = container.querySelector(
      ".vs-diff-line-content",
    )!.innerHTML;

    // 切到 dark · createEffect 应自动重 highlight
    setShikiTheme("dark");
    await waitForHighlight();
    const darkHtml = container.querySelector(
      ".vs-diff-line-content",
    )!.innerHTML;

    // light 与 dark 输出 HTML 应不同（颜色不同）
    expect(darkHtml).not.toBe(lightHtml);
  });
});

describe("DiffLineContent · 不识别 lang fallback", () => {
  it("不识别后缀的文件渲染纯文本（HTML escape）· 不含 shiki span", async () => {
    const { container } = render(() => (
      <DiffLineContent
        content="<script>evil</script>"
        filePath="data.unknown_ext"
        lineType="context"
      />
    ));

    flushVisible();
    await waitForHighlight(300);

    const span = container.querySelector(".vs-diff-line-content")!;
    // 不含 shiki HTML
    expect(span.innerHTML).not.toMatch(/class="shiki/);
    // 必须 escape 防 XSS
    expect(span.innerHTML).toContain("&lt;script&gt;");
    expect(span.innerHTML).not.toContain("<script>");
  });
});

describe("DiffLineContent · Phase C 保护逻辑", () => {
  it("disableHighlight=true 时直接走纯文本 fallback，不调用 shikiAdapter", async () => {
    const highlightSpy = vi
      .spyOn(shikiAdapter, "highlight")
      .mockResolvedValue("<span>mock</span>");

    const { container } = render(() => (
      <DiffLineContent
        content="<b>hello</b>"
        filePath="test.ts"
        lineType="context"
        disableHighlight
      />
    ));

    flushVisible();
    await waitForHighlight(300);

    const span = container.querySelector(".vs-diff-line-content")!;
    expect(span.innerHTML).toContain("&lt;b&gt;hello&lt;/b&gt;");
    expect(highlightSpy).not.toHaveBeenCalled();
  });

  it("单行内容超过 100KB 时截断并显示提示", () => {
    const longLine = "a".repeat(110 * 1024);
    const { container } = render(() => (
      <DiffLineContent
        content={longLine}
        filePath="bundle.js"
        lineType="context"
        disableHighlight
      />
    ));

    const content = container.querySelector(".vs-diff-line-content")!;
    const notice = container.querySelector(".vs-diff-line-truncated");
    expect(content.textContent!.length).toBeLessThan(longLine.length);
    expect(notice).not.toBeNull();
    expect(notice!.textContent).toContain("Line too long · truncated at 100KB");
  });

  it("调用 highlight 时透传 fileSize", async () => {
    const highlightSpy = vi
      .spyOn(shikiAdapter, "highlight")
      .mockResolvedValue("<span>mock</span>");

    render(() => (
      <DiffLineContent
        content="const x = 1;"
        filePath="test.ts"
        lineType="context"
        fileSize={5_000_000}
      />
    ));

    flushVisible();
    await waitForHighlight(300);

    expect(highlightSpy).toHaveBeenCalledWith(
      "const x = 1;",
      "typescript",
      "light",
      5_000_000,
    );
  });
});
