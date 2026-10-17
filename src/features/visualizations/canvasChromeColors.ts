import type { PaletteTheme } from "./categoryPalette";

/** Non-data colours of the canvas charts: ink, outlines and the gaps between blocks. */
export type CanvasChromeColors = {
  surface: string;
  primaryInk: string;
  secondaryInk: string;
  hoverOutline: string;
  selectionOutline: string;
};

const CHROME_COLORS: Record<PaletteTheme, CanvasChromeColors> = {
  light: {
    surface: "#ffffff",
    primaryInk: "#0b0b0b",
    secondaryInk: "#52514e",
    hoverOutline: "#0b0b0b",
    selectionOutline: "#4f46e5",
  },
  dark: {
    surface: "#0a0a0a",
    primaryInk: "#f0efec",
    secondaryInk: "#c3c2b7",
    hoverOutline: "#ffffff",
    selectionOutline: "#a5b4fc",
  },
};

export function canvasChromeColors(theme: PaletteTheme): CanvasChromeColors {
  return CHROME_COLORS[theme];
}
