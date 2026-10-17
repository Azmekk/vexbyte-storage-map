import { describe, expect, it } from "vitest";
import type { BreadcrumbItem, ChildrenPage, ChildSummary } from "@/lib/backend/tree";
import type { QueryState } from "@/lib/queryCache/scanQueryCache";
import { buildBreadcrumbTrail } from "./breadcrumbs/buildBreadcrumbTrail";
import { describeEmptyFolder } from "./tree/describeEmptyFolder";
import { flattenTreeRows } from "./tree/flattenTreeRows";
import { columnSortState, nextSortOrderForColumn } from "./tree/treeColumns";
import { treeKeyboardAction } from "./tree/treeKeyboardNavigation";

function createChild(node: number, overrides: Partial<ChildSummary> = {}): ChildSummary {
  return {
    node,
    name: `item ${node}`,
    kind: "directory",
    flags: {
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
    },
    logicalSize: 100,
    allocatedSize: 100,
    fileCount: 1,
    directoryCount: 0,
    modifiedAtUnixMilliseconds: null,
    shareOfParent: 0.5,
    hasChildren: true,
    extension: null,
    category: null,
    ...overrides,
  };
}

function loadedPage(
  parent: number,
  children: ChildSummary[],
  totalChildCount = children.length,
): QueryState<ChildrenPage> {
  return {
    status: "success",
    data: { parent, totalChildCount, offset: 0, children },
    isStale: false,
  };
}

describe("flattenTreeRows", () => {
  const pages = new Map<number, QueryState<ChildrenPage>>([
    [1, loadedPage(1, [createChild(2), createChild(3, { kind: "file", hasChildren: false })], 5)],
    [2, loadedPage(2, [createChild(4, { hasChildren: false })])],
  ]);

  it("lists the shown folder's children at depth 0 and a show-more row", () => {
    const { rows, foldersToLoad } = flattenTreeRows({
      rootFolder: 1,
      expandedFolders: new Set(),
      readChildren: (folder) => pages.get(folder),
    });

    expect(rows.map((row) => row.kind)).toEqual(["item", "item", "showMore"]);
    expect(rows[2]).toMatchObject({ hiddenChildCount: 3 });
    expect(foldersToLoad).toEqual([]);
  });

  it("nests expanded folders and asks for missing pages", () => {
    const { rows, foldersToLoad } = flattenTreeRows({
      rootFolder: 1,
      expandedFolders: new Set([2, 99]),
      readChildren: (folder) => pages.get(folder),
    });

    expect(rows.map((row) => [row.kind, row.depth])).toEqual([
      ["item", 0],
      ["item", 1],
      ["item", 0],
      ["showMore", 0],
    ]);
    expect(foldersToLoad).toEqual([]);

    const withUnloadedFolder = flattenTreeRows({
      rootFolder: 1,
      expandedFolders: new Set([2]),
      readChildren: (folder) => (folder === 2 ? undefined : pages.get(folder)),
    });
    expect(withUnloadedFolder.rows[1]).toMatchObject({ kind: "loading", depth: 1 });
    expect(withUnloadedFolder.foldersToLoad).toEqual([2]);
  });

  it("shows errors and empty folders as their own rows", () => {
    const { rows } = flattenTreeRows({
      rootFolder: 1,
      expandedFolders: new Set([2]),
      readChildren: (folder) =>
        folder === 2
          ? { status: "error", errorMessage: "Access denied" }
          : loadedPage(1, [createChild(2)]),
    });
    expect(rows[1]).toMatchObject({ kind: "error", errorMessage: "Access denied" });

    const empty = flattenTreeRows({
      rootFolder: 1,
      expandedFolders: new Set([2]),
      readChildren: (folder) =>
        folder === 2 ? loadedPage(2, []) : loadedPage(1, [createChild(2)]),
    });
    expect(empty.rows[1]).toMatchObject({ kind: "emptyFolder" });
  });
});

