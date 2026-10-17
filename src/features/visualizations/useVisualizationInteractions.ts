import { type MouseEvent, useCallback, useState } from "react";
import type { NodeActionTarget } from "@/features/explorer/actions/nodeActionTarget";
import { type NodeActions, useNodeActions } from "@/features/explorer/actions/useNodeActions";
import { useExplorerSelectionStore } from "@/features/explorer/useExplorerSelectionStore";
import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex, VisualizationNodeKind } from "@/lib/backend/tree";

type ClickableItem = { node: NodeIndex | null; name: string; kind: VisualizationNodeKind };

export type VisualizationInteractions = {
  actions: NodeActions;
  contextTarget: NodeActionTarget | null;
  /** Click: folders zoom in, files are selected for the inspector. */
  activateItem: (item: ClickableItem | null) => void;
  /** Right-click: remembers the item for the shared context menu, or suppresses the menu. */
  prepareContextMenu: (event: MouseEvent, item: ClickableItem | null) => void;
};

/** What clicks and right-clicks on treemap blocks and sunburst segments do. */
export function useVisualizationInteractions(scanId: ScanId): VisualizationInteractions {
  const actions = useNodeActions(scanId);
  const openFolder = useExplorerSelectionStore((state) => state.openFolder);
  const selectNode = useExplorerSelectionStore((state) => state.selectNode);
  const [contextTarget, setContextTarget] = useState<NodeActionTarget | null>(null);

  const activateItem = useCallback(
    (item: ClickableItem | null) => {
      if (item === null || item.node === null) {
        return;
      }
      if (item.kind === "directory") {
        openFolder(item.node);
      } else {
        selectNode(item.node);
      }
    },
    [openFolder, selectNode],
  );

  const prepareContextMenu = useCallback(
    (event: MouseEvent, item: ClickableItem | null) => {
      if (item === null || item.node === null || item.kind === "smallerItems") {
        event.preventDefault();
        setContextTarget(null);
        return;
      }
      selectNode(item.node);
      setContextTarget({ node: item.node, name: item.name, kind: item.kind, isSynthetic: false });
    },
    [selectNode],
  );

  return { actions, contextTarget, activateItem, prepareContextMenu };
}
