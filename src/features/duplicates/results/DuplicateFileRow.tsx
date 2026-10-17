import { motion } from "motion/react";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { exitTransition } from "@/lib/motion/transitions";
import { FileThumbnail } from "../thumbnails/FileThumbnail";
import type { DuplicateFileDescription } from "./describeDuplicateGroup";
import { FileRowMenu } from "./FileRowMenu";
import { KeepRemoveToggle } from "./KeepRemoveToggle";
import { RecycleFileButton } from "./RecycleFileButton";

type DuplicateFileRowProps = {
  file: DuplicateFileDescription;
  isSaving: boolean;
  canRecycle: boolean;
  onToggleKeep: (path: string) => void;
  onReveal: (path: string) => void;
  onOpen: (path: string) => void;
  onRecycle: (path: string) => void;
};

/**
 * One copy in a group card. Pressing Delete while the row or one of its controls has focus opens
 * the confirmation for moving this copy to the Recycle Bin.
 */
export function DuplicateFileRow({
  file,
  isSaving,
  canRecycle,
  onToggleKeep,
  onReveal,
  onOpen,
  onRecycle,
}: DuplicateFileRowProps) {
  return (
    <motion.li
      className="group/file-row overflow-hidden outline-none focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:ring-inset"
      exit={{ opacity: 0, height: 0, transition: exitTransition }}
      tabIndex={-1}
      aria-keyshortcuts="Delete"
      onKeyDown={(event) => {
        if (event.key === "Delete" && canRecycle) {
          event.preventDefault();
          onRecycle(file.path);
        }
      }}
    >
      <div
        className="flex h-16 items-center gap-3 px-3 transition-colors duration-150 data-[keep=false]:bg-muted/50"
        data-keep={file.keep}
      >
        <FileThumbnail path={file.path} category={file.category} isDimmed={!file.keep} />

        <div className="grid min-w-0 flex-1 gap-0.5">
          <span
            className="truncate font-medium text-sm data-[keep=false]:text-muted-foreground"
            data-keep={file.keep}
          >
            {file.fileName}
          </span>
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="w-fit max-w-full truncate text-muted-foreground text-xs">
                {file.shortFolderPath}
              </span>
            </TooltipTrigger>
            <TooltipContent className="max-w-[min(90vw,40rem)] break-all">
              {file.path}
            </TooltipContent>
          </Tooltip>
        </div>

        <Tooltip>
          <TooltipTrigger asChild>
            <span className="hidden w-28 shrink-0 text-right text-muted-foreground text-xs tabular-nums md:block">
              {file.modifiedRelativeText}
            </span>
          </TooltipTrigger>
          <TooltipContent>Modified {file.modifiedAbsoluteText}</TooltipContent>
        </Tooltip>

        <KeepRemoveToggle
          fileName={file.fileName}
          keep={file.keep}
          isSaving={isSaving}
          onToggle={() => onToggleKeep(file.path)}
        />
        <RecycleFileButton
          fileName={file.fileName}
          canRecycle={canRecycle}
          onRecycle={() => onRecycle(file.path)}
        />
        <FileRowMenu
          fileName={file.fileName}
          canRecycle={canRecycle}
          onReveal={() => onReveal(file.path)}
          onOpen={() => onOpen(file.path)}
          onRecycle={() => onRecycle(file.path)}
        />
      </div>
    </motion.li>
  );
}
