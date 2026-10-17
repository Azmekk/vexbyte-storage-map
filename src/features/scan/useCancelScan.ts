import { useCallback, useState } from "react";
import { toast } from "sonner";
import { cancelScan, type ScanId } from "@/lib/backend/scans";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";

export function useCancelScan(scanId: ScanId): { isCancelling: boolean; cancel: () => void } {
  const [cancelledScanId, setCancelledScanId] = useState<ScanId | null>(null);

  const cancel = useCallback(() => {
    setCancelledScanId(scanId);
    void cancelScan(scanId).catch((error: unknown) => {
      setCancelledScanId(null);
      toast.error("The scan could not be stopped", { description: readableErrorMessage(error) });
    });
  }, [scanId]);

  return { isCancelling: cancelledScanId === scanId, cancel };
}
