import { describe, expect, it } from "vitest";
import {
  categoryColor,
  categoryLabel,
  depthColor,
  FILE_CATEGORY_ORDER,
  legendEntries,
  paletteColor,
} from "./categoryPalette";
import { readableTextColorOn, relativeLuminance } from "./readableTextColor";

describe("category palette", () => {
  it("gives every category its own colour in both themes", () => {
    for (const theme of ["light", "dark"] as const) {
      const colors = FILE_CATEGORY_ORDER.map((category) => categoryColor(category, theme));
      expect(new Set(colors).size).toBe(FILE_CATEGORY_ORDER.length);
    }
  });

  it("keeps a category's colour fixed regardless of what else is shown", () => {
    expect(categoryColor("videos", "light")).toBe("#2a78d6");
    expect(categoryColor("pdfs", "light")).toBe("#e34948");
    expect(categoryColor("videos", "dark")).toBe("#3987e5");
  });

  it("uses neutral colours for files without a category and for folders", () => {
    expect(categoryColor(null, "light")).toBe(paletteColor("otherFiles", "light"));
    expect(paletteColor("folder", "dark")).not.toBe(paletteColor("folder", "light"));
  });

  it("clamps depth colours to the ramp", () => {
    expect(depthColor(0, "light")).toBe(depthColor(1, "light"));
    expect(depthColor(40, "light")).toBe(depthColor(6, "light"));
    expect(depthColor(2, "light")).not.toBe(depthColor(3, "light"));
  });

  it("labels categories for legends", () => {
    expect(categoryLabel("archivesAndInstallers")).toBe("Archives & installers");
    expect(categoryLabel(null)).toBe("Other files");
    expect(legendEntries()).toHaveLength(FILE_CATEGORY_ORDER.length + 1);
  });
});

describe("readableTextColorOn", () => {
  it("picks dark text on light fills and light text on dark fills", () => {
    expect(readableTextColorOn("#eda100")).toBe("#0b0b0b");
    expect(readableTextColorOn("#0d366b")).toBe("#ffffff");
    expect(readableTextColorOn("#ffffff")).toBe("#0b0b0b");
  });

  it("computes luminance at the extremes", () => {
    expect(relativeLuminance("#000000")).toBe(0);
    expect(relativeLuminance("#ffffff")).toBeCloseTo(1);
    expect(relativeLuminance("not a colour")).toBe(0);
  });
});
