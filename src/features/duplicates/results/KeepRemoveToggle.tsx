import { ArchiveRestoreIcon, CheckIcon } from "lucide-react";

type KeepRemoveToggleProps = {
  fileName: string;
  keep: boolean;
  isSaving: boolean;
  onToggle: () => void;
};

/** A two-way switch between keeping a copy and marking it for the Recycle Bin. */
export function KeepRemoveToggle({ fileName, keep, isSaving, onToggle }: KeepRemoveToggleProps) {
  const segmentClassName =
    "flex h-7 items-center gap-1 rounded-md px-2.5 font-medium text-xs outline-none transition-colors duration-150 focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-default";

  return (
    <fieldset
      className="flex shrink-0 rounded-lg border bg-background p-0.5 data-[saving=true]:opacity-60"
      data-saving={isSaving}
      aria-label={`What to do with ${fileName}`}
    >
      <button
        type="button"
        aria-pressed={keep}
        disabled={isSaving || keep}
        onClick={onToggle}
        className={`${segmentClassName} text-muted-foreground hover:text-foreground aria-pressed:bg-(--volume-used) aria-pressed:text-white`}
      >
        <CheckIcon className="size-3.5" aria-hidden />
        Keep
      </button>
      <button
        type="button"
        aria-pressed={!keep}
        disabled={isSaving || !keep}
        onClick={onToggle}
        className={`${segmentClassName} text-muted-foreground hover:text-foreground aria-pressed:bg-destructive/15 aria-pressed:text-destructive`}
      >
        <ArchiveRestoreIcon className="size-3.5" aria-hidden />
        Remove
      </button>
    </fieldset>
  );
}
