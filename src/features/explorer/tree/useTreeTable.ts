import { type KeyboardEvent, useCallback, useEffect, useMemo, useSyncExternalStore } from "react";
import type { ScanId } from "@/lib/backend/scans";
import { type ChildrenPage, getChildren, type NodeIndex } from "@/lib/backend/tree";
import { scanQueryCache } from "@/lib/queryCache/scanQueryCache";
import { useRecycleConfirmationStore } from "../actions/useRecycleConfirmationStore";
import { treeQueryKeys } from "../treeQueryKeys";
import { DEFAULT_CHILD_PAGE_LIMIT, useExplorerSelectionStore } from "../useExplorerSelectionStore";
import { useNodeDetails } from "../useTreeQueries";
import { describeEmptyFolder, type EmptyFolderDescription } from "./describeEmptyFolder";
import { flattenTreeRows, type TreeRow } from "./flattenTreeRows";
import { treeKeyboardAction } from "./treeKeyboardNavigation";

const REVEAL_ANIMATION_WINDOW_MILLISECONDS = 450;
/** Only this many rows of a newly expanded folder animate in. */
export const MAXIMUM_ANIMATED_REVEALED_ROWS = 24;

export type TreeTableModel = {
  rows: TreeRow[];
  focusedRowIndex: number;
  selectedNode: NodeIndex | null;
  lastExpandedFolder: NodeIndex | null;
  /** Set when the shown folder has nothing to list, with the reason. */
  emptyFolder: EmptyFolderDescription | null;
  /** Opens the parent of the shown folder; `null` at the scanned root. */
  goUp: (() => void) | null;
  handleKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  selectRow: (rowIndex: number) => void;
  toggleFolder: (folder: NodeIndex) => void;
  openFolder: (folder: NodeIndex) => void;
  showMoreChildren: (folder: NodeIndex) => void;
  retryFolder: (folder: NodeIndex) => void;
};

/**
 * Builds the rows of the tree table from cached children pages of the shown folder and every
 * expanded folder below it, loads what is missing and maps keys to tree actions.
 */
export function useTreeTable(
  scanId: ScanId,
  shownFolder: NodeIndex,
  parentOfShownFolder: NodeIndex | null,
): TreeTableModel {
  const expandedFolders = useExplorerSelectionStore((state) => state.expandedFolders);
  const childPageLimits = useExplorerSelectionStore((state) => state.childPageLimits);
  const sortOrder = useExplorerSelectionStore((state) => state.sortOrder);
  const selectedNode = useExplorerSelectionStore((state) => state.selectedNode);
  const lastExpandedFolder = useExplorerSelectionStore((state) => state.lastExpandedFolder);
  const selectNode = useExplorerSelectionStore((state) => state.selectNode);
  const toggleFolderExpanded = useExplorerSelectionStore((state) => state.toggleFolderExpanded);
  const setFolderExpanded = useExplorerSelectionStore((state) => state.setFolderExpanded);
  const navigateToNode = useExplorerSelectionStore((state) => state.navigateToNode);
  const showMoreChildren = useExplorerSelectionStore((state) => state.showMoreChildren);
  const clearLastExpandedFolder = useExplorerSelectionStore(
    (state) => state.clearLastExpandedFolder,
  );
  const requestRecycle = useRecycleConfirmationStore((state) => state.requestRecycle);

  const cacheVersion = useSyncExternalStore(scanQueryCache.subscribe, scanQueryCache.getVersion);

  const childrenKey = useCallback(
    (folder: NodeIndex) =>
      treeQueryKeys.childrenPage(
        scanId,
        folder,
        sortOrder,
        childPageLimits.get(folder) ?? DEFAULT_CHILD_PAGE_LIMIT,
      ),
    [scanId, sortOrder, childPageLimits],
  );

  const flattenedTree = useMemo(() => {
    void cacheVersion;
    return flattenTreeRows({
      rootFolder: shownFolder,
      expandedFolders,
      readChildren: (folder) => scanQueryCache.read<ChildrenPage>(childrenKey(folder)),
    });
  }, [cacheVersion, shownFolder, expandedFolders, childrenKey]);

  const { rows, foldersToLoad } = flattenedTree;
  const isEmptyFolder = rows.length === 0 || (rows.length === 1 && rows[0]?.kind === "emptyFolder");
  const shownFolderDetails = useNodeDetails(scanId, isEmptyFolder ? shownFolder : null).state;
  const emptyFolder = isEmptyFolder
    ? describeEmptyFolder(
        shownFolderDetails.status === "success" ? shownFolderDetails.data.flags : null,
      )
    : null;

  useEffect(() => {
    for (const folder of foldersToLoad) {
      const limit = childPageLimits.get(folder) ?? DEFAULT_CHILD_PAGE_LIMIT;
      scanQueryCache.ensureLoaded(childrenKey(folder), scanId, () =>
        getChildren(scanId, folder, sortOrder, 0, limit),
      );
    }
  }, [foldersToLoad, childrenKey, childPageLimits, scanId, sortOrder]);

  useEffect(() => {
    if (lastExpandedFolder === null) {
      return;
    }
    const timeoutId = window.setTimeout(
      clearLastExpandedFolder,
      REVEAL_ANIMATION_WINDOW_MILLISECONDS,
    );
    return () => window.clearTimeout(timeoutId);
  }, [lastExpandedFolder, clearLastExpandedFolder]);

  const focusedRowIndex = useMemo(
    () =>
      selectedNode === null
        ? -1
        : rows.findIndex((row) => row.kind === "item" && row.item.node === selectedNode),
    [rows, selectedNode],
  );

  const selectRow = useCallback(
    (rowIndex: number) => {
      const row = rows[rowIndex];
      if (row?.kind === "item") {
        selectNode(row.item.node);
      }
    },
    [rows, selectNode],
  );

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      if (event.altKey || event.ctrlKey || event.metaKey) {
        return;
      }
      const action = treeKeyboardAction(rows, focusedRowIndex, event.key);

      switch (action.kind) {
        case "none":
          return;
        case "focusRow":
          selectRow(action.rowIndex);
          break;
        case "expand":
          setFolderExpanded(action.folder, true);
          break;
        case "collapse":
          setFolderExpanded(action.folder, false);
          break;
        case "openFolder":
          navigateToNode(action.folder);
          break;
        case "showMore":
          showMoreChildren(action.folder);
          break;
        case "requestRecycle":
          requestRecycle(scanId, [action.target.node]);
          break;
        case "goUp":
          if (parentOfShownFolder === null) {
            return;
          }
          navigateToNode(parentOfShownFolder, { selectedNode: shownFolder });
          break;
      }
      event.preventDefault();
    },
    [
      rows,
      focusedRowIndex,
      selectRow,
      setFolderExpanded,
      navigateToNode,
      showMoreChildren,
      parentOfShownFolder,
      shownFolder,
      requestRecycle,
      scanId,
    ],
  );

  const goUp = useMemo(() => {
    if (parentOfShownFolder === null) {
      return null;
    }
    return () => navigateToNode(parentOfShownFolder, { selectedNode: shownFolder });
  }, [parentOfShownFolder, navigateToNode, shownFolder]);

  const retryFolder = useCallback(
    (folder: NodeIndex) => scanQueryCache.retry(childrenKey(folder)),
    [childrenKey],
  );

  return {
    rows,
    focusedRowIndex,
    selectedNode,
    lastExpandedFolder,
    emptyFolder,
    goUp,
    handleKeyDown,
    selectRow,
    toggleFolder: toggleFolderExpanded,
    openFolder: navigateToNode,
    showMoreChildren,
    retryFolder,
  };
}
