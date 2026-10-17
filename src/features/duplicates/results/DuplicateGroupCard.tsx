import { AnimatePresence, motion } from "motion/react";
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
};

export function DuplicateGroupCard({
  group,
  nowUnixMilliseconds,
  isLeaving,
  pathsBeingSaved,
  onToggleKeep,
  onReveal,
  onOpen,
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
      <header className="flex h-[52px] items-center justify-between gap-4 border-b bg-muted/30 px-4">
        <div className="flex min-w-0 items-baseline gap-3">
          <h3 className="truncate font-semibold text-sm">{description.copiesText}</h3>
          <span className="hidden truncate text-muted-foreground text-xs sm:inline">
            {description.removalText}
          </span>
        </div>
        <span className="shrink-0 font-medium text-sm tabular-nums">{description.wastedText}</span>
      </header>
      <ul className="divide-y">
        <AnimatePresence initial={false}>
          {description.files.map((file) => (
            <motion.li
              key={file.path}
              exit={{ opacity: 0, height: 0, transition: exitTransition }}
              className="overflow-hidden"
            >
              <DuplicateFileRow
                file={file}
                isSaving={pathsBeingSaved.has(file.path)}
                onToggleKeep={(path) => onToggleKeep(group, path)}
                onReveal={onReveal}
                onOpen={onOpen}
              />
            </motion.li>
          ))}
        </AnimatePresence>
      </ul>
    </motion.article>
  );
}
