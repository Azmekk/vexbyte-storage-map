import { useCallback, useState } from "react";
import { relaunchAsAdministrator } from "@/lib/backend/elevation";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import type { ElevationProblem } from "./useScanStarter";

const DEVELOPMENT_BUILD_PROBLEM: ElevationProblem = {
  title: "Restarting as admin needs the installed app",
  explanation:
    "This is a development build, which cannot restart itself as administrator. Start the built app instead.",
};

export type AdminRestart = {
  isRestarting: boolean;
  restartAsAdmin: () => void;
  elevationProblem: ElevationProblem | null;
  dismissElevationProblem: () => void;
};

/** Restarts the app as administrator without a scan, and explains when that did not happen. */
export function useAdminRestart(): AdminRestart {
  const [isRestarting, setIsRestarting] = useState(false);
  const [elevationProblem, setElevationProblem] = useState<ElevationProblem | null>(null);

  const restartAsAdmin = useCallback(() => {
    if (import.meta.env.DEV) {
      setElevationProblem(DEVELOPMENT_BUILD_PROBLEM);
      return;
    }

    setIsRestarting(true);
    void relaunchAsAdministrator()
      .catch((error: unknown) => {
        setElevationProblem({
          title: "The app is still running without admin rights",
          explanation: readableErrorMessage(error),
        });
      })
      .finally(() => setIsRestarting(false));
  }, []);

  const dismissElevationProblem = useCallback(() => setElevationProblem(null), []);

  return { isRestarting, restartAsAdmin, elevationProblem, dismissElevationProblem };
}
