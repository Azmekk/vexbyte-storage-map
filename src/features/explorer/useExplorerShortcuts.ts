import { useEffect, useRef } from "react";
import { useSearchPaletteStore } from "@/features/search/useSearchPaletteStore";
import {
  type ExplorerShortcutAction,
  explorerShortcutAction,
} from "@/features/shortcuts/shortcutActions";
import { readShortcutKeyPress } from "@/features/shortcuts/shortcutKeyPress";
import type { NodeIndex } from "@/lib/backend/tree";
import { mouseHistoryButtonAction } from "./history/mouseHistoryButtonAction";
import { canGoBack, canGoForward } from "./history/navigationHistory";
import { selectCurrentHistory, useExplorerSelectionStore } from "./useExplorerSelectionStore";

type ExplorerShortcutTargets = {
  rootNode: NodeIndex;
  shownFolder: NodeIndex;
  parentOfShownFolder: NodeIndex | null;
  onRescan: () => void;
};

/** Runs one explorer shortcut; returns false when it had nothing to do, so the key stays free. */
function runExplorerShortcut(
  action: ExplorerShortcutAction,
  targets: ExplorerShortcutTargets,
): boolean {
  const explorer = useExplorerSelectionStore.getState();
  const history = selectCurrentHistory(explorer);

  switch (action) {
    case "search":
      useSearchPaletteStore.getState().setOpen(true);
      return true;
    case "rescan":
      targets.onRescan();
      return true;
    case "goUp":
      if (targets.parentOfShownFolder === null) {
        return false;
      }
      explorer.navigateToNode(targets.parentOfShownFolder, { selectedNode: targets.shownFolder });
      return true;
    case "goBack":
      if (history === null || !canGoBack(history)) {
        return false;
      }
      explorer.goBack();
      return true;
    case "goForward":
      if (history === null || !canGoForward(history)) {
        return false;
      }
      explorer.goForward();
      return true;
    case "goToScanRoot":
      explorer.navigateToNode(targets.rootNode);
      return true;
    case "closeInspector":
      if (!explorer.isInspectorOpen) {
        return false;
      }
      explorer.setInspectorOpen(false);
      return true;
  }
}

/**
 * Ctrl+K search, going up, back and forward (keys and the mouse side buttons), rescanning and
 * closing the inspector while a scan is explored.
 */
export function useExplorerShortcuts(targets: ExplorerShortcutTargets): void {
  const targetsRef = useRef(targets);
  targetsRef.current = targets;

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const action = explorerShortcutAction(readShortcutKeyPress(event));
      if (action !== null && runExplorerShortcut(action, targetsRef.current)) {
        event.preventDefault();
      }
    };

    // The side buttons also make the WebView navigate its own history on release, so both the
    // press and the release are swallowed; the press does the work.
    const handleMouseDown = (event: MouseEvent) => {
      const action = mouseHistoryButtonAction(event.button);
      if (action === null) {
        return;
      }
      event.preventDefault();
      runExplorerShortcut(action, targetsRef.current);
    };
    const handleMouseUp = (event: MouseEvent) => {
      if (mouseHistoryButtonAction(event.button) !== null) {
        event.preventDefault();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("mousedown", handleMouseDown);
    window.addEventListener("mouseup", handleMouseUp);
    window.addEventListener("auxclick", handleMouseUp);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("mousedown", handleMouseDown);
      window.removeEventListener("mouseup", handleMouseUp);
      window.removeEventListener("auxclick", handleMouseUp);
    };
  }, []);
}
