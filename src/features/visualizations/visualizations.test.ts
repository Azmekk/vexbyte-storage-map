import { describe, expect, it } from "vitest";
import type { VisualizationNode } from "@/lib/backend/tree";
import {
  angleOfPoint,
  angularViewportForZoomIn,
  findSunburstArcAt,
  prepareSunburstLayout,
  projectAngle,
} from "./sunburst/prepareSunburstLayout";
import { findTreemapRectAt, prepareTreemapLayout } from "./treemap/prepareTreemapLayout";
import {
  interpolateViewport,
  projectRect,
  unprojectPoint,
  viewportForZoomIn,
  viewportForZoomOut,
} from "./treemap/treemapViewport";

function createNode(
  node: number | null,
  logicalSize: number,
  children: VisualizationNode[] = [],
  overrides: Partial<VisualizationNode> = {},
): VisualizationNode {
  return {
    node,
    name: `node ${node ?? "smaller"}`,
    kind: children.length > 0 ? "directory" : "file",
    logicalSize,
    category: null,
    mergedItemCount: 0,
    children,
    ...overrides,
  };
}

const sampleTree = createNode(0, 1000, [
  createNode(1, 600, [createNode(3, 400), createNode(4, 200)]),
  createNode(2, 399.9),
  createNode(5, 0.1),
]);

describe("prepareTreemapLayout", () => {
  it("lists parents before children and leaves the shown folder out", () => {
    const rects = prepareTreemapLayout(sampleTree, 400, 300);
    const ids = rects.map((rect) => rect.id);

    expect(ids).not.toContain("node-0");
    expect(ids.indexOf("node-1")).toBeLessThan(ids.indexOf("node-3"));
    expect(rects.find((rect) => rect.id === "node-1")?.isContainer).toBe(true);
  });

  it("gives areas proportional to sizes", () => {
    const rects = prepareTreemapLayout(
      createNode(0, 100, [createNode(1, 75), createNode(2, 25)]),
      200,
      100,
    );
    const area = (id: string) => {
      const rect = rects.find((candidate) => candidate.id === id);
      return rect === undefined ? 0 : (rect.x1 - rect.x0) * (rect.y1 - rect.y0);
    };

    expect(area("node-1") / area("node-2")).toBeCloseTo(3, 0);
  });

  it("prunes rectangles too small to see", () => {
    const rects = prepareTreemapLayout(sampleTree, 400, 300);
    expect(rects.some((rect) => rect.id === "node-5")).toBe(false);
  });

  it("only labels rectangles that are large enough", () => {
    const largeRects = prepareTreemapLayout(sampleTree, 800, 600);
    expect(largeRects.find((rect) => rect.id === "node-1")?.labelPlacement).toBe("header");
    expect(largeRects.find((rect) => rect.id === "node-2")?.labelPlacement).toBe("center");

    const tinyRects = prepareTreemapLayout(sampleTree, 40, 30);
    expect(tinyRects.every((rect) => rect.labelPlacement === "none")).toBe(true);
  });

  it("returns nothing for empty folders or canvases", () => {
    expect(prepareTreemapLayout(createNode(0, 0), 100, 100)).toEqual([]);
    expect(prepareTreemapLayout(sampleTree, 0, 100)).toEqual([]);
  });

  it("names merged blocks after their parent", () => {
    const tree = createNode(0, 100, [
      createNode(1, 60),
      createNode(null, 40, [], { kind: "smallerItems", mergedItemCount: 12 }),
    ]);
    const rects = prepareTreemapLayout(tree, 100, 100);
    expect(rects.find((rect) => rect.kind === "smallerItems")?.id).toBe("smaller-0");
  });

  it("hit-tests the deepest rectangle", () => {
    const rects = prepareTreemapLayout(sampleTree, 800, 600);
    const leaf = rects.find((rect) => rect.id === "node-3");
    expect(leaf).toBeDefined();
    if (leaf === undefined) {
      return;
    }
    const hit = findTreemapRectAt(rects, (leaf.x0 + leaf.x1) / 2, (leaf.y0 + leaf.y1) / 2);
    expect(hit?.id).toBe("node-3");
    expect(findTreemapRectAt(rects, -5, -5)).toBeNull();
  });
});

describe("treemap viewport", () => {
  const size = { width: 400, height: 200 };

  it("starts a zoom-in with the full layout inside the clicked rectangle", () => {
    const clickedRect = { x0: 100, y0: 50, x1: 200, y1: 100 };
    const startViewport = viewportForZoomIn(clickedRect, size);
    const projectedFullLayout = projectRect(
      { x0: 0, y0: 0, x1: size.width, y1: size.height },
      startViewport,
      size,
    );

    expect(projectedFullLayout.x0).toBeCloseTo(100);
    expect(projectedFullLayout.y0).toBeCloseTo(50);
    expect(projectedFullLayout.x1).toBeCloseTo(200);
    expect(projectedFullLayout.y1).toBeCloseTo(100);
  });

  it("starts a zoom-out with the previous folder filling the canvas", () => {
    const previousFolderRect = { x0: 20, y0: 10, x1: 220, y1: 110 };
    const projected = projectRect(previousFolderRect, viewportForZoomOut(previousFolderRect), size);
    expect(projected).toEqual({ x0: 0, y0: 0, x1: 400, y1: 200 });
  });

  it("interpolates and inverts projections", () => {
    const from = { x0: 0, y0: 0, x1: 100, y1: 100 };
    const to = { x0: 100, y0: 100, x1: 300, y1: 300 };
    expect(interpolateViewport(from, to, 0.5)).toEqual({ x0: 50, y0: 50, x1: 200, y1: 200 });

    const viewport = { x0: 50, y0: 25, x1: 250, y1: 125 };
    const layoutPoint = unprojectPoint(200, 100, viewport, size);
    expect(layoutPoint).toEqual({ x: 150, y: 75 });
  });
});

describe("prepareSunburstLayout", () => {
  it("splits the full turn by size and limits the ring count", () => {
    const arcs = prepareSunburstLayout(sampleTree, 1);
    expect(arcs.every((arc) => arc.ring === 1)).toBe(true);

    const largest = arcs.find((arc) => arc.id === "node-1");
    expect(largest?.endAngle).toBeDefined();
    if (largest === undefined) {
      return;
    }
    expect(largest.endAngle - largest.startAngle).toBeCloseTo(Math.PI * 2 * 0.6);
  });

  it("finds arcs by angle and ring", () => {
    const arcs = prepareSunburstLayout(sampleTree);
    const arcAtTop = findSunburstArcAt(arcs, 0.01, -12, 10, 5);
    expect(arcAtTop?.ring).toBe(1);
    expect(findSunburstArcAt(arcs, 0, 0, 10, 5)).toBeNull();
    expect(angleOfPoint(1, 0)).toBeCloseTo(Math.PI / 2);
  });

  it("starts zoom-in angles inside the clicked segment", () => {
    const viewport = angularViewportForZoomIn(1, 2);
    expect(projectAngle(0, viewport)).toBeCloseTo(1);
    expect(projectAngle(Math.PI * 2, viewport)).toBeCloseTo(2);
  });
});
