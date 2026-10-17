import { useCallback, useEffect } from "react";
import type { ScanResult } from "@/features/scan/useScanStore";
import type { ScanId } from "@/lib/backend/scans";
import type { BreadcrumbItem, NodeIndex } from "@/lib/backend/tree";
import { useLastSuccessfulData } from "@/lib/queryCache/useLastSuccessfulData";
import { useExplorerSelectionStore } from "./useExplorerSelectionStore";
import { usePathToRoot } from "./useTreeQueries";

export type ExplorerScreenModel = {
  isReady: boolean;
  shownFolder: NodeIndex;
  parentOfShownFolder: NodeIndex | null;
  crumbs: BreadcrumbItem[];
  navigateToNode: (folder: NodeIndex) => void;
};

/** Points the explorer stores at this scan and works out where the user is in its tree. */
export function useExplorerScreen(scanId: ScanId, result: ScanResult): ExplorerScreenModel {
  const showScan = useExplorerSelectionStore((state) => state.showScan);
  const boundScanId = useExplorerSelectionStore((state) => state.scanId);
  const currentFolder = useExplorerSelectionStore((state) => state.currentFolder);
  const navigateToNode = useExplorerSelectionStore((state) => state.navigateToNode);

  useEffect(() => {
    showScan(scanId, result.rootNode);
  }, [showScan, scanId, result.rootNode]);

  const isReady = boundScanId === scanId && currentFolder !== null;
  const shownFolder = isReady ? currentFolder : result.rootNode;
  const pathState = usePathToRoot(scanId, shownFolder).state;
  const lastCrumbs = useLastSuccessfulData(pathState);

  const crumbs = pathState.status === "error" ? [] : (lastCrumbs ?? []);

  useEffect(() => {
    if (pathState.status === "error" && isReady && shownFolder !== result.rootNode) {
      navigateToNode(result.rootNode, { replaceHistory: true });
    }
  }, [pathState.status, isReady, shownFolder, result.rootNode, navigateToNode]);

  const parentCrumb = crumbs.length >= 2 ? crumbs[crumbs.length - 2] : undefined;
  const isCrumbTrailCurrent = crumbs.at(-1)?.node === shownFolder;

  const navigateToFolder = useCallback(
    (folder: NodeIndex) => navigateToNode(folder),
    [navigateToNode],
  );

  return {
    isReady,
    shownFolder,
    parentOfShownFolder: isCrumbTrailCurrent ? (parentCrumb?.node ?? null) : null,
    crumbs,
    navigateToNode: navigateToFolder,
  };
}
