import { defineConfig } from "vitest/config";

// 专用配置 · 测 Node.js 构建脚本（setup-git-hooks.mjs / validate-task-spec.mjs）
// 不用 vite-plugin-solid（那些是浏览器端 Solid 组件插件 · 脚本测试不需要）
// 不用 jsdom 环境 · 用 node 原生环境（node: 内置模块需要）
export default defineConfig({
  test: {
    environment: "node",
    globals: true,
    include: ["tests/scripts/**/*.{test,spec}.ts"],
  },
});
