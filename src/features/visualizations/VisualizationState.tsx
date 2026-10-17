import { RotateCwIcon, ShapesIcon, TriangleAlertIcon } from "lucide-react";
import { EmptyState } from "@/components/EmptyState";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";

type VisualizationStateProps =
  | { kind: "loading" }
  | { kind: "error"; errorMessage: string; onRetry: () => void }
  | { kind: "empty"; description?: string };

/** Loading, error and "nothing to draw" states shared by every visualization tab. */
export function VisualizationState(props: VisualizationStateProps) {
  if (props.kind === "loading") {
    return <Skeleton className="m-3 flex-1" aria-busy />;
  }

  if (props.kind === "error") {
    return (
      <EmptyState
        className="my-auto"
        icon={TriangleAlertIcon}
        tone="warning"
        title="This view could not be loaded"
        description={props.errorMessage}
        actions={
          <Button variant="outline" onClick={props.onRetry}>
            <RotateCwIcon />
            Try again
          </Button>
        }
      />
    );
  }

  return (
    <EmptyState
      className="my-auto"
      icon={ShapesIcon}
      title="Nothing here takes up space"
      description={props.description ?? "This folder is empty or holds only empty files."}
    />
  );
}
