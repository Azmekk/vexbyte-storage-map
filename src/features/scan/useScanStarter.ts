import { useCallback, useState } from "react";
import { relaunchElevated } from "@/lib/backend/elevation";
import type { ScanMethod, ScanTarget } from "@/lib/backend/scans";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { startScanSession } from "./scanSession";

/** Why the restart as administrator did not happen, shown in a dialog. */
export type ElevationProblem = { title: string; explanation: string };

const DEVELOPMENT_BUILD_PROBLEM: ElevationProblem = {
  title: "Restarting as admin needs the installed app",
  explanation:
    "This is a development build, which cannot restart itself as administrator. Run a standard scan, or start the built app to use the fast scan.",
};

export type ScanStarter = {
  isStarting: boolean;
  startScanWithMethod: (method: ScanMethod) => void;
  restartAsAdminForFastScan: () => void;
  elevationProblem: ElevationProblem | null;
  dismissElevationProblem: () => void;
};

/** Starts scans of one target and handles the restart-as-admin offer. */
export function useScanStarter(target: ScanTarget): ScanStarter {
  const [isStarting, setIsStarting] = useState(false);
  const [elevationProblem, setElevationProblem] = useState<ElevationProblem | null>(null);

  const startScanWithMethod = useCallback(
    (method: ScanMethod) => {
      setIsStarting(true);
      void startScanSession(target, method).finally(() => setIsStarting(false));
    },
    [target],
  );

  const restartAsAdminForFastScan = useCallback(() => {
    if (import.meta.env.DEV) {
      setElevationProblem(DEVELOPMENT_BUILD_PROBLEM);
      return;
    }

    void relaunchElevated(target).catch((error: unknown) => {
      setElevationProblem({
        title: "The app is still running without admin rights",
        explanation: `${readableErrorMessage(error)} You can run a standard scan instead.`,
      });
    });
  }, [target]);

  const dismissElevationProblem = useCallback(() => setElevationProblem(null), []);

  return {
    isStarting,
    startScanWithMethod,
    restartAsAdminForFastScan,
    elevationProblem,
    dismissElevationProblem,
  };
}
