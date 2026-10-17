import { BackendCommandError } from "@/lib/backend/BackendCommandError";
import {
  cancelDuplicateSearch,
  type DuplicateSearchFailure,
  type DuplicateSearchFinished,
  type DuplicateSearchId,
  type DuplicateSearchSettings,
  discardDuplicateSearch,
  startDuplicateSearch,
  subscribeToDuplicateSearchFinished,
} from "@/lib/backend/duplicates";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { describeInvalidSettings } from "../setup/buildDuplicateSearchSettings";
import { searchIdOfRun, useDuplicateSearchStore } from "./useDuplicateSearchStore";

/*
 * The search outlives the Duplicates screen: the user may switch to another screen while it
 * runs. So its lifecycle lives here, at module level, and writes into the store; hooks only read
 * the store and call these functions.
 */

let currentRunNumber = 0;

function isInvalidSettingsError(error: unknown): error is BackendCommandError<{
  kind: "invalidSettings";
  details: string;
}> {
  if (!(error instanceof BackendCommandError)) {
    return false;
  }

  const commandError: unknown = error.commandError;
  return (
    typeof commandError === "object" &&
    commandError !== null &&
    "kind" in commandError &&
    commandError.kind === "invalidSettings"
  );
}

function describeSearchFailure(failure: DuplicateSearchFailure): string {
  switch (failure.kind) {
    case "invalidSettings":
      return describeInvalidSettings(failure.details);
    case "searchThreadFailed":
      return "The search stopped unexpectedly. Try again, or search fewer folders.";
  }
}

function applySearchFinished(searchFinished: DuplicateSearchFinished): void {
  const store = useDuplicateSearchStore.getState();
  const outcome = searchFinished.outcome;
  const searchId = searchFinished.id;

  switch (outcome.kind) {
    case "finished":
      store.resetResultPreferences();
      store.setRun({ status: "finished", searchId, summary: outcome.summary });
      return;
    case "cancelled":
      store.setRun({ status: "cancelled", searchId });
      return;
    case "failed":
      store.setRun({
        status: "failed",
        searchId,
        errorMessage: describeSearchFailure(outcome.failure),
      });
      return;
  }
}

function discardInBackground(searchId: DuplicateSearchId | null): void {
  if (searchId === null) {
    return;
  }

  discardDuplicateSearch(searchId).catch(() => {
    // The search may already be gone; nothing else holds on to it.
  });
}

/**
 * Starts a search with `settings`, replacing any previous one. Invalid settings send the user
 * back to the setup step with the reason; other failures show the failed state.
 */
export async function runDuplicateSearch(settings: DuplicateSearchSettings): Promise<void> {
  currentRunNumber += 1;
  const runNumber = currentRunNumber;
  const isCurrentRun = () => runNumber === currentRunNumber;
  const store = useDuplicateSearchStore.getState();

  discardInBackground(searchIdOfRun(store.run));
  store.setSetupErrorMessage(null);
  store.setLatestProgress(null);
  store.setRun({ status: "starting" });

  let startedSearchId: DuplicateSearchId | null = null;
  const finishedBeforeIdWasKnown = new Map<DuplicateSearchId, DuplicateSearchFinished>();

  // Subscribe first: a tiny search can finish before `startDuplicateSearch` resolves.
  const stopListening = await subscribeToDuplicateSearchFinished((searchFinished) => {
    if (startedSearchId === null) {
      finishedBeforeIdWasKnown.set(searchFinished.id, searchFinished);
      return;
    }

    if (searchFinished.id !== startedSearchId) {
      return;
    }

    stopListening();
    if (isCurrentRun()) {
      applySearchFinished(searchFinished);
    }
  });

  try {
    startedSearchId = await startDuplicateSearch(settings, (progress) => {
      if (isCurrentRun()) {
        useDuplicateSearchStore.getState().setLatestProgress(progress);
      }
    });
  } catch (error) {
    stopListening();
    if (!isCurrentRun()) {
      return;
    }

    if (isInvalidSettingsError(error)) {
      store.setRun({ status: "idle" });
      store.setSetupErrorMessage(describeInvalidSettings(error.commandError.details));
      return;
    }

    store.setRun({ status: "failed", searchId: null, errorMessage: readableErrorMessage(error) });
    return;
  }

  if (!isCurrentRun()) {
    stopListening();
    discardInBackground(startedSearchId);
    return;
  }

  const earlyFinish = finishedBeforeIdWasKnown.get(startedSearchId);
  if (earlyFinish !== undefined) {
    stopListening();
    applySearchFinished(earlyFinish);
    return;
  }

  store.setRun({ status: "running", searchId: startedSearchId });
}

/** Asks the running search to stop; the finished event then moves it to "cancelled". */
export async function cancelCurrentDuplicateSearch(): Promise<void> {
  const store = useDuplicateSearchStore.getState();
  const run = store.run;

  if (run.status !== "running") {
    return;
  }

  store.setRun({ status: "cancelling", searchId: run.searchId });

  try {
    await cancelDuplicateSearch(run.searchId);
  } catch (error) {
    store.setRun({
      status: "failed",
      searchId: run.searchId,
      errorMessage: readableErrorMessage(error),
    });
  }
}

/** Forgets the current search and its results and returns to the setup step. */
export function returnToDuplicateSetup(): void {
  currentRunNumber += 1;
  const store = useDuplicateSearchStore.getState();

  discardInBackground(searchIdOfRun(store.run));
  store.setLatestProgress(null);
  store.setRun({ status: "idle" });
}
