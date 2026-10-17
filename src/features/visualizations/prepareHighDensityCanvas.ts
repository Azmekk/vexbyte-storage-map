import type { CanvasSize } from "./treemap/treemapViewport";

/**
 * Sizes the canvas backing store for the screen's pixel density and returns a context that
 * draws in CSS pixels. Returns `null` when the canvas has no 2D context.
 */
export function prepareHighDensityCanvas(
  canvas: HTMLCanvasElement,
  size: CanvasSize,
): CanvasRenderingContext2D | null {
  const pixelRatio = window.devicePixelRatio || 1;
  const backingWidth = Math.max(1, Math.round(size.width * pixelRatio));
  const backingHeight = Math.max(1, Math.round(size.height * pixelRatio));

  if (canvas.width !== backingWidth || canvas.height !== backingHeight) {
    canvas.width = backingWidth;
    canvas.height = backingHeight;
  }

  const context = canvas.getContext("2d");
  context?.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
  return context;
}
