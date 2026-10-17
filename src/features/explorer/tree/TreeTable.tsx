import { useVirtualizer } from "@tanstack/react-virtual";
import { ArrowUpIcon, CloudIcon, FolderOpenIcon, ShieldAlertIcon } from "lucide-react";
import { useEffect, useRef } from "react";
import { EmptyState } from "@/components/EmptyState";
import { Button } from "@/components/ui/button";
import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex } from "@/lib/backend/tree";
import { useNodeActions } from "../actions/useNodeActions";
import { useExplorerSelectionStore } from "../useExplorerSelectionStore";
import { TreeItemRow } from "./TreeItemRow";
import { TreeStatusRow } from "./TreeStatusRow";
import { TreeTableHeader } from "./TreeTableHeader";
import { TREE_ROW_HEIGHT } from "./treeTableLayout";
import { MAXIMUM_ANIMATED_REVEALED_ROWS, useTreeTable } from "./useTreeTable";

type TreeTableProps = {
  scanId: ScanId;
  shownFolder: NodeIndex;
  parentOfShownFolder: NodeIndex | null;
};

export function TreeTable({ scanId, shownFolder, parentOfShownFolder }: TreeTableProps) {
  const model = useTreeTable(scanId, shownFolder, parentOfShownFolder);
  const actions = useNodeActions(scanId);
  const sortOrder = useExplorerSelectionStore((state) => state.sortOrder);
  const setSortOrder = useExplorerSelectionStore((state) => state.setSortOrder);
  const setOnlineOnlyVisibility = useExplorerSelectionStore(
    (state) => state.setOnlineOnlyVisibility,
  );
  const scrollElementRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: model.rows.length,
    getScrollElement: () => scrollElementRef.current,
    estimateSize: () => TREE_ROW_HEIGHT,
    overscan: 12,
    getItemKey: (rowIndex) => model.rows[rowIndex]?.key ?? rowIndex,
  });

  const { focusedRowIndex } = model;
  useEffect(() => {
    if (focusedRowIndex >= 0) {
      virtualizer.scrollToIndex(focusedRowIndex, { align: "auto" });
    }
  }, [focusedRowIndex, virtualizer]);

  const firstRevealedRowIndex =
    model.lastExpandedFolder === null
      ? -1
      : model.rows.findIndex((row) => row.parent === model.lastExpandedFolder);

  return (
    <div className="@container flex h-full min-h-0 flex-col">
      <TreeTableHeader sortOrder={sortOrder} onSortOrderChange={setSortOrder} />
      {model.emptyFolder !== null ? (
        <EmptyState
          icon={
            model.emptyFolder.isWarning
              ? ShieldAlertIcon
              : model.emptyFolder.hasHiddenOnlineOnlyFiles
                ? CloudIcon
                : FolderOpenIcon
          }
          tone={model.emptyFolder.isWarning ? "warning" : "neutral"}
          title={model.emptyFolder.title}
          description={model.emptyFolder.description}
          actions={
            <>
              {model.emptyFolder.hasHiddenOnlineOnlyFiles && (
                <Button variant="secondary" onClick={() => setOnlineOnlyVisibility("shown")}>
                  <CloudIcon />
                  Show online-only files
                </Button>
              )}
              {model.goUp !== null && (
                <Button variant="outline" onClick={model.goUp}>
                  <ArrowUpIcon />
                  Go up
                </Button>
              )}
            </>
          }
        />
      ) : (
        <div
          ref={scrollElementRef}
          role="tree"
          aria-label="Folder contents"
          aria-activedescendant={
            model.selectedNode === null ? undefined : `tree-row-${model.selectedNode}`
          }
          tabIndex={0}
          onKeyDown={model.handleKeyDown}
          className="min-h-0 flex-1 overflow-y-auto outline-none focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:ring-inset"
        >
          <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const row = model.rows[virtualRow.index];
              if (row === undefined) {
                return null;
              }
              const style = { top: virtualRow.start, height: virtualRow.size };

              if (row.kind !== "item") {
                return (
                  <TreeStatusRow
                    key={row.key}
                    row={row}
                    style={style}
                    onShowMore={() => model.showMoreChildren(row.parent)}
                    onRetry={() => model.retryFolder(row.parent)}
                  />
                );
              }

              const animatesIn =
                row.parent === model.lastExpandedFolder &&
                firstRevealedRowIndex >= 0 &&
                virtualRow.index - firstRevealedRowIndex < MAXIMUM_ANIMATED_REVEALED_ROWS;

              return (
                <TreeItemRow
                  key={row.key}
                  row={row}
                  style={style}
                  isSelected={virtualRow.index === focusedRowIndex}
                  animatesIn={animatesIn}
                  actions={actions}
                  onSelect={() => {
                    model.selectRow(virtualRow.index);
                    scrollElementRef.current?.focus({ preventScroll: true });
                  }}
                  onToggle={() => model.toggleFolder(row.item.node)}
                  onOpenFolder={() => model.openFolder(row.item.node)}
                />
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
