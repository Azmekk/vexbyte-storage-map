import { describe, expect, it } from "vitest";
import type { VolumeSummary } from "@/lib/backend/volumes";
import { volumeDisplayName, volumeDriveLabel } from "./volumeDisplayName";
import { calculateVolumeUsage, createVolumeUsageChartSegments } from "./volumeUsage";

function createVolume(overrides: Partial<VolumeSummary> = {}): VolumeSummary {
  return {
    driveLetter: "C",
    label: "Windows",
    fileSystemName: "NTFS",
    totalBytes: 1000,
    freeBytes: 250,
    driveType: "fixed",
    supportsMasterFileTableScan: true,
    ...overrides,
  };
}

describe("calculateVolumeUsage", () => {
  it("derives used bytes and percentage", () => {
    expect(calculateVolumeUsage(createVolume())).toEqual({
      usedBytes: 750,
      freeBytes: 250,
      totalBytes: 1000,
      usedPercentage: 75,
    });
  });

  it("handles empty volumes and free space larger than total", () => {
    expect(calculateVolumeUsage(createVolume({ totalBytes: 0, freeBytes: 0 })).usedPercentage).toBe(
      0,
    );
    expect(calculateVolumeUsage(createVolume({ totalBytes: 10, freeBytes: 20 })).usedBytes).toBe(0);
  });

  it("creates one chart segment for used and one for free space", () => {
    const segments = createVolumeUsageChartSegments(calculateVolumeUsage(createVolume()));

    expect(segments.map(({ segment, bytes }) => [segment, bytes])).toEqual([
      ["used", 750],
      ["free", 250],
    ]);
  });
});

describe("volumeDisplayName", () => {
  it("uses the volume label when there is one", () => {
    expect(volumeDisplayName(createVolume({ label: "  Games " }))).toBe("Games");
  });

  it("falls back to a name based on the drive type", () => {
    expect(volumeDisplayName(createVolume({ label: "" }))).toBe("Local Disk");
    expect(volumeDisplayName(createVolume({ label: "", driveType: "removable" }))).toBe(
      "Removable Disk",
    );
  });

  it("formats the drive letter with a colon", () => {
    expect(volumeDriveLabel(createVolume({ driveLetter: "D" }))).toBe("D:");
  });
});
