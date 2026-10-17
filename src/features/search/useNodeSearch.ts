import { useEffect, useState } from "react";
import type { ScanId } from "@/lib/backend/scans";
import { type SearchResult, searchNodes } from "@/lib/backend/tree";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";

const SEARCH_DEBOUNCE_MILLISECONDS = 180;
export const SEARCH_RESULT_LIMIT = 60;

export type NodeSearchState =
  | { status: "idle" }
  | { status: "searching"; previousResults: SearchResult[] }
  | { status: "done"; query: string; results: SearchResult[] }
  | { status: "error"; errorMessage: string };

/** Searches names in the scan a moment after the user stops typing; drops outdated answers. */
export function useNodeSearch(scanId: ScanId, query: string): NodeSearchState {
  const [searchState, setSearchState] = useState<NodeSearchState>({ status: "idle" });
  const trimmedQuery = query.trim();

  useEffect(() => {
    if (trimmedQuery.length === 0) {
      setSearchState({ status: "idle" });
      return;
    }

    setSearchState((previousState) => ({
      status: "searching",
      previousResults:
        previousState.status === "done"
          ? previousState.results
          : previousState.status === "searching"
            ? previousState.previousResults
            : [],
    }));

    let isCurrentQuery = true;
    const timeoutId = window.setTimeout(() => {
      searchNodes(scanId, trimmedQuery, SEARCH_RESULT_LIMIT).then(
        (results) => {
          if (isCurrentQuery) {
            setSearchState({ status: "done", query: trimmedQuery, results });
          }
        },
        (error: unknown) => {
          if (isCurrentQuery) {
            setSearchState({ status: "error", errorMessage: readableErrorMessage(error) });
          }
        },
      );
    }, SEARCH_DEBOUNCE_MILLISECONDS);

    return () => {
      isCurrentQuery = false;
      window.clearTimeout(timeoutId);
    };
  }, [scanId, trimmedQuery]);

  return searchState;
}
