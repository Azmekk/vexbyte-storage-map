import { AnimatePresence, motion } from "motion/react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { DuplicatesScreen } from "@/features/duplicates/DuplicatesScreen";
import { ScanScreen } from "@/features/scan/ScanScreen";
import { HomeScreen } from "@/features/volumes/HomeScreen";
import type { VolumesState } from "@/features/volumes/useVolumes";
import { viewFadeVariants } from "@/lib/motion/transitions";
import { TopBar } from "./TopBar";
import { TopBarTitle } from "./TopBarTitle";
import { useNavigationStore } from "./useNavigationStore";

type CurrentViewScreenProps = {
  volumesState: VolumesState;
  onRetryLoadingVolumes: () => void;
};

export function CurrentViewScreen({ volumesState, onRetryLoadingVolumes }: CurrentViewScreenProps) {
  const currentView = useNavigationStore((state) => state.currentView);
  const viewKey = currentView.kind === "scan" ? `scan-${currentView.scanId}` : currentView.kind;

  return (
    <AnimatePresence mode="wait" initial={false}>
      <motion.div
        key={viewKey}
        className="flex min-h-0 flex-1 flex-col"
        variants={viewFadeVariants}
        initial="hidden"
        animate="visible"
        exit="exit"
      >
        {currentView.kind === "home" && (
          <>
            <TopBar>
              <TopBarTitle title="Your drives" />
            </TopBar>
            <ScrollArea className="min-h-0 flex-1">
              <HomeScreen
                volumesState={volumesState}
                onRetryLoadingVolumes={onRetryLoadingVolumes}
              />
            </ScrollArea>
          </>
        )}
        {currentView.kind === "scan" && <ScanScreen scanId={currentView.scanId} />}
        {currentView.kind === "duplicates" && <DuplicatesScreen />}
      </motion.div>
    </AnimatePresence>
  );
}
