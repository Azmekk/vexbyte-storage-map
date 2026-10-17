import { useCallback } from "react";
import type { DuplicateSearchProgress } from "@/lib/backend/generated";
import { EMPTY_SEARCH_PROGRESS } from "../progress/describeSearchStages";
import { buildDuplicateSearchSettings } from "../setup/buildDuplicateSearchSettings";
import {
  cancelCurrentDuplicateSearch,
  returnToDuplicateSetup,
  runDuplicateSearch,
} from "./duplicateSearchRunner";
import {
  type DuplicateScreenStep,
  type DuplicateSearchRun,
  screenStepForRun,
  useDuplicateSearchStore,
} from "./useDuplicateSearchStore";

export type UseDuplicateSearchResult = {
  step: DuplicateScreenStep;
  run: DuplicateSearchRun;
  progress: DuplicateSearchProgress;
  cancelSearch: () => void;
  searchAgain: () => void;
  returnToSetup: () => void;
};

/** The state of the current search and the actions that move between the screen's steps. */
export function useDuplicateSearch(): UseDuplicateSearchResult {
  const run = useDuplicateSearchStore((state) => state.run);
  const latestProgress = useDuplicateSearchStore((state) => state.latestProgress);

  const cancelSearch = useCallback(() => {
    void cancelCurrentDuplicateSearch();
  }, []);

  const searchAgain = useCallback(() => {
    const state = useDuplicateSearchStore.getState();
    const settings = buildDuplicateSearchSettings({
      folderChoices: state.folderChoices ?? [],
      selectedCategories: state.selectedCategories,
      minimumFileSizeBytes: state.minimumFileSizeBytes,
    });
    void runDuplicateSearch(settings);
  }, []);

  return {
    step: screenStepForRun(run),
    run,
    progress: latestProgress ?? EMPTY_SEARCH_PROGRESS,
    cancelSearch,
    searchAgain,
    returnToSetup: returnToDuplicateSetup,
  };
}
