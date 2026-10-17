export type FilePathParts = {
  fileName: string;
  folderPath: string;
};

/** Splits a full path into the file name and the folder holding it. Accepts `\` and `/`. */
export function splitFilePath(path: string): FilePathParts {
  const lastSeparatorIndex = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));

  if (lastSeparatorIndex < 0) {
    return { fileName: path, folderPath: "" };
  }

  return {
    fileName: path.slice(lastSeparatorIndex + 1),
    folderPath: path.slice(0, lastSeparatorIndex),
  };
}
