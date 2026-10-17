import type { Virtualizer } from "@tanstack/react-virtual";
import { motion } from "motion/react";
import type { RefObject } from "react";
import type { SearchHit, SearchSortOrder } from "@/lib/backend/search";
import { cn } from "@/lib/classNames";
import { fadeSlideInVariants } from "@/lib/motion/transitions";
import { useNowEveryMinute } from "../useNowEveryMinute";
import { SearchResultRow } from "./SearchResultRow";
import { SearchResultsHeader } from "./SearchResultsHeader";

type SearchResultsListProps = {
  hits: readonly SearchHit[];
  selectedIndex: number | null;
  sortOrder: SearchSortOrder;
  isStale: boolean;
  scrollElementRef: RefObject<HTMLDivElement | null>;
  virtualizer: Virtualizer<HTMLDivElement, Element>;
  onChooseSortOrder: (sortOrder: SearchSortOrder) => void;
  onSelect: (index: number) => void;
  onOpen: (index: number) => void;
};

/** The results table. It fades in once when it first appears; later answers just replace rows. */
export function SearchResultsList({
  hits,
  selectedIndex,
  sortOrder,
  isStale,
  scrollElementRef,
  virtualizer,
  onChooseSortOrder,
  onSelect,
  onOpen,
}: SearchResultsListProps) {
  const nowUnixMilliseconds = useNowEveryMinute();

  return (
    <motion.div
      variants={fadeSlideInVariants}
      initial="hidden"
      animate="visible"
      className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border bg-card"
    >
      <div
        ref={scrollElementRef}
        role="listbox"
        data-inline-list
        aria-label="Search results"
        aria-activedescendant={
          selectedIndex === null ? undefined : `global-search-result-${selectedIndex}`
        }
        tabIndex={-1}
        className={cn(
          "min-h-0 flex-1 overflow-y-auto outline-none transition-opacity",
          isStale && "opacity-60",
        )}
      >
        <SearchResultsHeader sortOrder={sortOrder} onChooseSortOrder={onChooseSortOrder} />
        <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const hit = hits[virtualRow.index];
            if (hit === undefined) {
              return null;
            }
            return (
              <SearchResultRow
                key={`${hit.volumeMountPoint}:${hit.node}`}
                hit={hit}
                index={virtualRow.index}
                isSelected={virtualRow.index === selectedIndex}
                nowUnixMilliseconds={nowUnixMilliseconds}
                top={virtualRow.start - virtualizer.options.scrollMargin}
                height={virtualRow.size}
                onSelect={onSelect}
                onOpen={onOpen}
              />
            );
          })}
        </div>
      </div>
    </motion.div>
  );
}
