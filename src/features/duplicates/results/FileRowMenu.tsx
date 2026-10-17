import { EllipsisVerticalIcon, ExternalLinkIcon, FolderOpenIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

type FileRowMenuProps = {
  fileName: string;
  onReveal: () => void;
  onOpen: () => void;
};

export function FileRowMenu({ fileName, onReveal, onOpen }: FileRowMenuProps) {
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
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
