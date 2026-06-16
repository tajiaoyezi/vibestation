import { defineConfig } from "vitest/config";
import solid from "vite-plugin-solid";

export default defineConfig({
  plugins: [solid({ hot: false })],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./tests/setup.ts"],
    include: ["tests/**/*.{test,spec}.{ts,tsx}"],
    exclude: [
      "node_modules",
      "dist",
      // tests/scripts/ 测的是 Node.js 构建脚本（node:path / node:child_process 等
      // 内置模块）· Vite transform 把 node: 模块外部化为浏览器不兼容导致假失败。
      // 这些脚本测试改用 `node --test` 单独跑（见 package.json test:scripts）·
      // 不阻塞前端 vitest 套件
      "tests/scripts/**",
    ],
  },
  resolve: {
    conditions: ["development", "browser"],
  },
});
