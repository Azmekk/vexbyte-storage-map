import { useEffect, useState } from "react";
import { thumbnailImageUrl } from "@/lib/backend/fileActions";
import { cachedThumbnail, requestThumbnail, type ThumbnailResult } from "./thumbnailRequestQueue";

export type ThumbnailState =
  | { status: "loading" }
  | { status: "failed" }
  | { status: "ready"; imageUrl: string; isFileTypeIcon: boolean };

function thumbnailStateOf(result: ThumbnailResult | undefined): ThumbnailState {
  if (result === undefined) {
    return { status: "loading" };
  }

  if (result.kind === "failed") {
    return { status: "failed" };
  }

  return {
    status: "ready",
    imageUrl: thumbnailImageUrl(result.thumbnail),
    isFileTypeIcon: result.thumbnail.isFileTypeIcon,
  };
}

/**
 * The thumbnail of `path`, requested while the component is mounted. Only rows the virtualized
 * list renders (in or near the viewport) mount, so off-screen files are never requested.
 */
export function useThumbnail(path: string, sizePixels: number): ThumbnailState {
  const [result, setResult] = useState<{ key: string; state: ThumbnailState }>(() => ({
    key: `${sizePixels}|${path}`,
    state: thumbnailStateOf(cachedThumbnail(path, sizePixels)),
  }));
  const key = `${sizePixels}|${path}`;

  useEffect(() => {
    const cancelRequest = requestThumbnail(path, sizePixels, (thumbnailResult) => {
      setResult({ key: `${sizePixels}|${path}`, state: thumbnailStateOf(thumbnailResult) });
    });

    return cancelRequest;
  }, [path, sizePixels]);

  if (result.key !== key) {
    return thumbnailStateOf(cachedThumbnail(path, sizePixels));
  }

  return result.state;
}
