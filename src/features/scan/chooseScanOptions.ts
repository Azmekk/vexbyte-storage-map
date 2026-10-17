import type { ScanMethod } from "@/lib/backend/scans";

/** How the Scan button of a drive behaves. */
export type ScanOptions =
  /** One button that starts this method. */
  | { kind: "single"; method: ScanMethod }
  /** A standard scan, plus a menu entry that restarts as admin for the fast scan. */
  | { kind: "standardWithElevationOffer" };

/**
 * Elevated on NTFS: go straight to the fast scan. NTFS without admin rights: standard scan, and
 * offer the restart. Anything else: standard scan only. While elevation is unknown the standard
 * scan is offered, so the button is never blocked.
 */
export function chooseScanOptions(
  supportsFastScan: boolean,
  isElevated: boolean | null,
): ScanOptions {
  if (!supportsFastScan) {
    return { kind: "single", method: "folderWalk" };
  }

  if (isElevated === true) {
    return { kind: "single", method: "masterFileTable" };
  }

  return { kind: "standardWithElevationOffer" };
}
