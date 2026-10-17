import type { ScanResult } from "@/features/scan/useScanStore";
import type { NodeDetails, OnlineOnlyVisibility } from "@/lib/backend/tree";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatCount } from "@/lib/formatting/formatCount";
import { formatDurationInWords } from "@/lib/formatting/formatDuration";
import { formatModifiedDateAndTime } from "@/lib/formatting/formatModifiedDate";
import { formatShare } from "@/lib/formatting/formatShare";

export type InspectorDetailText = { label: string; value: string };

/** The facts listed for one item, in reading order. */
export function describeItemDetails(
  details: NodeDetails,
  onlineOnly: OnlineOnlyVisibility,
): InspectorDetailText[] {
  const rows: InspectorDetailText[] = [
    { label: "Size on disk", value: formatBytesAsReadableSize(details.allocatedSize) },
    { label: "Share of folder", value: formatShare(details.shareOfParent) },
  ];

  if (details.kind === "directory") {
    rows.push(
      { label: "Files", value: formatCount(details.fileCount) },
      { label: "Folders", value: formatCount(details.directoryCount) },
    );
    if (details.onlineOnlyFileCount > 0) {
      const count = formatCount(details.onlineOnlyFileCount);
      rows.push({
        label: "Online-only files",
        value: onlineOnly === "hidden" ? `${count} (not shown)` : count,
      });
    }
  } else if (details.extension !== null) {
    rows.push({ label: "Type", value: `.${details.extension}` });
  }

  rows.push({
    label: "Modified",
    value: formatModifiedDateAndTime(details.modifiedAtUnixMilliseconds),
  });
  return rows;
}

/** Facts about the whole scan, shown while the scanned folder itself is inspected. */
export function describeScanDetails(result: ScanResult): InspectorDetailText[] {
  const totals = result.currentTotals;
  const rows: InspectorDetailText[] = [
    { label: "Scan took", value: formatDurationInWords(result.summary.durationMilliseconds) },
  ];

  if (totals.inaccessibleDirectoryCount > 0) {
    rows.push({
      label: "Folders not readable",
      value: formatCount(totals.inaccessibleDirectoryCount),
    });
  }
  if (totals.cloudPlaceholderFileCount > 0) {
    rows.push({
      label: "In the cloud",
      value: `${formatCount(totals.cloudPlaceholderFileCount)} files (${formatBytesAsReadableSize(totals.cloudPlaceholderLogicalSize)} online, not counted)`,
    });
  }
  if (totals.additionalHardLinkCount > 0) {
    rows.push({ label: "Extra hard links", value: formatCount(totals.additionalHardLinkCount) });
  }
  if (totals.linkCount > 0) {
    rows.push({ label: "Links not followed", value: formatCount(totals.linkCount) });
  }

  return rows;
}
