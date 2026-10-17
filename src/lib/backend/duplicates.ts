import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  commands,
  type DuplicateCommandError,
  type DuplicateGroup,
  type DuplicateGroupId,
  type DuplicateGroupSortOrder,
  type DuplicateGroupsPage,
  type DuplicateRecycleResult,
  type DuplicateSearchFinished,
  type DuplicateSearchId,
  type DuplicateSearchOverview,
  type DuplicateSearchProgress,
  type DuplicateSearchSettings,
  type DuplicateSelectionSummary,
  events,
  type KeepRule,
  type SuggestedFolder,
} from "./generated";
import { createProgressChannel } from "./progressChannel";
import { unwrapCommandResult } from "./unwrapCommandResult";

export type {
  ChangedFile,
  DriveKind,
  DuplicateCommandError,
  DuplicateFile,
  DuplicateGroup,
  DuplicateGroupId,
  DuplicateGroupSortOrder,
  DuplicateGroupsPage,
  DuplicateRecycleResult,
  DuplicateSearchFailure,
  DuplicateSearchFinished,
  DuplicateSearchId,
  DuplicateSearchOutcome,
  DuplicateSearchOverview,
  DuplicateSearchProgress,
  DuplicateSearchSettings,
  DuplicateSearchStage,
  DuplicateSearchStatus,
  DuplicateSearchSummary,
  DuplicateSelectionSummary,
  FileChangeReason,
  FolderDriveKind,
  KeepRule,
  SuggestedFolder,
} from "./generated";

function describeDuplicateCommandError(commandError: DuplicateCommandError): string {
  switch (commandError.kind) {
    case "invalidSettings":
      return commandError.details;
    case "searchNotFound":
      return "This duplicate search is no longer available. Please search again.";
    case "searchNotFinished":
      return "This duplicate search has no results yet.";
    case "groupNotFound":
      return "This duplicate group no longer exists.";
    case "fileNotInGroup":
      return `${commandError.path} is not part of this duplicate group.`;
    case "fileNotInAnyGroup":
      return `${commandError.path} is not part of any duplicate group.`;
    case "groupWouldHaveNoKeptFile":
      return "Every group must keep at least one copy.";
    case "recycleFailed":
      return `The Recycle Bin operation failed. ${commandError.details}`;
    case "searchStartFailed":
      return `The duplicate search could not be started. ${commandError.details}`;
    case "backgroundTaskFailed":
      return "The operation stopped unexpectedly. Please try again.";
  }
}

/** The known personal folders that exist on this computer, to pre-fill the setup step. */
export async function getDefaultDuplicateFolders(): Promise<SuggestedFolder[]> {
  const result = await commands.getDefaultDuplicateFolders();
  return unwrapCommandResult(result, describeDuplicateCommandError);
}

/**
 * Validates `settings` and starts the search in the background; returns its id right away.
 * Invalid settings (no folders, a system folder, ...) reject with `invalidSettings`.
 */
export async function startDuplicateSearch(
  settings: DuplicateSearchSettings,
  onProgress: (progress: DuplicateSearchProgress) => void,
): Promise<DuplicateSearchId> {
  const progressChannel = createProgressChannel(onProgress);
  const result = await commands.startDuplicateSearch(settings, progressChannel);
  return unwrapCommandResult(result, describeDuplicateCommandError);
}

export async function cancelDuplicateSearch(searchId: DuplicateSearchId): Promise<void> {
  const result = await commands.cancelDuplicateSearch(searchId);
  unwrapCommandResult(result, describeDuplicateCommandError);
}

/** Status of a search plus, once it ended, how it ended. */
export async function getDuplicateSearchSummary(
  searchId: DuplicateSearchId,
): Promise<DuplicateSearchOverview> {
  const result = await commands.getDuplicateSearchSummary(searchId);
  return unwrapCommandResult(result, describeDuplicateCommandError);
}

/** Forgets a search and its groups. A running search is cancelled first. */
export async function discardDuplicateSearch(searchId: DuplicateSearchId): Promise<void> {
  const result = await commands.discardDuplicateSearch(searchId);
  unwrapCommandResult(result, describeDuplicateCommandError);
}

/** One page of groups, with totals and the current selection over all groups. */
export async function getDuplicateGroups(
  searchId: DuplicateSearchId,
  sort: DuplicateGroupSortOrder,
  offset: number,
  limit: number,
): Promise<DuplicateGroupsPage> {
  const result = await commands.getDuplicateGroups(searchId, sort, offset, limit);
  return unwrapCommandResult(result, describeDuplicateCommandError);
}

/** Applies a keep rule to every group, replacing choices made by hand. */
export async function setKeepRule(
  searchId: DuplicateSearchId,
  rule: KeepRule,
): Promise<DuplicateSelectionSummary> {
  const result = await commands.setKeepRule(searchId, rule);
  return unwrapCommandResult(result, describeDuplicateCommandError);
}

/**
 * Keeps or removes one copy by hand and returns the changed group. Removing the last kept copy
 * rejects with `groupWouldHaveNoKeptFile`.
 */
export async function setFileKeep(
  searchId: DuplicateSearchId,
  groupId: DuplicateGroupId,
  path: string,
  keep: boolean,
): Promise<DuplicateGroup> {
  const result = await commands.setFileKeep(searchId, groupId, path, keep);
  return unwrapCommandResult(result, describeDuplicateCommandError);
}

/**
 * Moves the listed copies to the Recycle Bin after checking every group keeps a copy and that
 * nothing changed since the search. Moved copies leave their groups; groups left with one copy
 * disappear, so refetch the visible pages afterwards.
 */
export async function recycleDuplicates(
  searchId: DuplicateSearchId,
  filesToRemove: string[],
): Promise<DuplicateRecycleResult> {
  const result = await commands.recycleDuplicates(searchId, filesToRemove);
  return unwrapCommandResult(result, describeDuplicateCommandError);
}

/** Calls `onSearchFinished` whenever any duplicate search ends. Resolves to an unlisten function. */
export function subscribeToDuplicateSearchFinished(
  onSearchFinished: (searchFinished: DuplicateSearchFinished) => void,
): Promise<UnlistenFn> {
  return events.duplicateSearchFinished.listen((event) => onSearchFinished(event.payload));
}
