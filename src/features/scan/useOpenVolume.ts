import { useCallback } from "react";
import { useNavigationStore } from "@/app/useNavigationStore";
import type { ScanRecord } from "./useScanStore";
import { useScanStore } from "./useScanStore";

/** The newest scan of this volume that is still worth showing (running or finished). */
export function findLatestScanOfVolume(
  scans: Record<number, ScanRecord>,
  mountPoint: string,
): ScanRecord | null {
  let latestScan: ScanRecord | null = null;

  for (const record of Object.values(scans)) {
    const isSameVolume = record.target.volume?.mountPoint === mountPoint;
    const isWorthShowing = record.status === "running" || record.status === "finished";
    if (
      isSameVolume &&
      isWorthShowing &&
      (latestScan === null || record.scanId > latestScan.scanId)
    ) {
      latestScan = record;
    }
  }

  return latestScan;
}

/** Selecting a drive in the sidebar shows its latest scan, or the home screen to start one. */
export function useOpenVolume(): (mountPoint: string) => void {
  const navigateToScan = useNavigationStore((state) => state.navigateToScan);
  const navigateHome = useNavigationStore((state) => state.navigateHome);

  return useCallback(
    (mountPoint: string) => {
      const latestScan = findLatestScanOfVolume(useScanStore.getState().scans, mountPoint);
      if (latestScan === null) {
        navigateHome();
      } else {
        navigateToScan(latestScan.scanId);
      }
    },
    [navigateHome, navigateToScan],
  );
}
