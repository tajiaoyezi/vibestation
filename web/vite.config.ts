import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// Tauri 2 dev server 约定端口 1420
export default defineConfig({
  plugins: [solid()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: false,
  },
  build: {
    target: "es2022",
    minify: "esbuild",
    sourcemap: false,
  },
});
