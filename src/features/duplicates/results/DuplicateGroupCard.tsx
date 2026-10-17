import { Trash2Icon } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { Button } from "@/components/ui/button";
import type { DuplicateGroup } from "@/lib/backend/generated";
import { enterTransition, exitTransition } from "@/lib/motion/transitions";
import { DuplicateFileRow } from "./DuplicateFileRow";
import { describeDuplicateGroup } from "./describeDuplicateGroup";

type DuplicateGroupCardProps = {
  group: DuplicateGroup;
  nowUnixMilliseconds: number;
  isLeaving: boolean;
  pathsBeingSaved: ReadonlySet<string>;
  onToggleKeep: (group: DuplicateGroup, path: string) => void;
  onReveal: (path: string) => void;
  onOpen: (path: string) => void;
  onRecycleFile: (group: DuplicateGroup, path: string) => void;
  onRecycleGroup: (group: DuplicateGroup) => void;
};

export function DuplicateGroupCard({
  group,
  nowUnixMilliseconds,
  isLeaving,
  pathsBeingSaved,
  onToggleKeep,
  onReveal,
  onOpen,
  onRecycleFile,
  onRecycleGroup,
}: DuplicateGroupCardProps) {
  const description = describeDuplicateGroup(group, nowUnixMilliseconds);

  return (
    <motion.article
      className="overflow-hidden rounded-xl border bg-card"
      initial={false}
      animate={isLeaving ? { opacity: 0, scale: 0.98 } : { opacity: 1, scale: 1 }}
      transition={isLeaving ? exitTransition : enterTransition}
      aria-label={description.copiesText}
    >
      <header className="flex h-[52px] items-center justify-between gap-4 border-b bg-muted/30 pr-2.5 pl-4">
        <div className="flex min-w-0 items-baseline gap-3">
          <h3 className="truncate font-semibold text-sm">{description.copiesText}</h3>
          <span className="hidden truncate text-muted-foreground text-xs sm:inline">
            {description.removalText}
          </span>
        </div>
        <div className="flex shrink-0 items-center gap-3">
          <span className="font-medium text-sm tabular-nums">{description.wastedText}</span>
          <Button
            variant="ghost"
            size="sm"
            className="text-destructive hover:bg-destructive/10 hover:text-destructive dark:hover:bg-destructive/20"
            disabled={description.markedForRemovalCount === 0 || isLeaving}
            onClick={() => onRecycleGroup(group)}
          >
            <Trash2Icon />
            {description.recycleMarkedButtonText}
          </Button>
        </div>
      </header>
      <ul className="divide-y">
        <AnimatePresence initial={false}>
          {description.files.map((file) => (
            <DuplicateFileRow
              key={file.path}
              file={file}
              isSaving={pathsBeingSaved.has(file.path)}
              canRecycle={description.canRecycleSingleFile && !isLeaving}
              onToggleKeep={(path) => onToggleKeep(group, path)}
              onReveal={onReveal}
              onOpen={onOpen}
              onRecycle={(path) => onRecycleFile(group, path)}
            />
          ))}
        </AnimatePresence>
      </ul>
    </motion.article>
  );
}
