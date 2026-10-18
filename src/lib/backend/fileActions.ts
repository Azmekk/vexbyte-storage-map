import { convertFileSrc } from "@tauri-apps/api/core";
import { platformWording } from "@/lib/platform/platformWording";
import {
  commands,
  type DuplicateSearchId,
  type FileActionCommandError,
  type NodeIndex,
  type ScanId,
  type ThumbnailHandle,
  type TreeRecycleResult,
} from "./generated";
import { unwrapCommandResult } from "./unwrapCommandResult";

export type {
  FileActionCommandError,
  RecycleFailure,
  RecycleFailureReason,
  RecycleResult,
  RemovalSummary,
  ThumbnailHandle,
  TreeRecycleResult,
} from "./generated";

function describeFileActionCommandError(commandError: FileActionCommandError): string {
  switch (commandError.kind) {
    case "scanNotFound":
      return "This scan is no longer available. Please scan again.";
    case "scanNotFinished":
      return "This scan has no results yet.";
    case "nodeNotFound":
      return "This item does not exist in the scan.";
    case "nodeRemoved":
      return `This item was already moved to the ${platformWording().binName}.`;
    case "nodeHasNoPath":
      return "This item is a summary of several files and has no single location on disk.";
    case "cannotRecycleScanRoot":
      return `The scanned folder itself cannot be moved to the ${platformWording().binName}.`;
    case "searchNotFound":
      return "This duplicate search is no longer available. Please search again.";
    case "fileNotInSearch":
      return `${commandError.path} is not part of this duplicate search.`;
    case "revealFailed":
      return `Could not show ${commandError.path} in ${platformWording().fileManagerName}. ${commandError.details}`;
    case "openFailed":
      return `Could not open ${commandError.path}. ${commandError.details}`;
    case "recycleFailed":
      return `The move to the ${platformWording().binName} failed. ${commandError.details}`;
    case "thumbnailFailed":
      return `Could not create a preview of ${commandError.path}.`;
    case "backgroundTaskFailed":
      return "The action stopped unexpectedly. Please try again.";
  }
}

/** Opens an Explorer window with the item selected. */
export async function revealInExplorer(scanId: ScanId, node: NodeIndex): Promise<void> {
  const result = await commands.revealInExplorer(scanId, node);
  unwrapCommandResult(result, describeFileActionCommandError);
}

/** Opens the item with its default application, or a folder in Explorer. */
export async function openWithDefaultApp(scanId: ScanId, node: NodeIndex): Promise<void> {
  const result = await commands.openWithDefaultApp(scanId, node);
  unwrapCommandResult(result, describeFileActionCommandError);
}

/** Opens an Explorer window with one copy of a duplicate selected. */
export async function revealDuplicateInExplorer(
  searchId: DuplicateSearchId,
  path: string,
): Promise<void> {
  const result = await commands.revealDuplicateInExplorer(searchId, path);
  unwrapCommandResult(result, describeFileActionCommandError);
}

/** Opens one copy of a duplicate with its default application. */
export async function openDuplicateWithDefaultApp(
  searchId: DuplicateSearchId,
  path: string,
): Promise<void> {
  const result = await commands.openDuplicateWithDefaultApp(searchId, path);
  unwrapCommandResult(result, describeFileActionCommandError);
}

/**
 * Moves items to the Recycle Bin and takes the moved ones out of the tree. Cached children
 * pages, subtrees and breakdowns of this scan are stale afterwards.
 */
export async function moveToRecycleBin(
  scanId: ScanId,
  nodes: NodeIndex[],
): Promise<TreeRecycleResult> {
  const result = await commands.moveToRecycleBin(scanId, nodes);
  return unwrapCommandResult(result, describeFileActionCommandError);
}

/** A cached PNG thumbnail of `path`, about `sizePixels` square. */
export async function getThumbnail(path: string, sizePixels: number): Promise<ThumbnailHandle> {
  const result = await commands.getThumbnail(path, sizePixels);
  return unwrapCommandResult(result, describeFileActionCommandError);
}

/** The URL an `<img>` loads a thumbnail from, through the asset protocol. */
export function thumbnailImageUrl(thumbnail: ThumbnailHandle): string {
  return convertFileSrc(thumbnail.pngPath);
}
