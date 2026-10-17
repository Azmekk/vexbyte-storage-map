import type { ScanId } from "@/lib/backend/scans";
import type {
  ChildSortOrder,
  FileCategory,
  NodeIndex,
  OnlineOnlyVisibility,
} from "@/lib/backend/tree";

/**
 * Cache keys of the tree queries. Every key starts with the scan id, and every query whose answer
 * depends on whether online-only files are listed ends with that choice.
 */
export const treeQueryKeys = {
  childrenPage: (
    scanId: ScanId,
    node: NodeIndex,
    sortOrder: ChildSortOrder,
    limit: number,
    onlineOnly: OnlineOnlyVisibility,
  ) => `${scanId}:children:${node}:${sortOrder}:${limit}:${onlineOnly}`,
  nodeDetails: (scanId: ScanId, node: NodeIndex, onlineOnly: OnlineOnlyVisibility) =>
    `${scanId}:details:${node}:${onlineOnly}`,
  pathToRoot: (scanId: ScanId, node: NodeIndex) => `${scanId}:path:${node}`,
  visualizationSubtree: (
    scanId: ScanId,
    node: NodeIndex,
    maxDepth: number,
    minimumShareOfParent: number,
    onlineOnly: OnlineOnlyVisibility,
  ) => `${scanId}:subtree:${node}:${maxDepth}:${minimumShareOfParent}:${onlineOnly}`,
  largestFiles: (
    scanId: ScanId,
    node: NodeIndex,
    categoryFilter: FileCategory | null,
    limit: number,
    onlineOnly: OnlineOnlyVisibility,
  ) => `${scanId}:largest:${node}:${categoryFilter ?? "any"}:${limit}:${onlineOnly}`,
  fileTypeBreakdown: (scanId: ScanId, node: NodeIndex, onlineOnly: OnlineOnlyVisibility) =>
    `${scanId}:types:${node}:${onlineOnly}`,
};
