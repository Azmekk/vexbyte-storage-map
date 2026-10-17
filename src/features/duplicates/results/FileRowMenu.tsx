import { EllipsisVerticalIcon, ExternalLinkIcon, FolderOpenIcon, Trash2Icon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

type FileRowMenuProps = {
  fileName: string;
  canRecycle: boolean;
  onReveal: () => void;
  onOpen: () => void;
  onRecycle: () => void;
};

export function FileRowMenu({
  fileName,
  canRecycle,
  onReveal,
  onOpen,
  onRecycle,
}: FileRowMenuProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon-sm" aria-label={`More actions for ${fileName}`}>
          <EllipsisVerticalIcon />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onSelect={onReveal}>
          <FolderOpenIcon />
          Reveal in Explorer
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={onOpen}>
          <ExternalLinkIcon />
          Open
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem variant="destructive" disabled={!canRecycle} onSelect={onRecycle}>
          <Trash2Icon />
          Move to Recycle Bin…
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
