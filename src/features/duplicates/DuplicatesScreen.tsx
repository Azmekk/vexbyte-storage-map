import type { JSX } from "react";
import { DuplicateProgressStep } from "./progress/DuplicateProgressStep";
import { DuplicateResultsStep } from "./results/DuplicateResultsStep";
import { useDuplicateSearch } from "./search/useDuplicateSearch";
import { DuplicateSetupStep } from "./setup/DuplicateSetupStep";

/**
 * The duplicate finder: setup, progress and results. It fills its parent and scrolls its own
 * content, so place it in a container with a fixed height (not inside a scroll area).
 */
export function DuplicatesScreen(): JSX.Element {
  const search = useDuplicateSearch();
  const { run } = search;

  return (
    <div className="flex h-full min-h-0 flex-1 flex-col">
      {search.step === "setup" && <DuplicateSetupStep />}
      {search.step === "progress" && (
        <DuplicateProgressStep
          run={run}
          progress={search.progress}
          onCancel={search.cancelSearch}
          onSearchAgain={search.searchAgain}
          onReturnToSetup={search.returnToSetup}
        />
      )}
      {run.status === "finished" && (
        <DuplicateResultsStep
          key={run.searchId}
          searchId={run.searchId}
          summary={run.summary}
          onSearchAgain={search.searchAgain}
          onReturnToSetup={search.returnToSetup}
        />
      )}
    </div>
  );
}
