import type { DuplicateGroup, DuplicateGroupId } from "@/lib/backend/generated";

export const GROUP_PAGE_SIZE = 50;

/** Pages of groups by page index. Page `n` starts at group `n * GROUP_PAGE_SIZE`. */
export type GroupPages = ReadonlyMap<number, readonly DuplicateGroup[]>;

export function pageIndexOfGroup(groupIndex: number, pageSize = GROUP_PAGE_SIZE): number {
  return Math.floor(groupIndex / pageSize);
}

/** The pages needed to show groups `firstIndex` to `lastIndex`, inclusive, within `totalCount`. */
export function pageIndexesCoveringRange(
  firstIndex: number,
  lastIndex: number,
  totalCount: number,
  pageSize = GROUP_PAGE_SIZE,
): number[] {
  if (totalCount <= 0 || lastIndex < firstIndex) {
    return [];
  }

  const clampedFirstIndex = Math.max(0, firstIndex);
  const clampedLastIndex = Math.min(totalCount - 1, lastIndex);
  const firstPage = pageIndexOfGroup(clampedFirstIndex, pageSize);
  const lastPage = pageIndexOfGroup(clampedLastIndex, pageSize);

  const pageIndexes: number[] = [];
  for (let pageIndex = firstPage; pageIndex <= lastPage; pageIndex += 1) {
    pageIndexes.push(pageIndex);
  }

  return pageIndexes;
}

export function groupAtIndex(
  pages: GroupPages,
  groupIndex: number,
  pageSize = GROUP_PAGE_SIZE,
): DuplicateGroup | undefined {
  const page = pages.get(pageIndexOfGroup(groupIndex, pageSize));
  return page?.[groupIndex % pageSize];
}

export function findGroupInPages(
  pages: GroupPages,
  groupId: DuplicateGroupId,
): DuplicateGroup | undefined {
  for (const page of pages.values()) {
    const matchingGroup = page.find((group) => group.id === groupId);
    if (matchingGroup !== undefined) {
      return matchingGroup;
    }
  }

  return undefined;
}

/** A copy of `pages` with the group of the same id replaced. Pages without it are reused. */
export function replaceGroupInPages(pages: GroupPages, updatedGroup: DuplicateGroup): GroupPages {
  const updatedPages = new Map<number, readonly DuplicateGroup[]>();

  for (const [pageIndex, page] of pages) {
    const containsGroup = page.some((group) => group.id === updatedGroup.id);
    updatedPages.set(
      pageIndex,
      containsGroup
        ? page.map((group) => (group.id === updatedGroup.id ? updatedGroup : group))
        : page,
    );
  }

  return updatedPages;
}

/**
 * The cached groups that will disappear once `movedPaths` are gone: those left with fewer than
 * two copies. Used to animate them out before the list is refetched.
 */
export function groupIdsLeavingAfterMove(
  pages: GroupPages,
  movedPaths: readonly string[],
): Set<DuplicateGroupId> {
  const movedPathSet = new Set(movedPaths);
  const leavingGroupIds = new Set<DuplicateGroupId>();

  for (const page of pages.values()) {
    for (const group of page) {
      const remainingCopyCount = group.files.filter((file) => !movedPathSet.has(file.path)).length;
      if (remainingCopyCount < 2) {
        leavingGroupIds.add(group.id);
      }
    }
  }

  return leavingGroupIds;
}

/** The loaded page indexes, ascending, limited to pages that still exist for `totalCount`. */
export function loadedPageIndexesWithin(
  pages: GroupPages,
  totalCount: number,
  pageSize = GROUP_PAGE_SIZE,
): number[] {
  const pageCount = Math.ceil(totalCount / pageSize);
  return [...pages.keys()].filter((pageIndex) => pageIndex < pageCount).sort((a, b) => a - b);
}
