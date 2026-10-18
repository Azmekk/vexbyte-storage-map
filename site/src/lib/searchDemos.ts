export type SearchDemo = {
  query: string;
  results: ReadonlyArray<readonly [path: string, size: string]>;
};

/** Example queries in the app's search syntax, with made-up results. */
export const SEARCH_DEMOS: readonly SearchDemo[] = [
  {
    query: "ext:iso size:>2gb",
    results: [
      ["D:\\Images\\Win11_24H2_English_x64.iso", "6.1 GB"],
      ["E:\\VMs\\ubuntu-24.04-desktop.iso", "5.8 GB"],
    ],
  },
  {
    query: "holiday ext:jpg;png",
    results: [
      ["C:\\Users\\Alex\\Pictures\\Holiday 2025\\IMG_0412.jpg", "4.2 MB"],
      ["C:\\Users\\Alex\\Pictures\\Holiday 2025\\IMG_0413.png", "3.9 MB"],
    ],
  },
  {
    query: "folder: node_modules",
    results: [
      ["E:\\code\\web\\node_modules", "612 MB"],
      ["E:\\code\\api\\node_modules", "448 MB"],
    ],
  },
];
