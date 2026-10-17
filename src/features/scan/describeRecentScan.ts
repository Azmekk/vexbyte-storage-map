import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { describeScanTarget, scanMethodLabel } from "./describeScanTarget";
import type { ScanRecord } from "./useScanStore";

export type RecentScanDescription = {
  scanId: number;
  title: string;
  location: string;
  status: ScanRecord["status"];
  detail: string;
};

function statusDetail(record: ScanRecord): string {
  switch (record.status) {
    case "running":
      return "Scanning…";
    case "cancelled":
      return "Cancelled";
    case "failed":
      return "Did not finish";
    case "finished": {
      const totals = record.result?.currentTotals ?? null;
      return totals === null ? "Finished" : formatBytesAsReadableSize(totals.logicalSize);
    }
  }
}

export function describeRecentScan(record: ScanRecord): RecentScanDescription {
  const { title, location } = describeScanTarget(record.target);
  return {
    scanId: record.scanId,
    title,
    location,
    status: record.status,
    detail: `${statusDetail(record)} · ${scanMethodLabel(record.method)}`,
  };
}

/** Newest first. */
export function describeRecentScans(scans: Record<number, ScanRecord>): RecentScanDescription[] {
  return Object.values(scans)
    .sort((first, second) => second.scanId - first.scanId)
    .map(describeRecentScan);
}
