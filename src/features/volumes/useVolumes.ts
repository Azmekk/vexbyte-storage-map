import { useCallback, useEffect, useState } from "react";
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

/** Loads and describes the mounted volumes once, with a retry for the error state. */
export function useVolumes(): UseVolumesResult {
  const [volumesState, setVolumesState] = useState<VolumesState>({ status: "loading" });

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

  useEffect(() => {
    let isMounted = true;
    void loadVolumes(() => isMounted);

    return () => {
      isMounted = false;
    };
  }, [loadVolumes]);

  const retryLoadingVolumes = useCallback(() => {
    void loadVolumes(() => true);
  }, [loadVolumes]);

  return { volumesState, retryLoadingVolumes };
}
