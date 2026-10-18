import type { NodeFlagSummary } from "@/lib/backend/tree";
import { currentPlatform } from "@/lib/platform/currentPlatform";
import { platformWording } from "@/lib/platform/platformWording";

export type NodeFlagBadge = {
  key: keyof NodeFlagSummary;
  label: string;
  explanation: string;
  tone: "neutral" | "warning";
};

function flagBadges(): NodeFlagBadge[] {
  const { systemName, systemNameAtSentenceStart } = platformWording();
  const isLinux = currentPlatform().os === "linux";

  return [
    {
      key: "inaccessible",
      label: "Access denied",
      explanation: `${systemNameAtSentenceStart} did not let the scan read this folder, so its size may be larger than shown.`,
      tone: "warning",
    },
    {
      key: "hardLinkCountedElsewhere",
      label: "Hard link, counted once",
      explanation: "Another path points to the same file, and its size is counted there.",
      tone: "neutral",
    },
    {
      key: "cloudPlaceholder",
      label: "In the cloud",
      explanation:
        "The content lives online. Only what is downloaded counts toward the sizes shown here.",
      tone: "neutral",
    },
    {
      key: "compressed",
      label: "Compressed",
      explanation: "NTFS compression makes this take less space on disk than its size.",
      tone: "neutral",
    },
    {
      key: "sparse",
      label: "Sparse",
      explanation: "Empty regions of this file take no space on disk.",
      tone: "neutral",
    },
    {
      key: "junction",
      label: isLinux ? "Mount point" : "Junction",
      explanation: isLinux
        ? "Another file system is mounted here. It is not followed or counted here."
        : "Points to another folder, which is not followed or counted here.",
      tone: "neutral",
    },
    {
      key: "symlink",
      label: "Symbolic link",
      explanation: "Points to another location, which is not followed or counted here.",
      tone: "neutral",
    },
    {
      key: "synthetic",
      label: "Summary item",
      explanation: "Groups several items together; it does not exist as one item on disk.",
      tone: "neutral",
    },
    {
      key: "system",
      label: "System",
      explanation: `Marked as a system file by ${systemName}.`,
      tone: "neutral",
    },
    {
      key: "hidden",
      label: "Hidden",
      explanation: isLinux
        ? "Its name starts with a dot, so file managers hide it by default."
        : "Hidden in File Explorer by default.",
      tone: "neutral",
    },
  ];
}

/** The badges the inspector shows for an item, most important first. */
export function describeNodeFlags(flags: NodeFlagSummary): NodeFlagBadge[] {
  return flagBadges().filter((badge) => flags[badge.key]);
}

/** Whether an item can be moved to the Recycle Bin or Trash from the UI at all. */
export function canRecycleNode(flags: NodeFlagSummary, isScanRoot: boolean): boolean {
  return !isScanRoot && !flags.synthetic;
}
