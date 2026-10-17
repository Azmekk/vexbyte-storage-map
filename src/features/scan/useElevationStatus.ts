import { useEffect, useState } from "react";
import { getElevationStatus } from "@/lib/backend/elevation";

let elevationStatusRequest: Promise<boolean> | null = null;

function loadIsElevated(): Promise<boolean> {
  if (elevationStatusRequest === null) {
    elevationStatusRequest = getElevationStatus().then(
      (status) => status.isElevated,
      () => false,
    );
  }
  return elevationStatusRequest;
}

/** Whether the app runs as administrator; `null` while unknown. Asked once per app run. */
export function useIsElevated(): boolean | null {
  const [isElevated, setIsElevated] = useState<boolean | null>(null);

  useEffect(() => {
    let isMounted = true;
    void loadIsElevated().then((elevated) => {
      if (isMounted) {
        setIsElevated(elevated);
      }
    });
    return () => {
      isMounted = false;
    };
  }, []);

  return isElevated;
}
