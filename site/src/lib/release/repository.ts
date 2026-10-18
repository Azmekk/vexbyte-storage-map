export const REPOSITORY = "Azmekk/vexbyte-storage-map";
export const REPOSITORY_URL = `https://github.com/${REPOSITORY}`;
export const RELEASES_URL = `${REPOSITORY_URL}/releases`;
export const LATEST_RELEASE_API_URL = `https://api.github.com/repos/${REPOSITORY}/releases/latest`;
export const ISSUES_URL = `${REPOSITORY_URL}/issues`;

// The release workflow names the installer after the version; update this if that ever changes.
export function installerFileName(version: string): string {
  return `Vexbyte.Storage.Map_${version}_x64-setup.exe`;
}

export function installerUrlForVersion(version: string): string {
  return `${REPOSITORY_URL}/releases/download/v${version}/${installerFileName(version)}`;
}
