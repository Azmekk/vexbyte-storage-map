import { ArrowLeftIcon, CopyIcon, LoaderCircleIcon, TriangleAlertIcon } from "lucide-react";
import { motion } from "motion/react";
import { useNavigationStore } from "@/app/useNavigationStore";
import { Button } from "@/components/ui/button";
import { fadeSlideInVariants } from "@/lib/motion/transitions";
import { CategoryChoiceGrid } from "./CategoryChoiceGrid";
import { FolderChoiceList } from "./FolderChoiceList";
import { MinimumSizeSelect } from "./MinimumSizeSelect";
import { useDuplicateSetup } from "./useDuplicateSetup";

export function DuplicateSetupStep() {
  const setup = useDuplicateSetup();
  const navigateHome = useNavigationStore((state) => state.navigateHome);
  const isFindDisabled =
    setup.setupBlocker !== null || setup.isStarting || setup.folderChoicesState.status !== "ready";
  const footerMessage = setup.setupErrorMessage ?? setup.setupBlocker;

  return (
    <motion.div
      className="flex min-h-0 flex-1 flex-col"
      variants={fadeSlideInVariants}
      initial="hidden"
      animate="visible"
    >
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto grid w-full max-w-6xl gap-8 p-6">
          <header className="grid max-w-2xl justify-items-start gap-1.5">
            <Button
              variant="ghost"
              size="sm"
              className="-ml-2.5 text-muted-foreground"
              onClick={navigateHome}
            >
              <ArrowLeftIcon />
              Back to drives
            </Button>
            <h1 className="font-semibold text-2xl tracking-tight">Find duplicate files</h1>
            <p className="text-muted-foreground text-sm">
              Pick where to look and what to compare. Nothing is changed until you review the copies
              and choose what to move to the Recycle Bin.
            </p>
          </header>

          <div className="grid gap-8 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <FolderChoiceList
              folderChoicesState={setup.folderChoicesState}
              onToggleFolder={setup.toggleFolder}
              onRemoveFolder={setup.removeFolder}
              onAddFolder={setup.addFolder}
              onRetryLoadingFolders={setup.retryLoadingFolders}
            />
            <div className="grid content-start gap-4">
              <CategoryChoiceGrid
                selectedCategories={setup.selectedCategories}
                onToggleCategory={setup.toggleCategory}
              />
              <MinimumSizeSelect
                minimumFileSizeBytes={setup.minimumFileSizeBytes}
                onMinimumFileSizeChange={setup.setMinimumFileSizeBytes}
              />
            </div>
          </div>
        </div>
      </div>

      <footer className="border-t bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/80">
        <div className="mx-auto flex w-full max-w-6xl flex-wrap items-center justify-end gap-4 px-6 py-4">
          {footerMessage !== null && (
            <p
              role={setup.setupErrorMessage !== null ? "alert" : "status"}
              data-error={setup.setupErrorMessage !== null}
              className="mr-auto flex min-w-0 items-center gap-2 text-muted-foreground text-sm data-[error=true]:text-destructive"
            >
              <TriangleAlertIcon className="size-4 shrink-0" aria-hidden />
              <span className="min-w-0">{footerMessage}</span>
            </p>
          )}
          <Button
            size="lg"
            className="h-11 min-w-48 px-5 text-base"
            onClick={setup.startSearch}
            disabled={isFindDisabled}
          >
            {setup.isStarting ? <LoaderCircleIcon className="animate-spin" /> : <CopyIcon />}
            Find duplicates
          </Button>
        </div>
      </footer>
    </motion.div>
  );
}
