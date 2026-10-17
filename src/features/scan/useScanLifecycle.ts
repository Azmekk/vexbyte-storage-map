import { useEffect } from "react";
import { getStartupScanRequest } from "@/lib/backend/elevation";
import { listScans, subscribeToScanFinished } from "@/lib/backend/scans";
import { handleScanFinished, startScanSession } from "./scanSession";
import { useScanStore } from "./useScanStore";

let startupScanRequestWasHandled = false;

async function startRequestedStartupScan(): Promise<void> {
  if (startupScanRequestWasHandled) {
    return;
  }
  startupScanRequestWasHandled = true;

  try {
    const startupScanRequest = await getStartupScanRequest();
    if (startupScanRequest !== null) {
      await startScanSession(startupScanRequest.target, startupScanRequest.method);
    }
  } catch {
    // Without a startup request the app simply opens on the home screen.
  }
}

async function restoreScansOfThisSession(): Promise<void> {
  try {
    const overviews = await listScans();
    useScanStore.getState().mergeOverviews(overviews);
  } catch {
    // The recent scans list stays empty; new scans still work.
  }
}

/**
 * Listens for finished scans for the whole app lifetime, restores the scans the backend still
 * holds (after a reload) and starts the scan an elevated relaunch asked for. The listener is
 * attached before anything can start a scan.
 */
export function useScanLifecycle(): void {
  useEffect(() => {
    let isSubscribed = true;
    let stopListening: (() => void) | null = null;

    void subscribeToScanFinished((scanFinished) => {
      void handleScanFinished(scanFinished);
    }).then((unlisten) => {
      if (isSubscribed) {
        stopListening = unlisten;
        void restoreScansOfThisSession();
        void startRequestedStartupScan();
      } else {
        unlisten();
      }
    });

    return () => {
      isSubscribed = false;
      stopListening?.();
    };
  }, []);
}
