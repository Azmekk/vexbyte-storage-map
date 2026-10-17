import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import { NodeContextMenuContent } from "@/features/explorer/actions/NodeContextMenuContent";
import { useExplorerSelectionStore } from "@/features/explorer/useExplorerSelectionStore";
import type { ScanId } from "@/lib/backend/scans";
import type { VisualizationNode } from "@/lib/backend/tree";
import { describeVisualizationItem } from "../describeVisualizationItem";
import { usePaletteTheme } from "../usePaletteTheme";
import { useVisualizationInteractions } from "../useVisualizationInteractions";
import { VisualizationTooltip } from "../VisualizationTooltip";
import { useTreemapCanvas } from "./useTreemapCanvas";

type TreemapViewProps = {
  scanId: ScanId;
  subtree: VisualizationNode;
};

export function TreemapView({ scanId, subtree }: TreemapViewProps) {
  const colorMode = useExplorerSelectionStore((state) => state.treemapColorMode);
  const selectedNode = useExplorerSelectionStore((state) => state.selectedNode);
  const theme = usePaletteTheme();
  const canvas = useTreemapCanvas({ subtree, colorMode, theme, selectedNode });
  const interactions = useVisualizationInteractions(scanId);

  return (
    <div ref={canvas.containerRef} className="relative min-h-0 flex-1 overflow-hidden rounded-md">
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <canvas
            ref={canvas.canvasRef}
            className="absolute inset-0 size-full cursor-pointer"
            role="img"
            aria-label={`Treemap of ${subtree.name}: ${canvas.rectCount} blocks. Use the table for keyboard access.`}
            onPointerMove={canvas.handlePointerMove}
            onPointerLeave={canvas.handlePointerLeave}
            onClick={(event) => interactions.activateItem(canvas.rectAtPointer(event))}
            onContextMenu={(event) =>
              interactions.prepareContextMenu(event, canvas.rectAtPointer(event))
            }
          />
        </ContextMenuTrigger>
        <NodeContextMenuContent
          target={interactions.contextTarget}
          actions={interactions.actions}
          isScanRoot={false}
        />
      </ContextMenu>
      {canvas.hover !== null && (
        <VisualizationTooltip
          item={describeVisualizationItem(
            { ...canvas.hover.rect, hasVisibleChildren: canvas.hover.rect.isContainer },
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
