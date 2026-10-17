import { HomeIcon, SearchXIcon } from "lucide-react";
import { TopBar } from "@/app/TopBar";
import { TopBarTitle } from "@/app/TopBarTitle";
import { useNavigationStore } from "@/app/useNavigationStore";
import { EmptyState } from "@/components/EmptyState";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { ExplorerScreen } from "@/features/explorer/ExplorerScreen";
import type { ScanId } from "@/lib/backend/scans";
import { describeScanTarget } from "./describeScanTarget";
import { ScanEndedState } from "./ScanEndedState";
import { ScanProgressScreen } from "./ScanProgressScreen";
import { startScanSession } from "./scanSession";
import { useCancelScan } from "./useCancelScan";
import { useScanStore } from "./useScanStore";

type ScanScreenProps = {
  scanId: ScanId;
};

/** One scan: its progress while running, the explorer once finished, or how it ended. */
export function ScanScreen({ scanId }: ScanScreenProps) {
  const record = useScanStore((state) => state.scans[scanId]);
  const navigateHome = useNavigationStore((state) => state.navigateHome);
  const { isCancelling, cancel } = useCancelScan(scanId);

  if (record === undefined) {
    return (
      <>
        <TopBar>
          <TopBarTitle title="Scan" />
        </TopBar>
        <EmptyState
          className="my-auto"
          icon={SearchXIcon}
          title="This scan is no longer available"
          description="It was removed from this session. Start a new scan from the home screen."
          actions={
            <Button onClick={navigateHome}>
              <HomeIcon />
              Back home
            </Button>
          }
        />
      </>
    );
  }

  if (record.status === "finished" && record.result !== null) {
    return <ExplorerScreen record={record} result={record.result} />;
  }

  const { title } = describeScanTarget(record.target);

  return (
    <>
      <TopBar>
        <TopBarTitle title={record.status === "running" ? `Scanning ${title}` : title} />
      </TopBar>
      <ScrollArea className="min-h-0 flex-1">
        <div className="flex min-h-[calc(100svh-3.5rem)] flex-col">
          {record.status === "running" && (
            <ScanProgressScreen record={record} isCancelling={isCancelling} onCancel={cancel} />
          )}
          {record.status === "finished" && (
            <div className="mx-auto grid w-full max-w-5xl gap-4 p-8" aria-busy>
              <p className="text-muted-foreground text-sm">Preparing the results…</p>
              <Skeleton className="h-10 w-1/2" />
              <Skeleton className="h-64 w-full" />
            </div>
          )}
          {(record.status === "cancelled" || record.status === "failed") && (
            <ScanEndedState
              record={record}
              onScanAgain={() => void startScanSession(record.target, record.method)}
              onStandardScan={() => void startScanSession(record.target, "folderWalk")}
              onBackHome={navigateHome}
            />
          )}
        </div>
      </ScrollArea>
    </>
  );
}
