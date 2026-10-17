import type { RecycleFailureReason, TreeRecycleResult } from "@/lib/backend/fileActions";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";

const FAILURE_REASON_TEXT: Record<RecycleFailureReason, string> = {
  notAbsolute: "its location could not be resolved",
  notFound: "it no longer exists",
  noRecycleBinOnDrive: "this drive has no Recycle Bin, so it was left untouched",
  nameNotSupportedByShell: "its name ends with a dot or a space, so it was left untouched",
  stillInPlace: "Windows kept it in place",
};

export type RecycleOutcomeMessages = {
  success: { title: string; description: string } | null;
  failure: { title: string; description: string } | null;
};

function itemCountText(count: number): string {
  return count === 1 ? "1 item" : `${count} items`;
}

/** The toasts shown after a Recycle Bin action, using the freed size of the tree removal. */
export function describeRecycleOutcome(result: TreeRecycleResult): RecycleOutcomeMessages {
  const movedCount = result.recycleResult.moved.length;
  const failures = result.recycleResult.failed;

  const success =
    movedCount === 0
      ? null
      : {
          title: `Moved ${itemCountText(movedCount)} to the Recycle Bin`,
          description: `${formatBytesAsReadableSize(result.removal.freedLogicalSize)} freed. You can restore them from the Recycle Bin.`,
        };

  const firstFailure = failures[0];
  const failure =
    firstFailure === undefined
      ? null
      : {
          title: `${itemCountText(failures.length)} could not be moved`,
          description: `${firstFailure.path}: ${FAILURE_REASON_TEXT[firstFailure.reason]}${firstFailure.detail === null ? "" : ` (${firstFailure.detail})`}.`,
        };

  return { success, failure };
}
