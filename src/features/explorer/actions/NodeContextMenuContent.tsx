import {
  ClipboardCopyIcon,
  ExternalLinkIcon,
  FolderInputIcon,
  FolderOpenIcon,
  Trash2Icon,
} from "lucide-react";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
} from "@/components/ui/context-menu";
import { platformWording } from "@/lib/platform/platformWording";
import type { NodeActionTarget } from "./nodeActionTarget";
import type { NodeActions } from "./useNodeActions";

type NodeContextMenuContentProps = {
  target: NodeActionTarget | null;
  actions: NodeActions;
  isScanRoot: boolean;
};

/** The same menu for tree rows, treemap blocks, sunburst segments and largest files. */
export function NodeContextMenuContent({
  target,
  actions,
  isScanRoot,
}: NodeContextMenuContentProps) {
  if (target === null) {
    return null;
  }

  const isFolder = target.kind === "directory";
  const hasLocation = !target.isSynthetic;

  return (
    <ContextMenuContent className="w-60">
      <ContextMenuLabel className="truncate">{target.name}</ContextMenuLabel>
      <ContextMenuSeparator />
      {isFolder && (
        <ContextMenuItem onSelect={() => actions.exploreFolder(target)}>
          <FolderInputIcon />
          Explore this folder
        </ContextMenuItem>
      )}
      <ContextMenuItem disabled={!hasLocation} onSelect={() => actions.openItem(target)}>
        {isFolder ? <FolderOpenIcon /> : <ExternalLinkIcon />}
        {isFolder ? platformWording().openInFileManagerLabel : "Open"}
      </ContextMenuItem>
      <ContextMenuItem disabled={!hasLocation} onSelect={() => actions.revealItem(target)}>
        <FolderOpenIcon />
        Show in File Explorer
      </ContextMenuItem>
      <ContextMenuItem disabled={!hasLocation} onSelect={() => actions.copyPath(target)}>
        <ClipboardCopyIcon />
        Copy path
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem
        variant="destructive"
        disabled={!hasLocation || isScanRoot}
        onSelect={() => actions.requestRecycle(target)}
      >
        <Trash2Icon />
        Move to Recycle Bin…
      </ContextMenuItem>
    </ContextMenuContent>
  );
}
