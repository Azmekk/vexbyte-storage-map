import type { DuplicateGroupSortOrder } from "@/lib/backend/generated";

export type SortOrderOption = {
  sortOrder: DuplicateGroupSortOrder;
  label: string;
};

export const SORT_ORDER_OPTIONS: readonly SortOrderOption[] = [
  { sortOrder: "wastedBytesDescending", label: "Most wasted space" },
  { sortOrder: "fileSizeDescending", label: "Largest files" },
  { sortOrder: "fileCountDescending", label: "Most copies" },
  { sortOrder: "pathAscending", label: "Location (A to Z)" },
];

export const DEFAULT_SORT_ORDER: DuplicateGroupSortOrder = "wastedBytesDescending";

export function isDuplicateGroupSortOrder(value: string): value is DuplicateGroupSortOrder {
  return SORT_ORDER_OPTIONS.some((option) => option.sortOrder === value);
}
