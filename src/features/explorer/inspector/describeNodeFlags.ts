import type { NodeFlagSummary } from "@/lib/backend/tree";

export type NodeFlagBadge = {
  key: keyof NodeFlagSummary;
  label: string;
  explanation: string;
  tone: "neutral" | "warning";
};

const FLAG_BADGES: NodeFlagBadge[] = [
  {
    key: "inaccessible",
    label: "Access denied",
    explanation:
      "Windows did not let the scan read this folder, so its size may be larger than shown.",
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
      "Only a placeholder is on this PC; the content lives online and takes almost no space here.",
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
    label: "Junction",
    explanation: "Points to another folder, which is not followed or counted here.",
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
    explanation: "Marked as a system file by Windows.",
    tone: "neutral",
  },
  {
    key: "hidden",
    label: "Hidden",
    explanation: "Hidden in File Explorer by default.",
    tone: "neutral",
  },
];

/** The badges the inspector shows for an item, most important first. */
export function describeNodeFlags(flags: NodeFlagSummary): NodeFlagBadge[] {
  return FLAG_BADGES.filter((badge) => flags[badge.key]);
}

/** Whether an item can be moved to the Recycle Bin from the UI at all. */
export function canRecycleNode(flags: NodeFlagSummary, isScanRoot: boolean): boolean {
  return !isScanRoot && !flags.synthetic;
}
