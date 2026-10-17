import type { NodeFlagSummary, NodeIndex, NodeKind } from "@/lib/backend/tree";

/** The item a context menu or action button acts on. */
export type NodeActionTarget = {
  node: NodeIndex;
  name: string;
  kind: NodeKind;
  /** Summary nodes such as "NTFS metadata" have no single place on disk. */
  isSynthetic: boolean;
};

export function nodeActionTarget(item: {
  node: NodeIndex;
  name: string;
  kind: NodeKind;
  flags: NodeFlagSummary;
}): NodeActionTarget {
  return { node: item.node, name: item.name, kind: item.kind, isSynthetic: item.flags.synthetic };
}
