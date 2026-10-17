import { FolderIcon } from "lucide-react";
import { motion } from "motion/react";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { layoutReorderTransition } from "@/lib/motion/transitions";
import type { TopLevelFolderBar } from "./describeScanProgress";

type TopLevelFolderListProps = {
  folders: TopLevelFolderBar[];
  isFastScan: boolean;
};

export function TopLevelFolderList({ folders, isFastScan }: TopLevelFolderListProps) {
  if (folders.length === 0) {
    return (
      <p className="text-muted-foreground text-sm">
        {isFastScan
          ? "Folder sizes appear once the file table has been read."
          : "The biggest folders show up here as they are found."}
      </p>
    );
  }

  return (
    <ol className="grid gap-1.5">
      {folders.map((folder) => (
        <motion.li
          key={folder.name}
          layout
          transition={layoutReorderTransition}
          className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1"
        >
          <span className="flex min-w-0 items-center gap-2 text-sm">
            <FolderIcon className="size-4 shrink-0 text-muted-foreground" aria-hidden />
            <span className="truncate">{folder.name}</span>
          </span>
          <span className="text-muted-foreground text-sm tabular-nums">
            {formatBytesAsReadableSize(folder.bytesFound)}
          </span>
          <div className="col-span-2 h-1.5 overflow-hidden rounded-full bg-muted">
            <motion.div
              className="h-full origin-left rounded-full bg-(--volume-used)"
              initial={false}
              animate={{ scaleX: Math.max(folder.shareOfLargest, 0.01) }}
              transition={layoutReorderTransition}
            />
          </div>
        </motion.li>
      ))}
    </ol>
  );
}
