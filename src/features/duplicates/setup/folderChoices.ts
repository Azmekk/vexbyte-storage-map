import type { SuggestedFolder } from "@/lib/backend/generated";
import { folderDisplayName } from "../paths/folderDisplayName";
import { isSameFolder } from "../paths/isPathInsideFolder";

export type FolderChoice = {
  path: string;
  name: string;
  isCustom: boolean;
  isSelected: boolean;
};

/** The known folders, all selected, ready for the setup step. */
export function folderChoicesFromSuggestions(
  suggestedFolders: readonly SuggestedFolder[],
): FolderChoice[] {
  return suggestedFolders.map((suggestedFolder) => ({
    path: suggestedFolder.path,
    name: suggestedFolder.name,
    isCustom: false,
    isSelected: true,
  }));
}

/**
 * Adds a folder picked by the user, selected. Picking a folder that is already listed selects it
 * instead of listing it twice.
 */
export function addCustomFolderChoice(
  folderChoices: readonly FolderChoice[],
  folderPath: string,
): FolderChoice[] {
  const alreadyListed = folderChoices.some((choice) => isSameFolder(choice.path, folderPath));

  if (alreadyListed) {
    return folderChoices.map((choice) =>
      isSameFolder(choice.path, folderPath) ? { ...choice, isSelected: true } : choice,
    );
  }

  return [
    ...folderChoices,
    { path: folderPath, name: folderDisplayName(folderPath), isCustom: true, isSelected: true },
  ];
}

export function removeFolderChoice(
  folderChoices: readonly FolderChoice[],
  folderPath: string,
): FolderChoice[] {
  return folderChoices.filter((choice) => choice.path !== folderPath);
}

export function toggleFolderChoice(
  folderChoices: readonly FolderChoice[],
  folderPath: string,
): FolderChoice[] {
  return folderChoices.map((choice) =>
    choice.path === folderPath ? { ...choice, isSelected: !choice.isSelected } : choice,
  );
}

export function selectedFolderPaths(folderChoices: readonly FolderChoice[]): string[] {
  return folderChoices.filter((choice) => choice.isSelected).map((choice) => choice.path);
}
