import { describe, expect, it } from "vitest";
import { KEYBOARD_SHORTCUT_GROUPS } from "./keyboardShortcutList";
import { appShortcutAction, explorerShortcutAction } from "./shortcutActions";
import type { ShortcutKeyPress } from "./shortcutKeyPress";

function press(key: string, overrides: Partial<ShortcutKeyPress> = {}): ShortcutKeyPress {
  return {
    key,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    isTyping: false,
    isOverlayOpen: false,
    isHandled: false,
    ...overrides,
  };
}

describe("appShortcutAction", () => {
  it("opens the shortcuts list with ? or Ctrl+/", () => {
    expect(appShortcutAction(press("?"))).toBe("showShortcuts");
    expect(appShortcutAction(press("/", { ctrlKey: true }))).toBe("showShortcuts");
  });

  it("leaves ? alone while typing or while a dialog is open", () => {
    expect(appShortcutAction(press("?", { isTyping: true }))).toBeNull();
    expect(appShortcutAction(press("?", { isOverlayOpen: true }))).toBeNull();
  });

  it("swallows the page reload keys that nobody else used", () => {
    expect(appShortcutAction(press("r", { ctrlKey: true }))).toBe("preventPageReload");
    expect(appShortcutAction(press("F5"))).toBe("preventPageReload");
    expect(appShortcutAction(press("F5", { isHandled: true }))).toBeNull();
  });

  it("goes to Your drives with Ctrl+H, but not under a dialog", () => {
    expect(appShortcutAction(press("h", { ctrlKey: true }))).toBe("goToYourDrives");
    expect(appShortcutAction(press("H", { ctrlKey: true, isTyping: true }))).toBe("goToYourDrives");
    expect(appShortcutAction(press("h", { ctrlKey: true, isOverlayOpen: true }))).toBeNull();
    expect(appShortcutAction(press("h"))).toBeNull();
  });
});

describe("explorerShortcutAction", () => {
  it("opens search with Ctrl+K from anywhere", () => {
    expect(explorerShortcutAction(press("k", { ctrlKey: true }))).toBe("search");
    expect(explorerShortcutAction(press("K", { ctrlKey: true, isTyping: true }))).toBe("search");
  });

  it("goes up with Backspace, but not while typing", () => {
    expect(explorerShortcutAction(press("Backspace"))).toBe("goUp");
    expect(explorerShortcutAction(press("Backspace", { isTyping: true }))).toBeNull();
    expect(explorerShortcutAction(press("Backspace", { altKey: true }))).toBeNull();
  });

  it("goes back and forward with Alt+Left and Alt+Right", () => {
    expect(explorerShortcutAction(press("ArrowLeft", { altKey: true }))).toBe("goBack");
    expect(explorerShortcutAction(press("ArrowRight", { altKey: true }))).toBe("goForward");
    expect(explorerShortcutAction(press("ArrowLeft"))).toBeNull();
    expect(explorerShortcutAction(press("ArrowLeft", { altKey: true, ctrlKey: true }))).toBeNull();
    expect(
      explorerShortcutAction(press("ArrowLeft", { altKey: true, isHandled: true })),
    ).toBeNull();
  });

  it("returns to the scanned root with Alt+Home", () => {
    expect(explorerShortcutAction(press("Home", { altKey: true }))).toBe("goToScanRoot");
    expect(explorerShortcutAction(press("Home"))).toBeNull();
  });

  it("rescans with Ctrl+R or F5", () => {
    expect(explorerShortcutAction(press("r", { ctrlKey: true }))).toBe("rescan");
    expect(explorerShortcutAction(press("F5"))).toBe("rescan");
  });

  it("closes the inspector with Escape unless something else used the key", () => {
    expect(explorerShortcutAction(press("Escape"))).toBe("closeInspector");
    expect(explorerShortcutAction(press("Escape", { isOverlayOpen: true }))).toBeNull();
    expect(explorerShortcutAction(press("Escape", { isHandled: true }))).toBeNull();
  });

  it("never acts while a dialog is open, except for search", () => {
    expect(explorerShortcutAction(press("Backspace", { isOverlayOpen: true }))).toBeNull();
    expect(explorerShortcutAction(press("r", { ctrlKey: true, isOverlayOpen: true }))).toBeNull();
  });
});

describe("keyboard shortcut list", () => {
  it("describes every shortcut once", () => {
    const descriptions = KEYBOARD_SHORTCUT_GROUPS.flatMap((group) =>
      group.shortcuts.map((shortcut) => shortcut.description),
    );
    expect(new Set(descriptions).size).toBe(descriptions.length);
  });
});
