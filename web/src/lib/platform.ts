// Task 4.1 · platform-windows-class
//
// 平台判定 single source（Phase 4）。detectPlatform 纯函数（不读 / 不写 DOM，
// 便于 vitest 直接断言）+ applyPlatformClass 副作用（把 class + data-platform
// 属性写到 documentElement）。

export type Platform = "macos" | "linux" | "windows" | "unknown";

/**
 * 纯函数 · 据 navigator.platform（大小写不敏感）+ userAgentData 补充信号判定平台。
 * 不读 / 不写 DOM，便于 vitest 直接断言。
 */
export function detectPlatform(
  platformString: string = navigator.platform,
  uaPlatform: string | undefined = (
    navigator as Navigator & {
      userAgentData?: { platform?: string };
    }
  ).userAgentData?.platform,
): Platform {
  const p = platformString.toLowerCase();
  const ua = (uaPlatform ?? "").toLowerCase();
  if (p.includes("mac") || ua.includes("mac")) return "macos";
  if (p.includes("win") || ua.includes("win")) return "windows";
  if (p.includes("linux") || ua.includes("linux")) return "linux";
  return "unknown";
}

/**
 * 副作用函数 · 把平台 class + data-platform 属性写到 documentElement。
 * unknown 平台不发 class、不设属性（与现有"只 mac/linux 发"语义一致）。
 */
export function applyPlatformClass(
  root: HTMLElement = document.documentElement,
  platform: Platform = detectPlatform(),
): void {
  if (platform === "unknown") return;
  const className =
    platform === "macos"
      ? "platform-macos"
      : platform === "windows"
        ? "platform-windows"
        : "platform-linux";
  root.classList.add(className);
  root.setAttribute("data-platform", platform);
}
