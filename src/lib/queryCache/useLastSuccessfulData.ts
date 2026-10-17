import { useRef } from "react";
import type { QueryState } from "./scanQueryCache";

/**
 * The data of the latest successful state, kept while a new key loads, so views change once the
 * new data is there instead of flashing a loading state.
 */
export function useLastSuccessfulData<Data>(state: QueryState<Data>): Data | null {
  const lastDataRef = useRef<Data | null>(null);

  if (state.status === "success") {
    lastDataRef.current = state.data;
  }

  return state.status === "success" ? state.data : lastDataRef.current;
}
