import type { UnlistenFn } from "@tauri-apps/api/event";
import { platformWording } from "@/lib/platform/platformWording";
import {
  commands,
  events,
  type SearchCommandError,
  type SearchIndexStatus,
  type SearchQueryError,
  type SearchResponse,
  type SearchSortOrder,
  type SkipReason,
} from "./generated";
import { unwrapCommandResult } from "./unwrapCommandResult";

export type {
  IndexSummary,
  OverallIndexStatus,
  SearchCommandError,
  SearchHit,
  SearchIndexStatus,
  SearchIndexStatusChanged,
  SearchQueryError,
  SearchResponse,
  SearchSortOrder,
  SkipReason,
  VolumeIndexState,
  VolumeIndexStatus,
} from "./generated";

/** The most results one `searchFiles` call returns; larger limits are clamped by the backend. */
export const MAXIMUM_SEARCH_RESULT_COUNT = 10_000;

/** A sentence explaining why a query could not be understood, for showing under the search box. */
export function describeSearchQueryError(queryError: SearchQueryError): string {
  switch (queryError.kind) {
    case "invalidSizeFilter":
      return `"${queryError.value}" is not a size. Try size:>100mb, size:<1kb or size:1mb..10mb.`;
    case "invalidTypeFilter":
      return `"${queryError.value}" is not a type. Use type:file or type:folder.`;
    case "unsupportedFilter":
      return `The ${queryError.filter}: filter is not supported yet.`;
  }
}

/** Why a drive is not searched, for the per-volume list of the warm-up screen. */
export function describeSkipReason(reason: SkipReason): string {
  switch (reason) {
    case "cloudDrive":
      return "Cloud drives aren't indexed, so nothing gets downloaded.";
    case "networkDrive":
      return "Network drives aren't indexed.";
    case "removableDrive":
      return "Removable drives aren't indexed.";
    case "opticalDrive":
      return "Optical drives aren't indexed.";
    case "ramDisk":
      return "RAM disks aren't indexed.";
    case "unknownDriveType":
      return "This drive type isn't indexed.";
  }
}

function describeSearchCommandError(commandError: SearchCommandError): string {
  switch (commandError.kind) {
    case "invalidQuery":
      return describeSearchQueryError(commandError.problem);
    case "volumeNotFound":
      return `${commandError.mountPoint} is not connected anymore.`;
    case "volumeNotIndexed":
      return `${commandError.mountPoint} is not indexed. ${describeSkipReason(commandError.reason)}`;
    case "pathNotInIndex":
      return `${commandError.path} is no longer in the search index. Refresh the index and search again.`;
    case "revealFailed":
      return `Could not show ${commandError.path} in ${platformWording().fileManagerName}. ${commandError.details}`;
    case "openFailed":
      return `Could not open ${commandError.path}. ${commandError.details}`;
    case "backgroundTaskFailed":
      return "The search stopped unexpectedly. Please try again.";
  }
}

/** Every volume's index state plus one overall word (`warming`, `ready`, ...). */
export async function getSearchIndexStatus(): Promise<SearchIndexStatus> {
  const result = await commands.getSearchIndexStatus();
  return unwrapCommandResult(result, describeSearchCommandError);
}

/**
 * Rebuilds the index of one drive, or of every local drive when `mountPoint` is omitted (which
 * also picks up drives connected since the app started). Returns right away; the old index keeps
 * answering searches until the new one is ready. Follow progress with
 * `subscribeToSearchIndexStatus`.
 */
export async function refreshSearchIndex(mountPoint?: string): Promise<void> {
  const result = await commands.refreshSearchIndex(mountPoint ?? null);
  unwrapCommandResult(result, describeSearchCommandError);
}

/**
 * Searches every indexed drive. See `docs/SEARCH.md` for the query syntax. An empty query
 * resolves with no hits right away. A malformed query rejects with a `BackendCommandError` whose
 * `commandError.kind` is `invalidQuery`; its message can be shown under the search box.
 */
export async function searchFiles(
  query: string,
  sort: SearchSortOrder,
  limit: number,
): Promise<SearchResponse> {
  const result = await commands.searchFiles(query, sort, limit);
  return unwrapCommandResult(result, describeSearchCommandError);
}

/** Opens an Explorer window with a search result selected. Only paths of search hits work. */
export async function revealSearchHit(path: string): Promise<void> {
  const result = await commands.revealSearchHit(path);
  unwrapCommandResult(result, describeSearchCommandError);
}

/** Opens a search result with its default application, or a folder in Explorer. */
export async function openSearchHit(path: string): Promise<void> {
  const result = await commands.openSearchHit(path);
  unwrapCommandResult(result, describeSearchCommandError);
}

/**
 * Calls `onStatusChanged` with the full status whenever a drive's index changes state, and about
 * four times per second per drive while one is read. Call `getSearchIndexStatus` once after
 * subscribing for the current state. Resolves to a function that stops listening.
 */
export function subscribeToSearchIndexStatus(
  onStatusChanged: (status: SearchIndexStatus) => void,
): Promise<UnlistenFn> {
  return events.searchIndexStatusChanged.listen((event) => onStatusChanged(event.payload.status));
}
