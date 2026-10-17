import { describe, expect, it } from "vitest";
import type { FileTypeBreakdown } from "@/lib/backend/tree";
import { categoryColor } from "../categoryPalette";
import {
  prepareCategorySlices,
  prepareExtensionBars,
  TOP_EXTENSION_COUNT,
} from "./prepareFileTypeCharts";

const breakdown: FileTypeBreakdown = {
  totalLogicalSize: 1000,
  categories: [
    { category: null, logicalSize: 100, allocatedSize: 100, fileCount: 4, shareOfTotal: 0.1 },
    { category: "images", logicalSize: 300, allocatedSize: 300, fileCount: 3, shareOfTotal: 0.3 },
    { category: "videos", logicalSize: 600, allocatedSize: 600, fileCount: 1, shareOfTotal: 0.6 },
    { category: "pdfs", logicalSize: 0, allocatedSize: 0, fileCount: 2, shareOfTotal: 0 },
  ],
  extensions: Array.from({ length: 20 }, (_, index) => ({
    extension: index === 0 ? null : `ext${index}`,
    category: index === 1 ? ("videos" as const) : null,
    logicalSize: 100 - index,
    allocatedSize: 100 - index,
    fileCount: 1,
    shareOfTotal: null,
  })),
};

describe("prepareCategorySlices", () => {
  it("orders slices by the palette order with other files last, and drops empty ones", () => {
    const slices = prepareCategorySlices(breakdown, "light");
    expect(slices.map((slice) => slice.key)).toEqual(["videos", "images", "otherFiles"]);
    expect(slices[0]?.share).toBeCloseTo(0.6);
    expect(slices[0]?.fill).toBe(categoryColor("videos", "light"));
  });
});

describe("prepareExtensionBars", () => {
  it("keeps the top extensions and colours them by category", () => {
    const bars = prepareExtensionBars(breakdown, "dark");
    expect(bars).toHaveLength(TOP_EXTENSION_COUNT);
    expect(bars[0]?.label).toBe("No extension");
    expect(bars[1]?.label).toBe(".ext1");
    expect(bars[1]?.fill).toBe(categoryColor("videos", "dark"));
  });
});
