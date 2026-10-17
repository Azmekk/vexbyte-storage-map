import type { TreemapColorMode } from "@/features/explorer/useExplorerSelectionStore";
import type { NodeIndex } from "@/lib/backend/tree";
import { canvasChromeColors } from "../canvasChromeColors";
import type { PaletteTheme } from "../categoryPalette";
import { treemapRectColor } from "../treemap/treemapRectColor";
import type { CanvasSize } from "../treemap/treemapViewport";
import { type AngularViewport, projectAngle, type SunburstArc } from "./prepareSunburstLayout";
import type { SunburstGeometry } from "./sunburstGeometry";

export type SunburstDrawingOptions = {
  arcs: readonly SunburstArc[];
  geometry: SunburstGeometry;
  viewport: AngularViewport;
  size: CanvasSize;
  colorMode: TreemapColorMode;
  theme: PaletteTheme;
  hoveredId: string | null;
  selectedNode: NodeIndex | null;
};

const FULL_TURN = Math.PI * 2;
const QUARTER_TURN = Math.PI / 2;
const MINIMUM_PROJECTED_ANGLE = 0.002;

function traceSegment(
  context: CanvasRenderingContext2D,
  geometry: SunburstGeometry,
  ring: number,
  startAngle: number,
  endAngle: number,
): void {
  const innerRadius = geometry.centerRadius + (ring - 1) * geometry.ringThickness;
  const outerRadius = innerRadius + geometry.ringThickness;
  const canvasStart = startAngle - QUARTER_TURN;
  const canvasEnd = endAngle - QUARTER_TURN;

  context.beginPath();
  context.arc(geometry.centerX, geometry.centerY, outerRadius, canvasStart, canvasEnd);
  context.arc(geometry.centerX, geometry.centerY, innerRadius, canvasEnd, canvasStart, true);
  context.closePath();
}

/** Draws the rings with a surface-coloured gap between segments, then the outlines. */
export function drawSunburst(context: CanvasRenderingContext2D, options: SunburstDrawingOptions) {
  const { arcs, geometry, viewport, size, colorMode, theme, hoveredId, selectedNode } = options;
  const chrome = canvasChromeColors(theme);

  context.clearRect(0, 0, size.width, size.height);
  context.lineJoin = "round";

  let hoveredArc: { arc: SunburstArc; start: number; end: number } | null = null;
  let selectedArc: { arc: SunburstArc; start: number; end: number } | null = null;

  for (const arc of arcs) {
    const start = Math.max(0, projectAngle(arc.startAngle, viewport));
    const end = Math.min(FULL_TURN, projectAngle(arc.endAngle, viewport));
    if (end - start < MINIMUM_PROJECTED_ANGLE) {
      continue;
    }

    traceSegment(context, geometry, arc.ring, start, end);
    context.fillStyle = treemapRectColor(
      {
        kind: arc.kind,
        category: arc.category,
        depth: arc.ring,
        hasVisibleChildren: arc.hasChildren,
      },
      colorMode,
      theme,
    );
    context.fill();
    context.strokeStyle = chrome.surface;
    context.lineWidth = 1;
    context.stroke();

    if (arc.id === hoveredId) {
      hoveredArc = { arc, start, end };
    }
    if (selectedNode !== null && arc.node === selectedNode) {
      selectedArc = { arc, start, end };
    }
  }

  if (selectedArc !== null) {
    traceSegment(context, geometry, selectedArc.arc.ring, selectedArc.start, selectedArc.end);
    context.strokeStyle = chrome.selectionOutline;
    context.lineWidth = 2;
    context.stroke();
  }

  if (hoveredArc !== null) {
    traceSegment(context, geometry, hoveredArc.arc.ring, hoveredArc.start, hoveredArc.end);
    context.strokeStyle = chrome.hoverOutline;
    context.lineWidth = 2;
    context.stroke();
  }
}
