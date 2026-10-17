import { HouseIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

type YourDrivesButtonProps = {
  onSelect: () => void;
};

/** Leaves the scan for the list of drives. */
export function YourDrivesButton({ onSelect }: YourDrivesButtonProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Your drives"
          onClick={onSelect}
          className="text-muted-foreground"
        >
          <HouseIcon />
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        Your drives
        <KbdGroup>
          <Kbd>Ctrl</Kbd>
          <Kbd>H</Kbd>
        </KbdGroup>
      </TooltipContent>
    </Tooltip>
  );
}
