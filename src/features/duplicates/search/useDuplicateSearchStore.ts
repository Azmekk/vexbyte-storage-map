import { create } from "zustand";
import type {
  DuplicateGroupSortOrder,
  DuplicateSearchId,
  DuplicateSearchProgress,
  DuplicateSearchSummary,
  FileCategory,
  KeepRule,
} from "@/lib/backend/generated";
import { DEFAULT_KEEP_RULE } from "../results/keepRuleOptions";
import { DEFAULT_SORT_ORDER } from "../results/sortOrderOptions";
import { ALL_FILE_CATEGORIES } from "../setup/fileCategoryOptions";
import type { FolderChoice } from "../setup/folderChoices";
import { DEFAULT_MINIMUM_FILE_SIZE_BYTES } from "../setup/minimumSizePresets";

/** Where the current (or last) search stands. The screen's step follows from it. */
export type DuplicateSearchRun =
  | { status: "idle" }
  | { status: "starting" }
  | { status: "running"; searchId: DuplicateSearchId }
  | { status: "cancelling"; searchId: DuplicateSearchId }
  | { status: "cancelled"; searchId: DuplicateSearchId }
  | { status: "failed"; searchId: DuplicateSearchId | null; errorMessage: string }
  | { status: "finished"; searchId: DuplicateSearchId; summary: DuplicateSearchSummary };

export type DuplicateScreenStep = "setup" | "progress" | "results";

type DuplicateSearchState = {
  run: DuplicateSearchRun;
  /** The latest snapshot of a running search; replaced about ten times per second. */
  latestProgress: DuplicateSearchProgress | null;
  /** `null` until the known folders have been loaded once. */
  folderChoices: FolderChoice[] | null;
  selectedCategories: FileCategory[];
  minimumFileSizeBytes: number;
  /** Shown inline on the setup step, for example when a system folder was chosen. */
  setupErrorMessage: string | null;
  keepRule: KeepRule;
  sortOrder: DuplicateGroupSortOrder;

  setRun: (run: DuplicateSearchRun) => void;
  setLatestProgress: (progress: DuplicateSearchProgress | null) => void;
  setFolderChoices: (folderChoices: FolderChoice[]) => void;
  setSelectedCategories: (categories: FileCategory[]) => void;
  setMinimumFileSizeBytes: (bytes: number) => void;
  setSetupErrorMessage: (message: string | null) => void;
  setKeepRule: (rule: KeepRule) => void;
  setSortOrder: (sortOrder: DuplicateGroupSortOrder) => void;
  resetResultPreferences: () => void;
};

/** Duplicate finder UI state: ids, choices and the latest progress, never pages of groups. */
export const useDuplicateSearchStore = create<DuplicateSearchState>((set) => ({
  run: { status: "idle" },
  latestProgress: null,
  folderChoices: null,
  selectedCategories: [...ALL_FILE_CATEGORIES],
  minimumFileSizeBytes: DEFAULT_MINIMUM_FILE_SIZE_BYTES,
  setupErrorMessage: null,
  keepRule: DEFAULT_KEEP_RULE,
  sortOrder: DEFAULT_SORT_ORDER,

  setRun: (run) => set({ run }),
  setLatestProgress: (latestProgress) => set({ latestProgress }),
  setFolderChoices: (folderChoices) => set({ folderChoices, setupErrorMessage: null }),
  setSelectedCategories: (selectedCategories) =>
    set({ selectedCategories, setupErrorMessage: null }),
  setMinimumFileSizeBytes: (minimumFileSizeBytes) => set({ minimumFileSizeBytes }),
  setSetupErrorMessage: (setupErrorMessage) => set({ setupErrorMessage }),
  setKeepRule: (keepRule) => set({ keepRule }),
  setSortOrder: (sortOrder) => set({ sortOrder }),
  resetResultPreferences: () => set({ keepRule: DEFAULT_KEEP_RULE, sortOrder: DEFAULT_SORT_ORDER }),
}));

export function screenStepForRun(run: DuplicateSearchRun): DuplicateScreenStep {
  switch (run.status) {
    case "idle":
      return "setup";
    case "finished":
      return "results";
    case "starting":
    case "running":
    case "cancelling":
    case "cancelled":
    case "failed":
      return "progress";
  }
}

/** The id of the search the backend still holds for this screen, if any. */
export function searchIdOfRun(run: DuplicateSearchRun): DuplicateSearchId | null {
  switch (run.status) {
    case "running":
    case "cancelling":
    case "cancelled":
    case "failed":
    case "finished":
      return run.searchId;
    case "idle":
    case "starting":
      return null;
  }
}
