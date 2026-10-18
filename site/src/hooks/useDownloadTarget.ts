import { useEffect, useState } from "react";
import { fallbackRelease, fetchLatestRelease, type Release } from "../lib/release/latestRelease";
import { detectPlatform, type Platform } from "../lib/release/platform";

export type DownloadTarget = {
  platform: Platform;
  release: Release;
};

export function useDownloadTarget(): DownloadTarget {
  const [target, setTarget] = useState<DownloadTarget>(() => ({
    platform: "unknown",
    release: fallbackRelease(),
  }));

  useEffect(() => {
    let isCurrent = true;
    detectPlatform().then(async (platform) => {
      if (!isCurrent) return;
      setTarget((current) => ({ ...current, platform }));
      const release = await fetchLatestRelease();
      if (isCurrent) setTarget({ platform, release });
    });
    return () => {
      isCurrent = false;
    };
  }, []);

  return target;
}
