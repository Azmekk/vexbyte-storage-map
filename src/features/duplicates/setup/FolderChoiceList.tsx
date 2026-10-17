import { FolderPlusIcon, RotateCwIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { FolderChoiceRow } from "./FolderChoiceRow";
import type { FolderChoicesState } from "./useDuplicateSetup";

const SKELETON_ROW_KEYS = ["first", "second", "third", "fourth"];

type FolderChoiceListProps = {
  folderChoicesState: FolderChoicesState;
  onToggleFolder: (folderPath: string) => void;
  onRemoveFolder: (folderPath: string) => void;
  onAddFolder: () => void;
  onRetryLoadingFolders: () => void;
};

export function FolderChoiceList({
  folderChoicesState,
  onToggleFolder,
  onRemoveFolder,
  onAddFolder,
  onRetryLoadingFolders,
}: FolderChoiceListProps) {
  return (
    <section className="grid content-start gap-3" aria-labelledby="duplicate-folders-heading">
      <div className="flex items-end justify-between gap-3">
        <div className="grid gap-0.5">
          <h2 id="duplicate-folders-heading" className="font-semibold text-base">
            Where to look
          </h2>
          <p className="text-muted-foreground text-sm">Subfolders are included.</p>
        </div>
        <Button variant="outline" onClick={onAddFolder}>
          <FolderPlusIcon />
          Add folder
        </Button>
      </div>

      <div className="rounded-xl border bg-card p-1.5">
        {folderChoicesState.status === "loading" && (
          <div className="grid gap-1 p-1.5" aria-busy>
            {SKELETON_ROW_KEYS.map((rowKey) => (
              <Skeleton key={rowKey} className="h-11 w-full" />
            ))}
          </div>
        )}

        {folderChoicesState.status === "error" && (
          <div role="alert" className="grid justify-items-start gap-3 p-4">
            <p className="text-sm">
              Your personal folders couldn't be found. {folderChoicesState.errorMessage}
            </p>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={onRetryLoadingFolders}>
                <RotateCwIcon />
                Try again
              </Button>
              <Button variant="ghost" size="sm" onClick={onAddFolder}>
                Choose a folder instead
              </Button>
            </div>
          </div>
        )}

        {folderChoicesState.status === "ready" && folderChoicesState.folderChoices.length === 0 && (
          <p className="p-4 text-muted-foreground text-sm">
            No folders yet. Add one to search it for duplicates.
          </p>
        )}

        {folderChoicesState.status === "ready" && folderChoicesState.folderChoices.length > 0 && (
          <ul className="grid">
            {folderChoicesState.folderChoices.map((folderChoice) => (
              <FolderChoiceRow
                key={folderChoice.path}
                folderChoice={folderChoice}
                onToggle={onToggleFolder}
                onRemove={onRemoveFolder}
              />
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
