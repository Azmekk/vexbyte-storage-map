import { useCallback, useState } from "react";
import { toast } from "sonner";
import { getDuplicateGroups, recycleDuplicates } from "@/lib/backend/duplicates";
import type {
  DuplicateGroup,
  DuplicateGroupId,
  DuplicateGroupSortOrder,
  DuplicateRecycleResult,
  DuplicateSearchId,
} from "@/lib/backend/generated";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { describeRecycleToast, hasSkippedFiles } from "./describeRecycleResult";
import type { RecycleScope } from "./describeRecycleScope";
import { groupIdsLeavingAfterMove } from "./groupPageCache";
import { isGroupWouldHaveNoKeptFileError } from "./isGroupWouldHaveNoKeptFileError";
import {
  groupRecycleRequest,
  type RecycleRequest,
  singleFileRecycleRequest,
} from "./recycleRequests";
import { collectFilesToRemove, type FileToRemove, totalBytesOf } from "./selectionSummary";
import type { UseDuplicateGroupsPageResult } from "./useDuplicateGroupsPage";

const ALL_GROUPS_CHUNK_SIZE = 500;

/** Long enough for the leaving cards' exit animation to finish before the list reflows. */
const LEAVING_ANIMATION_MILLISECONDS = 220;

const ALL_MARKED_SCOPE: RecycleScope = { kind: "allMarked" };

export type RecycleConfirmationState =
  | { status: "closed" }
  | { status: "loading"; scope: RecycleScope }
  | { status: "error"; scope: RecycleScope; errorMessage: string }
  | { status: "ready"; scope: RecycleScope; filesToRemove: FileToRemove[]; totalBytes: number }
  | {
      status: "recycling";
      scope: RecycleScope;
      filesToRemove: FileToRemove[];
      totalBytes: number;
    };

export type UseRecycleDuplicatesResult = {
  confirmation: RecycleConfirmationState;
  leavingGroupIds: ReadonlySet<DuplicateGroupId>;
  /** The last recycle's skipped files, until dismissed. */
  skippedFilesResult: DuplicateRecycleResult | null;
  /** Opens the dialog for every copy marked Remove, across all groups. */
  openConfirmation: () => void;
  /** Opens the dialog for one copy, whatever its Keep/Remove choice. */
  openFileConfirmation: (group: DuplicateGroup, path: string) => void;
  /** Opens the dialog for the copies of one group marked Remove. */
  openGroupConfirmation: (group: DuplicateGroup) => void;
  closeConfirmation: () => void;
  confirmRecycle: () => void;
  dismissSkippedFiles: () => void;
};

async function fetchAllGroups(
  searchId: DuplicateSearchId,
  sortOrder: DuplicateGroupSortOrder,
): Promise<DuplicateGroup[]> {
  const allGroups: DuplicateGroup[] = [];
  let offset = 0;
  let totalGroupCount = Number.POSITIVE_INFINITY;

  while (offset < totalGroupCount) {
    const page = await getDuplicateGroups(searchId, sortOrder, offset, ALL_GROUPS_CHUNK_SIZE);
    allGroups.push(...page.groups);
    totalGroupCount = page.totalGroupCount;
    if (page.groups.length === 0) {
      break;
    }
    offset += page.groups.length;
  }

  return allGroups;
}

function waitMilliseconds(milliseconds: number): Promise<void> {
  return new Promise((resolve) => {
    window.setTimeout(resolve, milliseconds);
  });
}

function showRecycleToast(result: DuplicateRecycleResult): void {
  const recycleToast = describeRecycleToast(result);
  const toastOptions = { description: recycleToast.description ?? undefined };

  if (recycleToast.kind === "success") {
    toast.success(recycleToast.title, toastOptions);
  } else if (recycleToast.kind === "warning") {
    toast.warning(recycleToast.title, toastOptions);
  } else {
    toast.error(recycleToast.title, toastOptions);
  }
}

function showRecycleErrorToast(error: unknown): void {
  toast.error("Nothing was moved to the Recycle Bin", {
    description: isGroupWouldHaveNoKeptFileError(error)
      ? "A group must keep at least one copy. Keep another copy, then try again."
      : readableErrorMessage(error),
  });
}

