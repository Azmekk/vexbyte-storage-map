export type SearchSyntaxEntry = {
  /** What to type, exactly as shown. */
  example: string;
  description: string;
};

export type SearchSyntaxSection = {
  title: string;
  entries: readonly SearchSyntaxEntry[];
};

/** The query syntax of `docs/SEARCH.md`, for the "Search syntax" popover. Keep them in step. */
export const SEARCH_SYNTAX_SECTIONS: readonly SearchSyntaxSection[] = [
  {
    title: "Names",
    entries: [
      { example: "report", description: "Names that contain report, ignoring case" },
      { example: "annual report 2024", description: "Names that contain every word, any order" },
      { example: '"annual report"', description: "Names that contain the exact phrase" },
      { example: "*.jpg", description: "* is any run of characters; matches the whole name" },
      { example: "img_????.png", description: "? is exactly one character" },
    ],
  },
  {
    title: "Paths",
    entries: [
      { example: "photos\\2024", description: "A term with \\ or / is matched against the path" },
      { example: "c:\\*\\*.iso", description: "Wildcards in a path term match the whole path" },
    ],
  },
  {
    title: "Filters",
    entries: [
      { example: "ext:jpg;png;heic", description: "Files with one of these extensions" },
      { example: "size:>100mb", description: "Larger than 100 MB (also <, >=, <=)" },
      { example: "size:1mb..10mb", description: "Between 1 MB and 10 MB, inclusive" },
      { example: "type:folder", description: "Only folders (type:file for only files)" },
      { example: "folder:photos", description: "Folders whose name contains photos" },
    ],
  },
];

/** Short notes under the table. */
export const SEARCH_SYNTAX_NOTES: readonly string[] = [
  "Everything you type must match: holiday ext:jpg size:>2mb.",
  "Sizes use binary units (kb, mb, gb, tb), like File Explorer.",
  "OR, NOT, regular expressions and date filters are not supported yet.",
];

/** A few queries for the starting state, each one clickable. */
export const SEARCH_STARTER_EXAMPLES: readonly SearchSyntaxEntry[] = [
  { example: "*.iso", description: "Every disk image" },
  { example: "ext:mp4;mkv size:>1gb", description: "Videos larger than 1 GB" },
  { example: "type:folder node_modules", description: "Folders named node_modules" },
  { example: "downloads\\ ext:zip", description: "Zip files under a Downloads folder" },
];
