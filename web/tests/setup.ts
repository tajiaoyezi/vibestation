import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

type TauriInternals = {
  transformCallback: (callback: unknown, once?: boolean) => number;
  unregisterCallback: (id: number) => void;
  invoke: (cmd: string, args?: unknown, options?: unknown) => Promise<unknown>;
};

type TauriEventPluginInternals = {
  unregisterListener: (event: string, eventId: number) => void;
};

const testWindow = window as Window & {
  __TAURI_INTERNALS__?: TauriInternals;
  __TAURI_EVENT_PLUGIN_INTERNALS__?: TauriEventPluginInternals;
};

if (!window.matchMedia) {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    writable: true,
    value: (query: string): MediaQueryList => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }),
  });
}

if (!window.ResizeObserver) {
  class TestResizeObserver implements ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }

  Object.defineProperty(window, "ResizeObserver", {
    configurable: true,
    writable: true,
    value: TestResizeObserver,
  });
  Object.defineProperty(globalThis, "ResizeObserver", {
    configurable: true,
    writable: true,
    value: TestResizeObserver,
  });
}

if (!testWindow.__TAURI_INTERNALS__) {
  let callbackId = 1;
  Object.defineProperty(testWindow, "__TAURI_INTERNALS__", {
    configurable: true,
    writable: true,
    value: {
      transformCallback: vi.fn(() => callbackId++),
      unregisterCallback: vi.fn(),
      invoke: vi.fn(async (cmd: string) => {
        if (cmd === "plugin:event|listen") return callbackId++;
        if (cmd === "plugin:event|unlisten") return null;
        return null;
      }),
    } satisfies TauriInternals,
  });
}

if (!testWindow.__TAURI_EVENT_PLUGIN_INTERNALS__) {
  Object.defineProperty(testWindow, "__TAURI_EVENT_PLUGIN_INTERNALS__", {
    configurable: true,
    writable: true,
    value: {
      unregisterListener: vi.fn(),
    } satisfies TauriEventPluginInternals,
  });
}
