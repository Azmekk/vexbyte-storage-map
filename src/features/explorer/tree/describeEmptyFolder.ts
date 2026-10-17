import type { NodeFlagSummary } from "@/lib/backend/tree";

export type EmptyFolderDescription = {
  title: string;
  description: string;
  isWarning: boolean;
};

/** Why the tree table has no rows to show for a folder, based on its flags when they are known. */
export function describeEmptyFolder(flags: NodeFlagSummary | null): EmptyFolderDescription {
  if (flags?.inaccessible) {
    return {
      title: "Windows did not let the scan look inside",
      description:
        "This folder is protected, so its contents are not counted. Scanning as administrator with the fast scan usually can read it.",
      isWarning: true,
    };
  }
  if (flags?.cloudPlaceholder) {
    return {
      title: "This folder lives in the cloud",
      description: "Its files are online-only, so they take no space here and were not downloaded.",
      isWarning: false,
    };
  }
  if (flags?.junction || flags?.symlink) {
    return {
      title: "This is a link to another folder",
      description: "Links are not followed, so the space is counted where the real folder is.",
      isWarning: false,
    };
  }
  return {
    title: "This folder is empty",
    description: "There is nothing inside it that takes up space.",
    isWarning: false,
  };
}
