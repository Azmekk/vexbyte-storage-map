import type {
  DuplicateGroup,
  DuplicateSelectionSummary,
  FileCategory,
} from "@/lib/backend/generated";
import { platformWording } from "@/lib/platform/platformWording";
import { fileCategoryForPath } from "../setup/fileCategoryOptions";

export type FileToRemove = {
  path: string;
  fileSize: number;
  category: FileCategory | null;
};

export function countFilesMarkedForRemoval(group: DuplicateGroup): number {
  return group.files.filter((file) => !file.keep).length;
}

export function countKeptFiles(group: DuplicateGroup): number {
  return group.files.filter((file) => file.keep).length;
}

/** Whether a copy may be switched to "remove": not when it is the only copy still kept. */
export function canMarkForRemoval(group: DuplicateGroup, path: string): boolean {
  const file = group.files.find((candidate) => candidate.path === path);

  if (file === undefined) {
    return false;
  }

  return !file.keep || countKeptFiles(group) > 1;
}

/**
 * The selection after one group changed by hand. `setFileKeep` returns only the changed group,
 * so the totals over all groups are adjusted by the difference.
 */
export function applyGroupChangeToSelection(
  selection: DuplicateSelectionSummary,
  previousGroup: DuplicateGroup,
  updatedGroup: DuplicateGroup,
): DuplicateSelectionSummary {
  const previousRemovalCount = countFilesMarkedForRemoval(previousGroup);
  const updatedRemovalCount = countFilesMarkedForRemoval(updatedGroup);
  const removalCountDifference = updatedRemovalCount - previousRemovalCount;
  const removalBytesDifference =
    updatedRemovalCount * updatedGroup.fileSize - previousRemovalCount * previousGroup.fileSize;

  return {
    filesSelectedForRemoval: Math.max(
      0,
      selection.filesSelectedForRemoval + removalCountDifference,
    ),
    bytesSelectedForRemoval: Math.max(
      0,
      selection.bytesSelectedForRemoval + removalBytesDifference,
    ),
  };
}

/** Every copy marked for removal across `groups`, in group order. */
export function collectFilesToRemove(groups: readonly DuplicateGroup[]): FileToRemove[] {
  return groups.flatMap((group) =>
    group.files
      .filter((file) => !file.keep)
      .map((file) => ({
        path: file.path,
        fileSize: group.fileSize,
        category: fileCategoryForPath(file.path),
      })),
  );
}

export function totalBytesOf(files: readonly FileToRemove[]): number {
  return files.reduce((total, file) => total + file.fileSize, 0);
}

/** The label of the footer button: "Move all 12 marked files (3.4 GB) to Recycle Bin". */
export function describeRecycleButton(fileCount: number, readableSize: string): string {
  if (fileCount === 1) {
    return `Move 1 marked file (${readableSize}) to ${platformWording().binName}`;
  }
  return `Move all ${fileCount.toLocaleString("en-US")} marked files (${readableSize}) to ${platformWording().binName}`;
}
