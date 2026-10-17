import { useState } from "react";
import type { FileCategory } from "@/lib/backend/generated";
import { cn } from "@/lib/classNames";
import { FileCategoryIcon } from "../shared/FileCategoryIcon";
import { useThumbnail } from "./useThumbnail";

/** Requested larger than shown so it stays sharp on high-density screens. */
const THUMBNAIL_REQUEST_PIXELS = 128;

type FileThumbnailProps = {
  path: string;
  category: FileCategory | null;
  isDimmed: boolean;
  className?: string;
};

export function FileThumbnail({ path, category, isDimmed, className }: FileThumbnailProps) {
  const thumbnail = useThumbnail(path, THUMBNAIL_REQUEST_PIXELS);
  const [failedImageUrl, setFailedImageUrl] = useState<string | null>(null);
  const canShowImage = thumbnail.status === "ready" && failedImageUrl !== thumbnail.imageUrl;

  return (
    <div
      className={cn(
        "relative flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted transition-opacity duration-200",
        isDimmed && "opacity-55 grayscale-[40%]",
        className,
      )}
    >
      {canShowImage ? (
        <img
          src={thumbnail.imageUrl}
          alt=""
          loading="lazy"
          decoding="async"
          draggable={false}
          onError={() => setFailedImageUrl(thumbnail.imageUrl)}
          className={cn(
            "size-full animate-in object-cover fade-in duration-200",
            thumbnail.isFileTypeIcon && "object-contain p-2 opacity-70",
          )}
        />
      ) : (
        <FileCategoryIcon
          category={category}
          className={cn(
            "size-5 text-muted-foreground",
            thumbnail.status === "loading" && "animate-pulse motion-reduce:animate-none",
          )}
        />
      )}
    </div>
  );
}
