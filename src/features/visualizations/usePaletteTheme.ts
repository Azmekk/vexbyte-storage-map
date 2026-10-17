import { useTheme } from "@/app/useTheme";
import type { PaletteTheme } from "./categoryPalette";

/** The palette variant matching the current light or dark theme. */
export function usePaletteTheme(): PaletteTheme {
  return useTheme().resolvedTheme;
}
