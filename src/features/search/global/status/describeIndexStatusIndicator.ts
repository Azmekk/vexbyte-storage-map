import type { SearchIndexStatus } from "@/lib/backend/search";
import { describeVolumeIndexState } from "./describeVolumeIndexState";

export type IndexStatusIndicator =
  | { kind: "none" }
  | { kind: "warming"; label: string }
  | { kind: "warning"; tooltip: string };

function describeUnsearchableDrives(status: SearchIndexStatus): string {
  const failedDrives = status.volumes
    .filter((volume) => volume.state.kind === "failed" && volume.index === null)
    .map((volume) => volume.mountPoint.replace(/[\\/]+$/, ""));
  return failedDrives.length === 0 ? "" : ` (${failedDrives.join(", ")})`;
}

/** The small marker next to "Search files" in the sidebar. */
export function describeIndexStatusIndicator(
  status: SearchIndexStatus | null,
  loadErrorMessage: string | null,
): IndexStatusIndicator {
  if (loadErrorMessage !== null) {
    return { kind: "warning", tooltip: `Search status unavailable. ${loadErrorMessage}` };
  }
  if (status === null) {
    return { kind: "none" };
  }

  switch (status.overall) {
    case "warming":
      return { kind: "warming", label: "Warming up…" };
    case "ready":
      return { kind: "none" };
    case "partiallyReady":
      return {
        kind: "warning",
        tooltip: `Some drives could not be indexed${describeUnsearchableDrives(status)}. The others can be searched.`,
      };
    case "failed": {
      const firstReason = status.volumes
        .map((volume) => describeVolumeIndexState(volume).detail)
        .find((detail) => detail !== null);
      return {
        kind: "warning",
        tooltip:
          status.volumes.length === 0
            ? "No local drive to search."
            : `Search is unavailable${firstReason === undefined ? "." : `: ${firstReason}`}`,
      };
    }
  }
}
