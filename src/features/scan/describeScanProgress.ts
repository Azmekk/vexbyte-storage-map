import type { ScanPhase, ScanProgressUpdate } from "@/lib/backend/scans";

const PHASE_LABELS: Record<ScanPhase, string> = {
  listingDirectories: "Listing folders",
  readingMasterFileTable: "Reading the NTFS file table",
  buildingTree: "Adding up folder sizes",
  finished: "Finishing up",
};

export function scanPhaseLabel(phase: ScanPhase | null): string {
  return phase === null ? "Starting" : PHASE_LABELS[phase];
}

export type TopLevelFolderBar = {
  name: string;
  bytesFound: number;
  /** Relative to the largest folder so far, 0 to 1. */
  shareOfLargest: number;
};

const SHOWN_TOP_LEVEL_FOLDER_COUNT = 8;

/** The biggest folders so far with bar lengths relative to the biggest one. */
export function topLevelFolderBars(progress: ScanProgressUpdate | null): TopLevelFolderBar[] {
  const folders = (progress?.largestTopLevelDirectories ?? []).slice(
    0,
    SHOWN_TOP_LEVEL_FOLDER_COUNT,
  );
  const largestBytes = Math.max(1, ...folders.map((folder) => folder.bytesFound));

  return folders.map((folder) => ({
    name: folder.name,
    bytesFound: folder.bytesFound,
    shareOfLargest: folder.bytesFound / largestBytes,
  }));
}
