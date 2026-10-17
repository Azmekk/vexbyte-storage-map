import type { FileCategory } from "@/lib/backend/tree";
import { categoryColor, type PaletteTheme, paletteColor } from "./categoryPalette";
import { mixHexColors } from "./mixHexColors";

/** How much of the category colour a folder takes: enough to read, far lighter than its files. */
const FOLDER_TINT_AMOUNT: Record<PaletteTheme, number> = { light: 0.28, dark: 0.32 };

const tintedFolderColorCache = new Map<string, string>();

/** The neutral folder colour with a light tint of the category that fills most of the folder. */
export function tintedFolderColor(dominantCategory: FileCategory, theme: PaletteTheme): string {
  const cacheKey = `${theme}:${dominantCategory}`;
  const cachedColor = tintedFolderColorCache.get(cacheKey);
  if (cachedColor !== undefined) {
    return cachedColor;
  }

  const tintedColor = mixHexColors(
    paletteColor("folder", theme),
    categoryColor(dominantCategory, theme),
    FOLDER_TINT_AMOUNT[theme],
  );
  tintedFolderColorCache.set(cacheKey, tintedColor);
  return tintedColor;
}
