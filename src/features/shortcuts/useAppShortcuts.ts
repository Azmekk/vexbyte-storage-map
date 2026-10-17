import { useEffect } from "react";
import { useNavigationStore } from "@/app/useNavigationStore";
import { useGlobalSearchStore } from "@/features/search/global/useGlobalSearchStore";
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
      if (action === "goToYourDrives") {
        useNavigationStore.getState().navigateHome();
      }
      if (action === "searchFiles") {
        useNavigationStore.getState().navigateToSearch();
        useGlobalSearchStore.getState().requestInputFocus();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setShortcutsDialogOpen]);
}
