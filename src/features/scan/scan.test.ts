import { describe, expect, it } from "vitest";
import { chooseScanOptions } from "./chooseScanOptions";
import { describeScanFailure } from "./describeScanFailure";
import { describeScanTarget, scanMethodLabel } from "./describeScanTarget";
import { useScanStore } from "./useScanStore";

describe("chooseScanOptions", () => {
  it("uses the fast scan directly when elevated on NTFS", () => {
    expect(chooseScanOptions(true, true)).toEqual({ kind: "single", method: "masterFileTable" });
  });

  it("offers the restart as admin when NTFS but not elevated", () => {
    expect(chooseScanOptions(true, false)).toEqual({ kind: "standardWithElevationOffer" });
    expect(chooseScanOptions(true, null)).toEqual({ kind: "standardWithElevationOffer" });
  });

  it("only offers the standard scan on other file systems", () => {
    expect(chooseScanOptions(false, true)).toEqual({ kind: "single", method: "folderWalk" });
  });
});

describe("describeScanTarget", () => {
  it("names volumes by their drive", () => {
    expect(describeScanTarget({ volume: { mountPoint: "C:\\" } })).toEqual({
      title: "C:",
      location: "C:\\",
    });
  });

  it("names folders by their last segment", () => {
    expect(describeScanTarget({ folder: { path: "D:\\Backup\\Photos\\" } }).title).toBe("Photos");
    expect(describeScanTarget({ folder: { path: "E:\\" } }).title).toBe("E:");
  });

  it("labels scan methods", () => {
    expect(scanMethodLabel("masterFileTable")).toBe("Fast scan");
    expect(scanMethodLabel("folderWalk")).toBe("Standard scan");
  });
});

describe("describeScanFailure", () => {
  it("does not offer another scan of a cloud drive", () => {
    const description = describeScanFailure({
      kind: "cloudVirtualVolumeNotScanned",
      mountPoint: "G:\\",
    });
    expect(description.suggestsStandardScan).toBe(false);
    expect(description.explanation).toContain("G:\\");
  });

  it("suggests a standard scan when the fast scan is not possible", () => {
    const description = describeScanFailure({ kind: "fastScanUnsupported", path: "F:\\" });
    expect(description.suggestsStandardScan).toBe(true);
    expect(description.explanation).toContain("F:\\");
  });
});

describe("useScanStore", () => {
  const target = { folder: { path: "C:\\Temp" } };

  it("keeps an outcome that arrives before the scan is registered", () => {
    useScanStore.getState().recordOutcome(91, { kind: "cancelled" });
    useScanStore.getState().registerStartedScan(91, target, "folderWalk", null);

    const record = useScanStore.getState().scans[91];
    expect(record?.status).toBe("cancelled");
    expect(useScanStore.getState().outcomesOfUnregisteredScans[91]).toBeUndefined();
  });

  it("ignores progress after a scan ended", () => {
    useScanStore.getState().registerStartedScan(92, target, "folderWalk", null);
    useScanStore.getState().recordOutcome(92, { kind: "cancelled" });
    useScanStore.getState().recordProgress(92, {
      phase: "listingDirectories",
      filesFound: 5,
      directoriesFound: 1,
      bytesFound: 10,
      currentPath: "C:\\Temp",
      elapsedMilliseconds: 100,
      largestTopLevelDirectories: [],
    });

    expect(useScanStore.getState().scans[92]?.latestProgress).toBeNull();
  });
});
