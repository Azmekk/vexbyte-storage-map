/** Thrown by the backend wrappers when a command returns its typed error. */
export class BackendCommandError<CommandError> extends Error {
  readonly commandError: CommandError;

  constructor(message: string, commandError: CommandError) {
    super(message);
    this.name = "BackendCommandError";
    this.commandError = commandError;
  }
}
