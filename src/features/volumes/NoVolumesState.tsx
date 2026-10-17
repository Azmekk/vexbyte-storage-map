import { HardDriveIcon, RotateCwIcon } from "lucide-react";
import { Button } from "@/components/ui/button";

type NoVolumesStateProps = {
  onRetry: () => void;
};

export function NoVolumesState({ onRetry }: NoVolumesStateProps) {
  return (
    <div className="flex flex-col items-center gap-4 rounded-xl border border-dashed px-6 py-12 text-center">
      <HardDriveIcon className="size-8 text-muted-foreground" />
      <div className="grid max-w-md gap-1">
        <h2 className="font-semibold text-lg">No drives are ready</h2>
        <p className="text-muted-foreground text-sm">
          Connect a drive or insert a disc, then look again.
        </p>
      </div>
      <Button variant="outline" onClick={onRetry}>
        <RotateCwIcon />
        Look again
      </Button>
    </div>
  );
}
