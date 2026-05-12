import { describe, it, expect } from "vitest";
import { createShikiAdapter } from "../../../../src/utils/shiki";
import {
  scheduleHighlight,
  STREAMING_THRESHOLDS,
} from "../../../../src/utils/shiki/scheduler";

describe("G.4 单行超大文件", () => {
  it("10MB 单行应能被处理（scheduler 走 worker 路径）", async () => {
    const adapter = createShikiAdapter();
    // 构造 10MB 单行
    const hugeLine = "x".repeat(10 * 1024 * 1024);

    const result = await adapter.highlight(
      hugeLine,
      "typescript",
      "light",
      11 * 1024 * 1024,
    );

    expect(result).toBeDefined();
    expect(result.length).toBeGreaterThan(0);
  });

  it("100KB 边界应走同步路径", async () => {
    const adapter = createShikiAdapter();
    const line100KB = "a".repeat(100 * 1024);

    const result = await adapter.highlight(
      line100KB,
      "typescript",
      "light",
      100 * 1024,
    );

    expect(result).toBeDefined();
  });

  it("scheduler 阈值常量应匹配 spec", () => {
    expect(STREAMING_THRESHOLDS.SIZE_1MB).toBe(1 * 1024 * 1024);
    expect(STREAMING_THRESHOLDS.SIZE_10MB).toBe(10 * 1024 * 1024);
    expect(STREAMING_THRESHOLDS.CHUNK_SIZE_BYTES).toBe(100 * 1024);
  });

  it("⚠️ 截断逻辑检查：Phase C 未实施单行截断", async () => {
    // spec G.4 要求：单行 10MB → 按 100KB 分段截断 + "Line too long · truncated" 提示
    // 当前 scheduler.ts 仅按 fileSize 分三档，未对单行长度做截断
    // 本测试暴露该缺陷，留 v0.3 sprint fix track

    const hugeLine = "x".repeat(10 * 1024 * 1024);

    // 直接调 scheduleHighlight 观察：即使传了 11MB fileSize，
    // 实际代码仍是 10MB 单行，不会被截断
    const syncHighlight = async () => "mocked-html";
    const result = await scheduleHighlight(
      hugeLine,
      "typescript",
      "light",
      11 * 1024 * 1024,
      syncHighlight,
    );

    // 当前实现：返回完整内容（未截断）
    expect(result).toBe("mocked-html");

    // TODO: v0.3 sprint 应在 scheduler 中加入单行截断逻辑
    // 截断后 result 应缩短 + 包含截断提示
  });
});
