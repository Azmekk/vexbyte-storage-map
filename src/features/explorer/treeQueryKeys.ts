import type { ScanId } from "@/lib/backend/scans";
import type { ChildSortOrder, FileCategory, NodeIndex } from "@/lib/backend/tree";

/** Cache keys of the tree queries. Every key starts with the scan id. */
export const treeQueryKeys = {
  childrenPage: (scanId: ScanId, node: NodeIndex, sortOrder: ChildSortOrder, limit: number) =>
    `${scanId}:children:${node}:${sortOrder}:${limit}`,
  nodeDetails: (scanId: ScanId, node: NodeIndex) => `${scanId}:details:${node}`,
  pathToRoot: (scanId: ScanId, node: NodeIndex) => `${scanId}:path:${node}`,
  visualizationSubtree: (
    scanId: ScanId,
    node: NodeIndex,
    maxDepth: number,
    minimumShareOfParent: number,
  ) => `${scanId}:subtree:${node}:${maxDepth}:${minimumShareOfParent}`,
  largestFiles: (
    scanId: ScanId,
    node: NodeIndex,
    categoryFilter: FileCategory | null,
    limit: number,
  ) => `${scanId}:largest:${node}:${categoryFilter ?? "any"}:${limit}`,
  fileTypeBreakdown: (scanId: ScanId, node: NodeIndex) => `${scanId}:types:${node}`,
};
