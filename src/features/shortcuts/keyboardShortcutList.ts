import { platformWording } from "@/lib/platform/platformWording";

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
export function keyboardShortcutGroups(): readonly KeyboardShortcutGroup[] {
  return [
    {
      title: "Anywhere",
      shortcuts: [
        { keyCombinations: [["?"], ["Ctrl", "/"]], description: "Show these shortcuts" },
        { keyCombinations: [["Ctrl", "H"]], description: "Go to Your drives" },
        {
          keyCombinations: [["Ctrl", "Shift", "F"], ["F3"]],
          description: "Search files on every drive",
        },
        { keyCombinations: [["Esc"]], description: "Close a dialog, a menu or the details panel" },
      ],
    },
    {
      title: "Exploring a scan",
      shortcuts: [
        { keyCombinations: [["Ctrl", "K"]], description: "Search names in the scan" },
        {
          keyCombinations: [["Alt", "←"], ["Mouse back"]],
          description: "Go back to the previous folder",
        },
        {
          keyCombinations: [["Alt", "→"], ["Mouse forward"]],
          description: "Go forward again",
        },
        { keyCombinations: [["Backspace"]], description: "Go up one folder" },
        { keyCombinations: [["Alt", "Home"]], description: "Go to the top of the scan" },
        { keyCombinations: [["Ctrl", "R"], ["F5"]], description: "Scan the same place again" },
      ],
    },
    {
      title: "In search results",
      shortcuts: [
        { keyCombinations: [["↑"], ["↓"]], description: "Move through the results" },
        { keyCombinations: [["Enter"]], description: "Open the selected result" },
        { keyCombinations: [["Ctrl", "C"]], description: "Copy the path of the selected result" },
        { keyCombinations: [["Esc"]], description: "Clear the search" },
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
          description: `Move the selected item to the ${platformWording().binName} (asks first)`,
        },
      ],
    },
  ];
}
