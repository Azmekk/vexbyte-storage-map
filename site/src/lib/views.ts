export type AppView = {
  id: string;
  name: string;
  swatch: string;
  screenshot: string;
  alt: string;
  description: string;
};

export const APP_VIEWS: readonly AppView[] = [
  {
    id: "treemap",
    name: "Treemap",
    swatch: "var(--archive)",
    screenshot: "screenshots/treemap-dark.png",
    alt: "Treemap of the Program Files folder",
    description:
      "Each rectangle is a folder or file, drawn to scale. Click to zoom in, colour by file type or by depth, and spot the cache folder you forgot about.",
  },
  {
    id: "sunburst",
    name: "Sunburst",
    swatch: "var(--presentation)",
    screenshot: "screenshots/sunburst-dark.png",
    alt: "Sunburst of the same folder, coloured by file type",
    description:
      "Rings radiate from the folder you're in. Inner rings are parents, outer rings their children, so deep nesting reads at a glance.",
  },
  {
    id: "tree",
    name: "Tree table",
    swatch: "var(--video)",
    screenshot: "screenshots/tree-table-dark.png",
    alt: "Sortable tree table with size bars and percentages",
    description:
      "A virtualized, sortable list with size bars and percentages. Keyboard first, with breadcrumbs, an inspector panel and Ctrl+K to jump anywhere.",
  },
];
