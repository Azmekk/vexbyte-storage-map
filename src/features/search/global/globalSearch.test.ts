import { describe, expect, it } from "vitest";
import type { VolumeIndexStatus } from "@/lib/backend/search";
import { createRequestSequence } from "./query/requestSequence";
import { isSearchSortOrder, SEARCH_SORT_OPTIONS } from "./query/searchSortOptions";
import {
  SEARCH_STARTER_EXAMPLES,
  SEARCH_SYNTAX_NOTES,
  SEARCH_SYNTAX_SECTIONS,
} from "./query/searchSyntaxHelp";
import { driveLetterLabel, searchHitFolderPath } from "./results/describeSearchHit";
import {
  describeSearchResultCount,
  describeShownHitLimit,
  formatMatchCount,
  formatSearchDuration,
} from "./results/describeSearchResultCount";
import {
  type SearchResultsKeyContext,
  type SearchResultsKeyPress,
  searchResultsKeyAction,
} from "./results/searchResultsKeyAction";
import { splitPathForMiddleTruncation } from "./results/splitPathForMiddleTruncation";
import { describeIndexStatusIndicator } from "./status/describeIndexStatusIndicator";
import {
  describeIndexAge,
  describeVolumeIndexState,
  indexMethodLabel,
  searchableVolumesKey,
  totalEstimatedIndexMemoryBytes,
  volumeIndexDisplayName,
  volumesWarmingUp,
} from "./status/describeVolumeIndexState";
import {
  describeStillWarmingVolumes,
  describeWarmingBannerTitle,
} from "./status/describeWarmingBanner";

const MINUTE = 60_000;

function volume(overrides: Partial<VolumeIndexStatus> = {}): VolumeIndexStatus {
  return {
    mountPoint: "C:\\",
    label: "Windows",
    volumeKind: "localDisk",
    state: { kind: "ready" },
    index: {
      builtAtUnixMilliseconds: 1_000,
      fileCount: 900,
      directoryCount: 100,
      method: "masterFileTable",
      durationMilliseconds: 4_000,
      estimatedMemoryBytes: 120_000,
    },
    ...overrides,
  };
}

describe("search syntax help", () => {
  it("lists every documented filter with an example", () => {
    const examples = SEARCH_SYNTAX_SECTIONS.flatMap((section) =>
      section.entries.map((entry) => entry.example),
    );
    for (const filter of ["ext:", "size:", "type:", "folder:"]) {
      expect(examples.some((example) => example.startsWith(filter))).toBe(true);
    }
    expect(examples.some((example) => example.includes("*"))).toBe(true);
    expect(examples.some((example) => example.includes("\\"))).toBe(true);
  });

  it("has unique examples and no empty text", () => {
    const entries = [
      ...SEARCH_SYNTAX_SECTIONS.flatMap((section) => section.entries),
      ...SEARCH_STARTER_EXAMPLES,
    ];
    const sectionExamples = SEARCH_SYNTAX_SECTIONS.flatMap((section) =>
      section.entries.map((entry) => entry.example),
    );
    expect(new Set(sectionExamples).size).toBe(sectionExamples.length);
    for (const entry of entries) {
      expect(entry.example.trim().length).toBeGreaterThan(0);
      expect(entry.description.trim().length).toBeGreaterThan(0);
    }
    expect(SEARCH_SYNTAX_NOTES.length).toBeGreaterThan(0);
  });
});

describe("search sort options", () => {
  it("offers name, size, modified and path", () => {
    expect(SEARCH_SORT_OPTIONS.map((option) => option.label)).toEqual([
      "Name",
      "Size",
      "Modified",
      "Path",
    ]);
    expect(isSearchSortOrder("sizeDescending")).toBe(true);
    expect(isSearchSortOrder("size")).toBe(false);
  });
});

