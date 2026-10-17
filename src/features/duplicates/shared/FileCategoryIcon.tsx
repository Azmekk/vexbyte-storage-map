import {
  FileArchiveIcon,
  FileIcon,
  FileSpreadsheetIcon,
  FileTextIcon,
  FilmIcon,
  ImageIcon,
  type LucideIcon,
  MusicIcon,
  NotebookTextIcon,
  PresentationIcon,
} from "lucide-react";
import type { FileCategory } from "@/lib/backend/generated";

const CATEGORY_ICONS: Record<FileCategory, LucideIcon> = {
  images: ImageIcon,
  videos: FilmIcon,
  audio: MusicIcon,
  pdfs: FileTextIcon,
  documents: NotebookTextIcon,
  spreadsheets: FileSpreadsheetIcon,
  presentations: PresentationIcon,
  archivesAndInstallers: FileArchiveIcon,
};

type FileCategoryIconProps = {
  category: FileCategory | null;
  className?: string;
};

export function FileCategoryIcon({ category, className }: FileCategoryIconProps) {
  const Icon = category === null ? FileIcon : CATEGORY_ICONS[category];
  return <Icon className={className} aria-hidden />;
}
