import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { useScanStore } from "@/features/scan/useScanStore";
import { moveToRecycleBin } from "@/lib/backend/fileActions";
import { getNodeDetails, type NodeDetails } from "@/lib/backend/tree";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { scanQueryCache } from "@/lib/queryCache/scanQueryCache";
import { useExplorerSelectionStore } from "../useExplorerSelectionStore";
import { describeRecycleOutcome } from "./describeRecycleOutcome";
import { useRecycleConfirmationStore } from "./useRecycleConfirmationStore";

export type RecycleConfirmation = {
  isOpen: boolean;
  items: NodeDetails[] | null;
  loadErrorMessage: string | null;
  totalSize: number;
  isMoving: boolean;
  confirm: () => void;
  cancel: () => void;
};

/**
 * Loads what the confirmation dialog lists, moves the items after the user confirmed, then
 * refreshes every cached view of the scan and keeps the selection on something that exists.
 */
export function useRecycleConfirmation(): RecycleConfirmation {
  const scanId = useRecycleConfirmationStore((state) => state.scanId);
  const pendingNodes = useRecycleConfirmationStore((state) => state.pendingNodes);
  const closeConfirmation = useRecycleConfirmationStore((state) => state.closeConfirmation);
  const [items, setItems] = useState<NodeDetails[] | null>(null);
  const [loadErrorMessage, setLoadErrorMessage] = useState<string | null>(null);
  const [isMoving, setIsMoving] = useState(false);

  useEffect(() => {
    setItems(null);
    setLoadErrorMessage(null);

    if (scanId === null || pendingNodes.length === 0) {
      return;
    }

    let isCurrentRequest = true;
    Promise.all(pendingNodes.map((node) => getNodeDetails(scanId, node))).then(
      (loadedItems) => {
        if (isCurrentRequest) {
          setItems(loadedItems);
        }
      },
      (error: unknown) => {
        if (isCurrentRequest) {
          setLoadErrorMessage(readableErrorMessage(error));
        }
      },
    );

    return () => {
      isCurrentRequest = false;
    };
  }, [scanId, pendingNodes]);

  const confirm = useCallback(() => {
    if (scanId === null || items === null || items.length === 0) {
      return;
    }

    setIsMoving(true);
    const nodes = items.map((item) => item.node);

    moveToRecycleBin(scanId, nodes)
      .then((result) => {
        scanQueryCache.invalidateScan(scanId);
        useScanStore.getState().updateCurrentTotals(scanId, result.totals);

        const movedPaths = new Set(result.recycleResult.moved);
        const movedItems = items.filter(
          (item) => item.fullPath !== null && movedPaths.has(item.fullPath),
        );
        const explorer = useExplorerSelectionStore.getState();
        for (const movedItem of movedItems) {
          if (explorer.currentFolder === movedItem.node && movedItem.parent !== null) {
            explorer.openFolder(movedItem.parent);
          }
          if (useExplorerSelectionStore.getState().selectedNode === movedItem.node) {
            useExplorerSelectionStore.getState().selectNode(null);
          }
        }

        const messages = describeRecycleOutcome(result);
        if (messages.success !== null) {
          toast.success(messages.success.title, { description: messages.success.description });
        }
        if (messages.failure !== null) {
          toast.warning(messages.failure.title, { description: messages.failure.description });
        }
        closeConfirmation();
      })
      .catch((error: unknown) => {
        toast.error("Nothing was moved to the Recycle Bin", {
          description: readableErrorMessage(error),
        });
      })
      .finally(() => setIsMoving(false));
  }, [scanId, items, closeConfirmation]);

  const cancel = useCallback(() => {
    if (!isMoving) {
      closeConfirmation();
    }
  }, [isMoving, closeConfirmation]);

  const totalSize = (items ?? []).reduce((sizeSoFar, item) => sizeSoFar + item.logicalSize, 0);

  return {
    isOpen: pendingNodes.length > 0,
    items,
    loadErrorMessage,
    totalSize,
    isMoving,
    confirm,
    cancel,
  };
}
