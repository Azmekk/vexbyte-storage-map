import { useEffect } from "react";
import { toast } from "sonner";
import { openInDefaultBrowser } from "@/lib/backend/externalLinks";
import { currentPlatform, isCurrentPlatformKnown } from "@/lib/platform/currentPlatform";
import { describeVersionChangeSinceLastRun } from "./updateCheckDecisions";
import { changelogUrlForVersion } from "./updateLinks";
import { readLastSeenAppVersion, storeLastSeenAppVersion } from "./updateStorage";

let hasHandledVersionChange = false;

/** The version this start updated to, or `null`; records the current version either way. */
function takeUpdatedVersion(): string | null {
  if (hasHandledVersionChange || !isCurrentPlatformKnown()) {
    return null;
  }
  hasHandledVersionChange = true;

  const currentVersion = currentPlatform().appVersion;
  const versionChange = describeVersionChangeSinceLastRun(readLastSeenAppVersion(), currentVersion);
  storeLastSeenAppVersion(currentVersion);

  return versionChange.kind === "updated" ? versionChange.version : null;
}

/**
 * After an update, shows a small notice with a link to the changelog once. A first install only
 * records the version. The browser opens only when the user clicks the link.
 */
export function useUpdatedNotice(): void {
  useEffect(() => {
    const updatedVersion = takeUpdatedVersion();
    if (updatedVersion === null) {
      return;
    }

    toast(`Updated to ${updatedVersion}`, {
      duration: Number.POSITIVE_INFINITY,
      closeButton: true,
      action: {
        label: "See what's new",
        onClick: () => {
          openInDefaultBrowser(changelogUrlForVersion(updatedVersion)).catch((error: unknown) => {
            console.error("Could not open the changelog in the browser.", error);
          });
        },
      },
    });
  }, []);
}
