import { useCallback, useEffect, useState } from "react";
import { openInDefaultBrowser } from "@/lib/backend/externalLinks";
import { checkForUpdate } from "@/lib/backend/updates";
import { isCurrentPlatformKnown } from "@/lib/platform/currentPlatform";
import { shouldCheckForUpdateNow, shouldOfferUpdate } from "./updateCheckDecisions";
import { changelogUrlForVersion, DOWNLOAD_PAGE_URL } from "./updateLinks";
import {
  readLastUpdateCheckAt,
  readSkippedUpdateVersion,
  storeLastUpdateCheckAt,
  storeSkippedUpdateVersion,
} from "./updateStorage";

let updateCheckRequest: Promise<string | null> | null = null;

/** The newer version to offer, or `null`. Runs at most once per app start; failures stay silent. */
function loadVersionToOffer(): Promise<string | null> {
  if (updateCheckRequest === null) {
    updateCheckRequest = findVersionToOffer().catch(() => null);
  }
  return updateCheckRequest;
}

async function findVersionToOffer(): Promise<string | null> {
  if (!isCurrentPlatformKnown() || !shouldCheckForUpdateNow(readLastUpdateCheckAt(), Date.now())) {
    return null;
  }

  const availableUpdate = await checkForUpdate();
  storeLastUpdateCheckAt(Date.now());

  if (
    availableUpdate === null ||
    !shouldOfferUpdate(availableUpdate.latestVersion, readSkippedUpdateVersion())
  ) {
    return null;
  }
  return availableUpdate.latestVersion;
}

function openPageSilently(url: string): void {
  openInDefaultBrowser(url).catch((error: unknown) => {
    console.error("Could not open the page in the browser.", error);
  });
}

export type UpdateCheck = {
  /** The version the dialog offers; `null` while there is nothing to show. */
  availableVersion: string | null;
  openDownloadPage: () => void;
  openWhatsNew: () => void;
  remindLater: () => void;
  skipThisVersion: () => void;
};

/**
 * Asks once on startup, and at most once a day, whether a newer version exists. The dialog
 * offers it unless the user skipped that version. The browser opens only on a button press.
 */
export function useUpdateCheck(): UpdateCheck {
  const [availableVersion, setAvailableVersion] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;
    void loadVersionToOffer().then((version) => {
      if (isMounted) {
        setAvailableVersion(version);
      }
    });
    return () => {
      isMounted = false;
    };
  }, []);

  const openDownloadPage = useCallback(() => {
    openPageSilently(DOWNLOAD_PAGE_URL);
    setAvailableVersion(null);
  }, []);

  const openWhatsNew = useCallback(() => {
    if (availableVersion !== null) {
      openPageSilently(changelogUrlForVersion(availableVersion));
    }
  }, [availableVersion]);

  const remindLater = useCallback(() => setAvailableVersion(null), []);

  const skipThisVersion = useCallback(() => {
    if (availableVersion !== null) {
      storeSkippedUpdateVersion(availableVersion);
    }
    setAvailableVersion(null);
  }, [availableVersion]);

  return { availableVersion, openDownloadPage, openWhatsNew, remindLater, skipThisVersion };
}
