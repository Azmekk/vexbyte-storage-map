import { Trash2Icon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

type RecycleFileButtonProps = {
  fileName: string;
  canRecycle: boolean;
  onRecycle: () => void;
};

/**
 * The row's trash button. It shows on hover and focus of its row, and always on touch screens.
 * The wrapper carries the tooltip so it still explains a disabled button.
 */
export function RecycleFileButton({ fileName, canRecycle, onRecycle }: RecycleFileButtonProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="inline-flex shrink-0 opacity-0 transition-opacity duration-150 group-focus-within/file-row:opacity-100 group-hover/file-row:opacity-100 pointer-coarse:opacity-100 motion-reduce:transition-none">
          <Button
            variant="ghost"
            size="icon-sm"
            className="text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
            disabled={!canRecycle}
            onClick={onRecycle}
            aria-label={`Move ${fileName} to Recycle Bin`}
          >
            <Trash2Icon />
          </Button>
        </span>
      </TooltipTrigger>
      <TooltipContent>
        {canRecycle ? "Move this file to Recycle Bin" : "A group must keep at least one copy"}
      </TooltipContent>
    </Tooltip>
  );
}
