import type { FileCategory } from "@/lib/backend/generated";

export type FileCategoryOption = {
  category: FileCategory;
  label: string;
  exampleExtensions: string;
  extensions: readonly string[];
};

/** Mirrors `file_categories.rs`; the extensions here only pick a fallback icon. */
export const FILE_CATEGORY_OPTIONS: readonly FileCategoryOption[] = [
  {
    category: "images",
    label: "Images",
    exampleExtensions: "JPG, PNG, HEIC, RAW",
    extensions: [
      "jpg",
      "jpeg",
      "png",
      "gif",
      "webp",
      "heic",
      "heif",
      "bmp",
      "tif",
      "tiff",
      "cr2",
      "cr3",
      "nef",
      "arw",
      "dng",
    ],
  },
  {
    category: "videos",
    label: "Videos",
    exampleExtensions: "MP4, MOV, MKV",
    extensions: ["mp4", "mov", "mkv", "avi", "wmv", "webm", "m4v", "mts"],
  },
  {
    category: "audio",
    label: "Music and audio",
    exampleExtensions: "MP3, FLAC, WAV",
    extensions: ["mp3", "flac", "wav", "m4a", "aac", "ogg", "wma", "opus"],
  },
  {
    category: "pdfs",
    label: "PDFs",
    exampleExtensions: "PDF",
    extensions: ["pdf"],
  },
  {
    category: "documents",
    label: "Documents",
    exampleExtensions: "DOCX, TXT, EPUB",
    extensions: ["doc", "docx", "odt", "rtf", "txt", "md", "epub"],
  },
  {
    category: "spreadsheets",
    label: "Spreadsheets",
    exampleExtensions: "XLSX, CSV",
    extensions: ["xls", "xlsx", "xlsm", "ods", "csv"],
  },
  {
    category: "presentations",
    label: "Presentations",
    exampleExtensions: "PPTX, ODP",
    extensions: ["ppt", "pptx", "odp"],
  },
  {
    category: "archivesAndInstallers",
    label: "Archives, installers",
    exampleExtensions: "ZIP, 7Z, ISO, EXE",
    extensions: ["zip", "rar", "7z", "tar", "gz", "iso", "exe", "msi"],
  },
];

export const ALL_FILE_CATEGORIES: readonly FileCategory[] = FILE_CATEGORY_OPTIONS.map(
  (option) => option.category,
);

/** The category a file belongs to by its extension, or `null` for anything else. */
export function fileCategoryForPath(path: string): FileCategory | null {
  const lastDotIndex = path.lastIndexOf(".");
  const lastSeparatorIndex = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));

  if (lastDotIndex <= lastSeparatorIndex + 1) {
    return null;
  }

  const extension = path.slice(lastDotIndex + 1).toLowerCase();
  const matchingOption = FILE_CATEGORY_OPTIONS.find((option) =>
    option.extensions.includes(extension),
  );

  return matchingOption?.category ?? null;
}

/** Toggles one category, keeping the canonical order of `ALL_FILE_CATEGORIES`. */
export function toggleFileCategory(
  selectedCategories: readonly FileCategory[],
  category: FileCategory,
): FileCategory[] {
  const isSelected = selectedCategories.includes(category);

  return ALL_FILE_CATEGORIES.filter((candidate) =>
    candidate === category ? !isSelected : selectedCategories.includes(candidate),
  );
}
