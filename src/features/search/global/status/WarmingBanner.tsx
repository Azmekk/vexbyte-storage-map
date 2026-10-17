import { motion } from "motion/react";
import type { VolumeIndexStatus } from "@/lib/backend/search";
import { fadeSlideInVariants } from "@/lib/motion/transitions";
import { describeStillWarmingVolumes, describeWarmingBannerTitle } from "./describeWarmingBanner";
import { WarmingVolumeRow } from "./WarmingVolumeRow";

type WarmingBannerProps = {
  warmingVolumes: readonly VolumeIndexStatus[];
  /** Drives the last search could not look at yet. */
  skippedVolumesStillWarming: readonly string[];
};

export function WarmingBanner({ warmingVolumes, skippedVolumesStillWarming }: WarmingBannerProps) {
  const stillWarmingNote = describeStillWarmingVolumes(skippedVolumesStillWarming);

  return (
    <motion.section
      variants={fadeSlideInVariants}
      initial="hidden"
      animate="visible"
      exit="exit"
      aria-label="Search engine warm-up"
      className="grid gap-2 rounded-xl border border-(--volume-used)/30 bg-(--volume-used)/5 px-4 py-3"
    >
      <header className="flex flex-wrap items-center gap-x-3 gap-y-1">
        <span className="relative flex size-2" aria-hidden>
          <span className="absolute inline-flex size-full animate-ping rounded-full bg-(--volume-used) opacity-60 motion-reduce:animate-none" />
          <span className="relative inline-flex size-2 rounded-full bg-(--volume-used)" />
        </span>
        <h2 className="font-medium text-sm">{describeWarmingBannerTitle(warmingVolumes.length)}</h2>
        {stillWarmingNote !== null && (
          <p className="text-muted-foreground text-sm" aria-live="polite">
            {stillWarmingNote}
          </p>
        )}
      </header>
      {warmingVolumes.length > 0 && (
        <ul className="divide-y divide-(--volume-used)/15">
          {warmingVolumes.map((volume) => (
            <WarmingVolumeRow key={volume.mountPoint} volume={volume} />
          ))}
        </ul>
      )}
    </motion.section>
  );
}
