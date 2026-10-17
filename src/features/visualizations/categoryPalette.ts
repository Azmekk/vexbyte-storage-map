import type { FileCategory } from "@/lib/backend/tree";

export type PaletteTheme = "light" | "dark";

/**
 * The fixed order categories take the categorical slots in. The order is part of the palette's
 * colour-blind safety (validated adjacent pairs), so it never changes and colours never follow
 * rank. The categories that usually take the most space come first.
 */
export const FILE_CATEGORY_ORDER: readonly FileCategory[] = [
  "videos",
  "images",
  "archivesAndInstallers",
  "audio",
  "documents",
  "spreadsheets",
  "presentations",
  "pdfs",
];

const KNOWN_CATEGORY_NAMES: readonly string[] = FILE_CATEGORY_ORDER;

/** Narrows a value reported by a select or toggle to a known category. */
export function isFileCategory(value: string): value is FileCategory {
  return KNOWN_CATEGORY_NAMES.includes(value);
}

/** Everything a treemap or chart rectangle can be coloured as. */
export type ColorKey = FileCategory | "otherFiles" | "folder" | "unexpandedFolder" | "smallerItems";

type NeutralColorKey = Exclude<ColorKey, FileCategory>;

const CATEGORICAL_SLOTS: Record<PaletteTheme, readonly string[]> = {
  light: ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"],
  dark: ["#3987e5", "#d95926", "#199e70", "#c98500", "#d55181", "#008300", "#9085e9", "#e66767"],
};

const NEUTRAL_COLORS: Record<PaletteTheme, Record<NeutralColorKey, string>> = {
  light: {
    otherFiles: "#9f9d96",
    folder: "#e6e5e0",
    unexpandedFolder: "#cfcec7",
    smallerItems: "#c9c8c0",
  },
  dark: {
    otherFiles: "#77756f",
    folder: "#2b2b29",
    unexpandedFolder: "#4a4a46",
    smallerItems: "#454542",
  },
};

function isNeutralColorKey(colorKey: ColorKey): colorKey is NeutralColorKey {
  return colorKey in NEUTRAL_COLORS.light;
}

/** One-hue blue ramp (light to dark) for colouring by depth. */
const DEPTH_RAMP: Record<PaletteTheme, readonly string[]> = {
  light: ["#9ec5f4", "#6da7ec", "#3987e5", "#256abf", "#184f95", "#0d366b"],
  dark: ["#104281", "#184f95", "#256abf", "#3987e5", "#6da7ec", "#9ec5f4"],
};

const CATEGORY_LABELS: Record<FileCategory, string> = {
  videos: "Videos",
  images: "Images",
  archivesAndInstallers: "Archives & installers",
  audio: "Audio",
  documents: "Documents",
  spreadsheets: "Spreadsheets",
  presentations: "Presentations",
  pdfs: "PDFs",
};

/** The colour key of a file; files of no known category are "other files". */
export function colorKeyForCategory(category: FileCategory | null): ColorKey {
  return category ?? "otherFiles";
}

export function paletteColor(colorKey: ColorKey, theme: PaletteTheme): string {
  if (isNeutralColorKey(colorKey)) {
    return NEUTRAL_COLORS[theme][colorKey];
  }

  const slotIndex = FILE_CATEGORY_ORDER.indexOf(colorKey);
  return CATEGORICAL_SLOTS[theme][slotIndex] ?? NEUTRAL_COLORS[theme].otherFiles;
}

export function categoryColor(category: FileCategory | null, theme: PaletteTheme): string {
  return paletteColor(colorKeyForCategory(category), theme);
}

/** Depth 1 is the first level below the shown folder; deeper levels clamp to the last step. */
export function depthColor(depth: number, theme: PaletteTheme): string {
  const ramp = DEPTH_RAMP[theme];
  const stepIndex = Math.min(Math.max(depth - 1, 0), ramp.length - 1);
  return ramp[stepIndex] ?? ramp[0] ?? "#3987e5";
}

export function categoryLabel(category: FileCategory | null): string {
  return category === null ? "Other files" : CATEGORY_LABELS[category];
}

/** Every colour key with its label, in legend order. */
export function legendEntries(): { colorKey: ColorKey; label: string }[] {
  const categoryEntries = FILE_CATEGORY_ORDER.map(
    (category): { colorKey: ColorKey; label: string } => ({
      colorKey: category,
      label: CATEGORY_LABELS[category],
    }),
  );

  return [...categoryEntries, { colorKey: "otherFiles", label: "Other files" }];
}
