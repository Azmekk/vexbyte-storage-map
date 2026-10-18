import { type AvailableUpdate, commands, type UpdateCommandError } from "./generated";
import { unwrapCommandResult } from "./unwrapCommandResult";

export type { AvailableUpdate, UpdateCommandError } from "./generated";

function describeUpdateCommandError(commandError: UpdateCommandError): string {
  switch (commandError.kind) {
    case "checkFailed":
      return `Could not check for a newer version. ${commandError.details}`;
    case "backgroundTaskFailed":
      return "The update check stopped unexpectedly.";
  }
}

/** The newest release when it is newer than this app, otherwise `null`. Rejects when offline. */
export async function checkForUpdate(): Promise<AvailableUpdate | null> {
  const result = await commands.checkForUpdate();
  return unwrapCommandResult(result, describeUpdateCommandError);
}
