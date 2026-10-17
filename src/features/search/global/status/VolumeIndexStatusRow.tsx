import { RefreshCwIcon } from "lucide-react";
import { NumberTicker } from "@/components/NumberTicker";
import { Button } from "@/components/ui/button";
import type { VolumeIndexStatus } from "@/lib/backend/search";
import { cn } from "@/lib/classNames";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatCount } from "@/lib/formatting/formatCount";
import { formatModifiedDateAndTime } from "@/lib/formatting/formatModifiedDate";
import {
  describeIndexAge,
  describeVolumeIndexState,
  volumeIndexDisplayName,
} from "./describeVolumeIndexState";
import { IndexMethodBadge } from "./IndexMethodBadge";
import { VolumeIndexStateBadge } from "./VolumeIndexStateBadge";

const VOLUME_INDEX_GRID_COLUMNS =
  "grid-cols-[minmax(0,1.3fr)_8rem_minmax(0,1fr)_minmax(0,1.1fr)_4.5rem_2rem]";

type VolumeIndexStatusRowProps = {
  volume: VolumeIndexStatus;
  nowUnixMilliseconds: number;
  onRefresh: (mountPoint: string) => void;
};

function formatItemCount(count: number): string {
  return `${formatCount(count)} items`;
}

function formatFilesSoFar(count: number): string {
  return `${formatCount(count)} files so far`;
}

export function VolumeIndexStatusRow({
  volume,
  nowUnixMilliseconds,
  onRefresh,
}: VolumeIndexStatusRowProps) {
  const description = describeVolumeIndexState(volume);
  const { index, state } = volume;
  const isBusy = state.kind === "building" || state.kind === "idle";
  const canRefresh = state.kind !== "skipped" && !isBusy;
  const displayName = volumeIndexDisplayName(volume);

  return (
    <li className="grid gap-1 py-2.5">
      <div className={cn("grid items-center gap-3 text-sm", VOLUME_INDEX_GRID_COLUMNS)}>
        <span className="truncate font-medium" title={volume.mountPoint}>
          {displayName}
        </span>
        <span>
          <VolumeIndexStateBadge description={description} />
        </span>
        <span className="truncate text-muted-foreground text-xs tabular-nums">
          {state.kind === "building" && (
            <NumberTicker value={state.filesSoFar} formatValue={formatFilesSoFar} />
          )}
          {state.kind !== "building" &&
            (index === null ? "–" : formatItemCount(index.fileCount + index.directoryCount))}
        </span>
        <span
          className="flex min-w-0 items-center gap-1.5 text-muted-foreground text-xs"
          title={
            index === null ? undefined : formatModifiedDateAndTime(index.builtAtUnixMilliseconds)
          }
        >
          {index !== null && (
            <>
              <span className="truncate">
                {describeIndexAge(index.builtAtUnixMilliseconds, nowUnixMilliseconds)}
              </span>
              <IndexMethodBadge method={index.method} />
            </>
          )}
        </span>
        <span
          className="text-right text-muted-foreground text-xs tabular-nums"
          title={index === null ? undefined : "Estimated memory"}
        >
          {index === null ? "" : formatBytesAsReadableSize(index.estimatedMemoryBytes)}
        </span>
        <Button
          variant="ghost"
          size="icon-sm"
          disabled={!canRefresh}
          onClick={() => onRefresh(volume.mountPoint)}
          aria-label={`Refresh the index of ${displayName}`}
          title={canRefresh ? `Read ${volume.mountPoint} again` : undefined}
        >
          <RefreshCwIcon className={isBusy ? "animate-spin" : undefined} />
        </Button>
      </div>
      {description.detail !== null && (
        <p
          className={cn(
            "text-xs",
            description.tone === "failed" ? "text-destructive" : "text-muted-foreground",
          )}
        >
          {description.detail}
        </p>
      )}
    </li>
  );
}
