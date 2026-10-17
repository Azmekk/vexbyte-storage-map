import { interpolateNumber } from "d3-interpolate";

/** The part of the layout that is stretched over the whole canvas. */
export type Viewport = { x0: number; y0: number; x1: number; y1: number };

export type CanvasSize = { width: number; height: number };

export function fullViewport(size: CanvasSize): Viewport {
  return { x0: 0, y0: 0, x1: size.width, y1: size.height };
}

/**
 * Zooming into a folder: the new layout fills the canvas, so to start the transition where the
 * folder's rectangle was, the viewport begins larger than the canvas by the inverse ratio.
 */
export function viewportForZoomIn(previousRect: Viewport, size: CanvasSize): Viewport {
  const rectWidth = Math.max(previousRect.x1 - previousRect.x0, 1);
  const rectHeight = Math.max(previousRect.y1 - previousRect.y0, 1);
  const scaleX = size.width / rectWidth;
  const scaleY = size.height / rectHeight;
  const x0 = -previousRect.x0 * scaleX;
  const y0 = -previousRect.y0 * scaleY;
  return { x0, y0, x1: x0 + size.width * scaleX, y1: y0 + size.height * scaleY };
}

/**
 * Zooming out to a parent: the previously shown folder is a rectangle of the new layout, so the
 * transition starts with exactly that rectangle filling the canvas.
 */
export function viewportForZoomOut(previousFolderRect: Viewport): Viewport {
  return { ...previousFolderRect };
}

export function interpolateViewport(from: Viewport, to: Viewport, progress: number): Viewport {
  return {
    x0: interpolateNumber(from.x0, to.x0)(progress),
    y0: interpolateNumber(from.y0, to.y0)(progress),
    x1: interpolateNumber(from.x1, to.x1)(progress),
    y1: interpolateNumber(from.y1, to.y1)(progress),
  };
}

/** Maps a layout rectangle to canvas coordinates under `viewport`. */
export function projectRect(rect: Viewport, viewport: Viewport, size: CanvasSize): Viewport {
  const scaleX = size.width / Math.max(viewport.x1 - viewport.x0, 1e-6);
  const scaleY = size.height / Math.max(viewport.y1 - viewport.y0, 1e-6);
  return {
    x0: (rect.x0 - viewport.x0) * scaleX,
    y0: (rect.y0 - viewport.y0) * scaleY,
    x1: (rect.x1 - viewport.x0) * scaleX,
    y1: (rect.y1 - viewport.y0) * scaleY,
  };
}

/** Maps a canvas point back to layout coordinates under `viewport`. */
export function unprojectPoint(
  canvasX: number,
  canvasY: number,
  viewport: Viewport,
  size: CanvasSize,
): { x: number; y: number } {
  return {
    x: viewport.x0 + (canvasX / size.width) * (viewport.x1 - viewport.x0),
    y: viewport.y0 + (canvasY / size.height) * (viewport.y1 - viewport.y0),
  };
}

/** Ease-out cubic: fast start, gentle landing. */
export function easeOutCubic(progress: number): number {
  const clamped = Math.min(Math.max(progress, 0), 1);
  return 1 - (1 - clamped) ** 3;
}
