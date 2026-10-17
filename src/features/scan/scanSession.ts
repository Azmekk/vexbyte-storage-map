import { toast } from "sonner";
import { useNavigationStore } from "@/app/useNavigationStore";
import {
  discardScan,
  getScanSummary,
  type ScanFinished,
  type ScanId,
  type ScanMethod,
  type ScanProgressUpdate,
  type ScanTarget,
  startScan,
} from "@/lib/backend/scans";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { scanQueryCache } from "@/lib/queryCache/scanQueryCache";
import { describeScanTarget } from "./describeScanTarget";
import { useScanStore } from "./useScanStore";

/**
 * Starts a scan, tracks its progress in the scan store and shows its screen. Progress that
 * arrives before the backend returned the id is kept and applied once the id is known.
 */
export async function startScanSession(target: ScanTarget, method: ScanMethod): Promise<void> {
  let knownScanId: ScanId | null = null;
  let progressBeforeIdWasKnown: ScanProgressUpdate | null = null;

  const handleProgress = (progress: ScanProgressUpdate) => {
    if (knownScanId === null) {
      progressBeforeIdWasKnown = progress;
      return;
    }
    useScanStore.getState().recordProgress(knownScanId, progress);
  };

  try {
    const scanId = await startScan(target, method, handleProgress);
    knownScanId = scanId;
    useScanStore.getState().registerStartedScan(scanId, target, method, progressBeforeIdWasKnown);
    useNavigationStore.getState().navigateToScan(scanId);

    const record = useScanStore.getState().scans[scanId];
    if (record?.status === "finished" && record.result === null) {
      await loadScanResult(scanId);
    }
  } catch (error) {
    const targetTitle = describeScanTarget(target).title;
    toast.error(`Could not scan ${targetTitle}`, { description: readableErrorMessage(error) });
  }
}

async function loadScanResult(scanId: ScanId): Promise<void> {
  try {
    const overview = await getScanSummary(scanId);
    if (overview.state.kind === "finished") {
      useScanStore.getState().recordResult(scanId, {
        rootNode: overview.state.rootNode,
        summary: overview.state.summary,
        currentTotals: overview.state.currentTotals,
      });
    }
  } catch (error) {
    toast.error("The scan finished, but its results could not be loaded", {
      description: readableErrorMessage(error),
    });
  }
}

/** Applies a `scan-finished` event: records the outcome and loads the result when finished. */
export async function handleScanFinished(scanFinished: ScanFinished): Promise<void> {
  const { scanId, outcome } = scanFinished;
  useScanStore.getState().recordOutcome(scanId, outcome);

  const isRegistered = useScanStore.getState().scans[scanId] !== undefined;
  if (outcome.kind === "finished" && isRegistered) {
    await loadScanResult(scanId);
  }
}

/** Forgets a scan in the backend and the UI, and leaves its screen if it is shown. */
export async function discardScanSession(scanId: ScanId): Promise<void> {
  try {
    await discardScan(scanId);
  } catch (error) {
    toast.error("Could not remove this scan", { description: readableErrorMessage(error) });
    return;
  }

  useScanStore.getState().removeScan(scanId);
  scanQueryCache.removeScan(scanId);

  const currentView = useNavigationStore.getState().currentView;
  if (currentView.kind === "scan" && currentView.scanId === scanId) {
    useNavigationStore.getState().navigateHome();
  }
}
