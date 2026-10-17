import { useVirtualizer } from "@tanstack/react-virtual";
import { FileSearchIcon } from "lucide-react";
import { useRef } from "react";
import { EmptyState } from "@/components/EmptyState";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useNodeActions } from "@/features/explorer/actions/useNodeActions";
import { revealNodeInExplorer } from "@/features/explorer/revealNodeInExplorer";
import { useExplorerSelectionStore } from "@/features/explorer/useExplorerSelectionStore";
import { useLargestFiles } from "@/features/explorer/useTreeQueries";
import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex } from "@/lib/backend/tree";
import { formatCount } from "@/lib/formatting/formatCount";
import { useLastSuccessfulData } from "@/lib/queryCache/useLastSuccessfulData";
import {
  categoryColor,
  categoryLabel,
  FILE_CATEGORY_ORDER,
  isFileCategory,
} from "../categoryPalette";
import { usePaletteTheme } from "../usePaletteTheme";
import { VisualizationState } from "../VisualizationState";
import { LargestFileRow } from "./LargestFileRow";

const LARGEST_FILES_LIMIT = 500;
const ROW_HEIGHT = 48;

type LargestFilesViewProps = {
  scanId: ScanId;
  shownFolder: NodeIndex;
};

export function LargestFilesView({ scanId, shownFolder }: LargestFilesViewProps) {
  const categoryFilter = useExplorerSelectionStore((state) => state.largestFilesCategory);
  const setCategoryFilter = useExplorerSelectionStore((state) => state.setLargestFilesCategory);
  const selectedNode = useExplorerSelectionStore((state) => state.selectedNode);
  const selectNode = useExplorerSelectionStore((state) => state.selectNode);
  const backendCategory = categoryFilter === "all" ? null : categoryFilter;
  const { state, retry } = useLargestFiles(
    scanId,
    shownFolder,
    backendCategory,
    LARGEST_FILES_LIMIT,
  );
  const files = useLastSuccessfulData(state);
  const actions = useNodeActions(scanId);
  const theme = usePaletteTheme();
  const scrollElementRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: files?.length ?? 0,
    getScrollElement: () => scrollElementRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
  });

  const largestSize = Math.max(1, files?.[0]?.logicalSize ?? 1);
  const filterLabel =
    categoryFilter === "all" ? "files" : categoryLabel(categoryFilter).toLowerCase();

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 px-3 pb-2">
        <Select
          value={categoryFilter}
          onValueChange={(value) =>
            (value === "all" || isFileCategory(value)) && setCategoryFilter(value)
          }
        >
          <SelectTrigger size="sm" className="w-52" aria-label="Kind of file">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All kinds of files</SelectItem>
            {FILE_CATEGORY_ORDER.map((category) => (
              <SelectItem key={category} value={category}>
                <span
                  aria-hidden
                  className="size-2.5 rounded-sm"
                  style={{ backgroundColor: categoryColor(category, theme) }}
                />
                {categoryLabel(category)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {files !== null && files.length > 0 && (
          <span className="text-muted-foreground text-xs">
            {files.length >= LARGEST_FILES_LIMIT
              ? `Largest ${formatCount(LARGEST_FILES_LIMIT)} ${filterLabel}`
              : `${formatCount(files.length)} ${filterLabel}`}
          </span>
        )}
      </div>

      {state.status === "error" && (
        <VisualizationState kind="error" errorMessage={state.errorMessage} onRetry={retry} />
      )}
      {state.status !== "error" && files === null && <VisualizationState kind="loading" />}
      {state.status !== "error" && files !== null && files.length === 0 && (
        <EmptyState
          className="my-auto"
          icon={FileSearchIcon}
          title={categoryFilter === "all" ? "No files here" : `No ${filterLabel} in this folder`}
          description="Pick another kind of file, or open a different folder."
          actions={
            categoryFilter === "all" ? undefined : (
              <Button variant="outline" onClick={() => setCategoryFilter("all")}>
                Show all kinds of files
              </Button>
            )
          }
        />
      )}
      {files !== null && files.length > 0 && (
        <div
          ref={scrollElementRef}
          role="listbox"
          aria-label="Largest files"
          className="min-h-0 flex-1 overflow-y-auto"
        >
          <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const file = files[virtualRow.index];
              if (file === undefined) {
                return null;
              }
              return (
                <LargestFileRow
                  key={file.node}
                  file={file}
                  shareOfLargest={file.logicalSize / largestSize}
                  barColor={categoryColor(file.category, theme)}
                  style={{ top: virtualRow.start, height: virtualRow.size }}
                  isSelected={file.node === selectedNode}
                  actions={actions}
                  onSelect={() => selectNode(file.node)}
                  onShowInTree={() => void revealNodeInExplorer(scanId, file.node)}
                />
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
