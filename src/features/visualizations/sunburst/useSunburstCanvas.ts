import { interpolateNumber } from "d3-interpolate";
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
import { usePrefersReducedMotion } from "@/lib/motion/usePrefersReducedMotion";
import type { PaletteTheme } from "../categoryPalette";
import { prepareHighDensityCanvas } from "../prepareHighDensityCanvas";
import { useElementSize } from "../useElementSize";
import { useFrameAnimation } from "../useFrameAnimation";
import { drawSunburst } from "./drawSunburst";
import {
  type AngularViewport,
  angularViewportForZoomIn,
  FULL_ANGULAR_VIEWPORT,
  findSunburstArcAt,
  prepareSunburstLayout,
  type SunburstArc,
} from "./prepareSunburstLayout";
import { type SunburstGeometry, sunburstGeometry } from "./sunburstGeometry";

export type SunburstHover = { arc: SunburstArc; pointerX: number; pointerY: number };

export type SunburstCanvasModel = {
  containerRef: RefObject<HTMLDivElement | null>;
  canvasRef: RefObject<HTMLCanvasElement | null>;
  geometry: SunburstGeometry;
  arcCount: number;
  hover: SunburstHover | null;
  handlePointerMove: (event: PointerEvent<HTMLCanvasElement>) => void;
  handlePointerLeave: () => void;
  arcAtPointer: (event: { clientX: number; clientY: number }) => SunburstArc | null;
};

type SunburstCanvasInput = {
  subtree: VisualizationNode | null;
  colorMode: TreemapColorMode;
  theme: PaletteTheme;
  selectedNode: NodeIndex | null;
};

type PreviousLayout = { rootNode: NodeIndex | null; arcs: SunburstArc[] };

function interpolateAngularViewport(
  from: AngularViewport,
  to: AngularViewport,
  progress: number,
): AngularViewport {
  return {
    start: interpolateNumber(from.start, to.start)(progress),
    end: interpolateNumber(from.end, to.end)(progress),
  };
}

/** Lays out, draws and hit-tests the sunburst, with the same zoom behaviour as the treemap. */
export function useSunburstCanvas({
  subtree,
  colorMode,
  theme,
  selectedNode,
}: SunburstCanvasInput): SunburstCanvasModel {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const size = useElementSize(containerRef);
  const prefersReducedMotion = usePrefersReducedMotion();
  const animation = useFrameAnimation();
  const [hover, setHover] = useState<SunburstHover | null>(null);

  const arcs = useMemo(() => (subtree === null ? [] : prepareSunburstLayout(subtree)), [subtree]);
  const geometry = useMemo(() => sunburstGeometry(arcs, size), [arcs, size]);

  const viewportRef = useRef<AngularViewport>(FULL_ANGULAR_VIEWPORT);
  const previousLayoutRef = useRef<PreviousLayout | null>(null);
  const drawInputRef = useRef({
    arcs,
    geometry,
    size,
    colorMode,
    theme,
    selectedNode,
    hoveredId: "",
  });
  drawInputRef.current = {
    arcs,
    geometry,
    size,
    colorMode,
    theme,
    selectedNode,
    hoveredId: hover?.arc.id ?? "",
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
    drawSunburst(context, {
      arcs: input.arcs,
      geometry: input.geometry,
      viewport: viewportRef.current,
      size: input.size,
      colorMode: input.colorMode,
      theme: input.theme,
      hoveredId: input.hoveredId === "" ? null : input.hoveredId,
      selectedNode: input.selectedNode,
    });
  }, []);

  useLayoutEffect(() => {
    const previousLayout = previousLayoutRef.current;
    const newRootNode = subtree?.node ?? null;
    let startViewport: AngularViewport | null = null;

    if (
      previousLayout !== null &&
      previousLayout.rootNode !== newRootNode &&
      !prefersReducedMotion
    ) {
      const clickedArc = previousLayout.arcs.find((arc) => arc.node === newRootNode);
      const previousFolderArc = arcs.find((arc) => arc.node === previousLayout.rootNode);
      if (clickedArc !== undefined) {
        startViewport = angularViewportForZoomIn(clickedArc.startAngle, clickedArc.endAngle);
      } else if (previousFolderArc !== undefined) {
        startViewport = { start: previousFolderArc.startAngle, end: previousFolderArc.endAngle };
      }
    }

    setHover(null);

    if (arcs.length > 0) {
      previousLayoutRef.current = { rootNode: newRootNode, arcs };
    }

    if (startViewport === null) {
      animation.cancel();
      viewportRef.current = FULL_ANGULAR_VIEWPORT;
      draw();
      return;
    }

    const fromViewport = startViewport;
    animation.run(VISUALIZATION_ZOOM_DURATION_MILLISECONDS, (progress) => {
      viewportRef.current = interpolateAngularViewport(
        fromViewport,
        FULL_ANGULAR_VIEWPORT,
        progress,
      );
      draw();
    });
  }, [arcs, subtree, prefersReducedMotion, animation, draw]);

  const hoveredId = hover?.arc.id ?? null;
  useEffect(() => {
    void colorMode;
    void theme;
    void selectedNode;
    void hoveredId;
    void geometry;
    draw();
  }, [colorMode, theme, selectedNode, hoveredId, geometry, draw]);

  const arcAtPointer = useCallback(
    (event: { clientX: number; clientY: number }) => {
      const canvas = canvasRef.current;
      if (canvas === null) {
        return null;
      }
      const bounds = canvas.getBoundingClientRect();
      return findSunburstArcAt(
        arcs,
        event.clientX - bounds.left - geometry.centerX,
        event.clientY - bounds.top - geometry.centerY,
        geometry.ringThickness,
        geometry.centerRadius,
        viewportRef.current,
      );
    },
    [arcs, geometry],
  );

  const handlePointerMove = useCallback(
    (event: PointerEvent<HTMLCanvasElement>) => {
      const arc = arcAtPointer(event);
      const bounds = event.currentTarget.getBoundingClientRect();
      setHover(
        arc === null
          ? null
          : { arc, pointerX: event.clientX - bounds.left, pointerY: event.clientY - bounds.top },
      );
    },
    [arcAtPointer],
  );

  const handlePointerLeave = useCallback(() => setHover(null), []);

  return {
    containerRef,
    canvasRef,
    geometry,
    arcCount: arcs.length,
    hover,
    handlePointerMove,
    handlePointerLeave,
    arcAtPointer,
  };
}
