import type { ScanMethod, ScanTarget } from "@/lib/backend/scans";

const TRAILING_PATH_SEPARATORS = /[\\/]+$/;
const PATH_SEPARATORS = /[\\/]/;

export type ScanTargetDescription = {
  /** Short name, for example `C:` or `Photos`. */
  title: string;
  /** The full location, for example `C:\` or `D:\Backup\Photos`. */
  location: string;
};

export function describeScanTarget(target: ScanTarget): ScanTargetDescription {
  if (target.volume !== undefined) {
    const mountPoint = target.volume.mountPoint;
    const withoutTrailingSeparator = mountPoint.replace(TRAILING_PATH_SEPARATORS, "");
    return {
      title: withoutTrailingSeparator.length > 0 ? withoutTrailingSeparator : mountPoint,
      location: mountPoint,
    };
  }

  const folderPath = target.folder.path;
  const pathSegments = folderPath
    .replace(TRAILING_PATH_SEPARATORS, "")
    .split(PATH_SEPARATORS)
    .filter((segment) => segment.length > 0);
  const lastSegment = pathSegments.at(-1);

  return { title: lastSegment ?? folderPath, location: folderPath };
}

export function scanMethodLabel(method: ScanMethod): string {
  return method === "masterFileTable" ? "Fast scan" : "Standard scan";
}
