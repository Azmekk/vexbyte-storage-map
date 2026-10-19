import { CircleArrowDownIcon, DownloadIcon } from "lucide-react";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { useUpdateCheck } from "./useUpdateCheck";

export function UpdateAvailableDialog() {
  const updateCheck = useUpdateCheck();

  return (
    <AlertDialog
      open={updateCheck.availableVersion !== null}
      onOpenChange={(isOpen) => !isOpen && updateCheck.remindLater()}
    >
      <AlertDialogContent className="sm:max-w-lg">
        <AlertDialogHeader>
          <AlertDialogMedia>
            <CircleArrowDownIcon />
          </AlertDialogMedia>
          <AlertDialogTitle>A new version is available</AlertDialogTitle>
          <AlertDialogDescription>
            Version {updateCheck.availableVersion} is available.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-between">
          <div className="flex flex-col-reverse gap-2 sm:flex-row">
            <Button variant="ghost" onClick={updateCheck.skipThisVersion}>
              Skip this version
            </Button>
            <Button variant="ghost" onClick={updateCheck.remindLater}>
              Later
            </Button>
          </div>
          <div className="flex flex-col-reverse gap-2 sm:flex-row">
            <Button variant="outline" onClick={updateCheck.openWhatsNew}>
              What's new
            </Button>
            <Button onClick={updateCheck.openDownloadPage}>
              <DownloadIcon />
              Download
            </Button>
          </div>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