describe("describeSearchResultCount", () => {
  it("counts matches with the right noun and the time taken", () => {
    expect(formatMatchCount(1)).toBe("1 match");
    expect(formatMatchCount(1284)).toBe("1,284 matches");
    expect(formatSearchDuration(0.4)).toBe("in under 1 ms");
    expect(formatSearchDuration(12)).toBe("in 12 ms");
    expect(formatSearchDuration(1430)).toBe("in 1.4 s");
    expect(
      describeSearchResultCount({
        shownHitCount: 1284,
        totalMatchCount: 1284,
        totalMatchesCapped: false,
        elapsedMilliseconds: 12,
      }),
    ).toBe("1,284 matches in 12 ms");
  });

  it("says when only the first results are shown", () => {
    const capped = {
      shownHitCount: 10_000,
      totalMatchCount: 48_210,
      totalMatchesCapped: true,
      elapsedMilliseconds: 31,
    };
    expect(describeShownHitLimit(capped)).toBe("showing the first 10,000");
    expect(describeSearchResultCount(capped)).toBe(
      "48,210 matches in 31 ms, showing the first 10,000",
    );
  });
});

describe("splitPathForMiddleTruncation", () => {
  it("keeps the last folder visible and lets the start be cut", () => {
    expect(splitPathForMiddleTruncation("C:\\Program Files\\Common Files\\System")).toEqual({
      head: "C:\\Program Files\\Common Files",
      tail: "\\System",
    });
    expect(splitPathForMiddleTruncation("D:/photos/2024")).toEqual({
      head: "D:/photos",
      tail: "/2024",
    });
  });

  it("leaves drive roots, top-level folders and plain names whole", () => {
    expect(splitPathForMiddleTruncation("C:\\")).toEqual({ head: "C:\\", tail: "" });
    expect(splitPathForMiddleTruncation("C:\\Windows")).toEqual({ head: "C:\\Windows", tail: "" });
    expect(splitPathForMiddleTruncation("notes")).toEqual({ head: "notes", tail: "" });
    expect(splitPathForMiddleTruncation("C:\\Users\\")).toEqual({ head: "C:\\Users\\", tail: "" });
  });

  it("does not let a very long last folder crowd out the start", () => {
    const longFolder = "a".repeat(40);
    expect(splitPathForMiddleTruncation(`C:\\Data\\${longFolder}`)).toEqual({
      head: `C:\\Data\\${longFolder}`,
      tail: "",
    });
    expect(splitPathForMiddleTruncation(`C:\\Data\\${longFolder}`, 50).tail).toBe(
      `\\${longFolder}`,
    );
  });
});

describe("describeSearchHit", () => {
  it("finds the folder of a hit", () => {
    expect(
      searchHitFolderPath({ path: "C:\\Windows\\System32\\kernel32.dll", name: "kernel32.dll" }),
    ).toBe("C:\\Windows\\System32");
    expect(searchHitFolderPath({ path: "C:\\pagefile.sys", name: "pagefile.sys" })).toBe("C:\\");
  });

  it("labels drives by their letter", () => {
    expect(driveLetterLabel("C:\\")).toBe("C:");
    expect(driveLetterLabel("F:")).toBe("F:");
  });
});

function key(name: string, overrides: Partial<SearchResultsKeyPress> = {}): SearchResultsKeyPress {
  return {
    key: name,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    hasTextSelection: false,
    ...overrides,
  };
}

function context(overrides: Partial<SearchResultsKeyContext> = {}): SearchResultsKeyContext {
  return {
    selectedIndex: null,
    resultCount: 50,
    pageSize: 10,
    isQueryEmpty: false,
    ...overrides,
  };
}

