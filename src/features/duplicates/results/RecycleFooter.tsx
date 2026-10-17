import { LoaderCircleIcon, Trash2Icon } from "lucide-react";
import { Button } from "@/components/ui/button";

type RecycleFooterProps = {
  buttonLabel: string;
  canRecycle: boolean;
  isPreparing: boolean;
  onRecycle: () => void;
};

export function RecycleFooter({
  buttonLabel,
  canRecycle,
  isPreparing,
  onRecycle,
}: RecycleFooterProps) {
  return (
    <footer className="border-t bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/80">
      <div className="mx-auto flex w-full max-w-6xl flex-wrap items-center justify-between gap-3 px-6 py-3.5">
        <p className="text-muted-foreground text-sm">
          {canRecycle
            ? "Kept copies stay where they are. Moved files can be restored from the Recycle Bin."
            : "Mark copies as Remove to free space."}
        </p>
        <Button
          size="lg"
          className="h-10 bg-destructive px-4 text-white hover:bg-destructive/90 dark:bg-destructive/90"
          disabled={!canRecycle || isPreparing}
          onClick={onRecycle}
        >
          {isPreparing ? <LoaderCircleIcon className="animate-spin" /> : <Trash2Icon />}
          {buttonLabel}
        </Button>
      </div>
    </footer>
  );
}
