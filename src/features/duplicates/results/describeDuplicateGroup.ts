import type { DuplicateFile, DuplicateGroup, FileCategory } from "@/lib/backend/generated";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatRelativeDate } from "@/lib/formatting/formatRelativeDate";
import { formatAbsoluteDate } from "../formatting/formatAbsoluteDate";
import { pluralize } from "../formatting/pluralize";
import { splitFilePath } from "../paths/splitFilePath";
import { truncatePathMiddle } from "../paths/truncatePathMiddle";
import { fileCategoryForPath } from "../setup/fileCategoryOptions";

const FOLDER_PATH_DISPLAY_LENGTH = 64;

export type DuplicateFileDescription = {
  path: string;
  fileName: string;
  folderPath: string;
  shortFolderPath: string;
  modifiedRelativeText: string;
  modifiedAbsoluteText: string;
  category: FileCategory | null;
  keep: boolean;
};

export type DuplicateGroupDescription = {
  copiesText: string;
  wastedText: string;
  removalText: string;
  files: DuplicateFileDescription[];
};

export function describeDuplicateFile(
  file: DuplicateFile,
  nowUnixMilliseconds: number,
): DuplicateFileDescription {
  const { fileName, folderPath } = splitFilePath(file.path);

  return {
    path: file.path,
    fileName,
    folderPath,
    shortFolderPath: truncatePathMiddle(folderPath, FOLDER_PATH_DISPLAY_LENGTH),
    modifiedRelativeText: formatRelativeDate(file.modifiedUnixMilliseconds, nowUnixMilliseconds),
    modifiedAbsoluteText: formatAbsoluteDate(file.modifiedUnixMilliseconds),
    category: fileCategoryForPath(file.path),
    keep: file.keep,
  };
}

/** The texts and rows a group card shows. */
export function describeDuplicateGroup(
  group: DuplicateGroup,
  nowUnixMilliseconds: number,
): DuplicateGroupDescription {
  const removalCount = group.files.filter((file) => !file.keep).length;

  return {
    copiesText: `${pluralize(group.files.length, "copy", "copies")} of ${formatBytesAsReadableSize(group.fileSize)}`,
    wastedText: `${formatBytesAsReadableSize(group.wastedBytes)} wasted`,
    removalText:
      removalCount === 0
        ? "Keeping every copy"
        : `${pluralize(removalCount, "copy", "copies")} to remove`,
    files: group.files.map((file) => describeDuplicateFile(file, nowUnixMilliseconds)),
  };
}

/** An estimate of a group card's height in pixels, used before the card is measured. */
export function estimateGroupCardHeight(fileCount: number | null): number {
  const headerHeight = 52;
  const rowHeight = 64;
  const gapBelowCard = 12;
  const typicalFileCount = 2;

  return headerHeight + rowHeight * (fileCount ?? typicalFileCount) + gapBelowCard;
}
