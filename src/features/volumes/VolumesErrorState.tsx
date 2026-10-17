import { RotateCwIcon, TriangleAlertIcon } from "lucide-react";
import { Button } from "@/components/ui/button";

type VolumesErrorStateProps = {
  errorMessage: string;
  onRetry: () => void;
};

export function VolumesErrorState({ errorMessage, onRetry }: VolumesErrorStateProps) {
  return (
    <div
      role="alert"
      className="flex flex-col items-center gap-4 rounded-xl border border-dashed px-6 py-12 text-center"
    >
      <div className="flex size-12 items-center justify-center rounded-full bg-destructive/10">
        <TriangleAlertIcon className="size-6 text-destructive" />
      </div>
      <div className="grid max-w-md gap-1">
        <h2 className="font-semibold text-lg">We couldn't read your drives</h2>
        <p className="text-muted-foreground text-sm">{errorMessage}</p>
      </div>
      <Button variant="outline" onClick={onRetry}>
        <RotateCwIcon />
        Try again
      </Button>
    </div>
  );
}
