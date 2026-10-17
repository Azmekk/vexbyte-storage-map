import { ClipboardCopyIcon, ExternalLinkIcon, FolderOpenIcon, TextCursorIcon } from "lucide-react";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  ContextMenuShortcut,
} from "@/components/ui/context-menu";
import type { SearchHit } from "@/lib/backend/search";
import { copyHitName, copyHitPath, openHit, revealHit } from "./searchHitActions";

type SearchResultContextMenuContentProps = {
  hit: SearchHit;
};

export function SearchResultContextMenuContent({ hit }: SearchResultContextMenuContentProps) {
  return (
    <ContextMenuContent className="w-60">
      <ContextMenuLabel className="truncate">{hit.name}</ContextMenuLabel>
      <ContextMenuSeparator />
      <ContextMenuItem onSelect={() => openHit(hit)}>
        {hit.isDirectory ? <FolderOpenIcon /> : <ExternalLinkIcon />}
        {hit.isDirectory ? "Open in File Explorer" : "Open"}
        <ContextMenuShortcut>Enter</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => revealHit(hit)}>
        <FolderOpenIcon />
        Show in File Explorer
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem onSelect={() => copyHitPath(hit)}>
        <ClipboardCopyIcon />
        Copy path
        <ContextMenuShortcut>Ctrl+C</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => copyHitName(hit)}>
        <TextCursorIcon />
        Copy name
      </ContextMenuItem>
    </ContextMenuContent>
  );
}
