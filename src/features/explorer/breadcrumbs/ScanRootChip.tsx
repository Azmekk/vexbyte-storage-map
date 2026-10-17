import { FolderIcon, HardDriveIcon } from "lucide-react";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/classNames";

type ScanRootChipProps = {
  label: string;
  isVolume: boolean;
  isCurrent: boolean;
  onSelect: () => void;
};

/** The first crumb: the scanned drive or folder, always one click back to the top of the scan. */
export function ScanRootChip({ label, isVolume, isCurrent, onSelect }: ScanRootChipProps) {
  const Icon = isVolume ? HardDriveIcon : FolderIcon;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          onClick={onSelect}
          aria-current={isCurrent ? "page" : undefined}
          className={cn(
            "inline-flex h-7 max-w-40 items-center gap-1.5 rounded-md border px-2 font-medium text-sm outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring [&_svg]:size-3.5 [&_svg]:shrink-0",
            isCurrent
              ? "border-border bg-muted text-foreground"
              : "border-transparent bg-muted/50 text-foreground/80 hover:border-border hover:bg-muted hover:text-foreground",
          )}
        >
          <Icon aria-hidden />
          <span className="truncate">{label}</span>
        </button>
      </TooltipTrigger>
      <TooltipContent>
        Go to the top of {label}
        <KbdGroup>
          <Kbd>Alt</Kbd>
          <Kbd>Home</Kbd>
        </KbdGroup>
      </TooltipContent>
    </Tooltip>
  );
}
