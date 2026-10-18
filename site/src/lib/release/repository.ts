export const REPOSITORY = "Azmekk/vexbyte-storage-map";
export const REPOSITORY_URL = `https://github.com/${REPOSITORY}`;
export const RELEASES_URL = `${REPOSITORY_URL}/releases`;
export const LATEST_RELEASE_API_URL = `https://api.github.com/repos/${REPOSITORY}/releases/latest`;
export const ISSUES_URL = `${REPOSITORY_URL}/issues`;
// The one issue where macOS interest is counted: visitors add a thumbs-up and can subscribe to hear
// when a macOS build ships. Its reaction count is the tally.
export const MAC_SUPPORT_REQUEST_URL = `${ISSUES_URL}/1`;

// GitHub turns the spaces in "Vexbyte Storage Map" into dots, so assets are matched by suffix.
export const WINDOWS_INSTALLER_SUFFIX = "_x64-setup.exe";

export type LinuxPackageKind = "deb" | "rpm" | "appimage";

export const LINUX_PACKAGE_SUFFIXES: Record<LinuxPackageKind, string> = {
  deb: "_amd64.deb",
  rpm: ".x86_64.rpm",
  appimage: "_amd64.AppImage",
};

// The release workflow names the installer after the version; update this if that ever changes.
export function installerFileName(version: string): string {
  return `Vexbyte.Storage.Map_${version}${WINDOWS_INSTALLER_SUFFIX}`;
}

export function assetUrl(version: string, fileName: string): string {
  return `${RELEASES_URL}/download/v${version}/${fileName}`;
}

export function installerUrlForVersion(version: string): string {
  return assetUrl(version, installerFileName(version));
}
