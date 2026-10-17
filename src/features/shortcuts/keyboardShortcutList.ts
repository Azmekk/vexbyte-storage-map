export type KeyboardShortcut = {
  /** Each entry is one way to press it; each inner list holds the keys pressed together. */
  keyCombinations: readonly (readonly string[])[];
  description: string;
};

export type KeyboardShortcutGroup = {
  title: string;
  shortcuts: readonly KeyboardShortcut[];
};

/** What the shortcuts dialog lists. Keep it in step with `shortcutActions` and the tree table. */
export const KEYBOARD_SHORTCUT_GROUPS: readonly KeyboardShortcutGroup[] = [
  {
    title: "Anywhere",
    shortcuts: [
      { keyCombinations: [["?"], ["Ctrl", "/"]], description: "Show these shortcuts" },
      { keyCombinations: [["Esc"]], description: "Close a dialog, a menu or the details panel" },
    ],
  },
  {
    title: "Exploring a scan",
    shortcuts: [
      { keyCombinations: [["Ctrl", "K"]], description: "Search names in the scan" },
      { keyCombinations: [["Backspace"], ["Alt", "←"]], description: "Go up one folder" },
      { keyCombinations: [["Ctrl", "R"], ["F5"]], description: "Scan the same place again" },
    ],
  },
  {
    title: "In the folder table",
    shortcuts: [
      { keyCombinations: [["↑"], ["↓"]], description: "Move between rows" },
      { keyCombinations: [["→"], ["←"]], description: "Expand or collapse a folder" },
      { keyCombinations: [["Enter"]], description: "Open the selected folder" },
      { keyCombinations: [["Home"], ["End"]], description: "Jump to the first or last row" },
      {
        keyCombinations: [["Delete"]],
        description: "Move the selected item to the Recycle Bin (asks first)",
      },
    ],
  },
];
