import { driveLetterLabel } from "../results/describeSearchHit";

function joinWithAnd(items: readonly string[]): string {
  if (items.length <= 1) {
    return items.join("");
  }
  return `${items.slice(0, -1).join(", ")} and ${items[items.length - 1]}`;
}

/**
 * The note for drives a search could not look at yet: `C: still warming, results will update`,
 * `C: and D: still warming, results will update`. `null` when every drive answered.
 */
export function describeStillWarmingVolumes(mountPoints: readonly string[]): string | null {
  if (mountPoints.length === 0) {
    return null;
  }
  return `${joinWithAnd(mountPoints.map(driveLetterLabel))} still warming, results will update`;
}

/** The banner title: how many drives are still being read. */
export function describeWarmingBannerTitle(warmingVolumeCount: number): string {
  if (warmingVolumeCount <= 1) {
    return "Warming up search engine";
  }
  return `Warming up search engine on ${warmingVolumeCount} drives`;
}
