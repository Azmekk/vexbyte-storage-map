import { useCallback, useEffect, useSyncExternalStore } from "react";
import { type QueryState, scanQueryCache } from "./scanQueryCache";

const LOADING_STATE: QueryState<never> = { status: "loading" };

export type ScanQueryResult<Data> = {
  state: QueryState<Data>;
  retry: () => void;
};

/**
 * Reads `key` from the shared tree query cache and loads it when missing or stale. Pass `null`
 * as the key to skip the query. `load` may change identity freely; only `key` decides what is
 * fetched.
 */
export function useScanQuery<Data>(
  scanId: number,
  key: string | null,
  load: () => Promise<Data>,
): ScanQueryResult<Data> {
  const readSnapshot = useCallback(
    () => (key === null ? undefined : scanQueryCache.read<Data>(key)),
    [key],
  );
  const cachedState = useSyncExternalStore(scanQueryCache.subscribe, readSnapshot);

  useEffect(() => {
    if (key === null) {
      return;
    }

    const needsLoad =
      cachedState === undefined || (cachedState.status === "success" && cachedState.isStale);

    if (needsLoad) {
      scanQueryCache.ensureLoaded(key, scanId, load);
    }
  }, [key, scanId, load, cachedState]);

  const retry = useCallback(() => {
    if (key !== null) {
      scanQueryCache.retry(key);
    }
  }, [key]);

  return { state: cachedState ?? LOADING_STATE, retry };
}
