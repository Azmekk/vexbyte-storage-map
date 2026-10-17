import type { TreemapColorMode } from "@/features/explorer/useExplorerSelectionStore";
import type { FileCategory, VisualizationNodeKind } from "@/lib/backend/tree";
import { categoryColor, depthColor, type PaletteTheme, paletteColor } from "../categoryPalette";
import { tintedFolderColor } from "../tintedFolderColor";

export type ColoredItem = {
  kind: VisualizationNodeKind;
  category: FileCategory | null;
  depth: number;
  /** False for folders whose contents were left out of the layout to keep it small. */
  hasVisibleChildren: boolean;
  /** Folders only: tints the folder by category. Left out, folders stay neutral. */
  dominantCategory?: FileCategory | null;
};

/**
 * By category, files take their category's colour and folders stay neutral so the files inside
 * them carry the colour; folders that know their dominant category (the sunburst) take a light
 * tint of it. By depth, every block takes the step of its level. Merged "smaller
 * items" blocks are always neutral.
 */
export function treemapRectColor(
  item: ColoredItem,
  colorMode: TreemapColorMode,
  theme: PaletteTheme,
): string {
  if (item.kind === "smallerItems") {
    return paletteColor("smallerItems", theme);
  }
  if (colorMode === "depth") {
    return depthColor(item.depth, theme);
  }
  if (item.kind === "directory") {
    if (!item.hasVisibleChildren) {
      return paletteColor("unexpandedFolder", theme);
    }
    const dominantCategory = item.dominantCategory ?? null;
    return dominantCategory === null
      ? paletteColor("folder", theme)
      : tintedFolderColor(dominantCategory, theme);
  }
  return categoryColor(item.category, theme);
}
