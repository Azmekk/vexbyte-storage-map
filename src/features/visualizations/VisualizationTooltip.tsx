import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatCount } from "@/lib/formatting/formatCount";
import { formatShare } from "@/lib/formatting/formatShare";

const TOOLTIP_WIDTH = 240;
const POINTER_OFFSET = 14;

export type VisualizationTooltipItem = {
  name: string;
  kind: "file" | "directory" | "smallerItems";
  logicalSize: number;
  mergedItemCount: number;
  colorSwatch: string;
  typeLabel: string;
};

type VisualizationTooltipProps = {
  item: VisualizationTooltipItem;
  pointerX: number;
  pointerY: number;
  containerWidth: number;
  shownFolderSize: number;
};

/** Follows the pointer over the treemap or sunburst; flips left near the right edge. */
export function VisualizationTooltip({
  item,
  pointerX,
  pointerY,
  containerWidth,
  shownFolderSize,
}: VisualizationTooltipProps) {
  const fitsOnTheRight = pointerX + POINTER_OFFSET + TOOLTIP_WIDTH < containerWidth;
  const left = fitsOnTheRight
    ? pointerX + POINTER_OFFSET
    : Math.max(4, pointerX - POINTER_OFFSET - TOOLTIP_WIDTH);
  const share = shownFolderSize > 0 ? item.logicalSize / shownFolderSize : null;
  const title =
    item.kind === "smallerItems" ? `${formatCount(item.mergedItemCount)} smaller items` : item.name;

  return (
    <div
      role="tooltip"
      className="pointer-events-none absolute z-10 grid gap-1 rounded-lg border bg-popover px-3 py-2 text-popover-foreground text-xs shadow-md"
      style={{ left, top: pointerY + POINTER_OFFSET, width: TOOLTIP_WIDTH }}
    >
      <span className="break-words font-medium text-sm leading-snug">{title}</span>
      <span className="flex items-center gap-1.5 text-muted-foreground">
        <span
          aria-hidden
          className="size-2.5 shrink-0 rounded-sm"
          style={{ backgroundColor: item.colorSwatch }}
        />
        {item.typeLabel}
      </span>
      <span className="tabular-nums">
        <span className="font-medium">{formatBytesAsReadableSize(item.logicalSize)}</span>
        <span className="text-muted-foreground"> · {formatShare(share)} of this folder</span>
      </span>
      {item.kind === "directory" && <span className="text-muted-foreground">Click to zoom in</span>}
    </div>
  );
}
