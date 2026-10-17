import type { TreemapColorMode } from "@/features/explorer/useExplorerSelectionStore";
import { depthColor, legendEntries, type PaletteTheme, paletteColor } from "./categoryPalette";

type VisualizationLegendProps = {
  colorMode: TreemapColorMode;
  theme: PaletteTheme;
};

const DEPTH_LEVELS = [1, 2, 3, 4, 5, 6];

export function VisualizationLegend({ colorMode, theme }: VisualizationLegendProps) {
  if (colorMode === "depth") {
    return (
      <div className="flex items-center gap-2 text-muted-foreground text-xs">
        <span>Top level</span>
        <div className="flex overflow-hidden rounded-sm" aria-hidden>
          {DEPTH_LEVELS.map((depth) => (
            <span
              key={depth}
              className="h-2.5 w-5"
              style={{ backgroundColor: depthColor(depth, theme) }}
            />
          ))}
        </div>
        <span>Deeper</span>
      </div>
    );
  }

  return (
    <ul className="flex flex-wrap items-center gap-x-3 gap-y-1 text-muted-foreground text-xs">
      {legendEntries().map((entry) => (
        <li key={entry.colorKey} className="flex items-center gap-1.5">
          <span
            aria-hidden
            className="size-2.5 rounded-sm"
            style={{ backgroundColor: paletteColor(entry.colorKey, theme) }}
          />
          {entry.label}
        </li>
      ))}
      <li className="flex items-center gap-1.5">
        <span
          aria-hidden
          className="size-2.5 rounded-sm border"
          style={{ backgroundColor: paletteColor("unexpandedFolder", theme) }}
        />
        Folders
      </li>
    </ul>
  );
}
