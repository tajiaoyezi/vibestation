import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";

export default defineConfig({
  plugins: [solid()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: [],
    include: ["tests/**/*.{test,spec}.{ts,tsx}"],
    exclude: ["node_modules", "dist"],
  },
  resolve: {
    conditions: ["development", "browser"],
  },
});
