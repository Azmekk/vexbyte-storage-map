import type { TreemapColorMode } from "@/features/explorer/useExplorerSelectionStore";
import type { NodeIndex } from "@/lib/backend/tree";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { canvasChromeColors } from "../canvasChromeColors";
import type { PaletteTheme } from "../categoryPalette";
import { fitTextToWidth } from "../fitTextToWidth";
import { readableTextColorOn } from "../readableTextColor";
import { TREEMAP_HEADER_HEIGHT, type TreemapRect } from "./prepareTreemapLayout";
import { treemapRectColor } from "./treemapRectColor";
import { type CanvasSize, projectRect, type Viewport } from "./treemapViewport";

export type TreemapDrawingOptions = {
  rects: readonly TreemapRect[];
  viewport: Viewport;
  size: CanvasSize;
  colorMode: TreemapColorMode;
  theme: PaletteTheme;
  hoveredId: string | null;
  selectedNode: NodeIndex | null;
};

const LABEL_FONT = "500 11px 'Geist Variable', system-ui, sans-serif";
const SIZE_FONT = "11px 'Geist Variable', system-ui, sans-serif";
const MINIMUM_PROJECTED_SIZE = 0.6;
const LABEL_PADDING = 4;

function strokeRect(context: CanvasRenderingContext2D, projected: Viewport, inset: number): void {
  context.strokeRect(
    projected.x0 + inset,
    projected.y0 + inset,
    Math.max(projected.x1 - projected.x0 - inset * 2, 0),
    Math.max(projected.y1 - projected.y0 - inset * 2, 0),
  );
}

/** Draws every rectangle under the viewport, then labels, then hover and selection outlines. */
export function drawTreemap(context: CanvasRenderingContext2D, options: TreemapDrawingOptions) {
  const { rects, viewport, size, colorMode, theme, hoveredId, selectedNode } = options;
  const chrome = canvasChromeColors(theme);

  context.clearRect(0, 0, size.width, size.height);
  context.textBaseline = "middle";

  let hoveredProjection: Viewport | null = null;
  let selectedProjection: Viewport | null = null;

  for (const rect of rects) {
    const projected = projectRect(rect, viewport, size);
    const width = projected.x1 - projected.x0;
    const height = projected.y1 - projected.y0;

    const isOutsideCanvas =
      projected.x1 < 0 ||
      projected.y1 < 0 ||
      projected.x0 > size.width ||
      projected.y0 > size.height;
    if (isOutsideCanvas || width < MINIMUM_PROJECTED_SIZE || height < MINIMUM_PROJECTED_SIZE) {
      continue;
    }

    const fillColor = treemapRectColor(
      { ...rect, hasVisibleChildren: rect.isContainer },
      colorMode,
      theme,
    );
    context.fillStyle = fillColor;
    context.fillRect(projected.x0, projected.y0, width, height);

    if (rect.id === hoveredId) {
      hoveredProjection = projected;
    }
    if (selectedNode !== null && rect.node === selectedNode) {
      selectedProjection = projected;
    }

    const labelWidth = width - LABEL_PADDING * 2;
    if (rect.isContainer) {
      if (width >= 48 && height >= 36) {
        context.font = LABEL_FONT;
        context.fillStyle = readableTextColorOn(fillColor);
        const label = fitTextToWidth(
          `${rect.name}  ${formatBytesAsReadableSize(rect.logicalSize)}`,
          labelWidth,
          (candidate) => context.measureText(candidate).width,
        );
        context.fillText(
          label,
          projected.x0 + LABEL_PADDING,
          projected.y0 + TREEMAP_HEADER_HEIGHT / 2 + 1,
        );
      }
      continue;
    }

    if (width >= 44 && height >= 18) {
      const textColor = readableTextColorOn(fillColor);
      context.fillStyle = textColor;
      context.font = LABEL_FONT;
      const name = fitTextToWidth(
        rect.name,
        labelWidth,
        (candidate) => context.measureText(candidate).width,
      );
      const showsSize = height >= 34;
      const nameY = showsSize ? projected.y0 + 12 : projected.y0 + height / 2;
      context.fillText(name, projected.x0 + LABEL_PADDING, nameY);

      if (showsSize) {
        context.font = SIZE_FONT;
        context.globalAlpha = 0.8;
        const sizeText = fitTextToWidth(
          formatBytesAsReadableSize(rect.logicalSize),
          labelWidth,
          (candidate) => context.measureText(candidate).width,
        );
        context.fillText(sizeText, projected.x0 + LABEL_PADDING, projected.y0 + 26);
        context.globalAlpha = 1;
      }
    }
  }

  if (selectedProjection !== null) {
    context.strokeStyle = chrome.selectionOutline;
    context.lineWidth = 2;
    strokeRect(context, selectedProjection, 1);
  }

  if (hoveredProjection !== null) {
    context.strokeStyle = chrome.hoverOutline;
    context.lineWidth = 2;
    strokeRect(context, hoveredProjection, 1);
  }
}
