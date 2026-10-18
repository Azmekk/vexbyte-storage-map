import { openUrl } from "@tauri-apps/plugin-opener";

/**
 * Opens a web page in the default browser. The capability only allows pages on the app's own
 * website, so any other URL is refused.
 */
export function openInDefaultBrowser(url: string): Promise<void> {
  return openUrl(url);
}
