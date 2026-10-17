import { toast } from "sonner";
import { openSearchHit, revealSearchHit, type SearchHit } from "@/lib/backend/search";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";

async function runHitAction(failureTitle: string, action: () => Promise<void>): Promise<void> {
  try {
    await action();
  } catch (error) {
    toast.error(failureTitle, { description: readableErrorMessage(error) });
  }
}

/** Opens a file with its default app, or a folder in File Explorer. */
export function openHit(hit: SearchHit): void {
  void runHitAction(`Could not open ${hit.name}`, () => openSearchHit(hit.path));
}

/** Opens File Explorer with the item selected. */
export function revealHit(hit: SearchHit): void {
  void runHitAction(`Could not show ${hit.name} in File Explorer`, () => revealSearchHit(hit.path));
}

export function copyHitPath(hit: SearchHit): void {
  void runHitAction("Could not copy the path", async () => {
    await navigator.clipboard.writeText(hit.path);
    toast.success("Path copied", { description: hit.path });
  });
}

export function copyHitName(hit: SearchHit): void {
  void runHitAction("Could not copy the name", async () => {
    await navigator.clipboard.writeText(hit.name);
    toast.success("Name copied", { description: hit.name });
  });
}