describe("treeKeyboardAction", () => {
  const { rows } = flattenTreeRows({
    rootFolder: 1,
    expandedFolders: new Set([2]),
    readChildren: (folder) =>
      folder === 1
        ? loadedPage(1, [createChild(2), createChild(3, { kind: "file", hasChildren: false })])
        : loadedPage(2, [createChild(4, { hasChildren: false })]),
  });

  it("moves focus up and down within bounds", () => {
    expect(treeKeyboardAction(rows, 0, "ArrowDown")).toEqual({ kind: "focusRow", rowIndex: 1 });
    expect(treeKeyboardAction(rows, 2, "ArrowDown")).toEqual({ kind: "focusRow", rowIndex: 2 });
    expect(treeKeyboardAction(rows, 0, "ArrowUp")).toEqual({ kind: "focusRow", rowIndex: 0 });
    expect(treeKeyboardAction(rows, -1, "ArrowDown")).toEqual({ kind: "focusRow", rowIndex: 0 });
    expect(treeKeyboardAction(rows, 0, "End")).toEqual({ kind: "focusRow", rowIndex: 2 });
  });

  it("collapses an expanded folder, then steps to the parent", () => {
    expect(treeKeyboardAction(rows, 0, "ArrowLeft")).toEqual({ kind: "collapse", folder: 2 });
    expect(treeKeyboardAction(rows, 1, "ArrowLeft")).toEqual({ kind: "focusRow", rowIndex: 0 });
  });

  it("steps into an expanded folder and expands a collapsed one", () => {
    expect(treeKeyboardAction(rows, 0, "ArrowRight")).toEqual({ kind: "focusRow", rowIndex: 1 });

    const collapsed = flattenTreeRows({
      rootFolder: 1,
      expandedFolders: new Set(),
      readChildren: () => loadedPage(1, [createChild(2)]),
    });
    expect(treeKeyboardAction(collapsed.rows, 0, "ArrowRight")).toEqual({
      kind: "expand",
      folder: 2,
    });
  });

  it("opens folders with Enter and goes up with Backspace", () => {
    expect(treeKeyboardAction(rows, 0, "Enter")).toEqual({ kind: "openFolder", folder: 2 });
    expect(treeKeyboardAction(rows, 2, "Enter")).toEqual({ kind: "none" });
    expect(treeKeyboardAction(rows, 2, "Backspace")).toEqual({ kind: "goUp" });
    expect(treeKeyboardAction([], -1, "Backspace")).toEqual({ kind: "goUp" });
  });

  it("asks to recycle the focused item with Delete, never a synthetic one", () => {
    expect(treeKeyboardAction(rows, 2, "Delete")).toEqual({
      kind: "requestRecycle",
      target: { node: 3, name: "item 3", kind: "file", isSynthetic: false },
    });
    expect(treeKeyboardAction(rows, -1, "Delete")).toEqual({ kind: "none" });

    const synthetic = createChild(9, { flags: { ...createChild(9).flags, synthetic: true } });
    const syntheticRows = flattenTreeRows({
      rootFolder: 1,
      expandedFolders: new Set(),
      readChildren: () => loadedPage(1, [synthetic]),
    }).rows;
    expect(treeKeyboardAction(syntheticRows, 0, "Delete")).toEqual({ kind: "none" });
  });
});

describe("tree column sorting", () => {
  it("sorts by a new column in its natural direction", () => {
    expect(nextSortOrderForColumn("logicalSizeDescending", "name")).toBe("nameAscending");
    expect(nextSortOrderForColumn("nameAscending", "modified")).toBe("modifiedNewestFirst");
  });

  it("flips the direction when the sorted column is clicked again", () => {
    expect(nextSortOrderForColumn("logicalSizeDescending", "size")).toBe("logicalSizeAscending");
    expect(nextSortOrderForColumn("logicalSizeAscending", "size")).toBe("logicalSizeDescending");
  });

  it("reports which column shows the sort indicator", () => {
    expect(columnSortState("nameDescending", "name")).toBe("descending");
    expect(columnSortState("logicalSizeAscending", "size")).toBe("ascending");
    expect(columnSortState("logicalSizeAscending", "share")).toBe("none");
    expect(columnSortState("nameAscending", "files")).toBe("none");
  });
});

describe("buildBreadcrumbTrail", () => {
  const crumbs: BreadcrumbItem[] = ["C:", "Users", "marti", "Videos", "2024"].map(
    (name, index) => ({ node: index, name }),
  );

  it("shows every crumb when they fit", () => {
    const trail = buildBreadcrumbTrail(crumbs, 6);
    expect(trail).toHaveLength(5);
    expect(trail.at(-1)).toMatchObject({ kind: "crumb", isCurrent: true });
    expect(trail[0]).toMatchObject({ kind: "crumb", isCurrent: false });
  });

  it("collapses the middle and keeps the root and the last crumbs", () => {
    const trail = buildBreadcrumbTrail(crumbs, 3);
    expect(trail.map((entry) => (entry.kind === "crumb" ? entry.crumb.name : "…"))).toEqual([
      "C:",
      "…",
      "Videos",
      "2024",
    ]);
    expect(trail[1]).toMatchObject({ kind: "collapsed", hiddenCrumbs: [crumbs[1], crumbs[2]] });
  });

  it("handles a single crumb", () => {
    expect(buildBreadcrumbTrail(crumbs.slice(0, 1), 3)).toEqual([
      { kind: "crumb", crumb: crumbs[0], isCurrent: true },
    ]);
  });
});

describe("describeEmptyFolder", () => {
  const flags = createChild(1).flags;

  it("explains protected folders instead of calling them empty", () => {
    const description = describeEmptyFolder({ ...flags, inaccessible: true });
    expect(description.isWarning).toBe(true);
    expect(description.title).not.toMatch(/empty/i);
  });

  it("calls a plain folder empty, also before its flags are known", () => {
    expect(describeEmptyFolder(flags).title).toBe("This folder is empty");
    expect(describeEmptyFolder(null).title).toBe("This folder is empty");
  });
});
