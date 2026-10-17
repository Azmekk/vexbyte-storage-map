import { type HierarchyRectangularNode, treemap, treemapSquarify } from "d3-hierarchy";
import type {
  FileCategory,
  NodeIndex,
  VisualizationNode,
  VisualizationNodeKind,
} from "@/lib/backend/tree";
import { createVisualizationHierarchy, visualizationItemId } from "../visualizationHierarchy";

/** One rectangle to draw, in layout coordinates (CSS pixels of the full canvas). */
export type TreemapRect = {
  id: string;
  node: NodeIndex | null;
  name: string;
  kind: VisualizationNodeKind;
  category: FileCategory | null;
  logicalSize: number;
  mergedItemCount: number;
  /** 1 for the children of the shown folder. */
  depth: number;
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  /** A folder whose children are drawn inside it. */
  isContainer: boolean;
  /** Where the label goes: a header strip for containers, centred for leaves. */
  labelPlacement: "header" | "center" | "none";
};

export const TREEMAP_HEADER_HEIGHT = 16;
const CONTAINER_PADDING = 2;
const MINIMUM_HEADER_CONTAINER_WIDTH = 48;
const MINIMUM_HEADER_CONTAINER_HEIGHT = 36;
const MINIMUM_LABEL_WIDTH = 44;
const MINIMUM_LABEL_HEIGHT = 18;
/** Rectangles smaller than this (in either direction) are not drawn, nor their descendants. */
export const MINIMUM_VISIBLE_RECT_SIZE = 1;

function hasHeaderRoom(node: HierarchyRectangularNode<VisualizationNode>): boolean {
  return (
    node.x1 - node.x0 >= MINIMUM_HEADER_CONTAINER_WIDTH &&
    node.y1 - node.y0 >= MINIMUM_HEADER_CONTAINER_HEIGHT
  );
}

function sidePadding(node: HierarchyRectangularNode<VisualizationNode>): number {
  return node.depth === 0 ? 0 : CONTAINER_PADDING;
}

function topPadding(node: HierarchyRectangularNode<VisualizationNode>): number {
  if (node.depth === 0) {
    return 0;
  }
  return hasHeaderRoom(node) ? TREEMAP_HEADER_HEIGHT : CONTAINER_PADDING;
}

function chooseLabelPlacement(
  width: number,
  height: number,
  isContainer: boolean,
): TreemapRect["labelPlacement"] {
  if (isContainer) {
    return width >= MINIMUM_HEADER_CONTAINER_WIDTH && height >= MINIMUM_HEADER_CONTAINER_HEIGHT
      ? "header"
      : "none";
  }
  return width >= MINIMUM_LABEL_WIDTH && height >= MINIMUM_LABEL_HEIGHT ? "center" : "none";
}

/**
 * Lays out the subtree as a squarified treemap filling `width` × `height` and flattens it into
 * rectangles in drawing order: every parent comes before its children, so iterating backwards
 * finds the deepest rectangle first. The shown folder itself is not part of the list.
 */
export function prepareTreemapLayout(
  root: VisualizationNode,
  width: number,
  height: number,
): TreemapRect[] {
  if (width <= 0 || height <= 0 || root.logicalSize <= 0) {
    return [];
  }

  const layoutRoot = treemap<VisualizationNode>()
    .tile(treemapSquarify)
    .size([width, height])
    .paddingInner(1)
    .paddingTop(topPadding)
    .paddingRight(sidePadding)
    .paddingBottom(sidePadding)
    .paddingLeft(sidePadding)
    .round(false)(createVisualizationHierarchy(root));

  const rects: TreemapRect[] = [];

  const visit = (node: HierarchyRectangularNode<VisualizationNode>) => {
    const rectWidth = node.x1 - node.x0;
    const rectHeight = node.y1 - node.y0;

    if (
      node.depth > 0 &&
      (rectWidth < MINIMUM_VISIBLE_RECT_SIZE || rectHeight < MINIMUM_VISIBLE_RECT_SIZE)
    ) {
      return;
    }

    const children = node.children ?? [];
    const isContainer = children.length > 0;

    if (node.depth > 0) {
      rects.push({
        id: visualizationItemId(node.data, node.parent?.data ?? null),
        node: node.data.node,
        name: node.data.name,
        kind: node.data.kind,
        category: node.data.category,
        logicalSize: node.value ?? 0,
        mergedItemCount: node.data.mergedItemCount,
        depth: node.depth,
        x0: node.x0,
        y0: node.y0,
        x1: node.x1,
        y1: node.y1,
        isContainer,
        labelPlacement: chooseLabelPlacement(rectWidth, rectHeight, isContainer),
      });
    }

    for (const child of children) {
      visit(child);
    }
  };

  visit(layoutRoot);
  return rects;
}

/** The deepest rectangle under a point in layout coordinates, or `null`. */
export function findTreemapRectAt(
  rects: readonly TreemapRect[],
  layoutX: number,
  layoutY: number,
): TreemapRect | null {
  for (let rectIndex = rects.length - 1; rectIndex >= 0; rectIndex -= 1) {
    const rect = rects[rectIndex];
    if (
      rect !== undefined &&
      layoutX >= rect.x0 &&
      layoutX < rect.x1 &&
      layoutY >= rect.y0 &&
      layoutY < rect.y1
    ) {
      return rect;
    }
  }
  return null;
}
