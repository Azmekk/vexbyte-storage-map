import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex } from "@/lib/backend/tree";
import { selectCurrentHistory, useExplorerSelectionStore } from "../useExplorerSelectionStore";
import { useNodeDetails } from "../useTreeQueries";
import { backHistoryTarget, forwardHistoryTarget } from "./navigationHistory";

export type HistoryButtonModel = {
  isEnabled: boolean;
  /** The name of the folder the button leads to, once known. */
  targetName: string | null;
  go: () => void;
};

export type ExplorerHistoryButtons = {
  back: HistoryButtonModel;
  forward: HistoryButtonModel;
};

function useTargetName(scanId: ScanId, target: NodeIndex | null): string | null {
  const detailsState = useNodeDetails(scanId, target).state;
  return detailsState.status === "success" ? detailsState.data.name : null;
}

/** Back and forward through the folders shown in this scan, with the names they lead to. */
export function useExplorerHistoryButtons(scanId: ScanId): ExplorerHistoryButtons {
  const history = useExplorerSelectionStore(selectCurrentHistory);
  const goBack = useExplorerSelectionStore((state) => state.goBack);
  const goForward = useExplorerSelectionStore((state) => state.goForward);

  const backTarget = history === null ? null : backHistoryTarget(history);
  const forwardTarget = history === null ? null : forwardHistoryTarget(history);
  const backTargetName = useTargetName(scanId, backTarget);
  const forwardTargetName = useTargetName(scanId, forwardTarget);

  return {
    back: { isEnabled: backTarget !== null, targetName: backTargetName, go: goBack },
    forward: { isEnabled: forwardTarget !== null, targetName: forwardTargetName, go: goForward },
  };
}
