import { BackendCommandError } from "@/lib/backend/BackendCommandError";

/** Whether the backend refused because a group would be left without a kept copy. */
export function isGroupWouldHaveNoKeptFileError(error: unknown): boolean {
  if (!(error instanceof BackendCommandError)) {
    return false;
  }

  const commandError: unknown = error.commandError;
  return (
    typeof commandError === "object" &&
    commandError !== null &&
    "kind" in commandError &&
    commandError.kind === "groupWouldHaveNoKeptFile"
  );
}
