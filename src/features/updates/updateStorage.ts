import { parseStoredTimestamp } from "./updateCheckDecisions";

const LAST_UPDATE_CHECK_STORAGE_KEY = "vsm.lastUpdateCheckAt";
const SKIPPED_VERSION_STORAGE_KEY = "vsm.skippedUpdateVersion";
const LAST_SEEN_VERSION_STORAGE_KEY = "vsm.lastSeenAppVersion";

// Storage can be unavailable or throw. Every read then answers "nothing stored" and every write is
// dropped, so the update prompt at worst shows up again on the next start.

function readStoredValue(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function storeValue(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // See above: the value then lasts for this session only.
  }
}

export function readLastUpdateCheckAt(): number | null {
  return parseStoredTimestamp(readStoredValue(LAST_UPDATE_CHECK_STORAGE_KEY));
}

export function storeLastUpdateCheckAt(unixMilliseconds: number): void {
  storeValue(LAST_UPDATE_CHECK_STORAGE_KEY, String(Math.trunc(unixMilliseconds)));
}

export function readSkippedUpdateVersion(): string | null {
  return readStoredValue(SKIPPED_VERSION_STORAGE_KEY);
}

export function storeSkippedUpdateVersion(version: string): void {
  storeValue(SKIPPED_VERSION_STORAGE_KEY, version);
}

export function readLastSeenAppVersion(): string | null {
  return readStoredValue(LAST_SEEN_VERSION_STORAGE_KEY);
}

export function storeLastSeenAppVersion(version: string): void {
  storeValue(LAST_SEEN_VERSION_STORAGE_KEY, version);
}
