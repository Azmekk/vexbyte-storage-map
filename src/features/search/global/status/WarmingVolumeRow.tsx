import { NumberTicker } from "@/components/NumberTicker";
import type { VolumeIndexStatus } from "@/lib/backend/search";
import { formatCount } from "@/lib/formatting/formatCount";
import { formatElapsedClock } from "@/lib/formatting/formatDuration";
import { volumeIndexDisplayName } from "./describeVolumeIndexState";
import { IndexMethodBadge } from "./IndexMethodBadge";

type WarmingVolumeRowProps = {
  volume: VolumeIndexStatus;
};

function formatFileCount(count: number): string {
  return `${formatCount(count)} files`;
}

export function WarmingVolumeRow({ volume }: WarmingVolumeRowProps) {
  const { state } = volume;

  return (
    <li className="grid grid-cols-[minmax(0,1fr)_auto_3.5rem_5rem] items-center gap-4 py-2 text-sm">
      <span className="flex min-w-0 items-baseline gap-2">
        <span className="truncate font-medium">{volumeIndexDisplayName(volume)}</span>
        <span className="shrink-0 text-muted-foreground text-xs">{volume.mountPoint}</span>
      </span>
      {state.kind === "building" ? (
        <>
          <NumberTicker
            value={state.filesSoFar}
            formatValue={formatFileCount}
            className="text-right"
          />
          <span className="text-right text-muted-foreground text-xs tabular-nums">
            {formatElapsedClock(state.elapsedMilliseconds)}
          </span>
          <span className="flex justify-end">
            <IndexMethodBadge method={state.method} />
          </span>
        </>
      ) : (
        <span className="col-span-3 text-right text-muted-foreground text-xs">
          Waiting for its turn
        </span>
      )}
    </li>
  );
}
