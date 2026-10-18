import { platformWording } from "@/lib/platform/platformWording";
import { pluralize } from "../formatting/pluralize";

/** Which files a Recycle Bin confirmation is about, with what stays behind in the group. */
export type RecycleScope =
  | {
      kind: "file";
      /** Copies of the group that are not moved. */
      remainingCopyCount: number;
      /** The file is the group's only copy marked Keep, so the others are marked Keep afterwards. */
      isOnlyKeptCopy: boolean;
    }
  | { kind: "group"; remainingCopyCount: number }
  | { kind: "allMarked" };

export type RecycleScopeText = {
  title: string;
  description: string;
  /** What happens to the copies left behind, or `null` when there is nothing specific to say. */
  note: string | null;
  confirmLabel: string;
};

/** The size and count of the files in scope, or `null` while the list is still loading. */
export type RecycleScopeAmount = { fileCount: number; readableSize: string } | null;

function describeFileNote(remainingCopyCount: number, isOnlyKeptCopy: boolean): string {
  if (remainingCopyCount === 1) {
    return "The other copy stays where it is, and the group leaves the list.";
  }

  const staysText = `The other ${pluralize(remainingCopyCount, "copy", "copies")} stay where they are`;
  return isOnlyKeptCopy
    ? `This is the copy marked Keep. ${staysText} and are all marked Keep afterwards.`
    : `${staysText}.`;
}

function describeGroupNote(remainingCopyCount: number): string {
  return remainingCopyCount === 1
    ? "The copy marked Keep stays where it is, and the group leaves the list."
    : `The ${pluralize(remainingCopyCount, "copy", "copies")} marked Keep stay where they are.`;
}

function describeAllMarkedTitle(amount: RecycleScopeAmount): string {
  if (amount === null) {
    return `Move all marked files to ${platformWording().binName}`;
  }

  return amount.fileCount === 1
    ? `Move 1 marked file (${amount.readableSize}) to ${platformWording().binName}`
    : `Move all ${pluralize(amount.fileCount, "marked file", "marked files")} (${amount.readableSize}) to ${platformWording().binName}`;
}

/** The title, explanation and confirm label of the Recycle Bin dialog for one scope. */
export function describeRecycleScope(
  scope: RecycleScope,
  amount: RecycleScopeAmount,
): RecycleScopeText {
  const confirmLabel =
    amount === null
      ? `Move to ${platformWording().binName}`
      : `Move ${pluralize(amount.fileCount, "file", "files")} to ${platformWording().binName}`;
  const amountText =
    amount === null
      ? ""
      : ` ${pluralize(amount.fileCount, "file", "files")} (${amount.readableSize})`;

  switch (scope.kind) {
    case "file":
      return {
        title: `Move${amountText} to ${platformWording().binName}`,
        description: `It goes to the ${platformWording().binName}, not away for good: you can restore it from there.`,
        note: describeFileNote(scope.remainingCopyCount, scope.isOnlyKeptCopy),
        confirmLabel,
      };
    case "group":
      return {
        title:
          amount === null
            ? `Move this group's marked files to ${platformWording().binName}`
            : `Move ${pluralize(amount.fileCount, "marked file", "marked files")} (${amount.readableSize}) from this group to ${platformWording().binName}`,
        description: `These copies go to the ${platformWording().binName}, not away for good: you can restore any of them from there.`,
        note: describeGroupNote(scope.remainingCopyCount),
        confirmLabel,
      };
    case "allMarked":
      return {
        title: describeAllMarkedTitle(amount),
        description: `These copies go to the ${platformWording().binName}, not away for good: you can restore any of them from there. At least one copy of every file stays where it is.`,
        note: null,
        confirmLabel,
      };
  }
}

/** The label of a group's button: "Move 2 marked (4.0 MB) to Recycle Bin". */
export function describeGroupRecycleButton(markedCount: number, readableSize: string): string {
  return `Move ${new Intl.NumberFormat("en-US").format(markedCount)} marked (${readableSize}) to ${platformWording().binName}`;
}
