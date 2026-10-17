import { useMemo } from "react";
import { toast } from "sonner";
import { openWithDefaultApp, revealInExplorer } from "@/lib/backend/fileActions";
import type { ScanId } from "@/lib/backend/scans";
import { getNodeDetails, type NodeIndex } from "@/lib/backend/tree";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { useExplorerSelectionStore } from "../useExplorerSelectionStore";
import type { NodeActionTarget } from "./nodeActionTarget";
import { useRecycleConfirmationStore } from "./useRecycleConfirmationStore";

export type NodeActions = {
  exploreFolder: (target: NodeActionTarget) => void;
  openItem: (target: NodeActionTarget) => void;
  revealItem: (target: NodeActionTarget) => void;
  copyPath: (target: NodeActionTarget) => void;
  requestRecycle: (target: NodeActionTarget) => void;
  selectItem: (node: NodeIndex) => void;
};

async function runFileAction(failureTitle: string, action: () => Promise<void>): Promise<void> {
  try {
    await action();
  } catch (error) {
    toast.error(failureTitle, { description: readableErrorMessage(error) });
  }
}

/** Everything a context menu, the inspector or a list row can do with one item. */
export function useNodeActions(scanId: ScanId): NodeActions {
  const navigateToNode = useExplorerSelectionStore((state) => state.navigateToNode);
  const selectNode = useExplorerSelectionStore((state) => state.selectNode);
  const requestRecycleConfirmation = useRecycleConfirmationStore((state) => state.requestRecycle);

  return useMemo<NodeActions>(
    () => ({
      exploreFolder: (target) => {
        if (target.kind === "directory") {
          navigateToNode(target.node);
        }
      },
      openItem: (target) =>
        void runFileAction(`Could not open ${target.name}`, () =>
          openWithDefaultApp(scanId, target.node),
        ),
      revealItem: (target) =>
        void runFileAction(`Could not show ${target.name} in File Explorer`, () =>
          revealInExplorer(scanId, target.node),
        ),
      copyPath: (target) =>
        void runFileAction("Could not copy the path", async () => {
          const details = await getNodeDetails(scanId, target.node);
          if (details.fullPath === null) {
            throw new Error(`${target.name} has no single location on disk.`);
          }
          await navigator.clipboard.writeText(details.fullPath);
          toast.success("Path copied", { description: details.fullPath });
        }),
      requestRecycle: (target) => requestRecycleConfirmation(scanId, [target.node]),
      selectItem: (node) => selectNode(node),
    }),
    [scanId, navigateToNode, selectNode, requestRecycleConfirmation],
  );
}
