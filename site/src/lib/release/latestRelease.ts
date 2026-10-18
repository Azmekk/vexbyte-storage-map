import { installerFileName, installerUrlForVersion, LATEST_RELEASE_API_URL } from "./repository";

export type Release = {
  version: string;
  installerUrl: string;
  sizeInBytes: number | null;
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

export function fallbackRelease(version: string = __APP_VERSION__): Release {
  return {
    version,
    installerUrl: installerUrlForVersion(version),
    sizeInBytes: null,
  };
}

/** Builds the direct installer link for the latest tag; the asset list only supplies its size. */
export function releaseFromGitHub(release: GitHubRelease): Release {
  const version = release.tag_name.replace(/^v/, "");
  const asset = release.assets.find((candidate) => candidate.name === installerFileName(version));
  return {
    version,
    installerUrl: installerUrlForVersion(version),
    sizeInBytes: asset?.size ?? null,
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
