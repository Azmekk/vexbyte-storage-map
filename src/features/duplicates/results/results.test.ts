import { describe, expect, it } from "vitest";
import type {
  DuplicateFile,
  DuplicateGroup,
  DuplicateRecycleResult,
} from "@/lib/backend/generated";
import { describeRecycleFailure, describeRecycleToast } from "./describeRecycleResult";
import {
  findGroupInPages,
  type GroupPages,
  groupAtIndex,
  groupIdsLeavingAfterMove,
  loadedPageIndexesWithin,
  pageIndexesCoveringRange,
  replaceGroupInPages,
} from "./groupPageCache";
import {
  DEFAULT_KEEP_RULE,
  isKeepRuleKind,
  KEEP_RULE_OPTIONS,
  keepRuleFromKind,
  keepRuleOptionFor,
  preferredFolderOf,
} from "./keepRuleOptions";
import {
  applyGroupChangeToSelection,
  canMarkForRemoval,
  collectFilesToRemove,
  describeRecycleButton,
  totalBytesOf,
} from "./selectionSummary";
import { DEFAULT_SORT_ORDER, isDuplicateGroupSortOrder } from "./sortOrderOptions";

function createFile(path: string, keep: boolean): DuplicateFile {
  return { path, keep, modifiedUnixMilliseconds: 0, createdUnixMilliseconds: 0 };
}

function createGroup(id: number, fileSize: number, files: DuplicateFile[]): DuplicateGroup {
  return { id, fileSize, files, wastedBytes: fileSize * (files.length - 1) };
}

describe("keep rule options", () => {
  it("offers the four rules with oldest as the default", () => {
    expect(KEEP_RULE_OPTIONS.map((option) => option.kind)).toEqual([
      "oldest",
      "newest",
      "preferredFolder",
      "shortestPath",
    ]);
    expect(DEFAULT_KEEP_RULE).toEqual({ kind: "oldest" });
  });

  it("maps selector choices to backend rules", () => {
    expect(keepRuleFromKind("oldest", null)).toEqual({ kind: "oldest" });
    expect(keepRuleFromKind("newest", "C:\\Photos")).toEqual({ kind: "newest" });
    expect(keepRuleFromKind("shortestPath", null)).toEqual({ kind: "shortestPath" });
    expect(keepRuleFromKind("preferredFolder", "C:\\Photos")).toEqual({
      kind: "preferredFolder",
      folder: "C:\\Photos",
    });
  });

  it("needs a folder for the preferred folder rule", () => {
    expect(keepRuleFromKind("preferredFolder", null)).toBeNull();
  });

  it("finds the option and folder of a rule", () => {
    const rule = { kind: "preferredFolder", folder: "D:\\Best" } as const;
    expect(keepRuleOptionFor(rule).kind).toBe("preferredFolder");
    expect(preferredFolderOf(rule)).toBe("D:\\Best");
    expect(preferredFolderOf({ kind: "newest" })).toBeNull();
  });

  it("recognizes only known kinds and sort orders", () => {
    expect(isKeepRuleKind("shortestPath")).toBe(true);
    expect(isKeepRuleKind("largest")).toBe(false);
    expect(isDuplicateGroupSortOrder(DEFAULT_SORT_ORDER)).toBe(true);
    expect(isDuplicateGroupSortOrder("random")).toBe(false);
  });
});

describe("selection summary", () => {
  const previousGroup = createGroup(1, 100, [
    createFile("C:\\a.jpg", true),
    createFile("C:\\b.jpg", false),
    createFile("C:\\c.jpg", false),
  ]);

  it("adds a newly removed copy to the selection", () => {
    const updatedGroup = createGroup(2, 100, [
      createFile("C:\\a.jpg", true),
      createFile("C:\\b.jpg", false),
      createFile("C:\\c.jpg", false),
    ]);
    const before = createGroup(2, 100, [
      createFile("C:\\a.jpg", true),
      createFile("C:\\b.jpg", true),
      createFile("C:\\c.jpg", false),
    ]);

    expect(
      applyGroupChangeToSelection(
        { filesSelectedForRemoval: 5, bytesSelectedForRemoval: 1000 },
        before,
        updatedGroup,
      ),
    ).toEqual({ filesSelectedForRemoval: 6, bytesSelectedForRemoval: 1100 });
  });

  it("takes a copy that is kept again out of the selection, never below zero", () => {
    const updatedGroup = createGroup(1, 100, [
      createFile("C:\\a.jpg", true),
      createFile("C:\\b.jpg", true),
      createFile("C:\\c.jpg", false),
    ]);

    expect(
      applyGroupChangeToSelection(
        { filesSelectedForRemoval: 2, bytesSelectedForRemoval: 200 },
        previousGroup,
        updatedGroup,
      ),
    ).toEqual({ filesSelectedForRemoval: 1, bytesSelectedForRemoval: 100 });
    expect(
      applyGroupChangeToSelection(
        { filesSelectedForRemoval: 0, bytesSelectedForRemoval: 0 },
        previousGroup,
        updatedGroup,
      ),
    ).toEqual({ filesSelectedForRemoval: 0, bytesSelectedForRemoval: 0 });
  });

  it("refuses to remove the last kept copy", () => {
    expect(canMarkForRemoval(previousGroup, "C:\\a.jpg")).toBe(false);
    expect(canMarkForRemoval(previousGroup, "C:\\b.jpg")).toBe(true);
    expect(canMarkForRemoval(previousGroup, "C:\\missing.jpg")).toBe(false);
  });

  it("collects every copy marked for removal with its size and category", () => {
    const otherGroup = createGroup(2, 50, [
      createFile("C:\\x.pdf", false),
      createFile("C:\\y.pdf", true),
    ]);
    const filesToRemove = collectFilesToRemove([previousGroup, otherGroup]);

    expect(filesToRemove).toEqual([
      { path: "C:\\b.jpg", fileSize: 100, category: "images" },
      { path: "C:\\c.jpg", fileSize: 100, category: "images" },
      { path: "C:\\x.pdf", fileSize: 50, category: "pdfs" },
    ]);
    expect(totalBytesOf(filesToRemove)).toBe(250);
  });

  it("labels the recycle button with count and size", () => {
    expect(describeRecycleButton(1, "2.0 MB")).toBe("Move 1 marked file (2.0 MB) to Recycle Bin");
    expect(describeRecycleButton(1204, "3.4 GB")).toBe(
      "Move all 1,204 marked files (3.4 GB) to Recycle Bin",
    );
  });
});

