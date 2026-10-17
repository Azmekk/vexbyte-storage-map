import type { SearchSortOrder } from "@/lib/backend/search";

export type SearchSortOption = {
  sortOrder: SearchSortOrder;
  label: string;
};

export const SEARCH_SORT_OPTIONS: readonly SearchSortOption[] = [
  { sortOrder: "nameAscending", label: "Name" },
  { sortOrder: "sizeDescending", label: "Size" },
  { sortOrder: "modifiedDescending", label: "Modified" },
  { sortOrder: "pathAscending", label: "Path" },
];

export function isSearchSortOrder(value: string): value is SearchSortOrder {
  return SEARCH_SORT_OPTIONS.some((option) => option.sortOrder === value);
}
