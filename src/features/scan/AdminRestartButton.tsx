import { ShieldAlertIcon, ShieldIcon } from "lucide-react";
import { useState } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { currentPlatform } from "@/lib/platform/currentPlatform";
import { shouldOfferAdminRestart } from "./shouldOfferAdminRestart";
import { useAdminRestart } from "./useAdminRestart";
import { useIsElevated } from "./useElevationStatus";

/**
 * The top bar button that restarts the app as administrator, so the fast scan is ready without
 * first picking a drive. Hidden when the app already runs as admin or cannot restart elevated.
 */
export function AdminRestartButton() {
  const isElevated = useIsElevated();
  const adminRestart = useAdminRestart();
  const [isConfirming, setIsConfirming] = useState(false);

  if (!shouldOfferAdminRestart(currentPlatform().os, isElevated)) {
    return null;
  }

  return (
    <>
      <Button
        variant="outline"
        size="sm"
        disabled={adminRestart.isRestarting}
        onClick={() => setIsConfirming(true)}
      >
        <ShieldIcon />
        Restart as admin
      </Button>

      <AlertDialog open={isConfirming} onOpenChange={setIsConfirming}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogMedia>
              <ShieldIcon />
            </AlertDialogMedia>
            <AlertDialogTitle>Restart as administrator?</AlertDialogTitle>
            <AlertDialogDescription>
              The app closes and opens again with admin rights, which the fast scan needs. Scans
              that are still running stop. Windows asks you to allow the restart first.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={adminRestart.restartAsAdmin}>Restart</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={adminRestart.elevationProblem !== null}
        onOpenChange={(isOpen) => !isOpen && adminRestart.dismissElevationProblem()}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogMedia>
              <ShieldAlertIcon />
            </AlertDialogMedia>
            <AlertDialogTitle>{adminRestart.elevationProblem?.title}</AlertDialogTitle>
            <AlertDialogDescription>
              {adminRestart.elevationProblem?.explanation}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Close</AlertDialogCancel>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
