import type { NodeIndex } from "@/lib/backend/tree";

/** Browser-style history of the folders shown in one scan. `entries[currentIndex]` is shown. */
export type NavigationHistory = {
  readonly entries: readonly NodeIndex[];
  readonly currentIndex: number;
};

/** Older entries are dropped beyond this, so a long session does not grow without bound. */
export const MAXIMUM_HISTORY_ENTRIES = 200;

export function createNavigationHistory(initialFolder: NodeIndex): NavigationHistory {
  return { entries: [initialFolder], currentIndex: 0 };
}

export function currentHistoryEntry(history: NavigationHistory): NodeIndex | undefined {
  return history.entries[history.currentIndex];
}

/**
 * Records a visit to `folder`: drops everything ahead of the current entry, as a browser does.
 * Visiting the folder that is already current changes nothing.
 */
export function pushHistoryEntry(history: NavigationHistory, folder: NodeIndex): NavigationHistory {
  if (currentHistoryEntry(history) === folder) {
    return history;
  }
  const entriesUpToCurrent = history.entries.slice(0, history.currentIndex + 1);
  const entries = [...entriesUpToCurrent, folder].slice(-MAXIMUM_HISTORY_ENTRIES);
  return { entries, currentIndex: entries.length - 1 };
}

/**
 * Swaps the current entry for `folder` without adding a step, for corrections such as leaving a
 * folder that no longer exists. Neighbours that end up equal are merged into one entry.
 */
export function replaceCurrentHistoryEntry(
  history: NavigationHistory,
  folder: NodeIndex,
): NavigationHistory {
  const entries = [...history.entries];
  entries[history.currentIndex] = folder;

  const mergedEntries: NodeIndex[] = [];
  let currentIndex = 0;
  entries.forEach((entry, entryIndex) => {
    if (mergedEntries.at(-1) !== entry) {
      mergedEntries.push(entry);
    }
    if (entryIndex === history.currentIndex) {
      currentIndex = mergedEntries.length - 1;
    }
  });

  return { entries: mergedEntries, currentIndex };
}

export function canGoBack(history: NavigationHistory): boolean {
  return history.currentIndex > 0;
}

export function canGoForward(history: NavigationHistory): boolean {
  return history.currentIndex < history.entries.length - 1;
}

/** The folder Back leads to, or `null` at the start of the history. */
export function backHistoryTarget(history: NavigationHistory): NodeIndex | null {
  return canGoBack(history) ? (history.entries[history.currentIndex - 1] ?? null) : null;
}

/** The folder Forward leads to, or `null` at the end of the history. */
export function forwardHistoryTarget(history: NavigationHistory): NodeIndex | null {
  return canGoForward(history) ? (history.entries[history.currentIndex + 1] ?? null) : null;
}

export function goBackInHistory(history: NavigationHistory): NavigationHistory {
  return canGoBack(history) ? { ...history, currentIndex: history.currentIndex - 1 } : history;
}

export function goForwardInHistory(history: NavigationHistory): NavigationHistory {
  return canGoForward(history) ? { ...history, currentIndex: history.currentIndex + 1 } : history;
}
