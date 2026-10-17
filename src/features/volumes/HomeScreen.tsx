import { motion } from "motion/react";
import { staggeredChildrenVariants } from "@/lib/motion/transitions";
import { NoVolumesState } from "./NoVolumesState";
import type { VolumesState } from "./useVolumes";
import { VolumeCard } from "./VolumeCard";
import { VolumeCardSkeleton } from "./VolumeCardSkeleton";
import { VolumesErrorState } from "./VolumesErrorState";

const SKELETON_CARD_KEYS = ["first", "second", "third"];
const VOLUME_GRID_CLASS_NAME = "grid gap-4 sm:grid-cols-2 xl:grid-cols-3";

type HomeScreenProps = {
  volumesState: VolumesState;
  onRetryLoadingVolumes: () => void;
};

export function HomeScreen({ volumesState, onRetryLoadingVolumes }: HomeScreenProps) {
  return (
    <div className="mx-auto grid w-full max-w-6xl gap-6 p-6">
      <header className="grid gap-1">
        <h1 className="font-semibold text-2xl tracking-tight">Your drives</h1>
        <p className="text-muted-foreground text-sm">Pick a drive to see where the space goes.</p>
      </header>

      {volumesState.status === "loading" && (
        <div className={VOLUME_GRID_CLASS_NAME} aria-busy>
          {SKELETON_CARD_KEYS.map((skeletonKey) => (
            <VolumeCardSkeleton key={skeletonKey} />
          ))}
        </div>
      )}

      {volumesState.status === "error" && (
        <VolumesErrorState
          errorMessage={volumesState.errorMessage}
          onRetry={onRetryLoadingVolumes}
        />
      )}

      {volumesState.status === "ready" && volumesState.volumes.length === 0 && (
        <NoVolumesState onRetry={onRetryLoadingVolumes} />
      )}

      {volumesState.status === "ready" && volumesState.volumes.length > 0 && (
        <motion.div
          className={VOLUME_GRID_CLASS_NAME}
          variants={staggeredChildrenVariants}
          initial="hidden"
          animate="visible"
        >
          {volumesState.volumes.map((volume) => (
            <VolumeCard key={volume.mountPoint} volume={volume} />
          ))}
        </motion.div>
      )}
    </div>
  );
}
