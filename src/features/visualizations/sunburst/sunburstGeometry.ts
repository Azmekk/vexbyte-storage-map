import type { CanvasSize } from "../treemap/treemapViewport";
import type { SunburstArc } from "./prepareSunburstLayout";

export type SunburstGeometry = {
  centerX: number;
  centerY: number;
  centerRadius: number;
  ringThickness: number;
  ringCount: number;
};

const OUTER_MARGIN = 8;
const CENTER_SHARE_OF_RADIUS = 0.3;

/** Where the rings go inside the canvas, sized to the deepest ring that has segments. */
export function sunburstGeometry(arcs: readonly SunburstArc[], size: CanvasSize): SunburstGeometry {
  const ringCount = Math.max(1, ...arcs.map((arc) => arc.ring));
  const outerRadius = Math.max(0, Math.min(size.width, size.height) / 2 - OUTER_MARGIN);
  const centerRadius = outerRadius * CENTER_SHARE_OF_RADIUS;

  return {
    centerX: size.width / 2,
    centerY: size.height / 2,
    centerRadius,
    ringThickness: ringCount === 0 ? 0 : (outerRadius - centerRadius) / ringCount,
    ringCount,
  };
}
