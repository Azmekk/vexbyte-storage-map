import {
  FileArchiveIcon,
  FileAudioIcon,
  FileIcon,
  FileImageIcon,
  FileSpreadsheetIcon,
  FileTextIcon,
  FileVideoCameraIcon,
  FolderIcon,
  FolderLockIcon,
  FolderOpenIcon,
  FolderSymlinkIcon,
  LayersIcon,
  type LucideIcon,
  FileTypeIcon as PdfFileIcon,
  PresentationIcon,
} from "lucide-react";
import type { FileCategory, NodeKind } from "@/lib/backend/tree";
import { cn } from "@/lib/classNames";

const ICON_BY_CATEGORY: Record<FileCategory, LucideIcon> = {
  videos: FileVideoCameraIcon,
  images: FileImageIcon,
  audio: FileAudioIcon,
  pdfs: PdfFileIcon,
  documents: FileTextIcon,
  spreadsheets: FileSpreadsheetIcon,
  presentations: PresentationIcon,
  archivesAndInstallers: FileArchiveIcon,
};

type FileTypeIconProps = {
  kind: NodeKind | "smallerItems";
  category: FileCategory | null;
  isExpanded?: boolean;
  isInaccessible?: boolean;
  isLink?: boolean;
  className?: string;
};

function chooseIcon({
  kind,
  category,
  isExpanded,
  isInaccessible,
  isLink,
}: FileTypeIconProps): LucideIcon {
  if (kind === "smallerItems") {
    return LayersIcon;
  }
  if (kind === "directory") {
    if (isInaccessible === true) {
      return FolderLockIcon;
    }
    if (isLink === true) {
      return FolderSymlinkIcon;
    }
    return isExpanded === true ? FolderOpenIcon : FolderIcon;
  }
  return category === null ? FileIcon : ICON_BY_CATEGORY[category];
}

export function FileTypeIcon(props: FileTypeIconProps) {
  const Icon = chooseIcon(props);
  return (
    <Icon
      aria-hidden
      className={cn(
        "size-4 shrink-0",
        props.kind === "directory" ? "text-foreground/70" : "text-muted-foreground",
        props.className,
      )}
    />
  );
}
