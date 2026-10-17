import { useVirtualizer } from "@tanstack/react-virtual";
import { useCallback, useEffect, useRef } from "react";
import type { SearchHit } from "@/lib/backend/search";
import { useGlobalSearch } from "./query/useGlobalSearch";
import { openHit } from "./results/searchHitActions";
import {
  SEARCH_RESULT_HEADER_HEIGHT,
  SEARCH_RESULT_ROW_HEIGHT,
} from "./results/searchResultColumns";
import { useSearchResultsKeyboard } from "./results/useSearchResultsKeyboard";
import { searchableVolumesKey, volumesWarmingUp } from "./status/describeVolumeIndexState";
import { useSearchIndexStatus } from "./status/useSearchIndexStatus";
import { useGlobalSearchStore } from "./useGlobalSearchStore";

const NO_HITS: readonly SearchHit[] = [];

/** Everything the Search screen shows and does, so the components only render. */
export function useGlobalSearchScreen() {
  const query = useGlobalSearchStore((state) => state.query);
  const sortOrder = useGlobalSearchStore((state) => state.sortOrder);
  const storedSelectedIndex = useGlobalSearchStore((state) => state.selectedIndex);
  const setQuery = useGlobalSearchStore((state) => state.setQuery);
  const clearQuery = useGlobalSearchStore((state) => state.clearQuery);
  const setSortOrder = useGlobalSearchStore((state) => state.setSortOrder);
  const selectIndex = useGlobalSearchStore((state) => state.selectIndex);

  const indexStatus = useSearchIndexStatus();
  const volumes = indexStatus.status?.volumes ?? [];
  const search = useGlobalSearch(query, sortOrder, searchableVolumesKey(volumes));

  const hits = search.response?.hits ?? NO_HITS;
  const selectedIndex =
    storedSelectedIndex !== null && storedSelectedIndex < hits.length ? storedSelectedIndex : null;

  const scrollElementRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: hits.length,
    getScrollElement: () => scrollElementRef.current,
    estimateSize: () => SEARCH_RESULT_ROW_HEIGHT,
    overscan: 12,
    scrollMargin: SEARCH_RESULT_HEADER_HEIGHT,
    scrollPaddingStart: SEARCH_RESULT_HEADER_HEIGHT,
  });

  // A new query or order starts at the top; an answer that only adds a drive keeps the position.
  useEffect(
    () =>
      useGlobalSearchStore.subscribe((state, previousState) => {
        if (state.query !== previousState.query || state.sortOrder !== previousState.sortOrder) {
          virtualizer.scrollToOffset(0);
        }
      }),
    [virtualizer],
  );

  const handleKeyDown = useSearchResultsKeyboard({
    hits,
    selectedIndex,
    rowHeight: SEARCH_RESULT_ROW_HEIGHT,
    virtualizer,
  });

  const openHitAtIndex = useCallback(
    (index: number) => {
      const hit = hits[index];
      if (hit !== undefined) {
        openHit(hit);
      }
    },
    [hits],
  );

  const isQueryEmpty = query.trim().length === 0;
  const isWarming = indexStatus.status?.overall === "warming";
  const warmingVolumes = isWarming ? volumesWarmingUp(volumes) : [];
  const skippedVolumesStillWarming = search.response?.skippedVolumesStillWarming ?? [];

  return {
    query,
    sortOrder,
    isQueryEmpty,
    indexStatus,
    search,
    hits,
    selectedIndex,
    scrollElementRef,
    virtualizer,
    showWarmingBanner:
      !isQueryEmpty && (warmingVolumes.length > 0 || skippedVolumesStillWarming.length > 0),
    warmingVolumes,
    skippedVolumesStillWarming,
    setQuery,
    clearQuery,
    setSortOrder,
    selectIndex,
    openHitAtIndex,
    handleKeyDown,
  };
}
