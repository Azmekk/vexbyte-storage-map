import { commands, type PlatformInfo } from "./generated";

export type { OperatingSystem, PlatformInfo } from "./generated";

/** The operating system the app runs on, how it compares paths and the app version. */
export function getPlatformInfo(): Promise<PlatformInfo> {
  return commands.getPlatformInfo();
}
