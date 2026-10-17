import { useMemo } from "react";
import { describeRecentScans, type RecentScanDescription } from "./describeRecentScan";
import { useScanStore } from "./useScanStore";

/** The scans of this session for the sidebar, newest first. */
export function useRecentScans(): RecentScanDescription[] {
  const scans = useScanStore((state) => state.scans);
  return useMemo(() => describeRecentScans(scans), [scans]);
}
