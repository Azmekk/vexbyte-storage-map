/** The parts of a key press the search results keyboard handling looks at. */
export type SearchResultsKeyPress = {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  /** Text is selected in the search box, so Ctrl+C must copy that text as usual. */
  hasTextSelection: boolean;
};

export type SearchResultsKeyContext = {
  selectedIndex: number | null;
  resultCount: number;
  /** How many rows fit on screen, for Page Up and Page Down. */
  pageSize: number;
  isQueryEmpty: boolean;
};

export type SearchResultsKeyAction =
  | { kind: "select"; index: number }
  | { kind: "open"; index: number }
  | { kind: "copyPath"; index: number }
  | { kind: "clearQuery" };

function hasNoModifier(press: SearchResultsKeyPress): boolean {
  return !press.ctrlKey && !press.metaKey && !press.altKey && !press.shiftKey;
}

function hasOnlyCommandModifier(press: SearchResultsKeyPress): boolean {
  return (press.ctrlKey || press.metaKey) && !press.altKey && !press.shiftKey;
}

function clampIndex(index: number, resultCount: number): number {
  return Math.min(Math.max(index, 0), resultCount - 1);
}

function moveSelection(context: SearchResultsKeyContext, step: number): SearchResultsKeyAction {
  if (context.selectedIndex === null) {
    return { kind: "select", index: 0 };
  }
  return { kind: "select", index: clampIndex(context.selectedIndex + step, context.resultCount) };
}

/**
 * What a key does on the search screen, where the focus stays in the search box while the arrows
 * move through the results: arrows, Page Up/Down and Ctrl+Home/End select, Enter opens, Ctrl+C
 * copies the selected path (unless text is selected in the box) and Escape clears the query.
 */
export function searchResultsKeyAction(
  press: SearchResultsKeyPress,
  context: SearchResultsKeyContext,
): SearchResultsKeyAction | null {
  if (press.key === "Escape" && hasNoModifier(press)) {
    return context.isQueryEmpty ? null : { kind: "clearQuery" };
  }
  if (context.resultCount === 0) {
    return null;
  }

  const pageStep = Math.max(1, context.pageSize - 1);
  const selectedIndex = context.selectedIndex;

  if (hasNoModifier(press)) {
    switch (press.key) {
      case "ArrowDown":
        return moveSelection(context, 1);
      case "ArrowUp":
        return selectedIndex === null ? null : moveSelection(context, -1);
      case "PageDown":
        return moveSelection(context, pageStep);
      case "PageUp":
        return selectedIndex === null ? null : moveSelection(context, -pageStep);
      case "Enter":
        return selectedIndex === null ? null : { kind: "open", index: selectedIndex };
      default:
        return null;
    }
  }

  if (hasOnlyCommandModifier(press)) {
    if (press.key === "Home") {
      return { kind: "select", index: 0 };
    }
    if (press.key === "End") {
      return { kind: "select", index: context.resultCount - 1 };
    }
    if (press.key.toLowerCase() === "c" && selectedIndex !== null && !press.hasTextSelection) {
      return { kind: "copyPath", index: selectedIndex };
    }
  }

  return null;
}
