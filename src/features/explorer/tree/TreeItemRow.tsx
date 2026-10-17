import { ChevronRightIcon, CloudIcon, LockIcon } from "lucide-react";
import { motion } from "motion/react";
import type { CSSProperties } from "react";
import { SizeBar } from "@/components/SizeBar";
import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import { cn } from "@/lib/classNames";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatCount } from "@/lib/formatting/formatCount";
import { formatModifiedDate } from "@/lib/formatting/formatModifiedDate";
import { formatShare } from "@/lib/formatting/formatShare";
import { revealedRowVariants } from "@/lib/motion/transitions";
import { NodeContextMenuContent } from "../actions/NodeContextMenuContent";
import { nodeActionTarget } from "../actions/nodeActionTarget";
import type { NodeActions } from "../actions/useNodeActions";
import { FileTypeIcon } from "../FileTypeIcon";
import type { TreeRow } from "./flattenTreeRows";
import {
  FILES_COLUMN_CLASS_NAME,
  MODIFIED_COLUMN_CLASS_NAME,
  SIZE_BAR_CLASS_NAME,
  TREE_GRID_COLUMNS_CLASS_NAME,
  TREE_INDENT_PER_LEVEL,
} from "./treeTableLayout";

type TreeItemRowProps = {
  row: Extract<TreeRow, { kind: "item" }>;
  style: CSSProperties;
  isSelected: boolean;
  animatesIn: boolean;
  actions: NodeActions;
  onSelect: () => void;
  onToggle: () => void;
  onOpenFolder: () => void;
};

export function TreeItemRow({
  row,
  style,
  isSelected,
  animatesIn,
  actions,
  onSelect,
  onToggle,
  onOpenFolder,
}: TreeItemRowProps) {
  const { item, depth, isExpanded } = row;
  const isFolder = item.kind === "directory";
  const isLink = item.flags.junction || item.flags.symlink;

  return (
    <ContextMenu onOpenChange={(isOpen) => isOpen && onSelect()}>
      <ContextMenuTrigger asChild>
        <motion.div
          role="treeitem"
          tabIndex={-1}
          id={`tree-row-${item.node}`}
          aria-selected={isSelected}
          aria-expanded={item.hasChildren ? isExpanded : undefined}
          aria-level={depth + 1}
          style={style}
          variants={revealedRowVariants}
          initial={animatesIn ? "hidden" : false}
          animate="visible"
          className={cn(
            TREE_GRID_COLUMNS_CLASS_NAME,
            "absolute inset-x-0 cursor-default select-none items-center pr-3 text-sm",
            isSelected ? "bg-accent text-accent-foreground" : "hover:bg-muted/60",
          )}
          onMouseDown={(event) => event.preventDefault()}
          onClick={onSelect}
          onDoubleClick={() => isFolder && onOpenFolder()}
        >
          <div
            className="flex min-w-0 items-center gap-1.5"
            style={{ paddingLeft: 8 + depth * TREE_INDENT_PER_LEVEL }}
          >
            {item.hasChildren ? (
              <button
                type="button"
                tabIndex={-1}
                aria-label={isExpanded ? `Collapse ${item.name}` : `Expand ${item.name}`}
                className="flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-foreground/10 hover:text-foreground"
                onClick={(event) => {
                  event.stopPropagation();
                  onToggle();
                }}
              >
                <ChevronRightIcon
                  className={cn(
                    "size-3.5 transition-transform duration-150 motion-reduce:transition-none",
                    isExpanded && "rotate-90",
                  )}
                />
              </button>
            ) : (
              <span className="size-5 shrink-0" />
            )}
            <FileTypeIcon
              kind={item.kind}
              category={item.category}
              isExpanded={isExpanded}
              isInaccessible={item.flags.inaccessible}
              isLink={isLink}
            />
            <span className={cn("truncate", item.flags.synthetic && "italic")} title={item.name}>
              {item.name}
            </span>
            {item.flags.inaccessible && (
              <LockIcon className="size-3 shrink-0 text-destructive" aria-label="Access denied" />
            )}
            {item.flags.cloudPlaceholder && (
              <CloudIcon
                className="size-3 shrink-0 text-muted-foreground"
                aria-label="In the cloud"
              />
            )}
          </div>
          <div className="flex items-center justify-end gap-2">
            <SizeBar
              share={item.shareOfParent ?? 0}
              className={cn("w-12 shrink-0", SIZE_BAR_CLASS_NAME)}
            />
            <span className="whitespace-nowrap text-right tabular-nums">
              {formatBytesAsReadableSize(item.logicalSize)}
            </span>
          </div>
          <div className="text-right text-muted-foreground tabular-nums">
            {formatShare(item.shareOfParent)}
          </div>
          <div
            className={cn("text-right text-muted-foreground tabular-nums", FILES_COLUMN_CLASS_NAME)}
          >
            {isFolder ? formatCount(item.fileCount) : ""}
          </div>
          <div
            className={cn(
              "text-right text-muted-foreground tabular-nums",
              MODIFIED_COLUMN_CLASS_NAME,
            )}
          >
            {formatModifiedDate(item.modifiedAtUnixMilliseconds)}
          </div>
        </motion.div>
      </ContextMenuTrigger>
      <NodeContextMenuContent
        target={nodeActionTarget(item)}
        actions={actions}
        isScanRoot={false}
      />
    </ContextMenu>
  );
}
