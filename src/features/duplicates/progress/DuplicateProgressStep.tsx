import { CloudOffIcon, LoaderCircleIcon, XIcon } from "lucide-react";
import { motion } from "motion/react";
import { NumberTicker } from "@/components/NumberTicker";
import { Button } from "@/components/ui/button";
import type { DuplicateSearchProgress } from "@/lib/backend/generated";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { fadeSlideInVariants } from "@/lib/motion/transitions";
import { formatCount } from "../formatting/formatCount";
import { pluralize } from "../formatting/pluralize";
import type { DuplicateSearchRun } from "../search/useDuplicateSearchStore";
import { describeCurrentActivity, describeSearchStages } from "./describeSearchStages";
import { SearchEndedState } from "./SearchEndedState";
import { SearchStageFunnel } from "./SearchStageFunnel";

type DuplicateProgressStepProps = {
  run: DuplicateSearchRun;
  progress: DuplicateSearchProgress;
  onCancel: () => void;
  onSearchAgain: () => void;
  onReturnToSetup: () => void;
};

export function DuplicateProgressStep({
  run,
  progress,
  onCancel,
  onSearchAgain,
  onReturnToSetup,
}: DuplicateProgressStepProps) {
  const stageDescriptions = describeSearchStages(progress);
  const hasEnded = run.status === "cancelled" || run.status === "failed";
  const isCancelling = run.status === "cancelling";

  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <motion.div
        className="mx-auto grid w-full max-w-3xl gap-8 p-6"
        variants={fadeSlideInVariants}
        initial="hidden"
        animate="visible"
      >
        <header className="flex flex-wrap items-start justify-between gap-4">
          <div className="grid gap-1.5">
            <h1 className="font-semibold text-2xl tracking-tight">
              {hasEnded ? "Duplicate search" : "Looking for duplicates"}
            </h1>
            <p className="text-muted-foreground text-sm" aria-live="polite">
              {hasEnded ? "The search is no longer running." : describeCurrentActivity(progress)}
            </p>
          </div>
          {!hasEnded && (
            <Button variant="outline" onClick={onCancel} disabled={isCancelling}>
              {isCancelling ? <LoaderCircleIcon className="animate-spin" /> : <XIcon />}
              {isCancelling ? "Cancelling…" : "Cancel"}
            </Button>
          )}
        </header>

        {hasEnded && (
          <SearchEndedState
            kind={run.status === "failed" ? "failed" : "cancelled"}
            errorMessage={run.status === "failed" ? run.errorMessage : null}
            onSearchAgain={onSearchAgain}
            onReturnToSetup={onReturnToSetup}
          />
        )}

        <SearchStageFunnel stageDescriptions={stageDescriptions} isStopped={hasEnded} />

        <dl className="grid grid-cols-2 gap-x-6 gap-y-4 border-t pt-6 text-sm sm:grid-cols-3">
          <div className="grid gap-0.5">
            <dt className="text-muted-foreground text-xs">Size of files found</dt>
            <dd className="font-medium">
              <NumberTicker
                value={progress.bytesCollected}
                formatValue={formatBytesAsReadableSize}
              />
            </dd>
          </div>
          <div className="grid gap-0.5">
            <dt className="text-muted-foreground text-xs">Folders looked through</dt>
            <dd className="font-medium">
              <NumberTicker value={progress.foldersListed} formatValue={formatCount} />
            </dd>
          </div>
          <div className="grid gap-0.5">
            <dt className="text-muted-foreground text-xs">Duplicate groups so far</dt>
            <dd className="font-medium">
              <NumberTicker value={progress.duplicateGroupsFound} formatValue={formatCount} />
            </dd>
          </div>
        </dl>

        {progress.onlineOnlyFilesSkipped > 0 && (
          <p className="flex items-center gap-2 text-muted-foreground text-sm">
            <CloudOffIcon className="size-4 shrink-0" aria-hidden />
            {pluralize(progress.onlineOnlyFilesSkipped, "file is", "files are")} online-only and
            won't be downloaded or checked.
          </p>
        )}
      </motion.div>
    </div>
  );
}
