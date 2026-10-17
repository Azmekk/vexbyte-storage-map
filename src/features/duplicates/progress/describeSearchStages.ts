import type { DuplicateSearchProgress, DuplicateSearchStage } from "@/lib/backend/generated";

export type PipelineStage = Exclude<DuplicateSearchStage, "done">;

export type SearchStageState = "pending" | "active" | "complete";

export type SearchStageDescription = {
  stage: PipelineStage;
  label: string;
  state: SearchStageState;
  /** The main number for this stage: files that are still candidates after it. */
  fileCount: number;
  /** What `fileCount` means, in a few words. */
  fileCountCaption: string;
  /** Progress through this stage from 0 to 1, or `null` when it has no known end. */
  completedFraction: number | null;
  /** `fileCount` relative to all collected files. */
  shareOfCollectedFiles: number;
  /** Width of this stage's funnel bar, from 0 to 1 of the full width. */
  funnelBarShare: number;
  /** Width of the filled part of the funnel bar, from 0 to 1 of the full width. */
  funnelFilledShare: number;
};

/** Keeps a sliver of bar visible for stages with very few files, so the funnel stays legible. */
const MINIMUM_VISIBLE_BAR_SHARE = 0.015;

export const PIPELINE_STAGES: readonly PipelineStage[] = [
  "collecting",
  "comparingSizes",
  "quickCheck",
  "fullCheck",
];

const STAGE_LABELS: Record<PipelineStage, string> = {
  collecting: "Collecting",
  comparingSizes: "Comparing sizes",
  quickCheck: "Quick check",
  fullCheck: "Full check",
};

export const EMPTY_SEARCH_PROGRESS: DuplicateSearchProgress = {
  stage: "collecting",
  foldersListed: 0,
  filesCollected: 0,
  bytesCollected: 0,
  onlineOnlyFilesSkipped: 0,
  filesWithMatchingSize: 0,
  quickCheckFilesDone: 0,
  quickCheckFilesTotal: 0,
  fullCheckFilesDone: 0,
  fullCheckFilesTotal: 0,
  fullCheckBytesDone: 0,
  fullCheckBytesTotal: 0,
  duplicateGroupsFound: 0,
};

function stagePosition(stage: DuplicateSearchStage): number {
  return stage === "done" ? PIPELINE_STAGES.length : PIPELINE_STAGES.indexOf(stage);
}

function stateOfStage(stage: PipelineStage, currentStage: DuplicateSearchStage): SearchStageState {
  const position = stagePosition(stage);
  const currentPosition = stagePosition(currentStage);

  if (position < currentPosition) {
    return "complete";
  }

  return position === currentPosition ? "active" : "pending";
}

function fractionOf(done: number, total: number): number | null {
  if (total <= 0) {
    return null;
  }

  return Math.min(1, Math.max(0, done / total));
}

function shareOf(count: number, collectedFiles: number): number {
  if (collectedFiles <= 0) {
    return 0;
  }

  return Math.min(1, count / collectedFiles);
}

function describeStage(
  stage: PipelineStage,
  progress: DuplicateSearchProgress,
): Pick<
  SearchStageDescription,
  "fileCount" | "fileCountCaption" | "completedFraction" | "shareOfCollectedFiles"
> {
  switch (stage) {
    case "collecting":
      return {
        fileCount: progress.filesCollected,
        fileCountCaption: "files found",
        completedFraction: null,
        shareOfCollectedFiles: progress.filesCollected > 0 ? 1 : 0,
      };
    case "comparingSizes":
      return {
        fileCount: progress.filesWithMatchingSize,
        fileCountCaption: "share their size with another file",
        completedFraction: null,
        shareOfCollectedFiles: shareOf(progress.filesWithMatchingSize, progress.filesCollected),
      };
    case "quickCheck":
      return {
        fileCount: progress.quickCheckFilesDone,
        fileCountCaption: `of ${progress.quickCheckFilesTotal.toLocaleString("en-US")} checked by their first and last bytes`,
        completedFraction: fractionOf(progress.quickCheckFilesDone, progress.quickCheckFilesTotal),
        shareOfCollectedFiles: shareOf(progress.quickCheckFilesTotal, progress.filesCollected),
      };
    case "fullCheck":
      return {
        fileCount: progress.fullCheckFilesDone,
        fileCountCaption: `of ${progress.fullCheckFilesTotal.toLocaleString("en-US")} compared byte for byte`,
        completedFraction: fractionOf(progress.fullCheckBytesDone, progress.fullCheckBytesTotal),
        shareOfCollectedFiles: shareOf(progress.fullCheckFilesTotal, progress.filesCollected),
      };
  }
}

/**
 * Turns a progress snapshot into the four stepper stages. Earlier stages keep their counts while
 * later ones run, so every stage can be shown at once.
 */
export function describeSearchStages(progress: DuplicateSearchProgress): SearchStageDescription[] {
  return PIPELINE_STAGES.map((stage) => {
    const state = stateOfStage(stage, progress.stage);
    const description = describeStage(stage, progress);
    const completedFraction = state === "complete" ? 1 : description.completedFraction;
    const hasStarted = state !== "pending";
    const minimumBarShare = description.fileCount > 0 ? MINIMUM_VISIBLE_BAR_SHARE : 0;
    const funnelBarShare = hasStarted
      ? Math.max(description.shareOfCollectedFiles, minimumBarShare)
      : 0;

    return {
      stage,
      label: STAGE_LABELS[stage],
      state,
      ...description,
      completedFraction,
      funnelBarShare,
      funnelFilledShare: funnelBarShare * (completedFraction ?? 0),
    };
  });
}

/** One plain sentence describing what the search is doing right now. */
export function describeCurrentActivity(progress: DuplicateSearchProgress): string {
  switch (progress.stage) {
    case "collecting":
      return `Looking through ${progress.foldersListed.toLocaleString("en-US")} folders for matching files.`;
    case "comparingSizes":
      return "Grouping files of the exact same size.";
    case "quickCheck":
      return "Reading the start and end of each candidate to rule out look-alikes.";
    case "fullCheck":
      return "Comparing the remaining candidates byte for byte.";
    case "done":
      return "Putting the results together.";
  }
}
