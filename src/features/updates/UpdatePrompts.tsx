import { UpdateAvailableDialog } from "./UpdateAvailableDialog";
import { useUpdatedNotice } from "./useUpdatedNotice";

/**
 * The update dialog and the "Updated to …" notice. Render it after the `Toaster`, so the toaster
 * is listening before the notice is shown.
 */
export function UpdatePrompts() {
  useUpdatedNotice();
  return <UpdateAvailableDialog />;
}
