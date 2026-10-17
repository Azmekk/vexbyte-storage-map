import type { ChildSortOrder } from "@/lib/backend/tree";

export type TreeColumn = "name" | "size" | "share" | "files" | "modified";

type ColumnSortOrders = {
  /** The order a first click picks, the one people usually want for this column. */
  primary: ChildSortOrder;
  secondary: ChildSortOrder;
};

const SORT_ORDERS_BY_COLUMN: Record<TreeColumn, ColumnSortOrders> = {
  name: { primary: "nameAscending", secondary: "nameDescending" },
  size: { primary: "logicalSizeDescending", secondary: "logicalSizeAscending" },
  share: { primary: "logicalSizeDescending", secondary: "logicalSizeAscending" },
  files: { primary: "fileCountDescending", secondary: "fileCountAscending" },
  modified: { primary: "modifiedNewestFirst", secondary: "modifiedOldestFirst" },
};

/** Clicking a column sorts by it; clicking the sorted column again flips the direction. */
export function nextSortOrderForColumn(
  currentSortOrder: ChildSortOrder,
  clickedColumn: TreeColumn,
): ChildSortOrder {
  const { primary, secondary } = SORT_ORDERS_BY_COLUMN[clickedColumn];
  return currentSortOrder === primary ? secondary : primary;
}

export type ColumnSortState = "ascending" | "descending" | "none";

const ASCENDING_SORT_ORDERS: ReadonlySet<ChildSortOrder> = new Set<ChildSortOrder>([
  "logicalSizeAscending",
  "allocatedSizeAscending",
  "nameAscending",
  "fileCountAscending",
  "modifiedOldestFirst",
]);

/** How a column header shows the current sort. Size and share sort the same way. */
export function columnSortState(sortOrder: ChildSortOrder, column: TreeColumn): ColumnSortState {
  const { primary, secondary } = SORT_ORDERS_BY_COLUMN[column];
  if (sortOrder !== primary && sortOrder !== secondary) {
    return "none";
  }
  if (column === "share") {
    return "none";
  }
  return ASCENDING_SORT_ORDERS.has(sortOrder) ? "ascending" : "descending";
}
