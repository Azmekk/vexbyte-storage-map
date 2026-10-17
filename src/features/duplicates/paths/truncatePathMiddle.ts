const ELLIPSIS = "…";

/**
 * Shortens a path to at most `maximumLength` characters by cutting out the middle, keeping the
 * drive at the start and the closest folders at the end: `C:\Users\…\Holiday\2024`.
 */
export function truncatePathMiddle(path: string, maximumLength: number): string {
  if (path.length <= maximumLength) {
    return path;
  }

  if (maximumLength <= ELLIPSIS.length) {
    return ELLIPSIS;
  }

  const availableLength = maximumLength - ELLIPSIS.length;
  const tailLength = Math.ceil(availableLength * 0.6);
  const headLength = availableLength - tailLength;

  return `${path.slice(0, headLength)}${ELLIPSIS}${path.slice(path.length - tailLength)}`;
}
