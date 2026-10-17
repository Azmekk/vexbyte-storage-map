import { useCallback, useRef, useState } from "react";
import { toast } from "sonner";
import { BackendCommandError } from "@/lib/backend/BackendCommandError";
import { setFileKeep, setKeepRule } from "@/lib/backend/duplicates";
import { openDuplicateWithDefaultApp, revealDuplicateInExplorer } from "@/lib/backend/fileActions";
import { pickFolder } from "@/lib/backend/folderPicker";
import type {
  DuplicateGroup,
  DuplicateGroupSortOrder,
  DuplicateSearchId,
  KeepRule,
} from "@/lib/backend/generated";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { useDuplicateSearchStore } from "../search/useDuplicateSearchStore";
import { findGroupInPages } from "./groupPageCache";
import { type KeepRuleKind, keepRuleFromKind, preferredFolderOf } from "./keepRuleOptions";
import { applyGroupChangeToSelection, canMarkForRemoval } from "./selectionSummary";
import {
  type UseDuplicateGroupsPageResult,
  useDuplicateGroupsPage,
} from "./useDuplicateGroupsPage";

const LAST_KEPT_COPY_TOAST_TITLE = "Every group keeps at least one copy";
const LAST_KEPT_COPY_TOAST_DESCRIPTION = "Mark another copy as kept first, then remove this one.";

export type UseDuplicateResultsResult = {
  groupsPage: UseDuplicateGroupsPageResult;
  keepRule: KeepRule;
  sortOrder: DuplicateGroupSortOrder;
  isApplyingKeepRule: boolean;
  /** Paths whose keep flag is being saved right now. */
  pathsBeingSaved: ReadonlySet<string>;
  chooseKeepRule: (kind: KeepRuleKind) => void;
  choosePreferredFolder: () => void;
  chooseSortOrder: (sortOrder: DuplicateGroupSortOrder) => void;
  toggleFileKeep: (group: DuplicateGroup, path: string) => void;
  revealFile: (path: string) => void;
  openFile: (path: string) => void;
};

function isLastKeptCopyError(error: unknown): boolean {
  if (!(error instanceof BackendCommandError)) {
    return false;
  }

  const commandError: unknown = error.commandError;
  return (
    typeof commandError === "object" &&
    commandError !== null &&
    "kind" in commandError &&
    commandError.kind === "groupWouldHaveNoKeptFile"
  );
}

function showLastKeptCopyToast(): void {
  toast.error(LAST_KEPT_COPY_TOAST_TITLE, { description: LAST_KEPT_COPY_TOAST_DESCRIPTION });
}

/** The results step's data and actions: groups, keep rule, sort order and per-file choices. */
export function useDuplicateResults(searchId: DuplicateSearchId): UseDuplicateResultsResult {
  const keepRule = useDuplicateSearchStore((state) => state.keepRule);
  const sortOrder = useDuplicateSearchStore((state) => state.sortOrder);
  const storeKeepRule = useDuplicateSearchStore((state) => state.setKeepRule);
  const storeSortOrder = useDuplicateSearchStore((state) => state.setSortOrder);
  const groupsPage = useDuplicateGroupsPage(searchId, sortOrder);
  const [isApplyingKeepRule, setIsApplyingKeepRule] = useState(false);
  const [pathsBeingSaved, setPathsBeingSaved] = useState<ReadonlySet<string>>(new Set());
  const groupsPageRef = useRef(groupsPage);
  groupsPageRef.current = groupsPage;

  const applyKeepRule = useCallback(
    async (rule: KeepRule) => {
      setIsApplyingKeepRule(true);

      try {
        await setKeepRule(searchId, rule);
        storeKeepRule(rule);
        await groupsPageRef.current.refreshLoadedPages();
      } catch (error) {
        toast.error("The keep rule could not be applied", {
          description: readableErrorMessage(error),
        });
      } finally {
        setIsApplyingKeepRule(false);
      }
    },
    [searchId, storeKeepRule],
  );

  const pickPreferredFolderThenApply = useCallback(async () => {
    const pickedFolder = await pickFolder("Choose the folder whose copies to keep");
    if (pickedFolder === null) {
      return;
    }

    const rule = keepRuleFromKind("preferredFolder", pickedFolder);
    if (rule !== null) {
      await applyKeepRule(rule);
    }
  }, [applyKeepRule]);

  const chooseKeepRule = useCallback(
    (kind: KeepRuleKind) => {
      const state = useDuplicateSearchStore.getState();
      const knownFolder = preferredFolderOf(state.keepRule) ?? state.lastPreferredFolder;
      const rule = keepRuleFromKind(kind, knownFolder);
      if (rule === null) {
        void pickPreferredFolderThenApply();
        return;
      }

      void applyKeepRule(rule);
    },
    [applyKeepRule, pickPreferredFolderThenApply],
  );

  const choosePreferredFolder = useCallback(() => {
    void pickPreferredFolderThenApply();
  }, [pickPreferredFolderThenApply]);

  const chooseSortOrder = useCallback(
    (nextSortOrder: DuplicateGroupSortOrder) => storeSortOrder(nextSortOrder),
    [storeSortOrder],
  );

  const toggleFileKeep = useCallback(
    (group: DuplicateGroup, path: string) => {
      const file = group.files.find((candidate) => candidate.path === path);
      if (file === undefined) {
        return;
      }

      if (file.keep && !canMarkForRemoval(group, path)) {
        showLastKeptCopyToast();
        return;
      }

      setPathsBeingSaved((previousPaths) => new Set(previousPaths).add(path));

      setFileKeep(searchId, group.id, path, !file.keep)
        .then((updatedGroup) => {
          const currentGroupsPage = groupsPageRef.current;
          const previousGroup = findGroupInPages(currentGroupsPage.pages, updatedGroup.id) ?? group;
          const currentSelection = currentGroupsPage.totals?.selection;
          if (currentSelection === undefined) {
            return;
          }

          currentGroupsPage.replaceGroup(
            updatedGroup,
            applyGroupChangeToSelection(currentSelection, previousGroup, updatedGroup),
          );
        })
        .catch((error: unknown) => {
          if (isLastKeptCopyError(error)) {
            showLastKeptCopyToast();
            return;
          }

          toast.error("This choice could not be saved", {
            description: readableErrorMessage(error),
          });
        })
        .finally(() => {
          setPathsBeingSaved((previousPaths) => {
            const remainingPaths = new Set(previousPaths);
            remainingPaths.delete(path);
            return remainingPaths;
          });
        });
    },
    [searchId],
  );

  const revealFile = useCallback(
    (path: string) => {
      revealDuplicateInExplorer(searchId, path).catch((error: unknown) => {
        toast.error("Could not show the file in Explorer", {
          description: readableErrorMessage(error),
        });
      });
    },
    [searchId],
  );

  const openFile = useCallback(
    (path: string) => {
      openDuplicateWithDefaultApp(searchId, path).catch((error: unknown) => {
        toast.error("Could not open the file", { description: readableErrorMessage(error) });
      });
    },
    [searchId],
  );

  return {
    groupsPage,
    keepRule,
    sortOrder,
    isApplyingKeepRule,
    pathsBeingSaved,
    chooseKeepRule,
    choosePreferredFolder,
    chooseSortOrder,
    toggleFileKeep,
    revealFile,
    openFile,
  };
}
