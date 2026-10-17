import type { SearchStageDescription } from "./describeSearchStages";
import { SearchStageRow } from "./SearchStageRow";

type SearchStageFunnelProps = {
  stageDescriptions: readonly SearchStageDescription[];
  /** The search was cancelled or failed: nothing is active any more. */
  isStopped: boolean;
};

/** The pipeline as a stepper whose bars narrow as candidates are ruled out. */
export function SearchStageFunnel({ stageDescriptions, isStopped }: SearchStageFunnelProps) {
  return (
    <ol className="grid" aria-label="Search steps">
      {stageDescriptions.map((stageDescription, position) => (
        <SearchStageRow
          key={stageDescription.stage}
          stageDescription={stageDescription}
          stepNumber={position + 1}
          isLastStage={position === stageDescriptions.length - 1}
          isStopped={isStopped}
        />
      ))}
    </ol>
  );
}
