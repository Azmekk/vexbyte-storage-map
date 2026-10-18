export type Platform =
  | "windows-x64"
  | "windows-arm64"
  | "windows-32"
  | "mac"
  | "linux"
  | "mobile"
  | "unknown";

export type PlatformHints = {
  platform?: string;
  architecture?: string;
  bitness?: string;
};

type NavigatorWithHints = Navigator & {
  userAgentData?: {
    getHighEntropyValues(hints: string[]): Promise<PlatformHints>;
  };
};

/** What to call a platform the installer can't run on, as in "Not available for …". */
export function unsupportedPlatformName(platform: Platform): string | null {
  switch (platform) {
    case "mobile":
      return "phones and tablets";
    case "mac":
      return "macOS";
    case "linux":
      return "Linux";
    case "windows-32":
      return "32-bit Windows";
    default:
      return null;
  }
}

export function classifyPlatform(userAgent: string, hints?: PlatformHints): Platform {
  if (/iPhone|iPad|iPod|Android/i.test(userAgent)) return "mobile";
  if (hints?.platform) {
    switch (hints.platform) {
      case "Windows":
        if (hints.architecture === "arm") return "windows-arm64";
        if (hints.bitness === "32") return "windows-32";
        return "windows-x64";
      case "macOS":
        return "mac";
      case "Linux":
      case "Chrome OS":
        return "linux";
      case "Android":
        return "mobile";
    }
  }
  if (/Windows/i.test(userAgent)) {
    if (/ARM64/i.test(userAgent)) return "windows-arm64";
    if (/Win64|WOW64|x64/i.test(userAgent)) return "windows-x64";
    // Browsers freeze the Windows token, so a bare "Windows NT" is usually still 64-bit.
    return "windows-x64";
  }
  // iPadOS reports itself as a Mac.
  if (/Macintosh|Mac OS X/i.test(userAgent)) return "mac";
  if (/Linux|CrOS/i.test(userAgent)) return "linux";
  return "unknown";
}

export async function detectPlatform(): Promise<Platform> {
  const browserNavigator = navigator as NavigatorWithHints;
  let hints: PlatformHints | undefined;
  try {
    hints = await browserNavigator.userAgentData?.getHighEntropyValues([
      "platform",
      "architecture",
      "bitness",
    ]);
  } catch {
    hints = undefined;
  }
  const platform = classifyPlatform(navigator.userAgent, hints);
  if (platform === "mac" && navigator.maxTouchPoints > 1) return "mobile";
  return platform;
}
