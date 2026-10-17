/** Turns anything thrown into a message that can be shown to the user. */
export function readableErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.length > 0) {
    return error.message;
  }

  if (typeof error === "string" && error.length > 0) {
    return error;
  }

  return "Something went wrong.";
}
