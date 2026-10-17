import { AnimatePresence } from "motion/react";
import { TopBar } from "@/app/TopBar";
import { useNavigationStore } from "@/app/useNavigationStore";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
import { Skeleton } from "@/components/ui/skeleton";
import { describeScanTarget } from "@/features/scan/describeScanTarget";
import { startScanSession } from "@/features/scan/scanSession";
import type { ScanRecord, ScanResult } from "@/features/scan/useScanStore";
import { SearchPalette } from "@/features/search/SearchPalette";
import { VisualizationTabs } from "@/features/visualizations/VisualizationTabs";
import { RecycleConfirmationDialog } from "./actions/RecycleConfirmationDialog";
import { ExplorerBreadcrumbs } from "./breadcrumbs/ExplorerBreadcrumbs";
import { ExplorerTopBarActions } from "./ExplorerTopBarActions";
import { ExplorerHistoryButtons } from "./history/ExplorerHistoryButtons";
import { InspectorPanel } from "./inspector/InspectorPanel";
import { TreeTable } from "./tree/TreeTable";
import { useExplorerScreen } from "./useExplorerScreen";
import { useExplorerSelectionStore } from "./useExplorerSelectionStore";
import { useExplorerShortcuts } from "./useExplorerShortcuts";

type ExplorerScreenProps = {
  record: ScanRecord;
  result: ScanResult;
};

/** The results of a finished scan: tree table, visualizations and the inspector. */
export function ExplorerScreen({ record, result }: ExplorerScreenProps) {
  const scanId = record.scanId;
  const explorer = useExplorerScreen(scanId, result);
  const isInspectorOpen = useExplorerSelectionStore((state) => state.isInspectorOpen);
  const navigateHome = useNavigationStore((state) => state.navigateHome);
  const targetTitle = describeScanTarget(record.target).title;
  const rescan = () => void startScanSession(record.target, record.method);
  useExplorerShortcuts({
    rootNode: result.rootNode,
    shownFolder: explorer.shownFolder,
    parentOfShownFolder: explorer.parentOfShownFolder,
    onRescan: rescan,
  });

  return (
    <>
      <TopBar
        navigation={<ExplorerHistoryButtons scanId={scanId} />}
        actions={
          <ExplorerTopBarActions
            method={record.method}
            targetTitle={targetTitle}
            endedAtUnixMilliseconds={record.endedAtUnixMilliseconds}
            onRescan={rescan}
          />
        }
      >
        {explorer.crumbs.length > 0 ? (
          <ExplorerBreadcrumbs
            crumbs={explorer.crumbs}
            scanRoot={{ label: targetTitle, isVolume: record.target.volume !== undefined }}
            onOpenFolder={explorer.navigateToNode}
            onGoToYourDrives={navigateHome}
          />
        ) : (
          <Skeleton className="h-4 w-48" />
        )}
      </TopBar>

      {explorer.isReady ? (
        <div className="flex min-h-0 flex-1">
          <ResizablePanelGroup orientation="horizontal" className="min-w-0 flex-1">
            <ResizablePanel defaultSize="50" minSize={300}>
              <TreeTable
                scanId={scanId}
                shownFolder={explorer.shownFolder}
                parentOfShownFolder={explorer.parentOfShownFolder}
              />
            </ResizablePanel>
            <ResizableHandle withHandle />
            <ResizablePanel defaultSize="50" minSize={320}>
              <VisualizationTabs
                scanId={scanId}
                shownFolder={explorer.shownFolder}
                parentOfShownFolder={explorer.parentOfShownFolder}
                rootNode={result.rootNode}
              />
            </ResizablePanel>
          </ResizablePanelGroup>
          <AnimatePresence initial={false}>
            {isInspectorOpen && (
              <InspectorPanel
                key="inspector"
                scanId={scanId}
                shownFolder={explorer.shownFolder}
                result={result}
              />
            )}
          </AnimatePresence>
        </div>
      ) : (
        <div className="grid flex-1 grid-cols-2 gap-4 p-4" aria-busy>
          <Skeleton className="h-full" />
          <Skeleton className="h-full" />
        </div>
      )}

      <SearchPalette scanId={scanId} />
      <RecycleConfirmationDialog />
    </>
  );
}
