import { BackendCommandError } from "./BackendCommandError";

/** The result shape tauri-specta generates for commands that return `Result<T, E>`. */
export type CommandResult<Data, CommandError> =
  | { status: "ok"; data: Data }
  | { status: "error"; error: CommandError };

/** Returns the data of a successful command, or throws a `BackendCommandError`. */
export function unwrapCommandResult<Data, CommandError>(
  result: CommandResult<Data, CommandError>,
  describeCommandError: (commandError: CommandError) => string,
): Data {
  if (result.status === "error") {
    throw new BackendCommandError(describeCommandError(result.error), result.error);
  }

  return result.data;
}
