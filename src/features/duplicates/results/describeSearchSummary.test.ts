import { describe, expect, it } from "vitest";
import type { DuplicateSearchSummary } from "@/lib/backend/generated";
import { describeSearchCaveats, describeSearchHeadline } from "./describeSearchSummary";

function createSummary(overrides: Partial<DuplicateSearchSummary>): DuplicateSearchSummary {
  return {
    duplicateGroupCount: 0,
    duplicateFileCount: 0,
    totalWastedBytes: 0,
    filesExamined: 0,
    onlineOnlyFilesSkipped: 0,
    hardLinksCollapsed: 0,
    unreadableFileCount: 0,
    inaccessibleFolderCount: 0,
    folderDriveKinds: [],
    elapsedMilliseconds: 0,
    ...overrides,
  };
}

describe("describeSearchHeadline", () => {
  it("states the files compared and the time taken", () => {
    expect(
      describeSearchHeadline(createSummary({ filesExamined: 1204, elapsedMilliseconds: 3240 })),
    ).toBe("Compared 1,204 files in 3.2 seconds.");
    expect(
      describeSearchHeadline(createSummary({ filesExamined: 1, elapsedMilliseconds: 200 })),
    ).toBe("Compared 1 file in under a second.");
    expect(
      describeSearchHeadline(createSummary({ filesExamined: 5, elapsedMilliseconds: 125_000 })),
    ).toBe("Compared 5 files in 2 minutes 5 seconds.");
  });
});

describe("describeSearchCaveats", () => {
  it("is empty when everything was checked", () => {
    expect(describeSearchCaveats(createSummary({}))).toEqual([]);
  });

  it("lists skipped, unreadable and inaccessible items", () => {
    expect(
      describeSearchCaveats(
        createSummary({
          onlineOnlyFilesSkipped: 3,
          unreadableFileCount: 1,
          inaccessibleFolderCount: 2,
        }),
      ),
    ).toEqual([
      "3 files are online-only and were not checked.",
      "1 file couldn't be read.",
      "2 folders couldn't be opened.",
    ]);
  });
});