/**
 * The "Move to Recycle Bin" flows: one copy, the marked copies of one group, or every marked
 * copy. Lists the files for the confirmation dialog, moves exactly those on confirm, animates
 * emptied groups out and refetches the list.
 */
export function useRecycleDuplicates(
  searchId: DuplicateSearchId,
  sortOrder: DuplicateGroupSortOrder,
  groupsPage: UseDuplicateGroupsPageResult,
): UseRecycleDuplicatesResult {
  const [confirmation, setConfirmation] = useState<RecycleConfirmationState>({
    status: "closed",
  });
  const [leavingGroupIds, setLeavingGroupIds] = useState<ReadonlySet<DuplicateGroupId>>(new Set());
  const [skippedFilesResult, setSkippedFilesResult] = useState<DuplicateRecycleResult | null>(null);
  const { pages, refreshLoadedPages } = groupsPage;

  const openConfirmation = useCallback(() => {
    setConfirmation({ status: "loading", scope: ALL_MARKED_SCOPE });

    fetchAllGroups(searchId, sortOrder)
      .then((allGroups) => {
        const filesToRemove = collectFilesToRemove(allGroups);
        setConfirmation((current) =>
          current.status === "loading"
            ? {
                status: "ready",
                scope: ALL_MARKED_SCOPE,
                filesToRemove,
                totalBytes: totalBytesOf(filesToRemove),
              }
            : current,
        );
      })
      .catch((error: unknown) => {
        setConfirmation((current) =>
          current.status === "loading"
            ? {
                status: "error",
                scope: ALL_MARKED_SCOPE,
                errorMessage: readableErrorMessage(error),
              }
            : current,
        );
      });
  }, [searchId, sortOrder]);

  const openRequestConfirmation = useCallback((request: RecycleRequest | null) => {
    if (request === null) {
      return;
    }

    setConfirmation((current) =>
      current.status === "closed"
        ? {
            status: "ready",
            scope: request.scope,
            filesToRemove: request.filesToRemove,
            totalBytes: totalBytesOf(request.filesToRemove),
          }
        : current,
    );
  }, []);

  const openFileConfirmation = useCallback(
    (group: DuplicateGroup, path: string) =>
      openRequestConfirmation(singleFileRecycleRequest(group, path)),
    [openRequestConfirmation],
  );

  const openGroupConfirmation = useCallback(
    (group: DuplicateGroup) => openRequestConfirmation(groupRecycleRequest(group)),
    [openRequestConfirmation],
  );

  const closeConfirmation = useCallback(() => {
    setConfirmation((current) => (current.status === "recycling" ? current : { status: "closed" }));
  }, []);

  const confirmRecycle = useCallback(() => {
    if (confirmation.status !== "ready") {
      return;
    }

    const { scope, filesToRemove, totalBytes } = confirmation;
    setConfirmation({ status: "recycling", scope, filesToRemove, totalBytes });

    const recycleAndRefresh = async () => {
      try {
        const result = await recycleDuplicates(
          searchId,
          filesToRemove.map((file) => file.path),
        );

        setConfirmation({ status: "closed" });
        setSkippedFilesResult(hasSkippedFiles(result) ? result : null);
        showRecycleToast(result);

        setLeavingGroupIds(groupIdsLeavingAfterMove(pages, result.moved));
        await waitMilliseconds(LEAVING_ANIMATION_MILLISECONDS);
        await refreshLoadedPages();
      } catch (error) {
        setConfirmation({ status: "closed" });
        showRecycleErrorToast(error);
        await refreshLoadedPages();
      } finally {
        setLeavingGroupIds(new Set());
      }
    };

    void recycleAndRefresh();
  }, [confirmation, searchId, pages, refreshLoadedPages]);

  const dismissSkippedFiles = useCallback(() => setSkippedFilesResult(null), []);

  return {
    confirmation,
    leavingGroupIds,
    skippedFilesResult,
    openConfirmation,
    openFileConfirmation,
    openGroupConfirmation,
    closeConfirmation,
    confirmRecycle,
    dismissSkippedFiles,
  };
}
