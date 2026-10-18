import type { OperatingSystem } from "@/lib/backend/platform";
import { currentPlatform } from "./currentPlatform";

/** The names of system features in user-facing text, which differ between operating systems. */
export type PlatformWording = {
  /** After "in" or "Show … in": `File Explorer`, `the file manager`. */
  fileManagerName: string;
  /** The menu item that opens a folder: `Open in File Explorer`, `Open in File Manager`. */
  openInFileManagerLabel: string;
  /** Used after "the" or "to": `Recycle Bin`, `Trash`. */
  binName: string;
  /** How to get an item back out of the bin, as one sentence. */
  restoreFromBinInstructions: string;
  /** Inside a sentence: `Windows`, `the system`. */
  systemName: string;
  /** At the start of a sentence: `Windows`, `The system`. */
  systemNameAtSentenceStart: string;
  /** One sentence on why some folders could not be read and what helps. */
  unreadableFoldersHint: string;
};

const WORDING_BY_OPERATING_SYSTEM: Record<OperatingSystem, PlatformWording> = {
  windows: {
    fileManagerName: "File Explorer",
    openInFileManagerLabel: "Open in File Explorer",
    binName: "Recycle Bin",
    restoreFromBinInstructions:
      "Restore anytime: open the Recycle Bin, right-click the item, then choose Restore.",
    systemName: "Windows",
    systemNameAtSentenceStart: "Windows",
    unreadableFoldersHint: "A scan as administrator can usually read them.",
  },
  linux: {
    fileManagerName: "the file manager",
    openInFileManagerLabel: "Open in File Manager",
    binName: "Trash",
    restoreFromBinInstructions:
      "Restore anytime: open the Trash in your file manager, right-click the item, then choose Restore.",
    systemName: "the system",
    systemNameAtSentenceStart: "The system",
    unreadableFoldersHint: "They usually belong to other users or to the system itself.",
  },
};

export function wordingForOperatingSystem(operatingSystem: OperatingSystem): PlatformWording {
  return WORDING_BY_OPERATING_SYSTEM[operatingSystem];
}

/** The wording for the operating system the app runs on. */
export function platformWording(): PlatformWording {
  return wordingForOperatingSystem(currentPlatform().os);
}
