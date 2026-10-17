import { splitFilePath } from "./splitFilePath";

const TRAILING_SEPARATORS = /[\\/]+$/;

/** The last part of a folder path, for example "Photos" for `D:\Backup\Photos\`. */
export function folderDisplayName(folderPath: string): string {
  const pathWithoutTrailingSeparator = folderPath.replace(TRAILING_SEPARATORS, "");
  return splitFilePath(pathWithoutTrailingSeparator).fileName || folderPath;
}
