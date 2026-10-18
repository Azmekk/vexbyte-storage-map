import type { PlatformInfo } from "@/lib/backend/platform";

/** Used until the backend answered, and in tests. */
const DEFAULT_PLATFORM_INFO: PlatformInfo = {
  os: "windows",
  pathsAreCaseSensitive: false,
  appVersion: "0.0.0",
};

let currentPlatformInfo: PlatformInfo = DEFAULT_PLATFORM_INFO;
let isPlatformKnown = false;

/** Set once at startup, before the first render, from `getPlatformInfo`. */
export function initializeCurrentPlatform(platformInfo: PlatformInfo): void {
  currentPlatformInfo = platformInfo;
  isPlatformKnown = true;
}

/** Puts the default (Windows) platform back, for tests. */
export function resetCurrentPlatform(): void {
  currentPlatformInfo = DEFAULT_PLATFORM_INFO;
  isPlatformKnown = false;
}

/** False when the backend could not be asked, so the app version is not known either. */
export function isCurrentPlatformKnown(): boolean {
  return isPlatformKnown;
}

/** The operating system the app runs on, how it compares paths and the app version. */
export function currentPlatform(): PlatformInfo {
  return currentPlatformInfo;
}
