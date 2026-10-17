import { useEffect } from "react";
import { appShortcutAction } from "./shortcutActions";
import { readShortcutKeyPress } from "./shortcutKeyPress";
import { useKeyboardShortcutsDialogStore } from "./useKeyboardShortcutsDialogStore";

/** Listens for the shortcuts that work on every screen. */
export function useAppShortcuts(): void {
  const setShortcutsDialogOpen = useKeyboardShortcutsDialogStore((state) => state.setOpen);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const action = appShortcutAction(readShortcutKeyPress(event));
      if (action === null) {
        return;
      }
      event.preventDefault();
      if (action === "showShortcuts") {
        setShortcutsDialogOpen(true);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setShortcutsDialogOpen]);
}
