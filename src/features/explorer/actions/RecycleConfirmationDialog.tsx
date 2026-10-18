import { LoaderCircleIcon, Trash2Icon, Undo2Icon } from "lucide-react";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { platformWording } from "@/lib/platform/platformWording";
import { FileTypeIcon } from "../FileTypeIcon";
import { useRecycleConfirmation } from "./useRecycleConfirmation";

export function RecycleConfirmationDialog() {
  const confirmation = useRecycleConfirmation();
  const itemCount = confirmation.items?.length ?? 0;
  const { binName, restoreFromBinInstructions } = platformWording();

  return (
    <AlertDialog
      open={confirmation.isOpen}
      onOpenChange={(isOpen) => !isOpen && confirmation.cancel()}
    >
      <AlertDialogContent className="sm:max-w-lg">
        <AlertDialogHeader>
          <AlertDialogMedia className="bg-destructive/10 text-destructive">
            <Trash2Icon />
          </AlertDialogMedia>
          <AlertDialogTitle>Move to the {binName}?</AlertDialogTitle>
          <AlertDialogDescription>
            {itemCount === 1 ? "This item moves" : "These items move"} to the {binName}. Nothing is
            deleted for good, and you can restore {itemCount === 1 ? "it" : "them"} from there.
          </AlertDialogDescription>
        </AlertDialogHeader>

        <div className="grid max-h-56 gap-1 overflow-y-auto rounded-lg border bg-muted/40 p-2">
          {confirmation.items === null && confirmation.loadErrorMessage === null && (
            <Skeleton className="h-9 w-full" />
          )}
          {confirmation.loadErrorMessage !== null && (
            <p className="p-2 text-destructive text-sm">{confirmation.loadErrorMessage}</p>
          )}
          {confirmation.items?.map((item) => (
            <div key={item.node} className="flex items-center gap-3 rounded-md px-2 py-1.5">
              <FileTypeIcon kind={item.kind} category={item.category} />
              <div className="grid min-w-0 flex-1">
                <span className="truncate font-medium text-sm">{item.name}</span>
                <span className="truncate font-mono text-muted-foreground text-xs">
                  {item.fullPath ?? "No single location on disk"}
                </span>
              </div>
              <span className="shrink-0 text-sm tabular-nums">
                {formatBytesAsReadableSize(item.logicalSize)}
              </span>
            </div>
          ))}
        </div>

        <p className="flex items-center gap-2 text-muted-foreground text-xs">
          <Undo2Icon className="size-3.5" aria-hidden />
          {restoreFromBinInstructions}
        </p>

        <AlertDialogFooter>
          <AlertDialogCancel disabled={confirmation.isMoving}>Keep</AlertDialogCancel>
          <Button
            variant="destructive"
            disabled={confirmation.items === null || confirmation.isMoving}
            onClick={confirmation.confirm}
          >
            {confirmation.isMoving ? (
              <LoaderCircleIcon className="animate-spin motion-reduce:animate-none" />
            ) : (
              <Trash2Icon />
            )}
            Move {formatBytesAsReadableSize(confirmation.totalSize)} to {binName}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
