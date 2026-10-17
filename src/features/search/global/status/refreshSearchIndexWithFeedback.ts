import { toast } from "sonner";
import { refreshSearchIndex } from "@/lib/backend/search";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";

/** Starts rebuilding one drive's index (or every drive's); progress shows up in the status. */
export async function refreshSearchIndexWithFeedback(mountPoint?: string): Promise<void> {
  try {
    await refreshSearchIndex(mountPoint);
  } catch (error) {
    toast.error(
      mountPoint === undefined
        ? "Could not refresh the search index"
        : `Could not refresh ${mountPoint}`,
      { description: readableErrorMessage(error) },
    );
  }
}
