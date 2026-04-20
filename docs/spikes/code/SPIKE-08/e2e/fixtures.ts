import { createTauriTest } from "@srsholmes/tauri-playwright";

export const { test, expect } = createTauriTest({
  devUrl: "http://127.0.0.1:1420",
  mcpSocket: "/tmp/tauri-playwright.sock",
  ipcMocks: {},
});
