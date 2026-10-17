import { useCallback } from "react";
import type { ScanId } from "@/lib/backend/scans";
import {
  type BreadcrumbItem,
  type ChildrenPage,
  type ChildSortOrder,
  type FileCategory,
  type FileSummary,
  type FileTypeBreakdown,
  getChildren,
  getFileTypeBreakdown,
  getLargestFiles,
  getNodeDetails,
  getPathToRoot,
  getVisualizationSubtree,
  type NodeDetails,
  type NodeIndex,
  type VisualizationNode,
} from "@/lib/backend/tree";
import { type ScanQueryResult, useScanQuery } from "@/lib/queryCache/useScanQuery";
import { treeQueryKeys } from "./treeQueryKeys";
import { useExplorerSelectionStore } from "./useExplorerSelectionStore";

/** The user's choice whether online-only cloud files are listed, shared by every tree query. */
function useOnlineOnlyVisibility() {
  return useExplorerSelectionStore((state) => state.onlineOnlyVisibility);
}

export function useChildrenPage(
  scanId: ScanId,
  node: NodeIndex | null,
  sortOrder: ChildSortOrder,
  limit: number,
): ScanQueryResult<ChildrenPage> {
  const onlineOnly = useOnlineOnlyVisibility();
  const key =
    node === null ? null : treeQueryKeys.childrenPage(scanId, node, sortOrder, limit, onlineOnly);
  const load = useCallback(
    () => getChildren(scanId, node ?? 0, sortOrder, 0, limit, onlineOnly),
    [scanId, node, sortOrder, limit, onlineOnly],
  );
  return useScanQuery(scanId, key, load);
}

export function useNodeDetails(
  scanId: ScanId,
  node: NodeIndex | null,
): ScanQueryResult<NodeDetails> {
  const onlineOnly = useOnlineOnlyVisibility();
  const key = node === null ? null : treeQueryKeys.nodeDetails(scanId, node, onlineOnly);
  const load = useCallback(
    () => getNodeDetails(scanId, node ?? 0, onlineOnly),
    [scanId, node, onlineOnly],
  );
  return useScanQuery(scanId, key, load);
}

export function usePathToRoot(
  scanId: ScanId,
  node: NodeIndex | null,
): ScanQueryResult<BreadcrumbItem[]> {
  const key = node === null ? null : treeQueryKeys.pathToRoot(scanId, node);
  const load = useCallback(() => getPathToRoot(scanId, node ?? 0), [scanId, node]);
  return useScanQuery(scanId, key, load);
}

/** Depth and pruning shared by the treemap and the sunburst, so they share one cached subtree. */
export const VISUALIZATION_MAX_DEPTH = 8;
export const VISUALIZATION_MINIMUM_SHARE_OF_PARENT = 0.002;

export function useVisualizationSubtree(
  scanId: ScanId,
  node: NodeIndex | null,
): ScanQueryResult<VisualizationNode> {
  const onlineOnly = useOnlineOnlyVisibility();
  const key =
    node === null
      ? null
      : treeQueryKeys.visualizationSubtree(
          scanId,
          node,
          VISUALIZATION_MAX_DEPTH,
          VISUALIZATION_MINIMUM_SHARE_OF_PARENT,
          onlineOnly,
        );
  const load = useCallback(
    () =>
      getVisualizationSubtree(
        scanId,
        node ?? 0,
        VISUALIZATION_MAX_DEPTH,
        VISUALIZATION_MINIMUM_SHARE_OF_PARENT,
        onlineOnly,
      ),
    [scanId, node, onlineOnly],
  );
  return useScanQuery(scanId, key, load);
}

export function useLargestFiles(
  scanId: ScanId,
  node: NodeIndex | null,
  categoryFilter: FileCategory | null,
  limit: number,
): ScanQueryResult<FileSummary[]> {
  const onlineOnly = useOnlineOnlyVisibility();
  const key =
    node === null
      ? null
      : treeQueryKeys.largestFiles(scanId, node, categoryFilter, limit, onlineOnly);
  const load = useCallback(
    () => getLargestFiles(scanId, node ?? 0, categoryFilter, limit, onlineOnly),
    [scanId, node, categoryFilter, limit, onlineOnly],
  );
  return useScanQuery(scanId, key, load);
}

export function useFileTypeBreakdown(
  scanId: ScanId,
  node: NodeIndex | null,
): ScanQueryResult<FileTypeBreakdown> {
  const onlineOnly = useOnlineOnlyVisibility();
  const key = node === null ? null : treeQueryKeys.fileTypeBreakdown(scanId, node, onlineOnly);
  const load = useCallback(
    () => getFileTypeBreakdown(scanId, node ?? 0, onlineOnly),
    [scanId, node, onlineOnly],
  );
  return useScanQuery(scanId, key, load);
}
