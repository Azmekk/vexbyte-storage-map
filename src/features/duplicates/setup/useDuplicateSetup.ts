import { useCallback, useEffect, useState } from "react";
import { getDefaultDuplicateFolders } from "@/lib/backend/duplicates";
import { pickFolder } from "@/lib/backend/folderPicker";
import type { FileCategory } from "@/lib/backend/generated";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { runDuplicateSearch } from "../search/duplicateSearchRunner";
import { useDuplicateSearchStore } from "../search/useDuplicateSearchStore";
import { buildDuplicateSearchSettings, describeSetupBlocker } from "./buildDuplicateSearchSettings";
import { toggleFileCategory } from "./fileCategoryOptions";
import {
  addCustomFolderChoice,
  type FolderChoice,
  folderChoicesFromSuggestions,
  removeFolderChoice,
  toggleFolderChoice,
} from "./folderChoices";

export type FolderChoicesState =
  | { status: "loading" }
  | { status: "error"; errorMessage: string }
  | { status: "ready"; folderChoices: FolderChoice[] };

export type UseDuplicateSetupResult = {
  folderChoicesState: FolderChoicesState;
  selectedCategories: FileCategory[];
  minimumFileSizeBytes: number;
  /** Why the search can't start yet (checked locally), or `null`. */
  setupBlocker: string | null;
  /** What the backend rejected last time, for example a system folder. */
  setupErrorMessage: string | null;
  isStarting: boolean;
  toggleFolder: (folderPath: string) => void;
  removeFolder: (folderPath: string) => void;
  addFolder: () => void;
  retryLoadingFolders: () => void;
  toggleCategory: (category: FileCategory) => void;
  setMinimumFileSizeBytes: (bytes: number) => void;
  startSearch: () => void;
};

/** Everything the setup step needs: folder and category choices and starting the search. */
export function useDuplicateSetup(): UseDuplicateSetupResult {
  const folderChoices = useDuplicateSearchStore((state) => state.folderChoices);
  const selectedCategories = useDuplicateSearchStore((state) => state.selectedCategories);
  const minimumFileSizeBytes = useDuplicateSearchStore((state) => state.minimumFileSizeBytes);
  const setupErrorMessage = useDuplicateSearchStore((state) => state.setupErrorMessage);
  const isStarting = useDuplicateSearchStore((state) => state.run.status === "starting");
  const setFolderChoices = useDuplicateSearchStore((state) => state.setFolderChoices);
  const setSelectedCategories = useDuplicateSearchStore((state) => state.setSelectedCategories);
  const setMinimumFileSizeBytes = useDuplicateSearchStore((state) => state.setMinimumFileSizeBytes);
  const [loadErrorMessage, setLoadErrorMessage] = useState<string | null>(null);

  const loadFolderChoices = useCallback(
    async (isResultStillWanted: () => boolean) => {
      setLoadErrorMessage(null);

      try {
        const suggestedFolders = await getDefaultDuplicateFolders();
        if (isResultStillWanted()) {
          setFolderChoices(folderChoicesFromSuggestions(suggestedFolders));
        }
      } catch (error) {
        if (isResultStillWanted()) {
          setLoadErrorMessage(readableErrorMessage(error));
        }
      }
    },
    [setFolderChoices],
  );

  const haveFolderChoices = folderChoices !== null;

  useEffect(() => {
    if (haveFolderChoices) {
      return;
    }

    let isMounted = true;
    void loadFolderChoices(() => isMounted);

    return () => {
      isMounted = false;
    };
  }, [haveFolderChoices, loadFolderChoices]);

  const currentFolderChoices = useCallback(
    () => useDuplicateSearchStore.getState().folderChoices ?? [],
    [],
  );

  const toggleFolder = useCallback(
    (folderPath: string) =>
      setFolderChoices(toggleFolderChoice(currentFolderChoices(), folderPath)),
    [setFolderChoices, currentFolderChoices],
  );

  const removeFolder = useCallback(
    (folderPath: string) =>
      setFolderChoices(removeFolderChoice(currentFolderChoices(), folderPath)),
    [setFolderChoices, currentFolderChoices],
  );

  const addFolder = useCallback(() => {
    pickFolder("Choose a folder to search for duplicates")
      .then((pickedFolder) => {
        if (pickedFolder !== null) {
          setFolderChoices(addCustomFolderChoice(currentFolderChoices(), pickedFolder));
        }
      })
      .catch((error: unknown) => {
        useDuplicateSearchStore.getState().setSetupErrorMessage(readableErrorMessage(error));
      });
  }, [setFolderChoices, currentFolderChoices]);

  const retryLoadingFolders = useCallback(() => {
    void loadFolderChoices(() => true);
  }, [loadFolderChoices]);

  const toggleCategory = useCallback(
    (category: FileCategory) =>
      setSelectedCategories(
        toggleFileCategory(useDuplicateSearchStore.getState().selectedCategories, category),
      ),
    [setSelectedCategories],
  );

  const setupChoices = {
    folderChoices: folderChoices ?? [],
    selectedCategories,
    minimumFileSizeBytes,
  };
  const setupBlocker = folderChoices === null ? null : describeSetupBlocker(setupChoices);

  const startSearch = useCallback(() => {
    const state = useDuplicateSearchStore.getState();
    const choices = {
      folderChoices: state.folderChoices ?? [],
      selectedCategories: state.selectedCategories,
      minimumFileSizeBytes: state.minimumFileSizeBytes,
    };

    if (describeSetupBlocker(choices) !== null) {
      return;
    }

    void runDuplicateSearch(buildDuplicateSearchSettings(choices));
  }, []);

  const folderChoicesState: FolderChoicesState =
    folderChoices !== null
      ? { status: "ready", folderChoices }
      : loadErrorMessage !== null
        ? { status: "error", errorMessage: loadErrorMessage }
        : { status: "loading" };

  return {
    folderChoicesState,
    selectedCategories,
    minimumFileSizeBytes,
    setupBlocker,
    setupErrorMessage,
    isStarting,
    toggleFolder,
    removeFolder,
    addFolder,
    retryLoadingFolders,
    toggleCategory,
    setMinimumFileSizeBytes,
    startSearch,
  };
}
