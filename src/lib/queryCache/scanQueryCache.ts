import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";

/** What a cached query knows right now. `isStale` data is shown while it is fetched again. */
export type QueryState<Data> =
  | { status: "loading" }
  | { status: "error"; errorMessage: string }
  | { status: "success"; data: Data; isStale: boolean };

type CacheRecord = {
  scanId: number;
  state: QueryState<unknown>;
  isFetching: boolean;
  fetchGeneration: number;
};

const MAXIMUM_CACHED_QUERIES = 600;

/**
 * A small cache for tree queries, keyed by strings that start with the scan id. Recycling items
 * marks every entry of that scan stale, so views refetch while still showing the old data.
 */
export class ScanQueryCache {
  private readonly records = new Map<string, CacheRecord>();
  private readonly listeners = new Set<() => void>();
  private nextFetchGeneration = 1;
  private version = 0;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  /** Changes whenever any entry changes; lets views that read many entries re-render. */
  getVersion = (): number => this.version;

  read<Data>(key: string): QueryState<Data> | undefined {
    return this.records.get(key)?.state as QueryState<Data> | undefined;
  }

  /** Starts loading `key` unless it is cached and fresh, or already loading. */
  ensureLoaded<Data>(key: string, scanId: number, load: () => Promise<Data>): void {
    const existingRecord = this.records.get(key);
    const needsFetch =
      existingRecord === undefined ||
      (!existingRecord.isFetching &&
        existingRecord.state.status === "success" &&
        existingRecord.state.isStale);

    if (!needsFetch) {
      return;
    }

    const fetchGeneration = this.nextFetchGeneration;
    this.nextFetchGeneration += 1;

    this.records.set(key, {
      scanId,
      state: existingRecord?.state ?? { status: "loading" },
      isFetching: true,
      fetchGeneration,
    });
    this.dropOldestRecordsAboveLimit();

    load().then(
      (data) => this.finishFetch(key, fetchGeneration, { status: "success", data, isStale: false }),
      (error: unknown) =>
        this.finishFetch(key, fetchGeneration, {
          status: "error",
          errorMessage: readableErrorMessage(error),
        }),
    );
  }

  /** Marks every cached query of the scan stale; errors are dropped so they are retried. */
  invalidateScan(scanId: number): void {
    for (const [key, record] of this.records) {
      if (record.scanId !== scanId) {
        continue;
      }

      if (record.state.status === "success") {
        this.records.set(key, {
          ...record,
          isFetching: false,
          fetchGeneration: 0,
          state: { ...record.state, isStale: true },
        });
      } else {
        this.records.delete(key);
      }
    }

    this.notifyListeners();
  }

  /** Forgets everything about a scan, for example after it was discarded. */
  removeScan(scanId: number): void {
    for (const [key, record] of this.records) {
      if (record.scanId === scanId) {
        this.records.delete(key);
      }
    }

    this.notifyListeners();
  }

  /** Drops a failed entry so the next read loads it again. */
  retry(key: string): void {
    const record = this.records.get(key);
    if (record !== undefined && record.state.status === "error") {
      this.records.delete(key);
      this.notifyListeners();
    }
  }

  private finishFetch(key: string, fetchGeneration: number, state: QueryState<unknown>): void {
    const record = this.records.get(key);

    if (record === undefined || record.fetchGeneration !== fetchGeneration) {
      return;
    }

    this.records.set(key, { ...record, state, isFetching: false });
    this.notifyListeners();
  }

  private dropOldestRecordsAboveLimit(): void {
    while (this.records.size > MAXIMUM_CACHED_QUERIES) {
      const oldestKey = this.records.keys().next().value;
      if (oldestKey === undefined) {
        return;
      }
      this.records.delete(oldestKey);
    }
  }

  private notifyListeners(): void {
    this.version += 1;
    for (const listener of this.listeners) {
      listener();
    }
  }
}

/** The one cache every tree query hook shares. */
export const scanQueryCache = new ScanQueryCache();
