import { ShieldAlertIcon } from "lucide-react";
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
import type { ElevationProblem } from "./useScanStarter";

type ElevationProblemDialogProps = {
  problem: ElevationProblem | null;
  onDismiss: () => void;
  onStartStandardScan: () => void;
};

export function ElevationProblemDialog({
  problem,
  onDismiss,
  onStartStandardScan,
}: ElevationProblemDialogProps) {
  return (
    <AlertDialog open={problem !== null} onOpenChange={(isOpen) => !isOpen && onDismiss()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogMedia>
            <ShieldAlertIcon />
          </AlertDialogMedia>
          <AlertDialogTitle>{problem?.title}</AlertDialogTitle>
          <AlertDialogDescription>{problem?.explanation}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Close</AlertDialogCancel>
          <AlertDialogAction
            onClick={() => {
              onDismiss();
              onStartStandardScan();
            }}
          >
            Run standard scan
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
