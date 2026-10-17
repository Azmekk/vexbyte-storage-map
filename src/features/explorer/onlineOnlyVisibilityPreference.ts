import type { OnlineOnlyVisibility } from "@/lib/backend/tree";

export const ONLINE_ONLY_VISIBILITIES: readonly OnlineOnlyVisibility[] = ["hidden", "shown"];

const ONLINE_ONLY_VISIBILITY_STORAGE_KEY = "vsm.onlineOnlyVisibility";

/** Online-only cloud files are hidden unless the user chose to see them; unknown values hide. */
export function parseOnlineOnlyVisibility(storedValue: string | null): OnlineOnlyVisibility {
  const knownVisibility = ONLINE_ONLY_VISIBILITIES.find((visibility) => visibility === storedValue);
  return knownVisibility ?? "hidden";
}

export function readStoredOnlineOnlyVisibility(): OnlineOnlyVisibility {
  try {
    return parseOnlineOnlyVisibility(
      window.localStorage.getItem(ONLINE_ONLY_VISIBILITY_STORAGE_KEY),
    );
  } catch {
    return "hidden";
  }
}

export function storeOnlineOnlyVisibility(visibility: OnlineOnlyVisibility): void {
  try {
    window.localStorage.setItem(ONLINE_ONLY_VISIBILITY_STORAGE_KEY, visibility);
  } catch {
    // Storage can be unavailable; the choice then lasts for this session only.
  }
}
