import { ArrowUpIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  isTreemapColorMode,
  isVisualizationTab,
  useExplorerSelectionStore,
} from "@/features/explorer/useExplorerSelectionStore";
import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex } from "@/lib/backend/tree";
import { FileTypesView } from "./file_types/FileTypesView";
import { HierarchyVisualization } from "./HierarchyVisualization";
import { LargestFilesView } from "./largest_files/LargestFilesView";
import { usePaletteTheme } from "./usePaletteTheme";
import { VisualizationLegend } from "./VisualizationLegend";

type VisualizationTabsProps = {
  scanId: ScanId;
  shownFolder: NodeIndex;
  parentOfShownFolder: NodeIndex | null;
  rootNode: NodeIndex;
};

const TAB_CONTENT_CLASS_NAME = "flex min-h-0 flex-1 flex-col data-[state=inactive]:hidden";

export function VisualizationTabs({
  scanId,
  shownFolder,
  parentOfShownFolder,
}: VisualizationTabsProps) {
  const visualizationTab = useExplorerSelectionStore((state) => state.visualizationTab);
  const setVisualizationTab = useExplorerSelectionStore((state) => state.setVisualizationTab);
  const colorMode = useExplorerSelectionStore((state) => state.treemapColorMode);
  const setColorMode = useExplorerSelectionStore((state) => state.setTreemapColorMode);
  const openFolder = useExplorerSelectionStore((state) => state.openFolder);
  const selectNode = useExplorerSelectionStore((state) => state.selectNode);
  const theme = usePaletteTheme();
  const isHierarchyTab = visualizationTab === "treemap" || visualizationTab === "sunburst";

  return (
    <Tabs
      value={visualizationTab}
      onValueChange={(value) => isVisualizationTab(value) && setVisualizationTab(value)}
      className="flex h-full min-h-0 flex-col gap-0 bg-background"
    >
      <div className="flex flex-wrap items-center gap-2 border-b px-3 py-1.5">
        <TabsList>
          <TabsTrigger value="treemap">Treemap</TabsTrigger>
          <TabsTrigger value="sunburst">Sunburst</TabsTrigger>
          <TabsTrigger value="fileTypes">File types</TabsTrigger>
          <TabsTrigger value="largestFiles">Largest files</TabsTrigger>
        </TabsList>
        {isHierarchyTab && (
          <div className="ml-auto flex items-center gap-1.5">
            <Button
              variant="ghost"
              size="sm"
              disabled={parentOfShownFolder === null}
              onClick={() => {
                if (parentOfShownFolder !== null) {
                  openFolder(parentOfShownFolder);
                  selectNode(shownFolder);
                }
              }}
            >
              <ArrowUpIcon />
              Up
            </Button>
            <ToggleGroup
              type="single"
              size="sm"
              variant="outline"
              value={colorMode}
              onValueChange={(value) => isTreemapColorMode(value) && setColorMode(value)}
              aria-label="Colour blocks by"
            >
              <ToggleGroupItem value="category">By type</ToggleGroupItem>
              <ToggleGroupItem value="depth">By depth</ToggleGroupItem>
            </ToggleGroup>
          </div>
        )}
      </div>

      <TabsContent value="treemap" className={TAB_CONTENT_CLASS_NAME}>
        <HierarchyVisualization
          kind="treemap"
          scanId={scanId}
          shownFolder={shownFolder}
          parentOfShownFolder={parentOfShownFolder}
        />
      </TabsContent>
      <TabsContent value="sunburst" className={TAB_CONTENT_CLASS_NAME}>
        <HierarchyVisualization
          kind="sunburst"
          scanId={scanId}
          shownFolder={shownFolder}
          parentOfShownFolder={parentOfShownFolder}
        />
      </TabsContent>
      <TabsContent value="fileTypes" className={TAB_CONTENT_CLASS_NAME}>
        <FileTypesView scanId={scanId} shownFolder={shownFolder} />
      </TabsContent>
      <TabsContent value="largestFiles" className={`${TAB_CONTENT_CLASS_NAME} pt-2`}>
        <LargestFilesView scanId={scanId} shownFolder={shownFolder} />
      </TabsContent>

      {isHierarchyTab && (
        <div className="border-t px-3 py-2">
          <VisualizationLegend colorMode={colorMode} theme={theme} />
        </div>
      )}
    </Tabs>
  );
}
