import type { LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { HistoryButtonModel } from "./useExplorerHistoryButtons";

type HistoryNavigationButtonProps = {
  model: HistoryButtonModel;
  icon: LucideIcon;
  /** "Back" or "Forward". */
  label: string;
  shortcutKey: string;
};

export function HistoryNavigationButton({
  model,
  icon: Icon,
  label,
  shortcutKey,
}: HistoryNavigationButtonProps) {
  const accessibleLabel = model.targetName === null ? label : `${label} to ${model.targetName}`;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={accessibleLabel}
          disabled={!model.isEnabled}
          onClick={model.go}
        >
          <Icon />
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        {accessibleLabel}
        <KbdGroup>
          <Kbd>Alt</Kbd>
          <Kbd>{shortcutKey}</Kbd>
        </KbdGroup>
      </TooltipContent>
    </Tooltip>
  );
}
