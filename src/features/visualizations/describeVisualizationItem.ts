import type { TreemapColorMode } from "@/features/explorer/useExplorerSelectionStore";
import type { FileCategory, NodeIndex, VisualizationNodeKind } from "@/lib/backend/tree";
import { categoryLabel, type PaletteTheme } from "./categoryPalette";
import { treemapRectColor } from "./treemap/treemapRectColor";
import type { VisualizationTooltipItem } from "./VisualizationTooltip";

type VisualizationItem = {
  node: NodeIndex | null;
  name: string;
  kind: VisualizationNodeKind;
  category: FileCategory | null;
  logicalSize: number;
  mergedItemCount: number;
  depth: number;
  hasVisibleChildren: boolean;
};

function typeLabel(item: VisualizationItem): string {
  switch (item.kind) {
    case "directory":
      return "Folder";
    case "smallerItems":
      return "Merged small items";
    case "file":
      return categoryLabel(item.category);
  }
}

export function describeVisualizationItem(
  item: VisualizationItem,
  colorMode: TreemapColorMode,
  theme: PaletteTheme,
): VisualizationTooltipItem {
  return {
    name: item.name,
    kind: item.kind,
    logicalSize: item.logicalSize,
    mergedItemCount: item.mergedItemCount,
    colorSwatch: treemapRectColor(item, colorMode, theme),
    typeLabel: typeLabel(item),
  };
}
