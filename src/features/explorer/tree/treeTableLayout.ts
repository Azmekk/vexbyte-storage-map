/** Shared by the header and the rows so the columns line up; narrow panels drop columns. */
export const TREE_GRID_COLUMNS_CLASS_NAME =
  "grid grid-cols-[minmax(0,1fr)_5rem_3.5rem] @sm:grid-cols-[minmax(0,1fr)_8rem_3.75rem] @xl:grid-cols-[minmax(0,1fr)_8rem_3.75rem_4.5rem] @2xl:grid-cols-[minmax(0,1fr)_8rem_3.75rem_4.5rem_6.5rem]";

export const SIZE_BAR_CLASS_NAME = "hidden @sm:block";

export const FILES_COLUMN_CLASS_NAME = "hidden @xl:block";
export const MODIFIED_COLUMN_CLASS_NAME = "hidden @2xl:block";

export const TREE_ROW_HEIGHT = 32;
export const TREE_INDENT_PER_LEVEL = 16;
