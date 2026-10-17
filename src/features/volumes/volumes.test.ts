import { describe, expect, it } from "vitest";
import type { VolumeSummary } from "@/lib/backend/volumes";
import { volumeDisplayName, volumeDriveLabel } from "./volumeDisplayName";
import { calculateVolumeUsage, createVolumeUsageChartSegments } from "./volumeUsage";

function createVolume(overrides: Partial<VolumeSummary> = {}): VolumeSummary {
  return {
    mountPoint: "C:\\",
    label: "Windows",
    fileSystemName: "NTFS",
    totalBytes: 1000,
    freeBytes: 250,
    driveType: "fixed",
    supportsFastScan: true,
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

  it("shows the mount point without its trailing separator", () => {
    expect(volumeDriveLabel(createVolume({ mountPoint: "D:\\" }))).toBe("D:");
    expect(volumeDriveLabel(createVolume({ mountPoint: "E:\\Mounts\\Backup\\" }))).toBe(
      "E:\\Mounts\\Backup",
    );
    expect(volumeDriveLabel(createVolume({ mountPoint: "/" }))).toBe("/");
  });
});
