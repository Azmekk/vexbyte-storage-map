import {
  type BreadcrumbItem,
  type ChildrenPage,
  type ChildSortOrder,
  commands,
  type FileCategory,
  type FileSummary,
  type FileTypeBreakdown,
  type NodeDetails,
  type NodeIndex,
  type ScanId,
  type SearchResult,
  type TreeCommandError,
  type VisualizationNode,
} from "./generated";
import { unwrapCommandResult } from "./unwrapCommandResult";

export type {
  BreadcrumbItem,
  ChildrenPage,
  ChildSortOrder,
  ChildSummary,
  FileCategory,
  FileCategoryShare,
  FileExtensionShare,
  FileSummary,
  FileTypeBreakdown,
  NodeDetails,
  NodeFlagSummary,
  NodeIndex,
  NodeKind,
  SearchResult,
  TreeCommandError,
  VisualizationNode,
  VisualizationNodeKind,
} from "./generated";

function describeTreeCommandError(commandError: TreeCommandError): string {
  switch (commandError.kind) {
    case "scanNotFound":
      return "This scan is no longer available. Please scan again.";
    case "scanNotFinished":
      return "This scan has no results yet.";
    case "nodeNotFound":
      return "This item does not exist in the scan.";
    case "nodeRemoved":
      return "This item was moved to the Recycle Bin.";
    case "backgroundTaskFailed":
      return "The query stopped unexpectedly. Please try again.";
  }
}

/** One page of a folder's children in the requested order. */
export async function getChildren(
  scanId: ScanId,
  node: NodeIndex,
  sort: ChildSortOrder,
  offset: number,
  limit: number,
): Promise<ChildrenPage> {
  const result = await commands.getChildren(scanId, node, sort, offset, limit);
  return unwrapCommandResult(result, describeTreeCommandError);
}

/** Full path, sizes, counts, dates and flags of one item. */
export async function getNodeDetails(scanId: ScanId, node: NodeIndex): Promise<NodeDetails> {
  const result = await commands.getNodeDetails(scanId, node);
  return unwrapCommandResult(result, describeTreeCommandError);
}

/** The breadcrumb trail from the scanned root down to `node`. */
export async function getPathToRoot(scanId: ScanId, node: NodeIndex): Promise<BreadcrumbItem[]> {
  const result = await commands.getPathToRoot(scanId, node);
  return unwrapCommandResult(result, describeTreeCommandError);
}

/** A pruned nested view for the treemap and sunburst, capped at about 5,000 nodes. */
export async function getVisualizationSubtree(
  scanId: ScanId,
  node: NodeIndex,
  maxDepth: number,
  minimumShareOfParent: number,
): Promise<VisualizationNode> {
  const result = await commands.getVisualizationSubtree(
    scanId,
    node,
    maxDepth,
    minimumShareOfParent,
  );
  return unwrapCommandResult(result, describeTreeCommandError);
}

/** The largest files under `node`, optionally of one category. */
export async function getLargestFiles(
  scanId: ScanId,
  node: NodeIndex,
  categoryFilter: FileCategory | null,
  limit: number,
): Promise<FileSummary[]> {
  const result = await commands.getLargestFiles(scanId, node, categoryFilter, limit);
  return unwrapCommandResult(result, describeTreeCommandError);
}

/** Space per file category and per extension under `node`. */
export async function getFileTypeBreakdown(
  scanId: ScanId,
  node: NodeIndex,
): Promise<FileTypeBreakdown> {
  const result = await commands.getFileTypeBreakdown(scanId, node);
  return unwrapCommandResult(result, describeTreeCommandError);
}

/** Items whose name contains `query`, ignoring case, largest first. */
export async function searchNodes(
  scanId: ScanId,
  query: string,
  limit: number,
): Promise<SearchResult[]> {
  const result = await commands.searchNodes(scanId, query, limit);
  return unwrapCommandResult(result, describeTreeCommandError);
}
