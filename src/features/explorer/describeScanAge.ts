import { formatRelativeDate } from "@/lib/formatting/formatRelativeDate";

/** "Scanned just now", "Scanned 12 minutes ago"; `null` when the end of the scan is unknown. */
export function describeScanAge(
  endedAtUnixMilliseconds: number | null,
  nowUnixMilliseconds: number,
): string | null {
  if (endedAtUnixMilliseconds === null) {
    return null;
  }
  return `Scanned ${formatRelativeDate(endedAtUnixMilliseconds, nowUnixMilliseconds)}`;
}
