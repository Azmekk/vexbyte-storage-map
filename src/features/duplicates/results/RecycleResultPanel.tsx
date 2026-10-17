import { TriangleAlertIcon, XIcon } from "lucide-react";
import { motion } from "motion/react";
import { Button } from "@/components/ui/button";
import type { DuplicateRecycleResult } from "@/lib/backend/generated";
import { fadeSlideInVariants } from "@/lib/motion/transitions";
import { pluralize } from "../formatting/pluralize";
import { describeFileChangeReason, describeRecycleFailure } from "./describeRecycleResult";

const MAXIMUM_LISTED_FILES = 100;

type RecycleResultPanelProps = {
  result: DuplicateRecycleResult;
  onDismiss: () => void;
};

export function RecycleResultPanel({ result, onDismiss }: RecycleResultPanelProps) {
  const skippedFiles = [
    ...result.failed.map((failure) => ({
      path: failure.path,
      reason: describeRecycleFailure(failure.reason, failure.detail),
    })),
    ...result.changed.map((changedFile) => ({
      path: changedFile.path,
      reason: describeFileChangeReason(changedFile.reason),
    })),
  ];
  const listedFiles = skippedFiles.slice(0, MAXIMUM_LISTED_FILES);
  const unlistedCount = skippedFiles.length - listedFiles.length;

  return (
    <motion.section
      role="status"
      variants={fadeSlideInVariants}
      initial="hidden"
      animate="visible"
      className="grid gap-3 rounded-xl border border-amber-500/40 bg-amber-500/8 p-4"
      aria-labelledby="duplicate-skipped-heading"
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-2">
          <TriangleAlertIcon className="mt-0.5 size-4 shrink-0 text-amber-600 dark:text-amber-400" />
          <div className="grid gap-0.5">
            <h2 id="duplicate-skipped-heading" className="font-medium text-sm">
              {pluralize(skippedFiles.length, "file was", "files were")} left in place
            </h2>
            <p className="text-muted-foreground text-xs">
              Nothing happened to these files. Review them and search again if needed.
            </p>
          </div>
        </div>
        <Button variant="ghost" size="icon-sm" onClick={onDismiss} aria-label="Dismiss">
          <XIcon />
        </Button>
      </div>
      <ul className="grid max-h-48 gap-2 overflow-y-auto pr-1">
        {listedFiles.map((skippedFile) => (
          <li key={skippedFile.path} className="grid gap-0.5 text-xs">
            <span className="break-all font-medium">{skippedFile.path}</span>
            <span className="text-muted-foreground">{skippedFile.reason}</span>
          </li>
        ))}
      </ul>
      {unlistedCount > 0 && (
        <p className="text-muted-foreground text-xs">
          And {pluralize(unlistedCount, "more file", "more files")}.
        </p>
      )}
    </motion.section>
  );
}
