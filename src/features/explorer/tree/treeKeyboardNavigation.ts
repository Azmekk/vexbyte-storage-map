import type { NodeIndex } from "@/lib/backend/tree";
import { type NodeActionTarget, nodeActionTarget } from "../actions/nodeActionTarget";
import type { TreeRow } from "./flattenTreeRows";

/** What a key press in the tree table asks for. */
export type TreeKeyboardAction =
  | { kind: "focusRow"; rowIndex: number }
  | { kind: "expand"; folder: NodeIndex }
  | { kind: "collapse"; folder: NodeIndex }
  | { kind: "openFolder"; folder: NodeIndex }
  | { kind: "goUp" }
  | { kind: "showMore"; folder: NodeIndex }
  | { kind: "requestRecycle"; target: NodeActionTarget }
  | { kind: "none" };

const PAGE_STEP = 10;

function isFocusableRow(row: TreeRow | undefined): boolean {
  return row !== undefined && (row.kind === "item" || row.kind === "showMore");
}

function nearestFocusableRow(rows: TreeRow[], startIndex: number, direction: 1 | -1): number {
  for (let rowIndex = startIndex; rowIndex >= 0 && rowIndex < rows.length; rowIndex += direction) {
    if (isFocusableRow(rows[rowIndex])) {
      return rowIndex;
    }
  }
  return -1;
}

function clampedFocusTarget(rows: TreeRow[], wantedIndex: number, direction: 1 | -1): number {
  const clampedIndex = Math.min(Math.max(wantedIndex, 0), rows.length - 1);
  const target = nearestFocusableRow(rows, clampedIndex, direction);
  return target === -1 ? nearestFocusableRow(rows, clampedIndex, direction === 1 ? -1 : 1) : target;
}

function parentRowIndex(rows: TreeRow[], rowIndex: number): number {
  const row = rows[rowIndex];
  if (row === undefined || row.depth === 0) {
    return -1;
  }
  for (let candidateIndex = rowIndex - 1; candidateIndex >= 0; candidateIndex -= 1) {
    const candidate = rows[candidateIndex];
    if (candidate?.kind === "item" && candidate.item.node === row.parent) {
      return candidateIndex;
    }
  }
  return -1;
}

/**
 * Maps a key to a tree action, following the usual tree-view conventions: up and down move,
 * right expands (then steps into the first child), left collapses (then steps to the parent),
 * Enter opens a folder and Backspace goes to the parent folder. Delete only asks to move the
 * item to the Recycle Bin; the confirmation dialog always comes first.
 */
export function treeKeyboardAction(
  rows: TreeRow[],
  focusedRowIndex: number,
  key: string,
): TreeKeyboardAction {
  if (rows.length === 0) {
    return key === "Backspace" ? { kind: "goUp" } : { kind: "none" };
  }

  const hasFocus = focusedRowIndex >= 0 && focusedRowIndex < rows.length;
  const focusedRow = hasFocus ? rows[focusedRowIndex] : undefined;

  switch (key) {
    case "ArrowDown":
      return {
        kind: "focusRow",
        rowIndex: hasFocus
          ? clampedFocusTarget(rows, focusedRowIndex + 1, 1)
          : clampedFocusTarget(rows, 0, 1),
      };
    case "ArrowUp":
      return {
        kind: "focusRow",
        rowIndex: hasFocus
          ? clampedFocusTarget(rows, focusedRowIndex - 1, -1)
          : clampedFocusTarget(rows, 0, 1),
      };
    case "PageDown":
      return {
        kind: "focusRow",
        rowIndex: clampedFocusTarget(rows, focusedRowIndex + PAGE_STEP, 1),
      };
    case "PageUp":
      return {
        kind: "focusRow",
        rowIndex: clampedFocusTarget(rows, focusedRowIndex - PAGE_STEP, -1),
      };
    case "Home":
      return { kind: "focusRow", rowIndex: clampedFocusTarget(rows, 0, 1) };
    case "End":
      return { kind: "focusRow", rowIndex: clampedFocusTarget(rows, rows.length - 1, -1) };
    case "Backspace":
      return { kind: "goUp" };
    case "Delete":
      return focusedRow?.kind === "item" && !focusedRow.item.flags.synthetic
        ? { kind: "requestRecycle", target: nodeActionTarget(focusedRow.item) }
        : { kind: "none" };
    case "ArrowRight": {
      if (focusedRow?.kind !== "item" || !focusedRow.item.hasChildren) {
        return { kind: "none" };
      }
      if (!focusedRow.isExpanded) {
        return { kind: "expand", folder: focusedRow.item.node };
      }
      const firstChildIndex = nearestFocusableRow(rows, focusedRowIndex + 1, 1);
      const firstChild = rows[firstChildIndex];
      return firstChild !== undefined && firstChild.depth > focusedRow.depth
        ? { kind: "focusRow", rowIndex: firstChildIndex }
        : { kind: "none" };
    }
    case "ArrowLeft": {
      if (focusedRow?.kind === "item" && focusedRow.isExpanded) {
        return { kind: "collapse", folder: focusedRow.item.node };
      }
      const parentIndex = parentRowIndex(rows, focusedRowIndex);
      return parentIndex === -1 ? { kind: "none" } : { kind: "focusRow", rowIndex: parentIndex };
    }
    case "Enter": {
      if (focusedRow?.kind === "showMore") {
        return { kind: "showMore", folder: focusedRow.parent };
      }
      if (focusedRow?.kind === "item" && focusedRow.item.kind === "directory") {
        return { kind: "openFolder", folder: focusedRow.item.node };
      }
      return { kind: "none" };
    }
    default:
      return { kind: "none" };
  }
}
