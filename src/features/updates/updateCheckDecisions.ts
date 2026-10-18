import { compareVersions } from "./compareVersions";

export const UPDATE_CHECK_INTERVAL_MILLISECONDS = 24 * 60 * 60 * 1000;

/**
 * Whether to ask GitHub again: never asked before, a day has passed, or the clock went back so
 * the stored time lies in the future.
 */
export function shouldCheckForUpdateNow(
  lastCheckedAtUnixMilliseconds: number | null,
  nowUnixMilliseconds: number,
): boolean {
  if (lastCheckedAtUnixMilliseconds === null) {
    return true;
  }

  const elapsedMilliseconds = nowUnixMilliseconds - lastCheckedAtUnixMilliseconds;
  return elapsedMilliseconds < 0 || elapsedMilliseconds >= UPDATE_CHECK_INTERVAL_MILLISECONDS;
}

/** Reads a stored timestamp; anything that is not a whole, positive number counts as missing. */
export function parseStoredTimestamp(storedValue: string | null): number | null {
  if (storedValue === null || !/^\d+$/.test(storedValue)) {
    return null;
  }

  const timestamp = Number(storedValue);
  return Number.isSafeInteger(timestamp) && timestamp > 0 ? timestamp : null;
}

/** Whether to show the update dialog for `latestVersion`, unless the user skipped exactly it. */
export function shouldOfferUpdate(latestVersion: string, skippedVersion: string | null): boolean {
  return skippedVersion === null || compareVersions(latestVersion, skippedVersion) !== 0;
}

export type VersionChangeSinceLastRun =
  | { kind: "firstRun" }
  | { kind: "unchanged" }
  | { kind: "updated"; version: string };

/**
 * What changed since the app last ran: a first install (nothing stored), the same or an older
 * version, or an update to `currentVersion`.
 */
export function describeVersionChangeSinceLastRun(
  lastSeenVersion: string | null,
  currentVersion: string,
): VersionChangeSinceLastRun {
  if (lastSeenVersion === null) {
    return { kind: "firstRun" };
  }

  const comparison = compareVersions(currentVersion, lastSeenVersion);
  return comparison !== null && comparison > 0
    ? { kind: "updated", version: currentVersion }
    : { kind: "unchanged" };
}
