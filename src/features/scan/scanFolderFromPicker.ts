import { toast } from "sonner";
import { pickFolder } from "@/lib/backend/folderPicker";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { startScanSession } from "./scanSession";

/** Lets the user pick a folder and starts a standard scan of it. Does nothing when cancelled. */
export async function scanFolderFromPicker(): Promise<void> {
  let chosenFolder: string | null;

  try {
    chosenFolder = await pickFolder("Choose a folder to scan");
  } catch (error) {
    toast.error("The folder picker could not open", { description: readableErrorMessage(error) });
    return;
  }

  if (chosenFolder !== null) {
    await startScanSession({ folder: { path: chosenFolder } }, "folderWalk");
  }
}
