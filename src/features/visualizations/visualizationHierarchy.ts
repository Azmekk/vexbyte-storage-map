import { type HierarchyNode, hierarchy } from "d3-hierarchy";
import type { VisualizationNode } from "@/lib/backend/tree";

/**
 * A d3 hierarchy whose values add up to each node's logical size. A folder's own value is the
 * part of its size its listed children do not cover, so pruned folders keep their full area.
 */
export function createVisualizationHierarchy(
  root: VisualizationNode,
): HierarchyNode<VisualizationNode> {
  return hierarchy(root, (node) => (node.children.length > 0 ? node.children : null))
    .sum((node) => {
      if (node.children.length === 0) {
        return Math.max(0, node.logicalSize);
      }
      const childrenSize = node.children.reduce(
        (sizeSoFar, child) => sizeSoFar + Math.max(0, child.logicalSize),
        0,
      );
      return Math.max(0, node.logicalSize - childrenSize);
    })
    .sort((first, second) => (second.value ?? 0) - (first.value ?? 0));
}

/** A stable id for a node of the subtree: its node index, or its parent's for merged blocks. */
export function visualizationItemId(
  node: VisualizationNode,
  parent: VisualizationNode | null,
): string {
  if (node.node !== null) {
    return `node-${node.node}`;
  }
  return `smaller-${parent?.node ?? "root"}`;
}
