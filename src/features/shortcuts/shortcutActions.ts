import type { ShortcutKeyPress } from "./shortcutKeyPress";

export type AppShortcutAction = "showShortcuts" | "goToYourDrives" | "preventPageReload";
export type ExplorerShortcutAction =
  | "search"
  | "goUp"
  | "goBack"
  | "goForward"
  | "goToScanRoot"
  | "rescan"
  | "closeInspector";

function hasCommandModifier(press: ShortcutKeyPress): boolean {
  return (press.ctrlKey || press.metaKey) && !press.altKey;
}

function hasOnlyAlt(press: ShortcutKeyPress): boolean {
  return press.altKey && !press.ctrlKey && !press.metaKey;
}

function hasNoModifier(press: ShortcutKeyPress): boolean {
  return !press.ctrlKey && !press.metaKey && !press.altKey;
}

/**
 * Shortcuts that work on every screen: `?` or Ctrl+/ open the list of shortcuts and Ctrl+H goes
 * to "Your drives". The WebView's own reload keys are swallowed, since reloading would drop every
 * open scan from the window.
 */
export function appShortcutAction(press: ShortcutKeyPress): AppShortcutAction | null {
  if (press.isHandled) {
    return null;
  }
  if (((press.ctrlKey || press.metaKey) && press.key.toLowerCase() === "r") || press.key === "F5") {
    return "preventPageReload";
  }
  if (hasCommandModifier(press) && press.key === "/") {
    return "showShortcuts";
  }
  if (hasCommandModifier(press) && press.key.toLowerCase() === "h" && !press.isOverlayOpen) {
    return "goToYourDrives";
  }
  if (press.key === "?" && hasNoModifier(press) && !press.isTyping && !press.isOverlayOpen) {
    return "showShortcuts";
  }
  return null;
}

/**
 * Shortcuts of the explorer. Ctrl+K works from anywhere; the others stay out of the way while a
 * dialog or menu is open, and keys another handler already used are left alone (the tree table
 * goes up on Backspace itself). Backspace and Escape keep their meaning in text fields.
 */
export function explorerShortcutAction(press: ShortcutKeyPress): ExplorerShortcutAction | null {
  const key = press.key.toLowerCase();

  if (hasCommandModifier(press) && key === "k") {
    return "search";
  }
  if (press.isOverlayOpen || press.isHandled) {
    return null;
  }
  if ((hasCommandModifier(press) && key === "r") || (press.key === "F5" && hasNoModifier(press))) {
    return "rescan";
  }
  if (hasOnlyAlt(press) && press.key === "ArrowLeft") {
    return "goBack";
  }
  if (hasOnlyAlt(press) && press.key === "ArrowRight") {
    return "goForward";
  }
  if (hasOnlyAlt(press) && press.key === "Home") {
    return "goToScanRoot";
  }
  if (press.isTyping || !hasNoModifier(press)) {
    return null;
  }
  if (press.key === "Backspace") {
    return "goUp";
  }
  if (press.key === "Escape") {
    return "closeInspector";
  }
  return null;
}
