import { useVisualizationSubtree } from "@/features/explorer/useTreeQueries";
import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex } from "@/lib/backend/tree";
import { useLastSuccessfulData } from "@/lib/queryCache/useLastSuccessfulData";
import { SunburstView } from "./sunburst/SunburstView";
import { TreemapView } from "./treemap/TreemapView";
import { VisualizationState } from "./VisualizationState";

type HierarchyVisualizationProps = {
  kind: "treemap" | "sunburst";
  scanId: ScanId;
  shownFolder: NodeIndex;
  parentOfShownFolder: NodeIndex | null;
};

/**
 * Loads the pruned subtree of the shown folder and hands it to the treemap or sunburst. The
 * previous subtree stays on screen until the new one arrives, so zooming animates from it.
 */
export function HierarchyVisualization({
  kind,
  scanId,
  shownFolder,
  parentOfShownFolder,
}: HierarchyVisualizationProps) {
  const { state, retry } = useVisualizationSubtree(scanId, shownFolder);
  const subtree = useLastSuccessfulData(state);

  if (state.status === "error") {
    return <VisualizationState kind="error" errorMessage={state.errorMessage} onRetry={retry} />;
  }
  if (subtree === null) {
    return <VisualizationState kind="loading" />;
  }
  if (subtree.logicalSize <= 0 || subtree.children.length === 0) {
    return <VisualizationState kind="empty" />;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col p-2">
      {kind === "treemap" ? (
        <TreemapView scanId={scanId} subtree={subtree} />
      ) : (
        <SunburstView scanId={scanId} subtree={subtree} parentOfShownFolder={parentOfShownFolder} />
      )}
    </div>
  );
}
