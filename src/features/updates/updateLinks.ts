/** The download page on the app's website. The opener capability allows only this site. */
export const DOWNLOAD_PAGE_URL = "https://storagemap.vexbyte.com/";

export const CHANGELOG_PAGE_URL = "https://storagemap.vexbyte.com/changelog/";

/** The changelog section of one version, for example `…/changelog/#0.2.0`. */
export function changelogUrlForVersion(version: string): string {
  return `${CHANGELOG_PAGE_URL}#${encodeURIComponent(version)}`;
}
