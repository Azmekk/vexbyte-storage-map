import { describe, expect, it } from "vitest";
import type { TreeRecycleResult } from "@/lib/backend/fileActions";
import type { NodeFlagSummary } from "@/lib/backend/tree";
import { initializeCurrentPlatform, resetCurrentPlatform } from "@/lib/platform/currentPlatform";
import { canRecycleNode, describeNodeFlags } from "../inspector/describeNodeFlags";
import { describeRecycleOutcome } from "./describeRecycleOutcome";

const noFlags: NodeFlagSummary = {
  hardLinkCountedElsewhere: false,
  symlink: false,
  junction: false,
  cloudPlaceholder: false,
  compressed: false,
  sparse: false,
  hidden: false,
  system: false,
  inaccessible: false,
  synthetic: false,
};

function recycleResult(overrides: Partial<TreeRecycleResult["recycleResult"]>): TreeRecycleResult {
  return {
    recycleResult: { moved: [], failed: [], bytesFreed: 0, ...overrides },
    removal: { removedNodeCount: 0, freedLogicalSize: 3 * 1024 * 1024, freedAllocatedSize: 0 },
    totals: {
      logicalSize: 0,
      allocatedSize: 0,
      fileCount: 0,
      directoryCount: 0,
      inaccessibleDirectoryCount: 0,
      cloudPlaceholderFileCount: 0,
      cloudPlaceholderLogicalSize: 0,
      onlineOnlyFileCount: 0,
      additionalHardLinkCount: 0,
      linkCount: 0,
    },
  };
}

describe("describeNodeFlags", () => {
  it("lists access problems first and skips unset flags", () => {
    const badges = describeNodeFlags({
      ...noFlags,
      compressed: true,
      inaccessible: true,
      hardLinkCountedElsewhere: true,
    });
    expect(badges.map((badge) => badge.label)).toEqual([
      "Access denied",
      "Hard link, counted once",
      "Compressed",
    ]);
    expect(badges[0]?.tone).toBe("warning");
    expect(describeNodeFlags(noFlags)).toEqual([]);
  });

  it("never offers to recycle the scan root or summary items", () => {
    expect(canRecycleNode(noFlags, true)).toBe(false);
    expect(canRecycleNode({ ...noFlags, synthetic: true }, false)).toBe(false);
    expect(canRecycleNode(noFlags, false)).toBe(true);
  });
});

describe("describeRecycleOutcome", () => {
  it("reports the freed size from the tree removal, folders included", () => {
    const messages = describeRecycleOutcome(recycleResult({ moved: ["C:\\a", "C:\\b"] }));
    expect(messages.success?.title).toBe("Moved 2 items to the Recycle Bin");
    expect(messages.success?.description).toContain("3.0 MB freed");
    expect(messages.failure).toBeNull();
  });

  it("explains why items stayed in place", () => {
    const messages = describeRecycleOutcome(
      recycleResult({
        failed: [{ path: "E:\\video.mp4", reason: "noRecycleBinOnDrive", detail: null }],
      }),
    );
    expect(messages.success).toBeNull();
    expect(messages.failure?.title).toBe("1 item could not be moved");
    expect(messages.failure?.description).toContain("no Recycle Bin");
  });

  it("talks about the Trash on Linux", () => {
    initializeCurrentPlatform({ os: "linux", pathsAreCaseSensitive: true, appVersion: "0.2.0" });
    try {
      const messages = describeRecycleOutcome(recycleResult({ moved: ["/home/ana/a.iso"] }));
      expect(messages.success?.title).toBe("Moved 1 item to the Trash");
      expect(messages.success?.description).toContain("restore them from the Trash");
    } finally {
      resetCurrentPlatform();
    }
  });
});
