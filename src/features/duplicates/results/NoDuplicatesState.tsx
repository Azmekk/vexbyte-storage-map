import { CircleCheckIcon, RotateCwIcon, SlidersHorizontalIcon } from "lucide-react";
import { motion } from "motion/react";
import { Button } from "@/components/ui/button";
import { fadeSlideInVariants } from "@/lib/motion/transitions";

type NoDuplicatesStateProps = {
  title: string;
  details: readonly string[];
  onSearchAgain: () => void;
  onChangeSettings: () => void;
};

export function NoDuplicatesState({
  title,
  details,
  onSearchAgain,
  onChangeSettings,
}: NoDuplicatesStateProps) {
  return (
    <motion.div
      variants={fadeSlideInVariants}
      initial="hidden"
      animate="visible"
      className="flex flex-col items-center gap-4 rounded-xl border border-dashed px-6 py-14 text-center"
    >
      <div className="flex size-12 items-center justify-center rounded-full bg-(--volume-used)/10">
        <CircleCheckIcon className="size-6 text-(--volume-used)" />
      </div>
      <div className="grid max-w-md gap-1.5">
        <h2 className="font-semibold text-lg">{title}</h2>
        {details.map((detail) => (
          <p key={detail} className="text-muted-foreground text-sm">
            {detail}
          </p>
        ))}
      </div>
      <div className="flex flex-wrap justify-center gap-2">
        <Button variant="outline" onClick={onChangeSettings}>
          <SlidersHorizontalIcon />
          Search other folders
        </Button>
        <Button variant="ghost" onClick={onSearchAgain}>
          <RotateCwIcon />
          Search again
        </Button>
      </div>
    </motion.div>
  );
}
