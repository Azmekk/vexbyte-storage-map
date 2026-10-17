import { BackendCommandError } from "@/lib/backend/BackendCommandError";
import type { DuplicateCommandError } from "@/lib/backend/duplicates";
import { describeInvalidSettings } from "./buildDuplicateSearchSettings";

function isDuplicateCommandError(
  error: unknown,
): error is BackendCommandError<DuplicateCommandError> {
  if (!(error instanceof BackendCommandError)) {
    return false;
  }

  const commandError: unknown = error.commandError;
  return typeof commandError === "object" && commandError !== null && "kind" in commandError;
}

/**
 * The message to show inline on the setup step when the backend refused the chosen folders, for
 * example a system folder or a folder on a cloud drive. `null` for any other error.
 */
export function describeSetupRejection(error: unknown): string | null {
  if (!isDuplicateCommandError(error)) {
    return null;
  }

  const commandError = error.commandError;
  switch (commandError.kind) {
    case "invalidSettings":
      return describeInvalidSettings(commandError.details);
    case "cloudVirtualFolderNotAllowed":
      return error.message;
    default:
      return null;
  }
}
