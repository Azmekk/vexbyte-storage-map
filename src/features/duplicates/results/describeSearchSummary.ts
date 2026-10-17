import type { DuplicateSearchSummary } from "@/lib/backend/generated";
import { pluralize } from "../formatting/pluralize";

function describeElapsedTime(elapsedMilliseconds: number): string {
  const elapsedSeconds = elapsedMilliseconds / 1000;

  if (elapsedSeconds < 1) {
    return "under a second";
  }

  if (elapsedSeconds < 60) {
    return `${elapsedSeconds.toFixed(1)} seconds`;
  }

  const minutes = Math.floor(elapsedSeconds / 60);
  const seconds = Math.round(elapsedSeconds % 60);
  return `${pluralize(minutes, "minute", "minutes")} ${pluralize(seconds, "second", "seconds")}`;
}

/** One line for the results header: what was compared and how long it took. */
export function describeSearchHeadline(summary: DuplicateSearchSummary): string {
  return `Compared ${pluralize(summary.filesExamined, "file", "files")} in ${describeElapsedTime(summary.elapsedMilliseconds)}.`;
}

/** Notes about files the search could not or would not look at. Empty when there are none. */
export function describeSearchCaveats(summary: DuplicateSearchSummary): string[] {
  const caveats: string[] = [];

  if (summary.onlineOnlyFilesSkipped > 0) {
    caveats.push(
      `${pluralize(summary.onlineOnlyFilesSkipped, "file is", "files are")} online-only and were not checked.`,
    );
  }

  if (summary.unreadableFileCount > 0) {
    caveats.push(`${pluralize(summary.unreadableFileCount, "file", "files")} couldn't be read.`);
  }

  if (summary.inaccessibleFolderCount > 0) {
    caveats.push(
      `${pluralize(summary.inaccessibleFolderCount, "folder", "folders")} couldn't be opened.`,
    );
  }

  return caveats;
}
