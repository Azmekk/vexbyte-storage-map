import type { SearchHit } from "@/lib/backend/search";

/** The folder that holds a hit: its path without the name. `C:\` for items at the drive root. */
export function searchHitFolderPath(hit: Pick<SearchHit, "path" | "name">): string {
  const lastSeparatorIndex = Math.max(hit.path.lastIndexOf("\\"), hit.path.lastIndexOf("/"));
  if (lastSeparatorIndex < 0) {
    return "";
  }
  const folderPath = hit.path.slice(0, lastSeparatorIndex);
  return /^[a-z]:$/i.test(folderPath) ? `${folderPath}\\` : folderPath;
}

/** `C:\` becomes `C:`, for compact drive columns and sentences. */
export function driveLetterLabel(mountPoint: string): string {
  return mountPoint.replace(/[\\/]+$/, "");
}
