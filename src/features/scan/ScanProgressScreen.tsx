import { CircleStopIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatCount } from "@/lib/formatting/formatCount";
import { formatElapsedClock } from "@/lib/formatting/formatDuration";
import { scanPhaseLabel, topLevelFolderBars } from "./describeScanProgress";
import { describeScanTarget, scanMethodLabel } from "./describeScanTarget";
import { ScanCounter } from "./ScanCounter";
import { TopLevelFolderList } from "./TopLevelFolderList";
import type { ScanRecord } from "./useScanStore";

type ScanProgressScreenProps = {
  record: ScanRecord;
  isCancelling: boolean;
  onCancel: () => void;
};

export function ScanProgressScreen({ record, isCancelling, onCancel }: ScanProgressScreenProps) {
  const progress = record.latestProgress;
  const { title, location } = describeScanTarget(record.target);
  const isFastScan = record.method === "masterFileTable";

  return (
    <div className="mx-auto grid w-full max-w-5xl gap-10 p-8 lg:grid-cols-[minmax(0,1.3fr)_minmax(0,1fr)]">
      <section className="grid content-start gap-8">
        <header className="grid gap-3">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="secondary">{scanMethodLabel(record.method)}</Badge>
            <span
              className="flex items-center gap-2 text-muted-foreground text-sm"
              aria-live="polite"
            >
              <span className="relative flex size-2">
                <span className="absolute inline-flex size-full animate-ping rounded-full bg-(--volume-used) opacity-60 motion-reduce:animate-none" />
                <span className="relative inline-flex size-2 rounded-full bg-(--volume-used)" />
              </span>
              {isCancelling ? "Stopping" : scanPhaseLabel(progress?.phase ?? null)}
            </span>
          </div>
          <h1 className="font-semibold text-3xl tracking-tight">Scanning {title}</h1>
          <p className="text-muted-foreground text-sm">{location}</p>
        </header>

        <div className="grid gap-6">
          <ScanCounter
            label="found so far"
            value={progress?.bytesFound ?? 0}
            format={formatBytesAsReadableSize}
            isProminent
          />
          <div className="flex flex-wrap gap-x-12 gap-y-4">
            <ScanCounter label="files" value={progress?.filesFound ?? 0} format={formatCount} />
            <ScanCounter
              label="folders"
              value={progress?.directoriesFound ?? 0}
              format={formatCount}
            />
            <div className="grid gap-1">
              <span className="font-semibold text-3xl text-foreground/85 tabular-nums tracking-tight">
                {formatElapsedClock(progress?.elapsedMilliseconds ?? 0)}
              </span>
              <span className="text-muted-foreground text-sm">elapsed</span>
            </div>
          </div>
        </div>

        <p
          className="truncate font-mono text-muted-foreground text-xs"
          title={progress?.currentPath ?? ""}
        >
          {progress?.currentPath !== undefined && progress.currentPath.length > 0
            ? progress.currentPath
            : " "}
        </p>

        <div>
          <Button variant="outline" onClick={onCancel} disabled={isCancelling}>
            <CircleStopIcon />
            {isCancelling ? "Stopping…" : "Cancel scan"}
          </Button>
        </div>
      </section>

      <section className="grid content-start gap-4 rounded-xl border bg-card p-5">
        <h2 className="font-medium text-sm">Biggest folders so far</h2>
        <TopLevelFolderList folders={topLevelFolderBars(progress)} isFastScan={isFastScan} />
      </section>
    </div>
  );
}
