import type { LinuxPackage } from "./latestRelease";

/** Which file the visitor clicked, so the page can show the matching install steps. */
export type StartedDownload =
  | { platform: "windows" }
  | { platform: "linux"; linuxPackage: LinuxPackage };
