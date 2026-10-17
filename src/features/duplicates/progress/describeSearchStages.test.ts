import { describe, expect, it } from "vitest";
import type { DuplicateSearchProgress } from "@/lib/backend/generated";
import { describeSearchStages, EMPTY_SEARCH_PROGRESS } from "./describeSearchStages";

function createProgress(overrides: Partial<DuplicateSearchProgress>): DuplicateSearchProgress {
  return { ...EMPTY_SEARCH_PROGRESS, ...overrides };
}

describe("describeSearchStages", () => {
  it("lists the four stages in pipeline order with readable labels", () => {
    const stages = describeSearchStages(EMPTY_SEARCH_PROGRESS);
    expect(stages.map((stage) => stage.label)).toEqual([
      "Collecting",
      "Comparing sizes",
      "Quick check",
      "Full check",
    ]);
  });

  it("marks earlier stages complete and later ones pending", () => {
    const stages = describeSearchStages(createProgress({ stage: "quickCheck" }));
    expect(stages.map((stage) => stage.state)).toEqual([
      "complete",
      "complete",
      "active",
      "pending",
    ]);
  });

  it("marks every stage complete when the search is done", () => {
    const stages = describeSearchStages(createProgress({ stage: "done" }));
    expect(stages.every((stage) => stage.state === "complete")).toBe(true);
    expect(stages.every((stage) => stage.completedFraction === 1)).toBe(true);
  });

  it("keeps the counts of earlier stages while later stages run", () => {
    const stages = describeSearchStages(
      createProgress({
        stage: "fullCheck",
        filesCollected: 1000,
        filesWithMatchingSize: 400,
        quickCheckFilesDone: 400,
        quickCheckFilesTotal: 400,
        fullCheckFilesDone: 50,
        fullCheckFilesTotal: 200,
        fullCheckBytesDone: 25,
        fullCheckBytesTotal: 100,
      }),
    );

    expect(stages.map((stage) => stage.fileCount)).toEqual([1000, 400, 400, 50]);
    expect(stages.map((stage) => stage.shareOfCollectedFiles)).toEqual([1, 0.4, 0.4, 0.2]);
    expect(stages[3]?.completedFraction).toBe(0.25);
    expect(stages[3]?.funnelBarShare).toBe(0.2);
    expect(stages[3]?.funnelFilledShare).toBe(0.05);
    expect(stages[1]?.funnelFilledShare).toBe(0.4);
    expect(stages[3]?.fileCountCaption).toBe("of 200 compared byte for byte");
  });

  it("has no fraction for stages without a known total and no share before anything is found", () => {
    const stages = describeSearchStages(createProgress({ stage: "collecting" }));
    expect(stages[0]?.completedFraction).toBeNull();
    expect(stages[2]?.completedFraction).toBeNull();
    expect(stages[0]?.shareOfCollectedFiles).toBe(0);
    expect(stages[3]?.funnelBarShare).toBe(0);
  });
});
