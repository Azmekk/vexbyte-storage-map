import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigationStore } from "@/app/useNavigationStore";
import { listVolumes } from "@/lib/backend/volumes";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { describeVolume, type VolumeDescription } from "./describeVolume";

export type VolumesState =
  | { status: "loading" }
  | { status: "error"; errorMessage: string }
  | { status: "ready"; volumes: VolumeDescription[] };

export type UseVolumesResult = {
  volumesState: VolumesState;
  retryLoadingVolumes: () => void;
};

/**
 * Loads and describes the mounted volumes, with a retry for the error state. Free space changes
 * when files are cleaned up, so the list is refreshed quietly whenever the home screen is shown
 * again or the window regains focus while it is shown.
 */
export function useVolumes(): UseVolumesResult {
  const [volumesState, setVolumesState] = useState<VolumesState>({ status: "loading" });
  const isHomeShown = useNavigationStore((state) => state.currentView.kind === "home");
  const isFirstHomeVisit = useRef(true);

  const loadVolumes = useCallback(async (isResultStillWanted: () => boolean) => {
    setVolumesState({ status: "loading" });

    try {
      const volumeSummaries = await listVolumes();
      const volumes = volumeSummaries.map(describeVolume);
      if (isResultStillWanted()) {
        setVolumesState({ status: "ready", volumes });
      }
    } catch (error) {
      if (isResultStillWanted()) {
        setVolumesState({ status: "error", errorMessage: readableErrorMessage(error) });
      }
    }
  }, []);

  const refreshVolumesQuietly = useCallback(async (isResultStillWanted: () => boolean) => {
    try {
      const volumeSummaries = await listVolumes();
      const volumes = volumeSummaries.map(describeVolume);
      if (isResultStillWanted()) {
        setVolumesState({ status: "ready", volumes });
      }
    } catch {
      // Keep showing the last known volumes; the next refresh tries again.
    }
  }, []);

  useEffect(() => {
    let isMounted = true;
    void loadVolumes(() => isMounted);

    return () => {
      isMounted = false;
    };
  }, [loadVolumes]);

  useEffect(() => {
    if (!isHomeShown) {
      return;
    }

    let isMounted = true;
    const refreshWhileMounted = () => void refreshVolumesQuietly(() => isMounted);
    if (isFirstHomeVisit.current) {
      // The initial load above already covers the first time the home screen is shown.
      isFirstHomeVisit.current = false;
    } else {
      refreshWhileMounted();
    }
    window.addEventListener("focus", refreshWhileMounted);

    return () => {
      isMounted = false;
      window.removeEventListener("focus", refreshWhileMounted);
    };
  }, [isHomeShown, refreshVolumesQuietly]);

  const retryLoadingVolumes = useCallback(() => {
    void loadVolumes(() => true);
  }, [loadVolumes]);

  return { volumesState, retryLoadingVolumes };
}
