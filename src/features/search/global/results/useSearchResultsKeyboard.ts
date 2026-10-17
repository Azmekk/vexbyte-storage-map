import type { Virtualizer } from "@tanstack/react-virtual";
import { type KeyboardEvent, useCallback } from "react";
import type { SearchHit } from "@/lib/backend/search";
import { useGlobalSearchStore } from "../useGlobalSearchStore";
import { copyHitPath, openHit } from "./searchHitActions";
import { searchResultsKeyAction } from "./searchResultsKeyAction";

function hasTextSelection(target: EventTarget): boolean {
  if (target instanceof HTMLInputElement) {
    return target.selectionStart !== target.selectionEnd;
  }
  const selection = window.getSelection();
  return selection !== null && selection.toString().length > 0;
}

/**
 * Only the search box and the results list drive the selection. Buttons, the sort menu and
 * context menus (rendered in portals, but bubbling through React) keep their own keys.
 */
function isResultsKeyboardTarget(target: EventTarget, screen: HTMLElement): boolean {
  if (!(target instanceof HTMLElement) || !screen.contains(target)) {
    return false;
  }
  return target instanceof HTMLInputElement || target.closest('[role="listbox"]') !== null;
}

type SearchResultsKeyboardOptions = {
  hits: readonly SearchHit[];
  selectedIndex: number | null;
  rowHeight: number;
  virtualizer: Virtualizer<HTMLDivElement, Element>;
};

/**
 * The key handler for the whole search screen (search box and list): the focus can stay in the
 * box while the arrows move through the results. Keys pressed in menus and popovers rendered in
 * portals are left to them.
 */
export function useSearchResultsKeyboard({
  hits,
  selectedIndex,
  rowHeight,
  virtualizer,
}: SearchResultsKeyboardOptions): (event: KeyboardEvent<HTMLElement>) => void {
  return useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      if (!isResultsKeyboardTarget(event.target, event.currentTarget) || event.defaultPrevented) {
        return;
      }

      const store = useGlobalSearchStore.getState();
      const visibleHeight = virtualizer.scrollElement?.clientHeight ?? rowHeight * 10;
      const action = searchResultsKeyAction(
        {
          key: event.key,
          ctrlKey: event.ctrlKey,
          metaKey: event.metaKey,
          altKey: event.altKey,
          shiftKey: event.shiftKey,
          hasTextSelection: hasTextSelection(event.target),
        },
        {
          selectedIndex,
          resultCount: hits.length,
          pageSize: Math.max(1, Math.floor(visibleHeight / rowHeight)),
          isQueryEmpty: store.query.length === 0,
        },
      );
      if (action === null) {
        return;
      }

      event.preventDefault();
      switch (action.kind) {
        case "select":
          store.selectIndex(action.index);
          virtualizer.scrollToIndex(action.index, { align: "auto" });
          return;
        case "open": {
          const hit = hits[action.index];
          if (hit !== undefined) {
            openHit(hit);
          }
          return;
        }
        case "copyPath": {
          const hit = hits[action.index];
          if (hit !== undefined) {
            copyHitPath(hit);
          }
          return;
        }
        case "clearQuery":
          store.clearQuery();
          return;
      }
    },
    [hits, selectedIndex, rowHeight, virtualizer],
  );
}