describe("searchResultsKeyAction", () => {
  it("moves the selection with the arrows and stops at the ends", () => {
    expect(searchResultsKeyAction(key("ArrowDown"), context())).toEqual({
      kind: "select",
      index: 0,
    });
    expect(searchResultsKeyAction(key("ArrowDown"), context({ selectedIndex: 3 }))).toEqual({
      kind: "select",
      index: 4,
    });
    expect(searchResultsKeyAction(key("ArrowDown"), context({ selectedIndex: 49 }))).toEqual({
      kind: "select",
      index: 49,
    });
    expect(searchResultsKeyAction(key("ArrowUp"), context({ selectedIndex: 0 }))).toEqual({
      kind: "select",
      index: 0,
    });
    expect(searchResultsKeyAction(key("ArrowUp"), context())).toBeNull();
  });

  it("pages and jumps to the first or last result", () => {
    expect(searchResultsKeyAction(key("PageDown"), context({ selectedIndex: 0 }))).toEqual({
      kind: "select",
      index: 9,
    });
    expect(searchResultsKeyAction(key("PageUp"), context({ selectedIndex: 5 }))).toEqual({
      kind: "select",
      index: 0,
    });
    expect(searchResultsKeyAction(key("End", { ctrlKey: true }), context())).toEqual({
      kind: "select",
      index: 49,
    });
    expect(searchResultsKeyAction(key("Home", { ctrlKey: true }), context())).toEqual({
      kind: "select",
      index: 0,
    });
    expect(searchResultsKeyAction(key("Home"), context())).toBeNull();
  });

  it("opens only a selected result with Enter", () => {
    expect(searchResultsKeyAction(key("Enter"), context())).toBeNull();
    expect(searchResultsKeyAction(key("Enter"), context({ selectedIndex: 7 }))).toEqual({
      kind: "open",
      index: 7,
    });
    expect(searchResultsKeyAction(key("Enter", { ctrlKey: true }), context())).toBeNull();
  });

  it("copies the path with Ctrl+C unless text is selected in the box", () => {
    expect(
      searchResultsKeyAction(key("c", { ctrlKey: true }), context({ selectedIndex: 2 })),
    ).toEqual({ kind: "copyPath", index: 2 });
    expect(
      searchResultsKeyAction(
        key("c", { ctrlKey: true, hasTextSelection: true }),
        context({ selectedIndex: 2 }),
      ),
    ).toBeNull();
    expect(searchResultsKeyAction(key("c", { ctrlKey: true }), context())).toBeNull();
  });

  it("clears a query with Escape, also without results", () => {
    expect(searchResultsKeyAction(key("Escape"), context({ resultCount: 0 }))).toEqual({
      kind: "clearQuery",
    });
    expect(searchResultsKeyAction(key("Escape"), context({ isQueryEmpty: true }))).toBeNull();
  });

  it("does nothing else without results", () => {
    expect(searchResultsKeyAction(key("ArrowDown"), context({ resultCount: 0 }))).toBeNull();
    expect(searchResultsKeyAction(key("a"), context())).toBeNull();
  });
});

describe("createRequestSequence", () => {
  it("treats only the newest request as current", () => {
    const sequence = createRequestSequence();
    const first = sequence.begin();
    expect(sequence.isLatest(first)).toBe(true);
    const second = sequence.begin();
    expect(sequence.isLatest(first)).toBe(false);
    expect(sequence.isLatest(second)).toBe(true);
  });

  it("keeps separate sequences independent", () => {
    const one = createRequestSequence();
    const other = createRequestSequence();
    const ticket = one.begin();
    other.begin();
    other.begin();
    expect(one.isLatest(ticket)).toBe(true);
  });
});

describe("warming banner text", () => {
  it("names the drives still warming", () => {
    expect(describeStillWarmingVolumes([])).toBeNull();
    expect(describeStillWarmingVolumes(["C:\\"])).toBe("C: still warming, results will update");
    expect(describeStillWarmingVolumes(["C:\\", "D:\\"])).toBe(
      "C: and D: still warming, results will update",
    );
    expect(describeStillWarmingVolumes(["C:\\", "D:\\", "E:\\"])).toBe(
      "C:, D: and E: still warming, results will update",
    );
  });

  it("titles the banner", () => {
    expect(describeWarmingBannerTitle(1)).toBe("Warming up search engine");
    expect(describeWarmingBannerTitle(2)).toBe("Warming up search engine on 2 drives");
  });
});

