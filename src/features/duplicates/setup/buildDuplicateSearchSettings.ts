import type { DuplicateSearchSettings, FileCategory } from "@/lib/backend/generated";
import { type FolderChoice, selectedFolderPaths } from "./folderChoices";

export type DuplicateSetupChoices = {
  folderChoices: readonly FolderChoice[];
  selectedCategories: readonly FileCategory[];
  minimumFileSizeBytes: number;
};

/** Why the search cannot start yet, or `null` when it can. Checked before calling the backend. */
export function describeSetupBlocker(choices: DuplicateSetupChoices): string | null {
  if (selectedFolderPaths(choices.folderChoices).length === 0) {
    return "Choose at least one folder to search.";
  }

  if (choices.selectedCategories.length === 0) {
    return "Choose at least one kind of file to compare.";
  }

  return null;
}

export function buildDuplicateSearchSettings(
  choices: DuplicateSetupChoices,
): DuplicateSearchSettings {
  return {
    folders: selectedFolderPaths(choices.folderChoices),
    categories: [...choices.selectedCategories],
    minimumFileSizeBytes: choices.minimumFileSizeBytes,
  };
}

/** Backend validation messages start in lower case; show them as sentences. */
export function describeInvalidSettings(details: string): string {
  const trimmedDetails = details.trim();

  if (trimmedDetails.length === 0) {
    return "These settings can't be used for a search.";
  }

  const sentence = `${trimmedDetails.charAt(0).toUpperCase()}${trimmedDetails.slice(1)}`;
  return /[.!?]$/.test(sentence) ? sentence : `${sentence}.`;
}
