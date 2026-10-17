import { useCallback, useEffect, useRef, useState } from "react";
import { getDuplicateGroups } from "@/lib/backend/duplicates";
import type {
  DuplicateGroup,
  DuplicateGroupSortOrder,
  DuplicateGroupsPage,
  DuplicateSearchId,
  DuplicateSelectionSummary,
} from "@/lib/backend/generated";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import {
  GROUP_PAGE_SIZE,
  type GroupPages,
  groupAtIndex,
  loadedPageIndexesWithin,
  pageIndexesCoveringRange,
  replaceGroupInPages,
} from "./groupPageCache";

export type DuplicateGroupTotals = {
  totalGroupCount: number;
  totalWastedBytes: number;
  totalDuplicateFileCount: number;
  selection: DuplicateSelectionSummary;
};

type GroupsCache = {
  cacheKey: string;
  /** Bumped by every refresh, so answers to requests made before it are dropped. */
  generation: number;
  pages: GroupPages;
  totals: DuplicateGroupTotals | null;
};

export type UseDuplicateGroupsPageResult = {
  totals: DuplicateGroupTotals | null;
  loadErrorMessage: string | null;
  pages: GroupPages;
  groupAt: (groupIndex: number) => DuplicateGroup | undefined;
  /** Loads the pages holding groups `firstIndex` to `lastIndex` unless they are cached. */
  requestGroupRange: (firstIndex: number, lastIndex: number) => void;
  /** Puts a group changed by `setFileKeep` into the cache, with the adjusted selection. */
  replaceGroup: (updatedGroup: DuplicateGroup, selection: DuplicateSelectionSummary) => void;
  /** Refetches every loaded page at once and swaps them in together, starting from offset 0. */
  refreshLoadedPages: () => Promise<void>;
  retryLoading: () => void;
};

function createEmptyCache(cacheKey: string): GroupsCache {
  return { cacheKey, generation: 0, pages: new Map(), totals: null };
}

function totalsOfPage(page: DuplicateGroupsPage): DuplicateGroupTotals {
  return {
    totalGroupCount: page.totalGroupCount,
    totalWastedBytes: page.totalWastedBytes,
    totalDuplicateFileCount: page.totalDuplicateFileCount,
    selection: page.selection,
  };
}

/**
 * Pages of duplicate groups fetched on demand and cached by page offset for one search and sort
 * order. Changing the sort order starts a fresh cache.
 */
export function useDuplicateGroupsPage(
  searchId: DuplicateSearchId,
  sortOrder: DuplicateGroupSortOrder,
): UseDuplicateGroupsPageResult {
  const cacheKey = `${searchId}:${sortOrder}`;
  const [storedCache, setCache] = useState<GroupsCache>(() => createEmptyCache(cacheKey));
  const [loadErrorMessage, setLoadErrorMessage] = useState<string | null>(null);
  const cache = storedCache.cacheKey === cacheKey ? storedCache : createEmptyCache(cacheKey);
  const cacheRef = useRef(cache);
  cacheRef.current = cache;
  const requestedPageKeys = useRef(new Set<string>());

  if (storedCache.cacheKey !== cacheKey) {
    setCache(cache);
  }

  const fetchPage = useCallback(
    async (pageIndex: number) => {
      const generation = cacheRef.current.generation;
      const requestKey = `${cacheKey}|${generation}|${pageIndex}`;

      if (requestedPageKeys.current.has(requestKey)) {
        return;
      }
      requestedPageKeys.current.add(requestKey);

      try {
        const page = await getDuplicateGroups(
          searchId,
          sortOrder,
          pageIndex * GROUP_PAGE_SIZE,
          GROUP_PAGE_SIZE,
        );
        setLoadErrorMessage(null);
        setCache((previousCache) => {
          const isStale =
            previousCache.cacheKey !== cacheKey || previousCache.generation !== generation;
          if (isStale) {
            return previousCache;
          }

          const pages = new Map(previousCache.pages);
          pages.set(pageIndex, page.groups);
          return { ...previousCache, pages, totals: totalsOfPage(page) };
        });
      } catch (error) {
        requestedPageKeys.current.delete(requestKey);
        setLoadErrorMessage(readableErrorMessage(error));
      }
    },
    [cacheKey, searchId, sortOrder],
  );

  const hasTotals = cache.totals !== null;

  useEffect(() => {
    if (!hasTotals) {
      void fetchPage(0);
    }
  }, [hasTotals, fetchPage]);

  const requestGroupRange = useCallback(
    (firstIndex: number, lastIndex: number) => {
      const currentCache = cacheRef.current;
      if (currentCache.totals === null) {
        return;
      }

      const neededPages = pageIndexesCoveringRange(
        firstIndex,
        lastIndex,
        currentCache.totals.totalGroupCount,
      );
      for (const pageIndex of neededPages) {
        if (!currentCache.pages.has(pageIndex)) {
          void fetchPage(pageIndex);
        }
      }
    },
    [fetchPage],
  );

  const replaceGroup = useCallback(
    (updatedGroup: DuplicateGroup, selection: DuplicateSelectionSummary) => {
      setCache((previousCache) => ({
        ...previousCache,
        pages: replaceGroupInPages(previousCache.pages, updatedGroup),
        totals: previousCache.totals === null ? null : { ...previousCache.totals, selection },
      }));
    },
    [],
  );

  const refreshLoadedPages = useCallback(async () => {
    const currentCache = cacheRef.current;
    const loadedPageIndexes = [...currentCache.pages.keys()].sort((a, b) => a - b);
    const pageIndexesToFetch = loadedPageIndexes.includes(0)
      ? loadedPageIndexes
      : [0, ...loadedPageIndexes];

    try {
      const fetchedPages = await Promise.all(
        pageIndexesToFetch.map((pageIndex) =>
          getDuplicateGroups(searchId, sortOrder, pageIndex * GROUP_PAGE_SIZE, GROUP_PAGE_SIZE),
        ),
      );
      const firstPage = fetchedPages[0];
      if (firstPage === undefined) {
        return;
      }

      const refreshedPages = new Map<number, readonly DuplicateGroup[]>();
      pageIndexesToFetch.forEach((pageIndex, position) => {
        const fetchedPage = fetchedPages[position];
        if (fetchedPage !== undefined) {
          refreshedPages.set(pageIndex, fetchedPage.groups);
        }
      });

      const totals = totalsOfPage(firstPage);
      const existingPageIndexes = new Set(
        loadedPageIndexesWithin(refreshedPages, totals.totalGroupCount),
      );
      for (const pageIndex of refreshedPages.keys()) {
        if (!existingPageIndexes.has(pageIndex)) {
          refreshedPages.delete(pageIndex);
        }
      }

      requestedPageKeys.current.clear();
      setLoadErrorMessage(null);
      setCache((previousCache) =>
        previousCache.cacheKey !== cacheKey
          ? previousCache
          : {
              cacheKey,
              generation: previousCache.generation + 1,
              pages: refreshedPages,
              totals,
            },
      );
    } catch (error) {
      setLoadErrorMessage(readableErrorMessage(error));
    }
  }, [cacheKey, searchId, sortOrder]);

  const retryLoading = useCallback(() => {
    void refreshLoadedPages();
  }, [refreshLoadedPages]);

  const pages = cache.pages;
  const groupAt = useCallback((groupIndex: number) => groupAtIndex(pages, groupIndex), [pages]);

  return {
    totals: cache.totals,
    loadErrorMessage,
    pages,
    groupAt,
    requestGroupRange,
    replaceGroup,
    refreshLoadedPages,
    retryLoading,
  };
}
