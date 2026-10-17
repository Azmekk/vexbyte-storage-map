import { RefreshCwIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import type { SearchIndexStatus } from "@/lib/backend/search";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { useNowEveryMinute } from "../useNowEveryMinute";
import { totalEstimatedIndexMemoryBytes } from "./describeVolumeIndexState";
import { refreshSearchIndexWithFeedback } from "./refreshSearchIndexWithFeedback";
import { VolumeIndexStatusRow } from "./VolumeIndexStatusRow";

type SearchIndexStatusCardProps = {
  status: SearchIndexStatus | null;
  loadErrorMessage: string | null;
};

const SKELETON_ROW_KEYS = ["first", "second"];

/** Every drive's index with its state, size and age, and buttons to read drives again. */
export function SearchIndexStatusCard({ status, loadErrorMessage }: SearchIndexStatusCardProps) {
  const nowUnixMilliseconds = useNowEveryMinute();
  const volumes = status?.volumes ?? [];
  const totalMemoryBytes = totalEstimatedIndexMemoryBytes(volumes);
  const isAnyDriveBusy = volumes.some(
    (volume) => volume.state.kind === "building" || volume.state.kind === "idle",
  );

  return (
    <section aria-label="Search index" className="grid gap-2 rounded-xl border bg-card p-4">
      <header className="flex items-start justify-between gap-3">
        <div className="grid gap-0.5">
          <h2 className="font-medium text-sm">Search index</h2>
          <p className="text-muted-foreground text-xs">
            {totalMemoryBytes > 0
              ? `About ${formatBytesAsReadableSize(totalMemoryBytes)}, kept in memory while the app runs.`
              : "Built when the app starts and kept in memory while it runs."}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          disabled={status === null || isAnyDriveBusy}
          onClick={() => void refreshSearchIndexWithFeedback()}
        >
          <RefreshCwIcon />
          Refresh all
        </Button>
      </header>

      {loadErrorMessage !== null && (
        <p role="alert" className="text-destructive text-sm">
          The index status could not be loaded. {loadErrorMessage}
        </p>
      )}
      {status === null && loadErrorMessage === null && (
        <div className="grid gap-2 py-2">
          {SKELETON_ROW_KEYS.map((key) => (
            <Skeleton key={key} className="h-6 w-full" />
          ))}
        </div>
      )}
      {status !== null && volumes.length === 0 && (
        <p className="py-2 text-muted-foreground text-sm">
          No local drive was found to index. Connect a drive and choose Refresh all.
        </p>
      )}
      {volumes.length > 0 && (
        <ul className="divide-y">
          {volumes.map((volume) => (
            <VolumeIndexStatusRow
              key={volume.mountPoint}
              volume={volume}
              nowUnixMilliseconds={nowUnixMilliseconds}
              onRefresh={(mountPoint) => void refreshSearchIndexWithFeedback(mountPoint)}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
