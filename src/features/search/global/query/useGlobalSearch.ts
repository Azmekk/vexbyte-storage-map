import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  describeSearchQueryError,
  MAXIMUM_SEARCH_RESULT_COUNT,
  type SearchResponse,
  type SearchSortOrder,
  searchFiles,
} from "@/lib/backend/search";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";
import { invalidQueryProblem } from "./invalidQueryProblem";
import { createRequestSequence } from "./requestSequence";

export const SEARCH_DEBOUNCE_MILLISECONDS = 120;

export type GlobalSearchState = {
  /** The latest answer for the current query, kept while a newer one is on its way. */
  response: SearchResponse | null;
  isSearching: boolean;
  /** Why the query could not be understood, shown under the search box. */
  queryErrorMessage: string | null;
  /** Any other failure. */
  failureMessage: string | null;
};

const IDLE_STATE: GlobalSearchState = {
  response: null,
  isSearching: false,
  queryErrorMessage: null,
  failureMessage: null,
};

/**
 * Runs the query a moment after the user stops typing, and again whenever the set of searchable
 * drives changes (a drive finished warming up or was refreshed). Answers to outdated requests are
 * dropped by a request sequence, so a slow answer never replaces a newer one.
 */
export function useGlobalSearch(
  query: string,
  sortOrder: SearchSortOrder,
  searchableVolumesKey: string,
): GlobalSearchState & { retry: () => void } {
  const [searchState, setSearchState] = useState<GlobalSearchState>(IDLE_STATE);
  const [attempt, setAttempt] = useState(0);
  const requestSequenceRef = useRef(createRequestSequence());

  const request = useMemo(
    () => ({ query, sortOrder, searchableVolumesKey, attempt }),
    [query, sortOrder, searchableVolumesKey, attempt],
  );

  useEffect(() => {
    const requestSequence = requestSequenceRef.current;
    const ticket = requestSequence.begin();

    if (request.query.trim().length === 0) {
      setSearchState(IDLE_STATE);
      return;
    }

    setSearchState((previousState) => ({ ...previousState, isSearching: true }));

    const timeoutId = window.setTimeout(() => {
      searchFiles(request.query, request.sortOrder, MAXIMUM_SEARCH_RESULT_COUNT).then(
        (response) => {
          if (requestSequence.isLatest(ticket)) {
            setSearchState({
              response,
              isSearching: false,
              queryErrorMessage: null,
              failureMessage: null,
            });
          }
        },
        (error: unknown) => {
          if (!requestSequence.isLatest(ticket)) {
            return;
          }
          const problem = invalidQueryProblem(error);
          setSearchState((previousState) => ({
            ...previousState,
            isSearching: false,
            queryErrorMessage: problem === null ? null : describeSearchQueryError(problem),
            failureMessage: problem === null ? readableErrorMessage(error) : null,
          }));
        },
      );
    }, SEARCH_DEBOUNCE_MILLISECONDS);

    return () => window.clearTimeout(timeoutId);
  }, [request]);

  const retry = useCallback(() => setAttempt((previousAttempt) => previousAttempt + 1), []);

  return { ...searchState, retry };
}
