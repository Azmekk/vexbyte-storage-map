import { FolderOpenIcon, LocateFixedIcon, Trash2Icon } from "lucide-react";
import type { CSSProperties } from "react";
import { SizeBar } from "@/components/SizeBar";
import { Button } from "@/components/ui/button";
import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { NodeContextMenuContent } from "@/features/explorer/actions/NodeContextMenuContent";
import { nodeActionTarget } from "@/features/explorer/actions/nodeActionTarget";
import type { NodeActions } from "@/features/explorer/actions/useNodeActions";
import { FileTypeIcon } from "@/features/explorer/FileTypeIcon";
import type { FileSummary } from "@/lib/backend/tree";
import { cn } from "@/lib/classNames";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { platformWording } from "@/lib/platform/platformWording";

type LargestFileRowProps = {
  file: FileSummary;
  shareOfLargest: number;
  barColor: string;
  style: CSSProperties;
  isSelected: boolean;
  actions: NodeActions;
  onSelect: () => void;
  onShowInTree: () => void;
};

export function LargestFileRow({
  file,
  shareOfLargest,
  barColor,
  style,
  isSelected,
  actions,
  onSelect,
  onShowInTree,
}: LargestFileRowProps) {
  const target = nodeActionTarget({ ...file, kind: "file" });

  return (
    <ContextMenu onOpenChange={(isOpen) => isOpen && onSelect()}>
      <ContextMenuTrigger asChild>
        <div
          role="option"
          tabIndex={-1}
          aria-selected={isSelected}
          data-selected={isSelected}
          style={style}
          onClick={onSelect}
          onDoubleClick={onShowInTree}
          onKeyDown={(event) => event.key === "Enter" && onShowInTree()}
          className={cn(
            "group absolute inset-x-0 grid cursor-default grid-cols-[minmax(0,1fr)_7rem_auto] items-center gap-3 px-3 text-sm",
            isSelected ? "bg-accent" : "hover:bg-muted/60",
          )}
        >
          <div className="flex min-w-0 items-center gap-2.5">
            <FileTypeIcon kind="file" category={file.category} />
            <div className="grid min-w-0">
              <span className="truncate" title={file.name}>
                {file.name}
              </span>
              <span
                className="truncate font-mono text-muted-foreground text-xs"
                title={file.fullPath ?? ""}
              >
                {file.fullPath ?? ""}
              </span>
            </div>
          </div>
          <div className="grid gap-1">
            <span className="text-right tabular-nums">
              {formatBytesAsReadableSize(file.logicalSize)}
            </span>
            <SizeBar share={shareOfLargest} color={barColor} />
          </div>
          <div className="flex gap-0.5 opacity-0 transition-opacity group-hover:opacity-100 group-data-[selected=true]:opacity-100 motion-reduce:transition-none">
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label={`Show ${file.name} in the tree`}
                  onClick={(event) => {
                    event.stopPropagation();
                    onShowInTree();
                  }}
                >
                  <LocateFixedIcon />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Show in the tree</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label={`Show ${file.name} in ${platformWording().fileManagerName}`}
                  onClick={(event) => {
                    event.stopPropagation();
                    actions.revealItem(target);
                  }}
                >
                  <FolderOpenIcon />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Show in File Explorer</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  className="text-destructive hover:text-destructive"
                  aria-label={`Move ${file.name} to the ${platformWording().binName}`}
                  onClick={(event) => {
                    event.stopPropagation();
                    actions.requestRecycle(target);
                  }}
                >
                  <Trash2Icon />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Move to Recycle Bin…</TooltipContent>
            </Tooltip>
          </div>
        </div>
      </ContextMenuTrigger>
      <NodeContextMenuContent target={target} actions={actions} isScanRoot={false} />
    </ContextMenu>
  );
}
