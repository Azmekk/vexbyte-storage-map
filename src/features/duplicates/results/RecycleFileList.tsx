import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef } from "react";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { platformWording } from "@/lib/platform/platformWording";
import { splitFilePath } from "../paths/splitFilePath";
import { truncatePathMiddle } from "../paths/truncatePathMiddle";
import { FileCategoryIcon } from "../shared/FileCategoryIcon";
import type { FileToRemove } from "./selectionSummary";

const ROW_HEIGHT_PIXELS = 44;
const FOLDER_PATH_DISPLAY_LENGTH = 40;

type RecycleFileListProps = {
  filesToRemove: readonly FileToRemove[];
};

/** The files about to be moved, virtualized since a selection can hold thousands. */
export function RecycleFileList({ filesToRemove }: RecycleFileListProps) {
  const scrollElementRef = useRef<HTMLDivElement | null>(null);
  const virtualizer = useVirtualizer({
    count: filesToRemove.length,
    getScrollElement: () => scrollElementRef.current,
    estimateSize: () => ROW_HEIGHT_PIXELS,
    overscan: 8,
  });

  return (
    <div ref={scrollElementRef} className="max-h-72 overflow-y-auto rounded-lg border bg-muted/20">
      <ul
        className="relative w-full"
        style={{ height: virtualizer.getTotalSize() }}
        aria-label={`Files to move to the ${platformWording().binName}`}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => {
          const file = filesToRemove[virtualItem.index];
          if (file === undefined) {
            return null;
          }
          const { fileName, folderPath } = splitFilePath(file.path);

          return (
            <li
              key={virtualItem.key}
              className="absolute top-0 left-0 flex w-full items-center gap-3 border-b px-3 last:border-b-0"
              style={{ height: ROW_HEIGHT_PIXELS, transform: `translateY(${virtualItem.start}px)` }}
              title={file.path}
            >
              <FileCategoryIcon
                category={file.category}
                className="size-4 shrink-0 text-muted-foreground"
              />
              <span className="grid min-w-0 flex-1">
                <span className="truncate text-sm">{fileName}</span>
                <span className="truncate text-muted-foreground text-xs">
                  {truncatePathMiddle(folderPath, FOLDER_PATH_DISPLAY_LENGTH)}
                </span>
              </span>
              <span className="shrink-0 text-muted-foreground text-xs tabular-nums">
                {formatBytesAsReadableSize(file.fileSize)}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
