import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import { NodeContextMenuContent } from "@/features/explorer/actions/NodeContextMenuContent";
import { useExplorerSelectionStore } from "@/features/explorer/useExplorerSelectionStore";
import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex, VisualizationNode } from "@/lib/backend/tree";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { describeVisualizationItem } from "../describeVisualizationItem";
import { usePaletteTheme } from "../usePaletteTheme";
import { useVisualizationInteractions } from "../useVisualizationInteractions";
import { VisualizationTooltip } from "../VisualizationTooltip";
import { useSunburstCanvas } from "./useSunburstCanvas";

type SunburstViewProps = {
  scanId: ScanId;
  subtree: VisualizationNode;
  parentOfShownFolder: NodeIndex | null;
};

export function SunburstView({ scanId, subtree, parentOfShownFolder }: SunburstViewProps) {
  const colorMode = useExplorerSelectionStore((state) => state.treemapColorMode);
  const selectedNode = useExplorerSelectionStore((state) => state.selectedNode);
  const openFolder = useExplorerSelectionStore((state) => state.openFolder);
  const theme = usePaletteTheme();
  const canvas = useSunburstCanvas({ subtree, colorMode, theme, selectedNode });
  const interactions = useVisualizationInteractions(scanId);
  const { geometry } = canvas;
  const centerDiameter = geometry.centerRadius * 2;

  return (
    <div ref={canvas.containerRef} className="relative min-h-0 flex-1 overflow-hidden">
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <canvas
            ref={canvas.canvasRef}
            className="absolute inset-0 size-full cursor-pointer"
            role="img"
            aria-label={`Sunburst of ${subtree.name}: ${canvas.arcCount} segments. Use the table for keyboard access.`}
            onPointerMove={canvas.handlePointerMove}
            onPointerLeave={canvas.handlePointerLeave}
            onClick={(event) => interactions.activateItem(canvas.arcAtPointer(event))}
            onContextMenu={(event) =>
              interactions.prepareContextMenu(event, canvas.arcAtPointer(event))
            }
          />
        </ContextMenuTrigger>
        <NodeContextMenuContent
          target={interactions.contextTarget}
          actions={interactions.actions}
          isScanRoot={false}
        />
      </ContextMenu>

      {centerDiameter > 40 && (
        <button
          type="button"
          disabled={parentOfShownFolder === null}
          onClick={() => parentOfShownFolder !== null && openFolder(parentOfShownFolder)}
          aria-label={parentOfShownFolder === null ? subtree.name : `Go up from ${subtree.name}`}
          className="absolute flex flex-col items-center justify-center rounded-full p-3 text-center outline-none transition-colors hover:bg-muted/60 focus-visible:ring-2 focus-visible:ring-ring disabled:hover:bg-transparent"
          style={{
            left: geometry.centerX - geometry.centerRadius,
            top: geometry.centerY - geometry.centerRadius,
            width: centerDiameter,
            height: centerDiameter,
          }}
        >
          <span className="font-semibold text-lg tabular-nums leading-tight">
            {formatBytesAsReadableSize(subtree.logicalSize)}
          </span>
          <span className="line-clamp-2 max-w-full break-all text-muted-foreground text-xs">
            {subtree.name}
          </span>
          {parentOfShownFolder !== null && (
            <span className="mt-0.5 text-[10px] text-muted-foreground">Click to go up</span>
          )}
        </button>
      )}

      {canvas.hover !== null && (
        <VisualizationTooltip
          item={describeVisualizationItem(
            {
              ...canvas.hover.arc,
              depth: canvas.hover.arc.ring,
              hasVisibleChildren: canvas.hover.arc.hasChildren,
            },
            colorMode,
            theme,
          )}
          pointerX={canvas.hover.pointerX}
          pointerY={canvas.hover.pointerY}
          containerWidth={canvas.containerRef.current?.clientWidth ?? 0}
          shownFolderSize={subtree.logicalSize}
        />
      )}
    </div>
  );
}
