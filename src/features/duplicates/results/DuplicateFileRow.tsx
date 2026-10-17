import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { FileThumbnail } from "../thumbnails/FileThumbnail";
import type { DuplicateFileDescription } from "./describeDuplicateGroup";
import { FileRowMenu } from "./FileRowMenu";
import { KeepRemoveToggle } from "./KeepRemoveToggle";

type DuplicateFileRowProps = {
  file: DuplicateFileDescription;
  isSaving: boolean;
  onToggleKeep: (path: string) => void;
  onReveal: (path: string) => void;
  onOpen: (path: string) => void;
};

export function DuplicateFileRow({
  file,
  isSaving,
  onToggleKeep,
  onReveal,
  onOpen,
}: DuplicateFileRowProps) {
  return (
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
          <TooltipContent className="max-w-[min(90vw,40rem)] break-all">{file.path}</TooltipContent>
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
      <FileRowMenu
        fileName={file.fileName}
        onReveal={() => onReveal(file.path)}
        onOpen={() => onOpen(file.path)}
      />
    </div>
  );
}
