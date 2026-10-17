import type { DriveType, VolumeSummary } from "@/lib/backend/volumes";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { volumeDisplayName, volumeDriveLabel } from "./volumeDisplayName";
import {
  calculateVolumeUsage,
  createVolumeUsageChartSegments,
  type VolumeUsageChartSegment,
} from "./volumeUsage";

/** Everything the volume card and sidebar item display, already formatted. */
export type VolumeDescription = {
  driveLetter: string;
  displayName: string;
  driveLabel: string;
  driveType: DriveType;
  fileSystemName: string;
  usedSizeText: string;
  freeSizeText: string;
  totalSizeText: string;
  usedPercentage: number;
  usedPercentageLabel: string;
  usageChartSegments: VolumeUsageChartSegment[];
  supportsFastScan: boolean;
};

export function describeVolume(volume: VolumeSummary): VolumeDescription {
  const usage = calculateVolumeUsage(volume);

  return {
    driveLetter: volume.driveLetter,
    displayName: volumeDisplayName(volume),
    driveLabel: volumeDriveLabel(volume),
    driveType: volume.driveType,
    fileSystemName: volume.fileSystemName,
    usedSizeText: formatBytesAsReadableSize(usage.usedBytes),
    freeSizeText: formatBytesAsReadableSize(usage.freeBytes),
    totalSizeText: formatBytesAsReadableSize(usage.totalBytes),
    usedPercentage: usage.usedPercentage,
    usedPercentageLabel: `${Math.round(usage.usedPercentage)}%`,
    usageChartSegments: createVolumeUsageChartSegments(usage),
    supportsFastScan: volume.supportsMasterFileTableScan,
  };
}
