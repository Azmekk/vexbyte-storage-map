import { FolderIcon, XIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { truncatePathMiddle } from "../paths/truncatePathMiddle";
import type { FolderChoice } from "./folderChoices";

const FOLDER_PATH_DISPLAY_LENGTH = 56;

type FolderChoiceRowProps = {
  folderChoice: FolderChoice;
  onToggle: (folderPath: string) => void;
  onRemove: (folderPath: string) => void;
};

export function FolderChoiceRow({ folderChoice, onToggle, onRemove }: FolderChoiceRowProps) {
  const checkboxId = `duplicate-folder-${folderChoice.path}`;

  return (
    <li className="group/folder flex items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-muted/60">
      <Checkbox
        id={checkboxId}
        checked={folderChoice.isSelected}
        onCheckedChange={() => onToggle(folderChoice.path)}
      />
      <label htmlFor={checkboxId} className="flex min-w-0 flex-1 cursor-pointer items-center gap-3">
        <FolderIcon
          className="size-4 shrink-0 text-muted-foreground data-[selected=true]:text-(--volume-used)"
          data-selected={folderChoice.isSelected}
          aria-hidden
        />
        <span className="grid min-w-0">
          <span className="truncate font-medium text-sm">{folderChoice.name}</span>
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="truncate text-muted-foreground text-xs">
                {truncatePathMiddle(folderChoice.path, FOLDER_PATH_DISPLAY_LENGTH)}
              </span>
            </TooltipTrigger>
            <TooltipContent>{folderChoice.path}</TooltipContent>
          </Tooltip>
        </span>
      </label>
      {folderChoice.isCustom && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => onRemove(folderChoice.path)}
              aria-label={`Remove ${folderChoice.name} from the list`}
            >
              <XIcon />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Remove from the list</TooltipContent>
        </Tooltip>
      )}
    </li>
  );
}
