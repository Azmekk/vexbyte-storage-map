import type { ChildrenPage, ChildSummary, NodeIndex } from "@/lib/backend/tree";
import type { QueryState } from "@/lib/queryCache/scanQueryCache";

/** One visible line of the tree table. */
export type TreeRow =
  | {
      kind: "item";
      key: string;
      item: ChildSummary;
      depth: number;
      parent: NodeIndex;
      isExpanded: boolean;
    }
  | { kind: "loading"; key: string; depth: number; parent: NodeIndex }
  | { kind: "error"; key: string; depth: number; parent: NodeIndex; errorMessage: string }
  | { kind: "showMore"; key: string; depth: number; parent: NodeIndex; hiddenChildCount: number }
  | { kind: "emptyFolder"; key: string; depth: number; parent: NodeIndex };

export type FlattenTreeRowsInput = {
  rootFolder: NodeIndex;
  expandedFolders: ReadonlySet<NodeIndex>;
  readChildren: (folder: NodeIndex) => QueryState<ChildrenPage> | undefined;
};

export type FlattenedTree = {
  rows: TreeRow[];
  /** Folders whose children are shown but not loaded yet (or stale). */
  foldersToLoad: NodeIndex[];
};

/**
 * Turns the loaded children pages into the rows the virtualized table shows, depth first,
 * expanding only folders in `expandedFolders`. The shown folder's own children are at depth 0.
 */
export function flattenTreeRows({
  rootFolder,
  expandedFolders,
  readChildren,
}: FlattenTreeRowsInput): FlattenedTree {
  const rows: TreeRow[] = [];
  const foldersToLoad: NodeIndex[] = [];

  const appendChildrenOf = (folder: NodeIndex, depth: number) => {
    const childrenState = readChildren(folder);

    if (childrenState === undefined || childrenState.status === "loading") {
      foldersToLoad.push(folder);
      rows.push({ kind: "loading", key: `loading-${folder}`, depth, parent: folder });
      return;
    }

    if (childrenState.status === "error") {
      rows.push({
        kind: "error",
        key: `error-${folder}`,
        depth,
        parent: folder,
        errorMessage: childrenState.errorMessage,
      });
      return;
    }

    if (childrenState.isStale) {
      foldersToLoad.push(folder);
    }

    const page = childrenState.data;

    if (page.children.length === 0 && depth > 0) {
      rows.push({ kind: "emptyFolder", key: `empty-${folder}`, depth, parent: folder });
      return;
    }

    for (const child of page.children) {
      const isExpanded = child.hasChildren && expandedFolders.has(child.node);
      rows.push({
        kind: "item",
        key: `item-${child.node}`,
        item: child,
        depth,
        parent: folder,
        isExpanded,
      });

      if (isExpanded) {
        appendChildrenOf(child.node, depth + 1);
      }
    }

    const hiddenChildCount = page.totalChildCount - page.offset - page.children.length;
    if (hiddenChildCount > 0) {
      rows.push({
        kind: "showMore",
        key: `more-${folder}`,
        depth,
        parent: folder,
        hiddenChildCount,
      });
    }
  };

  appendChildrenOf(rootFolder, 0);
  return { rows, foldersToLoad };
}
