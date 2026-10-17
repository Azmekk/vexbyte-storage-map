import { memo } from "react";
import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import { FileTypeIcon } from "@/features/explorer/FileTypeIcon";
import type { SearchHit } from "@/lib/backend/search";
import { cn } from "@/lib/classNames";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatModifiedDateAndTime } from "@/lib/formatting/formatModifiedDate";
import { formatRelativeDate } from "@/lib/formatting/formatRelativeDate";
import { driveLetterLabel, searchHitFolderPath } from "./describeSearchHit";
import { MiddleTruncatedPath } from "./MiddleTruncatedPath";
import { SearchResultContextMenuContent } from "./SearchResultContextMenuContent";
import { SEARCH_RESULT_GRID_COLUMNS } from "./searchResultColumns";

type SearchResultRowProps = {
  hit: SearchHit;
  index: number;
  isSelected: boolean;
  nowUnixMilliseconds: number;
  top: number;
  height: number;
  onSelect: (index: number) => void;
  onOpen: (index: number) => void;
};

export const SearchResultRow = memo(function SearchResultRow({
  hit,
  index,
  isSelected,
  nowUnixMilliseconds,
  top,
  height,
  onSelect,
  onOpen,
}: SearchResultRowProps) {
  const modifiedAt = hit.modifiedAtUnixMilliseconds;

  return (
    <ContextMenu onOpenChange={(isOpen) => isOpen && onSelect(index)}>
      <ContextMenuTrigger asChild>
        <div
          role="option"
          tabIndex={-1}
          aria-selected={isSelected}
          id={`global-search-result-${index}`}
          style={{ top, height }}
          onClick={() => onSelect(index)}
          onDoubleClick={() => onOpen(index)}
          onKeyDown={(event) => {
            if (event.key === " ") {
              event.preventDefault();
              onSelect(index);
            }
          }}
          className={cn(
            "absolute inset-x-0 grid cursor-default items-center gap-4 px-4 text-sm outline-none",
            SEARCH_RESULT_GRID_COLUMNS,
            isSelected
              ? "bg-(--volume-used)/12 shadow-[inset_2px_0_0_var(--volume-used)]"
              : "hover:bg-muted/60",
          )}
        >
          <span className="flex min-w-0 items-center gap-2.5">
            <FileTypeIcon kind={hit.isDirectory ? "directory" : "file"} category={hit.category} />
            <span className="truncate" title={hit.name}>
              {hit.name}
            </span>
          </span>
          <MiddleTruncatedPath
            path={searchHitFolderPath(hit)}
            className="text-muted-foreground text-xs"
          />
          <span className="text-right tabular-nums">
            {formatBytesAsReadableSize(hit.logicalSize)}
          </span>
          <span
            className="truncate text-muted-foreground text-xs"
            title={formatModifiedDateAndTime(modifiedAt)}
          >
            {modifiedAt === null ? "–" : formatRelativeDate(modifiedAt, nowUnixMilliseconds)}
          </span>
          <span className="text-muted-foreground text-xs tabular-nums">
            {driveLetterLabel(hit.volumeMountPoint)}
          </span>
        </div>
      </ContextMenuTrigger>
      <SearchResultContextMenuContent hit={hit} />
    </ContextMenu>
  );
});
