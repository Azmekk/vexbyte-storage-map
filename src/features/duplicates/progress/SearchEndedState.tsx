import {
  CircleSlashIcon,
  RotateCwIcon,
  SlidersHorizontalIcon,
  TriangleAlertIcon,
} from "lucide-react";
import { motion } from "motion/react";
import { Button } from "@/components/ui/button";
import { fadeSlideInVariants } from "@/lib/motion/transitions";

type SearchEndedStateProps = {
  kind: "cancelled" | "failed";
  errorMessage: string | null;
  onSearchAgain: () => void;
  onReturnToSetup: () => void;
};

export function SearchEndedState({
  kind,
  errorMessage,
  onSearchAgain,
  onReturnToSetup,
}: SearchEndedStateProps) {
  const isFailure = kind === "failed";

  return (
    <motion.div
      role={isFailure ? "alert" : "status"}
      variants={fadeSlideInVariants}
      initial="hidden"
      animate="visible"
      className="flex flex-col items-start gap-4 rounded-xl border border-dashed p-6"
    >
      <div
        className="flex size-10 items-center justify-center rounded-full bg-muted data-[failure=true]:bg-destructive/10"
        data-failure={isFailure}
      >
        {isFailure ? (
          <TriangleAlertIcon className="size-5 text-destructive" />
        ) : (
          <CircleSlashIcon className="size-5 text-muted-foreground" />
        )}
      </div>
      <div className="grid gap-1">
        <h2 className="font-semibold text-lg">
          {isFailure ? "The search stopped" : "Search cancelled"}
        </h2>
        <p className="max-w-prose text-muted-foreground text-sm">
          {isFailure
            ? errorMessage
            : "No files were changed. Search again with the same settings, or adjust them first."}
        </p>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button onClick={onSearchAgain}>
          <RotateCwIcon />
          Search again
        </Button>
        <Button variant="outline" onClick={onReturnToSetup}>
          <SlidersHorizontalIcon />
          Change settings
        </Button>
      </div>
    </motion.div>
  );
}
