import type { BreadcrumbItem } from "@/lib/backend/tree";

export type BreadcrumbTrailEntry =
  | { kind: "crumb"; crumb: BreadcrumbItem; isCurrent: boolean }
  | { kind: "collapsed"; hiddenCrumbs: BreadcrumbItem[] };

/**
 * Shows the root, then an ellipsis for the hidden middle, then the last few crumbs. With
 * `maximumVisibleCrumbs` or fewer crumbs nothing is hidden. The last crumb is the current one.
 */
export function buildBreadcrumbTrail(
  crumbs: BreadcrumbItem[],
  maximumVisibleCrumbs: number,
): BreadcrumbTrailEntry[] {
  const lastIndex = crumbs.length - 1;
  const toEntry = (crumb: BreadcrumbItem, index: number): BreadcrumbTrailEntry => ({
    kind: "crumb",
    crumb,
    isCurrent: index === lastIndex,
  });

  const visibleLimit = Math.max(2, maximumVisibleCrumbs);
  if (crumbs.length <= visibleLimit) {
    return crumbs.map(toEntry);
  }

  const trailingCount = visibleLimit - 1;
  const firstTrailingIndex = crumbs.length - trailingCount;
  const rootCrumb = crumbs[0];
  const entries: BreadcrumbTrailEntry[] = rootCrumb === undefined ? [] : [toEntry(rootCrumb, 0)];

  entries.push({ kind: "collapsed", hiddenCrumbs: crumbs.slice(1, firstTrailingIndex) });

  crumbs.slice(firstTrailingIndex).forEach((crumb, offset) => {
    entries.push(toEntry(crumb, firstTrailingIndex + offset));
  });

  return entries;
}
