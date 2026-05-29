// Task 4.1 · platform-windows-class · RED stub（待 GREEN 实现）
//
// 平台判定 single source（Phase 4）。detectPlatform 纯函数 + applyPlatformClass 副作用。
// 本 RED 阶段仅声明签名以便测试编译，实现留待 GREEN commit。

export type Platform = "macos" | "linux" | "windows" | "unknown";

export function detectPlatform(
  _platformString: string = navigator.platform,
  _uaPlatform: string | undefined = (
    navigator as Navigator & {
      userAgentData?: { platform?: string };
    }
  ).userAgentData?.platform,
): Platform {
  // RED stub · 未实现
  return "unknown";
}

export function applyPlatformClass(
  _root: HTMLElement = document.documentElement,
  _platform: Platform = detectPlatform(),
): void {
  // RED stub · 未实现
}
