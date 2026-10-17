import {
  CircleSlashIcon,
  HomeIcon,
  RotateCwIcon,
  ScanSearchIcon,
  TriangleAlertIcon,
} from "lucide-react";
import { EmptyState } from "@/components/EmptyState";
import { Button } from "@/components/ui/button";
import { describeScanFailure } from "./describeScanFailure";
import { describeScanTarget } from "./describeScanTarget";
import type { ScanRecord } from "./useScanStore";

type ScanEndedStateProps = {
  record: ScanRecord;
  onScanAgain: () => void;
  onStandardScan: () => void;
  onBackHome: () => void;
};

/** What the scan screen shows after a scan was cancelled or failed. */
export function ScanEndedState({
  record,
  onScanAgain,
  onStandardScan,
  onBackHome,
}: ScanEndedStateProps) {
  const { title } = describeScanTarget(record.target);
  const backHomeButton = (
    <Button variant="ghost" onClick={onBackHome}>
      <HomeIcon />
      Back home
    </Button>
  );

  if (record.outcome?.kind === "failed") {
    const failure = describeScanFailure(record.outcome.failure);
    const offersStandardScan = failure.suggestsStandardScan && record.method === "masterFileTable";

    return (
      <EmptyState
        className="my-auto"
        icon={TriangleAlertIcon}
        tone="warning"
        title={failure.title}
        description={failure.explanation}
        actions={
          <>
            {offersStandardScan ? (
              <Button onClick={onStandardScan}>
                <ScanSearchIcon />
                Run standard scan
              </Button>
            ) : (
              <Button onClick={onScanAgain}>
                <RotateCwIcon />
                Try again
              </Button>
            )}
            {backHomeButton}
          </>
        }
      />
    );
  }

  return (
    <EmptyState
      className="my-auto"
      icon={CircleSlashIcon}
      title="Scan cancelled"
      description={`The scan of ${title} was stopped before it finished, so there is nothing to show yet.`}
      actions={
        <>
          <Button onClick={onScanAgain}>
            <RotateCwIcon />
            Scan again
          </Button>
          {backHomeButton}
        </>
      }
    />
  );
}
