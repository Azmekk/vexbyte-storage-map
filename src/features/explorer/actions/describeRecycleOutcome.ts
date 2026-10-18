import type { RecycleFailureReason, TreeRecycleResult } from "@/lib/backend/fileActions";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { platformWording } from "@/lib/platform/platformWording";

function failureReasonText(reason: RecycleFailureReason): string {
  const { binName, systemName } = platformWording();
  switch (reason) {
    case "notAbsolute":
      return "its location could not be resolved";
    case "notFound":
      return "it no longer exists";
    case "noRecycleBinOnDrive":
      return `this drive has no ${binName}, so it was left untouched`;
    case "nameNotSupportedByShell":
      return "its name ends with a dot or a space, so it was left untouched";
    case "stillInPlace":
      return `${systemName} kept it in place`;
  }
}

export type RecycleOutcomeMessages = {
  success: { title: string; description: string } | null;
  failure: { title: string; description: string } | null;
};

function itemCountText(count: number): string {
  return count === 1 ? "1 item" : `${count} items`;
}

/** The toasts shown after a move to the bin, using the freed size of the tree removal. */
export function describeRecycleOutcome(result: TreeRecycleResult): RecycleOutcomeMessages {
  const { binName } = platformWording();
  const movedCount = result.recycleResult.moved.length;
  const failures = result.recycleResult.failed;

  const success =
    movedCount === 0
      ? null
      : {
          title: `Moved ${itemCountText(movedCount)} to the ${binName}`,
          description: `${formatBytesAsReadableSize(result.removal.freedLogicalSize)} freed. You can restore them from the ${binName}.`,
        };

  const firstFailure = failures[0];
  const failure =
    firstFailure === undefined
      ? null
      : {
          title: `${itemCountText(failures.length)} could not be moved`,
          description: `${firstFailure.path}: ${failureReasonText(firstFailure.reason)}${firstFailure.detail === null ? "" : ` (${firstFailure.detail})`}.`,
        };

  return { success, failure };
}
