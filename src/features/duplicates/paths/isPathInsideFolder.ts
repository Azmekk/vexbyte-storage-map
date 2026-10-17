function normalizeForComparison(path: string): string {
  return path.replaceAll("/", "\\").replace(/\\+$/, "").toLowerCase();
}

/** Whether `path` is `folder` itself or lies anywhere below it. Case-insensitive, like Windows. */
export function isPathInsideFolder(path: string, folder: string): boolean {
  const normalizedPath = normalizeForComparison(path);
  const normalizedFolder = normalizeForComparison(folder);

  if (normalizedFolder.length === 0) {
    return false;
  }

  return normalizedPath === normalizedFolder || normalizedPath.startsWith(`${normalizedFolder}\\`);
}

/** Whether two folder paths name the same folder, ignoring case and trailing separators. */
export function isSameFolder(firstFolder: string, secondFolder: string): boolean {
  return normalizeForComparison(firstFolder) === normalizeForComparison(secondFolder);
}
