import type {
  DuplicateRecycleResult,
  FileChangeReason,
  RecycleFailureReason,
} from "@/lib/backend/generated";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { pluralize } from "../formatting/pluralize";

const FILE_CHANGE_REASON_TEXT: Record<FileChangeReason, string> = {
  missing: "It was moved, renamed or deleted since the search.",
  sizeChanged: "Its size changed since the search.",
  modifiedTimeChanged: "It was edited since the search.",
  becameOnlineOnly: "It became an online-only file since the search.",
  noKeptCopyLeft: "No unchanged kept copy is left, so it was kept to avoid losing the content.",
};

const RECYCLE_FAILURE_REASON_TEXT: Record<RecycleFailureReason, string> = {
  notAbsolute: "The path is not a full path.",
  notFound: "The file is no longer there.",
  noRecycleBinOnDrive:
    "This drive has no Recycle Bin, so the file would be deleted for good. It was left alone.",
  nameNotSupportedByShell:
    "Its name ends with a dot or a space, which Windows cannot handle safely. It was left alone.",
  stillInPlace: "Windows could not move it. It is still in place.",
};

export function describeFileChangeReason(reason: FileChangeReason): string {
  return FILE_CHANGE_REASON_TEXT[reason];
}

export function describeRecycleFailure(
  reason: RecycleFailureReason,
  detail: string | null,
): string {
  const reasonText = RECYCLE_FAILURE_REASON_TEXT[reason];
  return detail !== null && detail.trim().length > 0
    ? `${reasonText} ${detail.trim()}`
    : reasonText;
}

export type RecycleToast = {
  kind: "success" | "warning" | "error";
  title: string;
  description: string | null;
};

/** The toast shown after a recycle: what moved, how much space was freed, what was skipped. */
export function describeRecycleToast(result: DuplicateRecycleResult): RecycleToast {
  const skippedCount = result.failed.length + result.changed.length;
  const skippedText =
    skippedCount > 0 ? `${pluralize(skippedCount, "file was", "files were")} skipped.` : null;

  if (result.moved.length === 0) {
    return {
      kind: "error",
      title: "Nothing was moved to the Recycle Bin",
      description: skippedText,
    };
  }

  const title = `Moved ${pluralize(result.moved.length, "file", "files")} to the Recycle Bin`;
  const restoreText =
    result.moved.length === 1
      ? "You can restore it from the Recycle Bin."
      : "You can restore them from the Recycle Bin.";
  const freedText = `Freed ${formatBytesAsReadableSize(result.bytesFreed)}. ${restoreText}`;

  return {
    kind: skippedCount > 0 ? "warning" : "success",
    title,
    description: skippedText === null ? freedText : `${freedText} ${skippedText}`,
  };
}

export function hasSkippedFiles(result: DuplicateRecycleResult): boolean {
  return result.failed.length > 0 || result.changed.length > 0;
}
