import { ScrollArea } from "@/components/ui/scroll-area";
import { useExplorerSelectionStore } from "@/features/explorer/useExplorerSelectionStore";
import { useFileTypeBreakdown } from "@/features/explorer/useTreeQueries";
import type { ScanId } from "@/lib/backend/scans";
import type { FileCategory, NodeIndex } from "@/lib/backend/tree";
import { useLastSuccessfulData } from "@/lib/queryCache/useLastSuccessfulData";
import { canvasChromeColors } from "../canvasChromeColors";
import { usePaletteTheme } from "../usePaletteTheme";
import { VisualizationState } from "../VisualizationState";
import { CategoryDonut } from "./CategoryDonut";
import { ExtensionBars } from "./ExtensionBars";
import {
  prepareCategorySlices,
  prepareExtensionBars,
  TOP_EXTENSION_COUNT,
} from "./prepareFileTypeCharts";

type FileTypesViewProps = {
  scanId: ScanId;
  shownFolder: NodeIndex;
};

export function FileTypesView({ scanId, shownFolder }: FileTypesViewProps) {
  const { state, retry } = useFileTypeBreakdown(scanId, shownFolder);
  const breakdown = useLastSuccessfulData(state);
  const theme = usePaletteTheme();
  const chrome = canvasChromeColors(theme);
  const showLargestFilesOfCategory = useExplorerSelectionStore(
    (store) => store.showLargestFilesOfCategory,
  );

  if (state.status === "error") {
    return <VisualizationState kind="error" errorMessage={state.errorMessage} onRetry={retry} />;
  }
  if (breakdown === null) {
    return <VisualizationState kind="loading" />;
  }
  if (breakdown.totalLogicalSize <= 0) {
    return <VisualizationState kind="empty" />;
  }

  const handleSelectCategory = (category: FileCategory | null) =>
    showLargestFilesOfCategory(category ?? "all");

  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className="grid gap-8 p-4">
        <section className="grid gap-3">
          <h3 className="font-medium text-sm">Space by kind of file</h3>
          <CategoryDonut
            slices={prepareCategorySlices(breakdown, theme)}
            totalSize={breakdown.totalLogicalSize}
            surfaceColor={chrome.surface}
            onSelectCategory={handleSelectCategory}
          />
        </section>
        <section className="grid gap-1">
          <h3 className="font-medium text-sm">Top {TOP_EXTENSION_COUNT} extensions by size</h3>
          <ExtensionBars
            bars={prepareExtensionBars(breakdown, theme)}
            inkColor={chrome.secondaryInk}
            onSelectCategory={handleSelectCategory}
          />
        </section>
      </div>
    </ScrollArea>
  );
}
