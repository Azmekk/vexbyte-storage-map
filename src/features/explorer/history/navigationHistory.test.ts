import { describe, expect, it } from "vitest";
import { mouseHistoryButtonAction } from "./mouseHistoryButtonAction";
import {
  backHistoryTarget,
  canGoBack,
  canGoForward,
  createNavigationHistory,
  currentHistoryEntry,
  forwardHistoryTarget,
  goBackInHistory,
  goForwardInHistory,
  MAXIMUM_HISTORY_ENTRIES,
  type NavigationHistory,
  pushHistoryEntry,
  replaceCurrentHistoryEntry,
} from "./navigationHistory";

function visit(folders: number[]): NavigationHistory {
  const [first = 0, ...rest] = folders;
  return rest.reduce(pushHistoryEntry, createNavigationHistory(first));
}

describe("navigation history", () => {
  it("starts at one folder with nowhere to go", () => {
    const history = createNavigationHistory(1);
    expect(currentHistoryEntry(history)).toBe(1);
    expect(canGoBack(history)).toBe(false);
    expect(canGoForward(history)).toBe(false);
    expect(backHistoryTarget(history)).toBeNull();
    expect(forwardHistoryTarget(history)).toBeNull();
  });

  it("goes back and forward through visited folders", () => {
    const history = visit([1, 2, 3]);
    expect(backHistoryTarget(history)).toBe(2);

    const afterBack = goBackInHistory(history);
    expect(currentHistoryEntry(afterBack)).toBe(2);
    expect(canGoForward(afterBack)).toBe(true);
    expect(forwardHistoryTarget(afterBack)).toBe(3);

    const afterSecondBack = goBackInHistory(afterBack);
    expect(currentHistoryEntry(afterSecondBack)).toBe(1);
    expect(canGoBack(afterSecondBack)).toBe(false);
    expect(goBackInHistory(afterSecondBack)).toBe(afterSecondBack);

    const afterForward = goForwardInHistory(afterSecondBack);
    expect(currentHistoryEntry(afterForward)).toBe(2);
  });

  it("does nothing going forward at the end", () => {
    const history = visit([1, 2]);
    expect(goForwardInHistory(history)).toBe(history);
  });

  it("drops the forward entries when a new folder is visited after going back", () => {
    const history = pushHistoryEntry(goBackInHistory(visit([1, 2, 3])), 4);
    expect(history.entries).toEqual([1, 2, 4]);
    expect(canGoForward(history)).toBe(false);
    expect(backHistoryTarget(history)).toBe(2);
  });

  it("ignores a visit to the folder that is already shown", () => {
    const history = visit([1, 2]);
    expect(pushHistoryEntry(history, 2)).toBe(history);
    expect(visit([1, 1, 2, 2, 2, 3]).entries).toEqual([1, 2, 3]);
  });

  it("keeps a visit to the folder before the current one as a new step", () => {
    expect(visit([1, 2, 1]).entries).toEqual([1, 2, 1]);
  });

  it("keeps only the most recent entries", () => {
    const folders = Array.from({ length: MAXIMUM_HISTORY_ENTRIES + 10 }, (_, index) => index);
    const history = visit(folders);
    expect(history.entries).toHaveLength(MAXIMUM_HISTORY_ENTRIES);
    expect(currentHistoryEntry(history)).toBe(MAXIMUM_HISTORY_ENTRIES + 9);
    expect(history.entries[0]).toBe(10);
  });

  it("replaces the current entry and merges neighbours that became equal", () => {
    const replaced = replaceCurrentHistoryEntry(visit([1, 2, 3]), 4);
    expect(replaced.entries).toEqual([1, 2, 4]);
    expect(replaced.currentIndex).toBe(2);

    const merged = replaceCurrentHistoryEntry(visit([1, 2, 3]), 2);
    expect(merged.entries).toEqual([1, 2]);
    expect(currentHistoryEntry(merged)).toBe(2);

    const mergedInMiddle = replaceCurrentHistoryEntry(goBackInHistory(visit([1, 2, 1])), 1);
    expect(mergedInMiddle.entries).toEqual([1]);
    expect(currentHistoryEntry(mergedInMiddle)).toBe(1);
  });
});

describe("mouseHistoryButtonAction", () => {
  it("maps the side buttons to back and forward and ignores the others", () => {
    expect(mouseHistoryButtonAction(3)).toBe("goBack");
    expect(mouseHistoryButtonAction(4)).toBe("goForward");
    expect(mouseHistoryButtonAction(0)).toBeNull();
    expect(mouseHistoryButtonAction(1)).toBeNull();
    expect(mouseHistoryButtonAction(2)).toBeNull();
  });
});
