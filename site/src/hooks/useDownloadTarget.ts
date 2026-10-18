import { useEffect, useState } from "react";
import { fallbackRelease, fetchLatestRelease, type Release } from "../lib/release/latestRelease";
import { detectPlatform, type Platform, preferredLinuxPackageKind } from "../lib/release/platform";
import type { LinuxPackageKind } from "../lib/release/repository";

export type DownloadTarget = {
  platform: Platform;
  release: Release;
  preferredLinuxPackage: LinuxPackageKind;
};

export function useDownloadTarget(): DownloadTarget {
  const [target, setTarget] = useState<DownloadTarget>(() => ({
    platform: "unknown",
    release: fallbackRelease(),
    preferredLinuxPackage: preferredLinuxPackageKind(navigator.userAgent),
  }));

  useEffect(() => {
    let isCurrent = true;
    detectPlatform().then(async (platform) => {
      if (!isCurrent) return;
      setTarget((current) => ({ ...current, platform }));
      const release = await fetchLatestRelease();
      if (isCurrent) setTarget((current) => ({ ...current, platform, release }));
    });
    return () => {
      isCurrent = false;
    };
  }, []);

  return target;
}
