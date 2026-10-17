import { SlidersHorizontalIcon } from "lucide-react";
import type { JSX } from "react";
import { TopBar } from "@/app/TopBar";
import { TopBarTitle } from "@/app/TopBarTitle";
import { Button } from "@/components/ui/button";
import { DuplicateProgressStep } from "./progress/DuplicateProgressStep";
import { DuplicateResultsStep } from "./results/DuplicateResultsStep";
import { useDuplicateSearch } from "./search/useDuplicateSearch";
import { DuplicateSetupStep } from "./setup/DuplicateSetupStep";

/**
 * The duplicate finder: setup, progress and results, under its own top bar. It fills its parent
 * and scrolls its own content, so place it in a container with a fixed height.
 */
export function DuplicatesScreen(): JSX.Element {
  const search = useDuplicateSearch();
  const { run } = search;

  return (
    <>
      <TopBar
        actions={
          run.status === "finished" && (
            <Button variant="outline" size="sm" onClick={search.returnToSetup}>
              <SlidersHorizontalIcon />
              New search
            </Button>
          )
        }
      >
        <TopBarTitle title="Duplicates" />
      </TopBar>
      <div className="flex min-h-0 flex-1 flex-col overflow-auto">
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
    </>
  );
}
