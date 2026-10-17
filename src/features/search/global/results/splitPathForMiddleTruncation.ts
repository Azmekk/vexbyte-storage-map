/** The part of a path that may be cut with an ellipsis, and the end that always stays visible. */
export type MiddleTruncatedPath = {
  head: string;
  tail: string;
};

const DEFAULT_MAXIMUM_TAIL_LENGTH = 32;
/** `C:\` is three characters; a separator at or before it belongs to the drive. */
const DRIVE_ROOT_LENGTH = 3;

/**
 * Splits a folder path so it can be shortened in the middle at any column width: the head is
 * rendered with an ellipsis where it overflows, the tail (the last folder, with its separator) is
 * always shown. `C:\Program Files\Common Files\System` becomes `C:\Program Files\Common Files` +
 * `\System`, displayed as `C:\Program Fi…\System` in a narrow column. A last folder longer than
 * `maximumTailLength` stays in the head, so it is cut instead of crowding out the start.
 */
export function splitPathForMiddleTruncation(
  path: string,
  maximumTailLength: number = DEFAULT_MAXIMUM_TAIL_LENGTH,
): MiddleTruncatedPath {
  const lastSeparatorIndex = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  const isSeparatorAtEnd = lastSeparatorIndex === path.length - 1;

  if (lastSeparatorIndex < DRIVE_ROOT_LENGTH || isSeparatorAtEnd) {
    return { head: path, tail: "" };
  }

  const tail = path.slice(lastSeparatorIndex);
  if (tail.length > maximumTailLength) {
    return { head: path, tail: "" };
  }

  return { head: path.slice(0, lastSeparatorIndex), tail };
}
