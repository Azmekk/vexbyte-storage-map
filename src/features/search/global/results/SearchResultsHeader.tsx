import { ArrowDownIcon, ArrowUpIcon } from "lucide-react";
import type { SearchSortOrder } from "@/lib/backend/search";
import { cn } from "@/lib/classNames";
import { SEARCH_RESULT_GRID_COLUMNS } from "./searchResultColumns";

type SearchResultsHeaderProps = {
  sortOrder: SearchSortOrder;
  onChooseSortOrder: (sortOrder: SearchSortOrder) => void;
};

type SortableColumn = {
  label: string;
  sortOrder: SearchSortOrder | null;
  isDescending: boolean;
  alignEnd?: boolean;
};

const COLUMNS: readonly SortableColumn[] = [
  { label: "Name", sortOrder: "nameAscending", isDescending: false },
  { label: "Folder", sortOrder: "pathAscending", isDescending: false },
  { label: "Size", sortOrder: "sizeDescending", isDescending: true, alignEnd: true },
  { label: "Modified", sortOrder: "modifiedDescending", isDescending: true },
  { label: "Drive", sortOrder: null, isDescending: false },
];

export function SearchResultsHeader({ sortOrder, onChooseSortOrder }: SearchResultsHeaderProps) {
  return (
    <div
      className={cn(
        "sticky top-0 z-10 grid h-9 items-center gap-4 border-b bg-card px-4 text-muted-foreground text-xs",
        SEARCH_RESULT_GRID_COLUMNS,
      )}
    >
      {COLUMNS.map((column) => {
        const columnSortOrder = column.sortOrder;
        if (columnSortOrder === null) {
          return <span key={column.label}>{column.label}</span>;
        }
        const isActive = columnSortOrder === sortOrder;
        const SortIcon = column.isDescending ? ArrowDownIcon : ArrowUpIcon;
        return (
          <button
            key={column.label}
            type="button"
            onClick={() => onChooseSortOrder(columnSortOrder)}
            aria-pressed={isActive}
            className={cn(
              "flex items-center gap-1 rounded-sm outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring",
              column.alignEnd === true && "justify-end",
              isActive && "font-medium text-foreground",
            )}
          >
            {column.label}
            {isActive && <SortIcon aria-hidden className="size-3" />}
          </button>
        );
      })}
    </div>
  );
}
