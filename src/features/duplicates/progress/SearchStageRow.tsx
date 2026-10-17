import { CheckIcon } from "lucide-react";
import { motion } from "motion/react";
import { NumberTicker } from "@/components/NumberTicker";
import { enterTransition } from "@/lib/motion/transitions";
import { formatCount } from "../formatting/formatCount";
import type { SearchStageDescription } from "./describeSearchStages";

type SearchStageRowProps = {
  stageDescription: SearchStageDescription;
  stepNumber: number;
  isLastStage: boolean;
  isStopped: boolean;
};

export function SearchStageRow({
  stageDescription,
  stepNumber,
  isLastStage,
  isStopped,
}: SearchStageRowProps) {
  const {
    state,
    label,
    fileCount,
    fileCountCaption,
    completedFraction,
    funnelBarShare,
    funnelFilledShare,
  } = stageDescription;
  const hasStarted = state !== "pending";
  const isRunningHere = state === "active" && !isStopped;

  return (
    <li
      className="relative grid grid-cols-[2rem_minmax(0,1fr)] gap-x-4"
      data-state={state}
      aria-current={state === "active" ? "step" : undefined}
    >
      {!isLastStage && (
        <span
          className="absolute top-9 bottom-0 left-4 w-px -translate-x-1/2 bg-border data-[complete=true]:bg-(--volume-used)/60"
          data-complete={state === "complete"}
          aria-hidden
        />
      )}
      <span
        className="relative z-10 mt-0.5 flex size-8 items-center justify-center rounded-full border bg-background font-medium text-muted-foreground text-xs tabular-nums data-[state=active]:border-(--volume-used) data-[state=complete]:border-(--volume-used) data-[state=complete]:bg-(--volume-used) data-[state=active]:text-(--volume-used) data-[state=complete]:text-white"
        data-state={state}
        aria-hidden
      >
        {state === "complete" ? <CheckIcon className="size-4" /> : stepNumber}
        {isRunningHere && (
          <span className="absolute inset-[-4px] animate-pulse rounded-full border border-(--volume-used)/40 motion-reduce:animate-none" />
        )}
      </span>

      <div className="grid gap-2 pb-8">
        <div className="flex flex-wrap items-baseline justify-between gap-x-6 gap-y-1">
          <h3
            className="font-medium text-muted-foreground text-sm data-[started=true]:text-foreground"
            data-started={hasStarted}
          >
            {label}
          </h3>
          <p className="flex items-baseline gap-2 text-muted-foreground text-sm">
            <NumberTicker
              value={hasStarted ? fileCount : 0}
              formatValue={formatCount}
              className="font-semibold text-2xl text-foreground tracking-tight"
            />
            <span>{hasStarted ? fileCountCaption : "waiting"}</span>
          </p>
        </div>

        <div className="relative h-2 overflow-hidden rounded-full bg-muted" aria-hidden>
          <motion.div
            className="absolute inset-y-0 left-0 rounded-full bg-(--volume-used)/25"
            initial={false}
            animate={{ width: `${funnelBarShare * 100}%` }}
            transition={enterTransition}
          />
          <motion.div
            className="absolute inset-y-0 left-0 rounded-full bg-(--volume-used)"
            initial={false}
            animate={{ width: `${funnelFilledShare * 100}%` }}
            transition={enterTransition}
          />
          {isRunningHere && completedFraction === null && (
            <div className="absolute inset-0 animate-pulse bg-(--volume-used)/20 motion-reduce:animate-none" />
          )}
        </div>
      </div>
    </li>
  );
}
