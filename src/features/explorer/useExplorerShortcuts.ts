import { useEffect, useRef } from "react";
import { useSearchPaletteStore } from "@/features/search/useSearchPaletteStore";
import {
  type ExplorerShortcutAction,
  explorerShortcutAction,
} from "@/features/shortcuts/shortcutActions";
import { readShortcutKeyPress } from "@/features/shortcuts/shortcutKeyPress";
import type { NodeIndex } from "@/lib/backend/tree";
import { useExplorerSelectionStore } from "./useExplorerSelectionStore";

type ExplorerShortcutTargets = {
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
      explorer.openFolder(targets.parentOfShownFolder);
      explorer.selectNode(targets.shownFolder);
      return true;
    case "closeInspector":
      if (!explorer.isInspectorOpen) {
        return false;
      }
      explorer.setInspectorOpen(false);
      return true;
  }
}

/** Ctrl+K search, going up, rescanning and closing the inspector while a scan is explored. */
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

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);
}