describe("group page cache", () => {
  const firstPageGroups = [
    createGroup(10, 1, [createFile("C:\\1a", true), createFile("C:\\1b", false)]),
    createGroup(11, 1, [
      createFile("C:\\2a", true),
      createFile("C:\\2b", false),
      createFile("C:\\2c", false),
    ]),
  ];
  const secondPageGroups = [
    createGroup(12, 1, [createFile("C:\\3a", true), createFile("C:\\3b", false)]),
  ];
  const pages: GroupPages = new Map([
    [0, firstPageGroups],
    [1, secondPageGroups],
  ]);

  it("lists the pages covering a range, clamped to the total", () => {
    expect(pageIndexesCoveringRange(0, 9, 100, 10)).toEqual([0]);
    expect(pageIndexesCoveringRange(8, 25, 100, 10)).toEqual([0, 1, 2]);
    expect(pageIndexesCoveringRange(-5, 500, 15, 10)).toEqual([0, 1]);
    expect(pageIndexesCoveringRange(0, 10, 0, 10)).toEqual([]);
  });

  it("finds groups by index and by id", () => {
    expect(groupAtIndex(pages, 1, 2)?.id).toBe(11);
    expect(groupAtIndex(pages, 2, 2)?.id).toBe(12);
    expect(groupAtIndex(pages, 4, 2)).toBeUndefined();
    expect(findGroupInPages(pages, 12)?.fileSize).toBe(1);
    expect(findGroupInPages(pages, 99)).toBeUndefined();
  });

  it("replaces one group and reuses untouched pages", () => {
    const updatedGroup = createGroup(11, 1, [
      createFile("C:\\2a", true),
      createFile("C:\\2b", true),
      createFile("C:\\2c", false),
    ]);
    const updatedPages = replaceGroupInPages(pages, updatedGroup);

    expect(groupAtIndex(updatedPages, 1, 2)).toBe(updatedGroup);
    expect(updatedPages.get(1)).toBe(secondPageGroups);
    expect(groupAtIndex(pages, 1, 2)).not.toBe(updatedGroup);
  });

  it("finds the groups left with fewer than two copies after a move", () => {
    const leavingGroupIds = groupIdsLeavingAfterMove(pages, ["C:\\1b", "C:\\2b"]);
    expect([...leavingGroupIds]).toEqual([10]);
  });

  it("lists loaded pages that still exist", () => {
    expect(loadedPageIndexesWithin(pages, 3, 2)).toEqual([0, 1]);
    expect(loadedPageIndexesWithin(pages, 2, 2)).toEqual([0]);
  });
});

describe("recycle result descriptions", () => {
  function createResult(overrides: Partial<DuplicateRecycleResult>): DuplicateRecycleResult {
    return {
      moved: [],
      failed: [],
      changed: [],
      bytesFreed: 0,
      selection: { filesSelectedForRemoval: 0, bytesSelectedForRemoval: 0 },
      ...overrides,
    };
  }

  it("reports the space freed", () => {
    const toast = describeRecycleToast(
      createResult({ moved: ["C:\\a", "C:\\b"], bytesFreed: 2048 }),
    );
    expect(toast.kind).toBe("success");
    expect(toast.title).toBe("Moved 2 files to the Recycle Bin");
    expect(toast.description).toBe("Freed 2.0 KB. You can restore them from the Recycle Bin.");
  });

  it("warns when some files were skipped", () => {
    const toast = describeRecycleToast(
      createResult({
        moved: ["C:\\a"],
        bytesFreed: 1024,
        changed: [{ path: "C:\\b", reason: "sizeChanged" }],
      }),
    );
    expect(toast.kind).toBe("warning");
    expect(toast.description).toContain("You can restore it from the Recycle Bin.");
    expect(toast.description).toContain("1 file was skipped.");
  });

  it("reports an error when nothing moved", () => {
    const toast = describeRecycleToast(
      createResult({
        failed: [{ path: "C:\\a", reason: "stillInPlace", detail: null }],
      }),
    );
    expect(toast.kind).toBe("error");
  });

  it("adds the system detail to a failure reason", () => {
    expect(describeRecycleFailure("notFound", null)).toBe("The file is no longer there.");
    expect(describeRecycleFailure("stillInPlace", " Access is denied. ")).toBe(
      "Windows could not move it. It is still in place. Access is denied.",
    );
  });
});
