import { RotateCwIcon, TriangleAlertIcon } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import type { DuplicateSearchId, DuplicateSearchSummary } from "@/lib/backend/generated";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { DuplicateGroupList } from "./DuplicateGroupList";
import { describeSearchCaveats, describeSearchHeadline } from "./describeSearchSummary";
import { KeepRuleSelect } from "./KeepRuleSelect";
import { NoDuplicatesState } from "./NoDuplicatesState";
import { RecycleConfirmationDialog } from "./RecycleConfirmationDialog";
import { RecycleFooter } from "./RecycleFooter";
import { RecycleResultPanel } from "./RecycleResultPanel";
import { SortOrderSelect } from "./SortOrderSelect";
import { SummaryCards } from "./SummaryCards";
import { describeRecycleButton } from "./selectionSummary";
import { useDuplicateResults } from "./useDuplicateResults";
import { useRecycleDuplicates } from "./useRecycleDuplicates";

type DuplicateResultsStepProps = {
  searchId: DuplicateSearchId;
  summary: DuplicateSearchSummary;
  onSearchAgain: () => void;
  onReturnToSetup: () => void;
};

export function DuplicateResultsStep({
  searchId,
  summary,
  onSearchAgain,
  onReturnToSetup,
}: DuplicateResultsStepProps) {
  const results = useDuplicateResults(searchId);
  const recycle = useRecycleDuplicates(searchId, results.sortOrder, results.groupsPage);
  const [nowUnixMilliseconds] = useState(() => Date.now());
  const { totals, loadErrorMessage } = results.groupsPage;

  const groupCount = totals?.totalGroupCount ?? summary.duplicateGroupCount;
  const selection = totals?.selection ?? { filesSelectedForRemoval: 0, bytesSelectedForRemoval: 0 };
  const hasNoGroups = totals !== null && totals.totalGroupCount === 0;
  const caveats = describeSearchCaveats(summary);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="border-b">
        <div className="mx-auto grid w-full max-w-6xl gap-5 px-6 pt-6 pb-4">
          <p className="text-muted-foreground text-sm">
            {describeSearchHeadline(summary)} {caveats.join(" ")}
          </p>

          {!hasNoGroups && (
            <SummaryCards
              groupCount={groupCount}
              duplicateFileCount={totals?.totalDuplicateFileCount ?? summary.duplicateFileCount}
              wastedBytes={totals?.totalWastedBytes ?? summary.totalWastedBytes}
              filesSelectedForRemoval={selection.filesSelectedForRemoval}
              bytesSelectedForRemoval={selection.bytesSelectedForRemoval}
            />
          )}

          {recycle.skippedFilesResult !== null && (
            <RecycleResultPanel
              result={recycle.skippedFilesResult}
              onDismiss={recycle.dismissSkippedFiles}
            />
          )}

          {loadErrorMessage !== null && (
            <div
              role="alert"
              className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm"
            >
              <span className="flex items-center gap-2 text-destructive">
                <TriangleAlertIcon className="size-4 shrink-0" aria-hidden />
                {loadErrorMessage}
              </span>
              <Button variant="outline" size="sm" onClick={results.groupsPage.retryLoading}>
                <RotateCwIcon />
                Try again
              </Button>
            </div>
          )}

          {!hasNoGroups && (
            <div className="flex flex-wrap items-center justify-between gap-3">
              <KeepRuleSelect
                keepRule={results.keepRule}
                isApplying={results.isApplyingKeepRule}
                onChooseKeepRule={results.chooseKeepRule}
                onChoosePreferredFolder={results.choosePreferredFolder}
              />
              <SortOrderSelect
                sortOrder={results.sortOrder}
                onChooseSortOrder={results.chooseSortOrder}
              />
            </div>
          )}
        </div>
      </div>

      {hasNoGroups ? (
        <div className="min-h-0 flex-1 overflow-y-auto">
          <div className="mx-auto w-full max-w-6xl p-6">
            <NoDuplicatesState
              title={summary.duplicateGroupCount > 0 ? "No duplicates left" : "No duplicates found"}
              details={
                summary.duplicateGroupCount > 0
                  ? ["Every group now has a single copy.", ...caveats]
                  : ["Every file in these folders is unique.", ...caveats]
              }
              onSearchAgain={onSearchAgain}
              onChangeSettings={onReturnToSetup}
            />
          </div>
        </div>
      ) : (
        <DuplicateGroupList
          groupCount={groupCount}
          groupAt={results.groupsPage.groupAt}
          requestGroupRange={results.groupsPage.requestGroupRange}
          nowUnixMilliseconds={nowUnixMilliseconds}
          leavingGroupIds={recycle.leavingGroupIds}
          pathsBeingSaved={results.pathsBeingSaved}
          onToggleKeep={results.toggleFileKeep}
          onReveal={results.revealFile}
          onOpen={results.openFile}
          onRecycleFile={recycle.openFileConfirmation}
          onRecycleGroup={recycle.openGroupConfirmation}
        />
      )}

      {!hasNoGroups && (
        <RecycleFooter
          buttonLabel={describeRecycleButton(
            selection.filesSelectedForRemoval,
            formatBytesAsReadableSize(selection.bytesSelectedForRemoval),
          )}
          canRecycle={selection.filesSelectedForRemoval > 0}
          isPreparing={recycle.confirmation.status === "loading"}
          onRecycle={recycle.openConfirmation}
        />
      )}

      <RecycleConfirmationDialog
        confirmation={recycle.confirmation}
        onClose={recycle.closeConfirmation}
        onConfirm={recycle.confirmRecycle}
      />
    </div>
  );
}
