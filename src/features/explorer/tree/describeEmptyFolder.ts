import type { NodeDetails } from "@/lib/backend/tree";
import { formatCount } from "@/lib/formatting/formatCount";

export type EmptyFolderDescription = {
  title: string;
  description: string;
  isWarning: boolean;
  /** The folder holds only online-only cloud files, which the user can choose to show. */
  hasHiddenOnlineOnlyFiles: boolean;
};

/** What the tree table needs to know about a folder that has no rows to show. */
export type EmptyFolderFacts = Pick<NodeDetails, "flags" | "onlineOnlyFileCount">;

/** Why the tree table has no rows to show for a folder, based on its details when they are known. */
export function describeEmptyFolder(facts: EmptyFolderFacts | null): EmptyFolderDescription {
  if (facts?.flags.inaccessible) {
    return {
      title: "Windows did not let the scan look inside",
      description:
        "This folder is protected, so its contents are not counted. Scanning as administrator with the fast scan usually can read it.",
      isWarning: true,
      hasHiddenOnlineOnlyFiles: false,
    };
  }
  if (facts?.flags.cloudPlaceholder) {
    return {
      title: "This folder lives in the cloud",
      description: "Its files are online-only, so they take no space here and were not downloaded.",
      isWarning: false,
      hasHiddenOnlineOnlyFiles: false,
    };
  }
  if (facts !== null && facts.onlineOnlyFileCount > 0) {
    const fileCount = formatCount(facts.onlineOnlyFileCount);
    return {
      title: "Only online-only files here",
      description: `${fileCount} ${facts.onlineOnlyFileCount === 1 ? "file is" : "files are"} in the cloud and take no space on this PC, so they are not shown.`,
      isWarning: false,
      hasHiddenOnlineOnlyFiles: true,
    };
  }
  if (facts?.flags.junction || facts?.flags.symlink) {
    return {
      title: "This is a link to another folder",
      description: "Links are not followed, so the space is counted where the real folder is.",
      isWarning: false,
      hasHiddenOnlineOnlyFiles: false,
    };
  }
  return {
    title: "This folder is empty",
    description: "There is nothing inside it that takes up space.",
    isWarning: false,
    hasHiddenOnlineOnlyFiles: false,
  };
}
