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
import { groupIdsLeavingAfterMove } from "./groupPageCache";
import { collectFilesToRemove, type FileToRemove, totalBytesOf } from "./selectionSummary";
import type { UseDuplicateGroupsPageResult } from "./useDuplicateGroupsPage";

const ALL_GROUPS_CHUNK_SIZE = 500;

/** Long enough for the leaving cards' exit animation to finish before the list reflows. */
const LEAVING_ANIMATION_MILLISECONDS = 220;

export type RecycleConfirmationState =
  | { status: "closed" }
  | { status: "loading" }
  | { status: "error"; errorMessage: string }
  | { status: "ready"; filesToRemove: FileToRemove[]; totalBytes: number }
  | { status: "recycling"; filesToRemove: FileToRemove[]; totalBytes: number };

export type UseRecycleDuplicatesResult = {
  confirmation: RecycleConfirmationState;
  leavingGroupIds: ReadonlySet<DuplicateGroupId>;
  /** The last recycle's skipped files, until dismissed. */
  skippedFilesResult: DuplicateRecycleResult | null;
  openConfirmation: () => void;
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

/**
 * The "Move to Recycle Bin" flow: list every copy marked for removal for the confirmation
 * dialog, move them on confirm, animate emptied groups out and refetch the list.
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
    setConfirmation({ status: "loading" });

    fetchAllGroups(searchId, sortOrder)
      .then((allGroups) => {
        const filesToRemove = collectFilesToRemove(allGroups);
        setConfirmation((current) =>
          current.status === "loading"
            ? { status: "ready", filesToRemove, totalBytes: totalBytesOf(filesToRemove) }
            : current,
        );
      })
      .catch((error: unknown) => {
        setConfirmation((current) =>
          current.status === "loading"
            ? { status: "error", errorMessage: readableErrorMessage(error) }
            : current,
        );
      });
  }, [searchId, sortOrder]);

  const closeConfirmation = useCallback(() => {
    setConfirmation((current) => (current.status === "recycling" ? current : { status: "closed" }));
  }, []);

  const confirmRecycle = useCallback(() => {
    if (confirmation.status !== "ready") {
      return;
    }

    const { filesToRemove, totalBytes } = confirmation;
    setConfirmation({ status: "recycling", filesToRemove, totalBytes });

    const recycleAndRefresh = async () => {
      try {
        const result = await recycleDuplicates(
          searchId,
          filesToRemove.map((file) => file.path),
        );
        const recycleToast = describeRecycleToast(result);
        const toastOptions = { description: recycleToast.description ?? undefined };

        setConfirmation({ status: "closed" });
        setSkippedFilesResult(hasSkippedFiles(result) ? result : null);
        if (recycleToast.kind === "success") {
          toast.success(recycleToast.title, toastOptions);
        } else if (recycleToast.kind === "warning") {
          toast.warning(recycleToast.title, toastOptions);
        } else {
          toast.error(recycleToast.title, toastOptions);
        }

        setLeavingGroupIds(groupIdsLeavingAfterMove(pages, result.moved));
        await waitMilliseconds(LEAVING_ANIMATION_MILLISECONDS);
        await refreshLoadedPages();
      } catch (error) {
        setConfirmation({ status: "closed" });
        toast.error("Nothing was moved to the Recycle Bin", {
          description: readableErrorMessage(error),
        });
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
    closeConfirmation,
    confirmRecycle,
    dismissSkippedFiles,
  };
}
