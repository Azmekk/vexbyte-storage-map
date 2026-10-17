import { ArrowDownIcon, ArrowUpIcon } from "lucide-react";
import type { ChildSortOrder } from "@/lib/backend/tree";
import { cn } from "@/lib/classNames";
import { columnSortState, nextSortOrderForColumn, type TreeColumn } from "./treeColumns";
import {
  FILES_COLUMN_CLASS_NAME,
  MODIFIED_COLUMN_CLASS_NAME,
  TREE_GRID_COLUMNS_CLASS_NAME,
} from "./treeTableLayout";

type HeaderColumn = { column: TreeColumn; label: string; className: string };

const HEADER_COLUMNS: HeaderColumn[] = [
  { column: "name", label: "Name", className: "justify-start pl-3" },
  { column: "size", label: "Size", className: "justify-end" },
  { column: "share", label: "%", className: "justify-end" },
  { column: "files", label: "Files", className: cn("justify-end", FILES_COLUMN_CLASS_NAME) },
  {
    column: "modified",
    label: "Modified",
    className: cn("justify-end", MODIFIED_COLUMN_CLASS_NAME),
  },
];

type TreeTableHeaderProps = {
  sortOrder: ChildSortOrder;
  onSortOrderChange: (sortOrder: ChildSortOrder) => void;
};

export function TreeTableHeader({ sortOrder, onSortOrderChange }: TreeTableHeaderProps) {
  return (
    <div
      className={cn(
        TREE_GRID_COLUMNS_CLASS_NAME,
        "h-9 shrink-0 items-center border-b pr-3 text-muted-foreground text-xs",
      )}
    >
      {HEADER_COLUMNS.map(({ column, label, className }) => {
        const sortState = columnSortState(sortOrder, column);
        const sortDescription =
          sortState === "none"
            ? ""
            : sortState === "ascending"
              ? ", sorted ascending"
              : ", sorted descending";
        return (
          <div key={column} className={cn("flex", className)}>
            <button
              type="button"
              className={cn(
                "flex items-center gap-1 rounded px-1 py-0.5 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring",
                sortState !== "none" && "text-foreground",
              )}
              aria-label={`Sort by ${column === "share" ? "share" : label.toLowerCase()}${sortDescription}`}
              onClick={() => onSortOrderChange(nextSortOrderForColumn(sortOrder, column))}
            >
              {label}
              {sortState === "ascending" && <ArrowUpIcon className="size-3" aria-hidden />}
              {sortState === "descending" && <ArrowDownIcon className="size-3" aria-hidden />}
            </button>
          </div>
        );
      })}
    </div>
  );
}
