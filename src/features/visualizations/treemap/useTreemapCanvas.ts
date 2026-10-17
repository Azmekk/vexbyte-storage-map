import { useReducedMotion } from "motion/react";
import {
  type PointerEvent,
  type RefObject,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { TreemapColorMode } from "@/features/explorer/useExplorerSelectionStore";
import type { NodeIndex, VisualizationNode } from "@/lib/backend/tree";
import { VISUALIZATION_ZOOM_DURATION_MILLISECONDS } from "@/lib/motion/transitions";
import type { PaletteTheme } from "../categoryPalette";
import { prepareHighDensityCanvas } from "../prepareHighDensityCanvas";
import { useElementSize } from "../useElementSize";
import { useFrameAnimation } from "../useFrameAnimation";
import { drawTreemap } from "./drawTreemap";
import { findTreemapRectAt, prepareTreemapLayout, type TreemapRect } from "./prepareTreemapLayout";
import {
  fullViewport,
  interpolateViewport,
  unprojectPoint,
  type Viewport,
  viewportForZoomIn,
  viewportForZoomOut,
} from "./treemapViewport";

export type TreemapHover = { rect: TreemapRect; pointerX: number; pointerY: number };

export type TreemapCanvasModel = {
  containerRef: RefObject<HTMLDivElement | null>;
  canvasRef: RefObject<HTMLCanvasElement | null>;
  rectCount: number;
  hover: TreemapHover | null;
  handlePointerMove: (event: PointerEvent<HTMLCanvasElement>) => void;
  handlePointerLeave: () => void;
  rectAtPointer: (event: { clientX: number; clientY: number }) => TreemapRect | null;
};

type TreemapCanvasInput = {
  subtree: VisualizationNode | null;
  colorMode: TreemapColorMode;
  theme: PaletteTheme;
  selectedNode: NodeIndex | null;
};

type PreviousLayout = { rootNode: NodeIndex | null; rects: TreemapRect[]; width: number };

/**
 * Lays out, draws and hit-tests the treemap. When the shown folder changes, the new layout
 * grows out of the clicked block (zoom in) or shrinks the old folder into its place (zoom out).
 */
export function useTreemapCanvas({
  subtree,
  colorMode,
  theme,
  selectedNode,
}: TreemapCanvasInput): TreemapCanvasModel {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const size = useElementSize(containerRef);
  const prefersReducedMotion = useReducedMotion() === true;
  const animation = useFrameAnimation();
  const [hover, setHover] = useState<TreemapHover | null>(null);

  const rects = useMemo(
    () => (subtree === null ? [] : prepareTreemapLayout(subtree, size.width, size.height)),
    [subtree, size],
  );

  const viewportRef = useRef<Viewport>(fullViewport(size));
  const previousLayoutRef = useRef<PreviousLayout | null>(null);
  const drawInputRef = useRef({ rects, size, colorMode, theme, selectedNode, hoveredId: "" });
  drawInputRef.current = {
    rects,
    size,
    colorMode,
    theme,
    selectedNode,
    hoveredId: hover?.rect.id ?? "",
  };

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const input = drawInputRef.current;
    if (canvas === null || input.size.width === 0 || input.size.height === 0) {
      return;
    }
    const context = prepareHighDensityCanvas(canvas, input.size);
    if (context === null) {
      return;
    }
    drawTreemap(context, {
      rects: input.rects,
      viewport: viewportRef.current,
      size: input.size,
      colorMode: input.colorMode,
      theme: input.theme,
      hoveredId: input.hoveredId === "" ? null : input.hoveredId,
      selectedNode: input.selectedNode,
    });
  }, []);

  useLayoutEffect(() => {
    const targetViewport = fullViewport(size);
    const previousLayout = previousLayoutRef.current;
    const newRootNode = subtree?.node ?? null;
    let startViewport: Viewport | null = null;

    const isFolderChange =
      previousLayout !== null &&
      previousLayout.rootNode !== newRootNode &&
      previousLayout.width === size.width;

    if (isFolderChange && !prefersReducedMotion) {
      const clickedRect = previousLayout.rects.find((rect) => rect.node === newRootNode);
      const previousFolderRect = rects.find((rect) => rect.node === previousLayout.rootNode);
      if (clickedRect !== undefined) {
        startViewport = viewportForZoomIn(clickedRect, size);
      } else if (previousFolderRect !== undefined) {
        startViewport = viewportForZoomOut(previousFolderRect);
      }
    }

    setHover(null);

    if (rects.length > 0) {
      previousLayoutRef.current = { rootNode: newRootNode, rects, width: size.width };
    }

    if (startViewport === null) {
      animation.cancel();
      viewportRef.current = targetViewport;
      draw();
      return;
    }

    const fromViewport = startViewport;
    animation.run(VISUALIZATION_ZOOM_DURATION_MILLISECONDS, (progress) => {
      viewportRef.current = interpolateViewport(fromViewport, targetViewport, progress);
      draw();
    });
  }, [rects, size, subtree, prefersReducedMotion, animation, draw]);

  const hoveredId = hover?.rect.id ?? null;
  useEffect(() => {
    void colorMode;
    void theme;
    void selectedNode;
    void hoveredId;
    draw();
  }, [colorMode, theme, selectedNode, hoveredId, draw]);

  const rectAtPointer = useCallback(
    (event: { clientX: number; clientY: number }) => {
      const canvas = canvasRef.current;
      if (canvas === null || size.width === 0) {
        return null;
      }
      const bounds = canvas.getBoundingClientRect();
      const layoutPoint = unprojectPoint(
        event.clientX - bounds.left,
        event.clientY - bounds.top,
        viewportRef.current,
        size,
      );
      return findTreemapRectAt(rects, layoutPoint.x, layoutPoint.y);
    },
    [rects, size],
  );

  const handlePointerMove = useCallback(
    (event: PointerEvent<HTMLCanvasElement>) => {
      const rect = rectAtPointer(event);
      const bounds = event.currentTarget.getBoundingClientRect();
      setHover(
        rect === null
          ? null
          : { rect, pointerX: event.clientX - bounds.left, pointerY: event.clientY - bounds.top },
      );
    },
    [rectAtPointer],
  );

  const handlePointerLeave = useCallback(() => setHover(null), []);

  return {
    containerRef,
    canvasRef,
    rectCount: rects.length,
    hover,
    handlePointerMove,
    handlePointerLeave,
    rectAtPointer,
  };
}
