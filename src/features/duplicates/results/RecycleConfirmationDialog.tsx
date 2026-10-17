import { ArchiveRestoreIcon, LoaderCircleIcon, Trash2Icon, TriangleAlertIcon } from "lucide-react";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { describeRecycleScope } from "./describeRecycleScope";
import { RecycleFileList } from "./RecycleFileList";
import type { RecycleConfirmationState } from "./useRecycleDuplicates";

type RecycleConfirmationDialogProps = {
  confirmation: RecycleConfirmationState;
  onClose: () => void;
  onConfirm: () => void;
};

export function RecycleConfirmationDialog({
  confirmation,
  onClose,
  onConfirm,
}: RecycleConfirmationDialogProps) {
  const isOpen = confirmation.status !== "closed";
  const hasFiles = confirmation.status === "ready" || confirmation.status === "recycling";
  const isRecycling = confirmation.status === "recycling";
  const fileCount = hasFiles ? confirmation.filesToRemove.length : 0;
  const scopeText =
    confirmation.status === "closed"
      ? null
      : describeRecycleScope(
          confirmation.scope,
          hasFiles
            ? { fileCount, readableSize: formatBytesAsReadableSize(confirmation.totalBytes) }
            : null,
        );

  return (
    <AlertDialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
    >
      <AlertDialogContent className="sm:max-w-xl">
        <AlertDialogHeader>
          <AlertDialogTitle>{scopeText?.title}</AlertDialogTitle>
          <AlertDialogDescription>{scopeText?.description}</AlertDialogDescription>
        </AlertDialogHeader>

        {confirmation.status === "loading" && (
          <div className="grid gap-2" aria-busy>
            <Skeleton className="h-11 w-full" />
            <Skeleton className="h-11 w-full" />
            <Skeleton className="h-11 w-full" />
          </div>
        )}

        {confirmation.status === "error" && (
          <p role="alert" className="flex items-start gap-2 text-destructive text-sm">
            <TriangleAlertIcon className="mt-0.5 size-4 shrink-0" aria-hidden />
            The list of copies couldn't be loaded. {confirmation.errorMessage}
          </p>
        )}

        {hasFiles && <RecycleFileList filesToRemove={confirmation.filesToRemove} />}

        {hasFiles && scopeText?.note && (
          <p className="text-muted-foreground text-sm">{scopeText.note}</p>
        )}

        {hasFiles && (
          <p className="flex items-center gap-2 text-muted-foreground text-xs">
            <ArchiveRestoreIcon className="size-3.5 shrink-0" aria-hidden />
            Files whose size or date changed since the search are skipped and listed afterwards.
          </p>
        )}

        <AlertDialogFooter>
          <AlertDialogCancel disabled={isRecycling}>Cancel</AlertDialogCancel>
          <Button
            className="bg-destructive text-white hover:bg-destructive/90 dark:bg-destructive/90"
            disabled={confirmation.status !== "ready" || fileCount === 0}
            onClick={onConfirm}
          >
            {isRecycling ? <LoaderCircleIcon className="animate-spin" /> : <Trash2Icon />}
            {isRecycling ? "Moving…" : scopeText?.confirmLabel}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
