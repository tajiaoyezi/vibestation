import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  timeout: 60000,
  retries: 0,
  reporter: "list",
  outputDir: "../../../raw/SPIKE-08/tauri-playwright-results",
  use: {
    mode: "tauri",
    screenshot: "on",
    trace: "on",
  },
  projects: [
    {
      name: "tauri",
      use: {
        mode: "tauri",
      },
    },
  ],
});
