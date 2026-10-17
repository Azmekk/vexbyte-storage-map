import { BackendCommandError } from "@/lib/backend/BackendCommandError";
import type { SearchCommandError, SearchQueryError } from "@/lib/backend/search";

function isSearchCommandError(value: unknown): value is SearchCommandError {
  return typeof value === "object" && value !== null && "kind" in value;
}

/** The reason a query was refused, when `error` is the backend's `invalidQuery` answer. */
export function invalidQueryProblem(error: unknown): SearchQueryError | null {
  if (!(error instanceof BackendCommandError)) {
    return null;
  }
  const commandError: unknown = error.commandError;
  if (isSearchCommandError(commandError) && commandError.kind === "invalidQuery") {
    return commandError.problem;
  }
  return null;
}
