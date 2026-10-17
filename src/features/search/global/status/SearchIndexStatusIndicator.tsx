import { LoaderCircleIcon, TriangleAlertIcon } from "lucide-react";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { IndexStatusIndicator } from "./describeIndexStatusIndicator";

type SearchIndexStatusIndicatorProps = {
  indicator: IndexStatusIndicator;
};

/** The marker at the end of the "Search files" sidebar item. Nothing while the index is ready. */
export function SearchIndexStatusIndicator({ indicator }: SearchIndexStatusIndicatorProps) {
  if (indicator.kind === "none") {
    return null;
  }

  if (indicator.kind === "warming") {
    return (
      <span className="ml-auto flex shrink-0 items-center gap-1 text-sidebar-foreground/60 text-xs group-data-[collapsible=icon]:hidden">
        <LoaderCircleIcon aria-hidden className="size-3! animate-spin" />
        {indicator.label}
      </span>
    );
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span
          role="img"
          aria-label={indicator.tooltip}
          className="ml-auto flex shrink-0 text-amber-600 group-data-[collapsible=icon]:hidden dark:text-amber-400"
        >
          <TriangleAlertIcon aria-hidden className="size-3.5!" />
        </span>
      </TooltipTrigger>
      <TooltipContent side="right" className="max-w-64">
        {indicator.tooltip}
      </TooltipContent>
    </Tooltip>
  );
}
