import { toast } from "sonner";
import type { ScanId } from "@/lib/backend/scans";
import { getNodeDetails, type NodeIndex } from "@/lib/backend/tree";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { useExplorerSelectionStore } from "./useExplorerSelectionStore";

/**
 * Shows an item found elsewhere (search, largest files) in the explorer: a folder is opened, a
 * file is selected inside its folder.
 */
export async function revealNodeInExplorer(scanId: ScanId, node: NodeIndex): Promise<void> {
  try {
    const details = await getNodeDetails(scanId, node);
    const explorer = useExplorerSelectionStore.getState();

    if (details.kind === "directory" || details.parent === null) {
      explorer.openFolder(details.node);
      return;
    }

    explorer.revealNode(details.parent, details.node);
  } catch (error) {
    toast.error("Could not show this item", { description: readableErrorMessage(error) });
  }
}
