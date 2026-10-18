import {
  assetUrl,
  installerFileName,
  LATEST_RELEASE_API_URL,
  LINUX_PACKAGE_SUFFIXES,
  type LinuxPackageKind,
  WINDOWS_INSTALLER_SUFFIX,
} from "./repository";

export type ReleaseAsset = {
  fileName: string;
  url: string;
  sizeInBytes: number | null;
};

export type LinuxPackage = ReleaseAsset & {
  kind: LinuxPackageKind;
};

export type Release = {
  version: string;
  windowsInstaller: ReleaseAsset;
  /** Empty when the release has no Linux builds, or when GitHub couldn't be reached. */
  linuxPackages: LinuxPackage[];
};

export type GitHubAsset = {
  name: string;
  size: number;
};

export type GitHubRelease = {
  // biome-ignore lint/style/useNamingConvention: GitHub API field name.
  tag_name: string;
  assets: GitHubAsset[];
};

const CACHE_KEY = "vsm-latest-release";
const REQUEST_TIMEOUT_MILLISECONDS = 4000;
const LINUX_PACKAGE_ORDER: LinuxPackageKind[] = ["deb", "rpm", "appimage"];

function constructedWindowsInstaller(version: string): ReleaseAsset {
  const fileName = installerFileName(version);
  return { fileName, url: assetUrl(version, fileName), sizeInBytes: null };
}

export function fallbackRelease(version: string = __APP_VERSION__): Release {
  return {
    version,
    windowsInstaller: constructedWindowsInstaller(version),
    linuxPackages: [],
  };
}

function findAsset(version: string, assets: GitHubAsset[], suffix: string): ReleaseAsset | null {
  const asset = assets.find((candidate) => candidate.name.endsWith(suffix));
  if (!asset) return null;
  return { fileName: asset.name, url: assetUrl(version, asset.name), sizeInBytes: asset.size };
}

/** Picks each platform's file from the release assets by suffix. */
export function releaseFromGitHub(release: GitHubRelease): Release {
  const version = release.tag_name.replace(/^v/, "");
  const linuxPackages = LINUX_PACKAGE_ORDER.flatMap((kind): LinuxPackage[] => {
    const asset = findAsset(version, release.assets, LINUX_PACKAGE_SUFFIXES[kind]);
    return asset ? [{ ...asset, kind }] : [];
  });
  return {
    version,
    windowsInstaller:
      findAsset(version, release.assets, WINDOWS_INSTALLER_SUFFIX) ??
      constructedWindowsInstaller(version),
    linuxPackages,
  };
}

function readCachedRelease(): GitHubRelease | null {
  try {
    const cached = sessionStorage.getItem(CACHE_KEY);
    return cached ? (JSON.parse(cached) as GitHubRelease) : null;
  } catch {
    return null;
  }
}

function writeCachedRelease(release: GitHubRelease): void {
  try {
    sessionStorage.setItem(CACHE_KEY, JSON.stringify(release));
  } catch {
    // Storage can be blocked or full; the next page load fetches again.
  }
}

async function requestLatestRelease(): Promise<GitHubRelease> {
  const response = await fetch(LATEST_RELEASE_API_URL, {
    signal: AbortSignal.timeout(REQUEST_TIMEOUT_MILLISECONDS),
    headers: { accept: "application/vnd.github+json" },
  });
  if (!response.ok) throw new Error(`GitHub responded with ${response.status}`);
  return (await response.json()) as GitHubRelease;
}

export async function fetchLatestRelease(): Promise<Release> {
  try {
    const gitHubRelease = readCachedRelease() ?? (await requestLatestRelease());
    writeCachedRelease(gitHubRelease);
    return releaseFromGitHub(gitHubRelease);
  } catch {
    return fallbackRelease();
  }
}
