import type { DuplicateGroup } from "@/lib/backend/generated";
import { fileCategoryForPath } from "../setup/fileCategoryOptions";
import type { RecycleScope } from "./describeRecycleScope";
import { countKeptFiles, type FileToRemove } from "./selectionSummary";

/** The files one confirmation dialog would move, and how the dialog should describe them. */
export type RecycleRequest = {
  scope: RecycleScope;
  filesToRemove: FileToRemove[];
};

/** Whether a single copy of `group` may be moved: the backend keeps at least one other copy. */
export function canRecycleSingleFile(group: DuplicateGroup): boolean {
  return group.files.length >= 2;
}

/** A request to move only `path`, whatever its Keep/Remove choice, or `null` if not allowed. */
export function singleFileRecycleRequest(
  group: DuplicateGroup,
  path: string,
): RecycleRequest | null {
  const file = group.files.find((candidate) => candidate.path === path);
  if (file === undefined || !canRecycleSingleFile(group)) {
    return null;
  }

  return {
    scope: {
      kind: "file",
      remainingCopyCount: group.files.length - 1,
      isOnlyKeptCopy: file.keep && countKeptFiles(group) === 1,
    },
    filesToRemove: [
      { path: file.path, fileSize: group.fileSize, category: fileCategoryForPath(file.path) },
    ],
  };
}

/** A request to move the copies of `group` marked Remove, or `null` when none are. */
export function groupRecycleRequest(group: DuplicateGroup): RecycleRequest | null {
  const markedFiles = group.files.filter((file) => !file.keep);
  if (markedFiles.length === 0) {
    return null;
  }

  return {
    scope: { kind: "group", remainingCopyCount: group.files.length - markedFiles.length },
    filesToRemove: markedFiles.map((file) => ({
      path: file.path,
      fileSize: group.fileSize,
      category: fileCategoryForPath(file.path),
    })),
  };
}