describe("volume index state", () => {
  it("describes each state", () => {
    expect(describeVolumeIndexState(volume()).label).toBe("Ready");
    expect(
      describeVolumeIndexState(
        volume({
          index: null,
          state: {
            kind: "building",
            filesSoFar: 1,
            directoriesSoFar: 1,
            elapsedMilliseconds: 10,
            method: "folderWalk",
          },
        }),
      ).label,
    ).toBe("Reading");
    expect(describeVolumeIndexState(volume({ state: { kind: "idle" } })).label).toBe(
      "Refresh queued",
    );
    const skipped = describeVolumeIndexState(
      volume({ index: null, state: { kind: "skipped", reason: "cloudDrive" } }),
    );
    expect(skipped.tone).toBe("skipped");
    expect(skipped.detail).toContain("Cloud drives");
    const failedRefresh = describeVolumeIndexState(
      volume({ state: { kind: "failed", reason: "Access denied." } }),
    );
    expect(failedRefresh.label).toBe("Refresh failed");
    expect(failedRefresh.detail).toContain("previous index");
  });

  it("formats names, methods and ages", () => {
    expect(volumeIndexDisplayName(volume())).toBe("Windows (C:)");
    expect(volumeIndexDisplayName(volume({ label: " " }))).toBe("Local Disk (C:)");
    expect(indexMethodLabel("masterFileTable")).toBe("Fast");
    expect(indexMethodLabel("folderWalk")).toBe("Standard");
    expect(describeIndexAge(0, 5 * MINUTE)).toBe("built 5 minutes ago");
  });

  it("finds warming drives, sums memory and keys searchable drives", () => {
    const ready = volume();
    const warming = volume({
      mountPoint: "D:\\",
      index: null,
      state: { kind: "idle" },
    });
    expect(volumesWarmingUp([ready, warming]).map((item) => item.mountPoint)).toEqual(["D:\\"]);
    expect(totalEstimatedIndexMemoryBytes([ready, warming, ready])).toBe(240_000);
    const keyBefore = searchableVolumesKey([ready, warming]);
    const progressTick = volume({
      mountPoint: "D:\\",
      index: null,
      state: {
        kind: "building",
        filesSoFar: 5,
        directoriesSoFar: 1,
        elapsedMilliseconds: 250,
        method: "folderWalk",
      },
    });
    expect(searchableVolumesKey([ready, progressTick])).toBe(keyBefore);
    expect(searchableVolumesKey([ready, volume({ mountPoint: "D:\\" })])).not.toBe(keyBefore);
  });
});

describe("describeIndexStatusIndicator", () => {
  it("shows a spinner while warming and nothing when ready", () => {
    expect(describeIndexStatusIndicator(null, null)).toEqual({ kind: "none" });
    expect(describeIndexStatusIndicator({ overall: "warming", volumes: [] }, null).kind).toBe(
      "warming",
    );
    expect(describeIndexStatusIndicator({ overall: "ready", volumes: [] }, null)).toEqual({
      kind: "none",
    });
  });

  it("warns about drives that could not be indexed", () => {
    const partial = describeIndexStatusIndicator(
      {
        overall: "partiallyReady",
        volumes: [
          volume(),
          volume({ mountPoint: "D:\\", index: null, state: { kind: "failed", reason: "Oops." } }),
        ],
      },
      null,
    );
    expect(partial).toEqual({
      kind: "warning",
      tooltip: "Some drives could not be indexed (D:). The others can be searched.",
    });
    expect(describeIndexStatusIndicator({ overall: "failed", volumes: [] }, null)).toEqual({
      kind: "warning",
      tooltip: "No local drive to search.",
    });
    expect(describeIndexStatusIndicator(null, "Broken.").kind).toBe("warning");
  });
});
