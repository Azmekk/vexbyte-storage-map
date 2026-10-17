const BYTES_PER_UNIT_STEP = 1024;
const SIZE_UNITS = ["B", "KB", "MB", "GB", "TB"] as const;

/**
 * Formats a byte count with 1024-based units: whole bytes, one decimal from KB to TB.
 * For example `1536` becomes `1.5 KB`.
 */
export function formatBytesAsReadableSize(byteCount: number): string {
  if (!Number.isFinite(byteCount) || byteCount <= 0) {
    return "0 B";
  }

  if (byteCount < BYTES_PER_UNIT_STEP) {
    return `${Math.round(byteCount)} B`;
  }

  let scaledSize = byteCount;
  let unitIndex = 0;

  while (scaledSize >= BYTES_PER_UNIT_STEP && unitIndex < SIZE_UNITS.length - 1) {
    scaledSize /= BYTES_PER_UNIT_STEP;
    unitIndex += 1;
  }

  const roundedSize = Math.round(scaledSize * 10) / 10;
  const roundsUpToNextUnit =
    roundedSize >= BYTES_PER_UNIT_STEP && unitIndex < SIZE_UNITS.length - 1;

  if (roundsUpToNextUnit) {
    return `1.0 ${SIZE_UNITS[unitIndex + 1]}`;
  }

  return `${roundedSize.toFixed(1)} ${SIZE_UNITS[unitIndex]}`;
}
