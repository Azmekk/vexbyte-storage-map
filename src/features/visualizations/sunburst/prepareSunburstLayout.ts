import { type HierarchyRectangularNode, partition } from "d3-hierarchy";
import type {
  FileCategory,
  NodeIndex,
  VisualizationNode,
  VisualizationNodeKind,
} from "@/lib/backend/tree";
import { dominantFileCategories } from "../dominantFileCategories";
import { createVisualizationHierarchy, visualizationItemId } from "../visualizationHierarchy";

/** One ring segment. Angles in radians from 12 o'clock, clockwise; rings counted from 1. */
export type SunburstArc = {
  id: string;
  node: NodeIndex | null;
  name: string;
  kind: VisualizationNodeKind;
  category: FileCategory | null;
  logicalSize: number;
  mergedItemCount: number;
  ring: number;
  startAngle: number;
  endAngle: number;
  hasChildren: boolean;
  /** For folders: the known category taking most of the bytes listed inside, if any. */
  dominantCategory: FileCategory | null;
};

export const SUNBURST_RING_COUNT = 5;
const FULL_TURN = Math.PI * 2;
/** Segments thinner than this many radians are skipped, together with their descendants. */
export const MINIMUM_ARC_ANGLE = 0.003;

/** Partitions the subtree into rings around the shown folder, at most `ringCount` deep. */
export function prepareSunburstLayout(
  root: VisualizationNode,
  ringCount: number = SUNBURST_RING_COUNT,
): SunburstArc[] {
  if (root.logicalSize <= 0) {
    return [];
  }

  const layoutRoot = partition<VisualizationNode>().size([FULL_TURN, 1])(
    createVisualizationHierarchy(root),
  );

  const dominantCategories = dominantFileCategories(layoutRoot);
  const arcs: SunburstArc[] = [];

  const visit = (node: HierarchyRectangularNode<VisualizationNode>) => {
    if (node.depth > ringCount) {
      return;
    }
    if (node.depth > 0 && node.x1 - node.x0 < MINIMUM_ARC_ANGLE) {
      return;
    }

    const children = node.children ?? [];

    if (node.depth > 0) {
      arcs.push({
        id: visualizationItemId(node.data, node.parent?.data ?? null),
        node: node.data.node,
        name: node.data.name,
        kind: node.data.kind,
        category: node.data.category,
        logicalSize: node.value ?? 0,
        mergedItemCount: node.data.mergedItemCount,
        ring: node.depth,
        startAngle: node.x0,
        endAngle: node.x1,
        hasChildren: children.length > 0,
        dominantCategory: dominantCategories.get(node) ?? null,
      });
    }

    for (const child of children) {
      visit(child);
    }
  };

  visit(layoutRoot);
  return arcs;
}

/** The angle of a point around the centre, from 12 o'clock clockwise, in `[0, 2π)`. */
export function angleOfPoint(offsetX: number, offsetY: number): number {
  const angle = Math.atan2(offsetX, -offsetY);
  return angle < 0 ? angle + FULL_TURN : angle;
}

/** The arc under a point given relative to the centre, or `null` (also for the centre). */
export function findSunburstArcAt(
  arcs: readonly SunburstArc[],
  offsetX: number,
  offsetY: number,
  ringThickness: number,
  centerRadius: number,
  viewport: AngularViewport = FULL_ANGULAR_VIEWPORT,
): SunburstArc | null {
  const distance = Math.hypot(offsetX, offsetY);
  if (distance < centerRadius || ringThickness <= 0) {
    return null;
  }

  const ring = Math.floor((distance - centerRadius) / ringThickness) + 1;
  const screenAngle = angleOfPoint(offsetX, offsetY);
  const layoutAngle = viewport.start + (screenAngle / FULL_TURN) * (viewport.end - viewport.start);

  for (const arc of arcs) {
    if (arc.ring === ring && layoutAngle >= arc.startAngle && layoutAngle < arc.endAngle) {
      return arc;
    }
  }
  return null;
}

/** An angle range of the layout spread over the full turn while zooming. */
export type AngularViewport = { start: number; end: number };

export const FULL_ANGULAR_VIEWPORT: AngularViewport = { start: 0, end: FULL_TURN };

/** Maps a layout angle into screen angles for the given angular viewport. */
export function projectAngle(angle: number, viewport: AngularViewport): number {
  const span = Math.max(viewport.end - viewport.start, 1e-6);
  return ((angle - viewport.start) / span) * FULL_TURN;
}

/** Zooming in: the new rings start squeezed into the clicked segment's angles. */
export function angularViewportForZoomIn(
  clickedStartAngle: number,
  clickedEndAngle: number,
): AngularViewport {
  const span = Math.max(clickedEndAngle - clickedStartAngle, 1e-6);
  const scale = FULL_TURN / span;
  const start = -clickedStartAngle * scale;
  return { start, end: start + FULL_TURN * scale };
}
