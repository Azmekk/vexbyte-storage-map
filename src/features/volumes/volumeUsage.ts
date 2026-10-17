import type { VolumeSummary } from "@/lib/backend/volumes";

export type VolumeUsage = {
  usedBytes: number;
  freeBytes: number;
  totalBytes: number;
  /** Between 0 and 100. */
  usedPercentage: number;
};

export type VolumeUsageChartSegment = {
  segment: "used" | "free";
  bytes: number;
  fill: string;
};

export function calculateVolumeUsage(volume: VolumeSummary): VolumeUsage {
  const totalBytes = Math.max(0, volume.totalBytes);
  const freeBytes = Math.min(Math.max(0, volume.freeBytes), totalBytes);
  const usedBytes = totalBytes - freeBytes;
  const usedPercentage = totalBytes === 0 ? 0 : (usedBytes / totalBytes) * 100;

  return { usedBytes, freeBytes, totalBytes, usedPercentage };
}

/** Segments for the usage donut. Colors refer to the chart config's CSS variables. */
export function createVolumeUsageChartSegments(usage: VolumeUsage): VolumeUsageChartSegment[] {
  return [
    { segment: "used", bytes: usage.usedBytes, fill: "var(--color-used)" },
    { segment: "free", bytes: usage.freeBytes, fill: "var(--color-free)" },
  ];
}
