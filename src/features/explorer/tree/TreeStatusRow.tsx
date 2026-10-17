import { ChevronsDownIcon, FolderXIcon, TriangleAlertIcon } from "lucide-react";
import type { CSSProperties } from "react";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/classNames";
import { formatCount } from "@/lib/formatting/formatCount";
import type { TreeRow } from "./flattenTreeRows";
import { TREE_INDENT_PER_LEVEL } from "./treeTableLayout";

type TreeStatusRowProps = {
  row: Exclude<TreeRow, { kind: "item" }>;
  style: CSSProperties;
  onShowMore: () => void;
  onRetry: () => void;
};

/** Loading, error, empty and "show more" lines inside the tree. */
export function TreeStatusRow({ row, style, onShowMore, onRetry }: TreeStatusRowProps) {
  const indentStyle = { paddingLeft: 8 + 26 + row.depth * TREE_INDENT_PER_LEVEL };
  const className = "absolute inset-x-0 flex items-center gap-2 pr-3 text-muted-foreground text-xs";

  switch (row.kind) {
    case "loading":
      return (
        <div style={{ ...style, ...indentStyle }} className={className}>
          <Skeleton className="h-3 w-40" />
        </div>
      );
    case "emptyFolder":
      return (
        <div style={{ ...style, ...indentStyle }} className={className}>
          <FolderXIcon className="size-3.5" aria-hidden />
          Empty folder
        </div>
      );
    case "error":
      return (
        <div style={{ ...style, ...indentStyle }} className={cn(className, "text-destructive")}>
          <TriangleAlertIcon className="size-3.5 shrink-0" aria-hidden />
          <span className="truncate">{row.errorMessage}</span>
          <button type="button" className="shrink-0 underline underline-offset-2" onClick={onRetry}>
            Try again
          </button>
        </div>
      );
    case "showMore":
      return (
        <div style={{ ...style, ...indentStyle }} className={className}>
          <button
            type="button"
            className="flex items-center gap-1.5 rounded px-1 py-0.5 hover:text-foreground"
            onClick={onShowMore}
          >
            <ChevronsDownIcon className="size-3.5" aria-hidden />
            {`Show ${formatCount(Math.min(row.hiddenChildCount, 1000))} more (${formatCount(row.hiddenChildCount)} not shown)`}
          </button>
        </div>
      );
  }
}
