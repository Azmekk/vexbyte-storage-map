import { useEffect } from "react";
import { useSearchPaletteStore } from "./useSearchPaletteStore";

/** Ctrl+K (or Cmd+K) opens the search palette while the explorer is shown. */
export function useSearchShortcut(): void {
  const setOpen = useSearchPaletteStore((state) => state.setOpen);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const isSearchShortcut =
        (event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === "k";
      if (isSearchShortcut) {
        event.preventDefault();
        setOpen(true);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setOpen]);
}
