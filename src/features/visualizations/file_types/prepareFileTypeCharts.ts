import type { FileCategory, FileTypeBreakdown } from "@/lib/backend/tree";
import {
  categoryColor,
  categoryLabel,
  FILE_CATEGORY_ORDER,
  type PaletteTheme,
} from "../categoryPalette";

export type CategorySlice = {
  key: string;
  category: FileCategory | null;
  label: string;
  logicalSize: number;
  fileCount: number;
  share: number;
  fill: string;
};

export type ExtensionBar = {
  key: string;
  label: string;
  category: FileCategory | null;
  logicalSize: number;
  fileCount: number;
  share: number;
  fill: string;
};

export const TOP_EXTENSION_COUNT = 15;

function shareOf(size: number, total: number): number {
  return total > 0 ? size / total : 0;
}

/**
 * Donut slices in the palette's fixed category order (so colours sit next to their validated
 * neighbours), with "Other files" last. Categories without any size are left out.
 */
export function prepareCategorySlices(
  breakdown: FileTypeBreakdown,
  theme: PaletteTheme,
): CategorySlice[] {
  const total = breakdown.totalLogicalSize;
  const orderedCategories: (FileCategory | null)[] = [...FILE_CATEGORY_ORDER, null];

  return orderedCategories.flatMap((category) => {
    const share = breakdown.categories.find((candidate) => candidate.category === category);
    if (share === undefined || share.logicalSize <= 0) {
      return [];
    }
    return [
      {
        key: category ?? "otherFiles",
        category,
        label: categoryLabel(category),
        logicalSize: share.logicalSize,
        fileCount: share.fileCount,
        share: shareOf(share.logicalSize, total),
        fill: categoryColor(category, theme),
      },
    ];
  });
}

/** The largest extensions, each bar coloured by the category its extension belongs to. */
export function prepareExtensionBars(
  breakdown: FileTypeBreakdown,
  theme: PaletteTheme,
): ExtensionBar[] {
  return breakdown.extensions
    .filter((extension) => extension.logicalSize > 0)
    .slice(0, TOP_EXTENSION_COUNT)
    .map((extension) => ({
      key: extension.extension ?? "(none)",
      label: extension.extension === null ? "No extension" : `.${extension.extension}`,
      category: extension.category,
      logicalSize: extension.logicalSize,
      fileCount: extension.fileCount,
      share: shareOf(extension.logicalSize, breakdown.totalLogicalSize),
      fill: categoryColor(extension.category, theme),
    }));
}
