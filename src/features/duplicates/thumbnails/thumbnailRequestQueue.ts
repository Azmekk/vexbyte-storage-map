import { getThumbnail, type ThumbnailHandle } from "@/lib/backend/fileActions";

/** The shell renders thumbnails on a few threads; more requests at once only queue up there. */
const MAXIMUM_CONCURRENT_REQUESTS = 4;

export type ThumbnailResult = { kind: "ready"; thumbnail: ThumbnailHandle } | { kind: "failed" };

type QueuedRequest = {
  cacheKey: string;
  path: string;
  sizePixels: number;
  listeners: Set<(result: ThumbnailResult) => void>;
};

const finishedResults = new Map<string, ThumbnailResult>();
const waitingRequests = new Map<string, QueuedRequest>();
const inFlightRequests = new Map<string, QueuedRequest>();

function cacheKeyOf(path: string, sizePixels: number): string {
  return `${sizePixels}|${path}`;
}

function startNextRequests(): void {
  while (inFlightRequests.size < MAXIMUM_CONCURRENT_REQUESTS) {
    const nextRequest = waitingRequests.values().next().value;
    if (nextRequest === undefined) {
      return;
    }

    waitingRequests.delete(nextRequest.cacheKey);
    inFlightRequests.set(nextRequest.cacheKey, nextRequest);
    void runRequest(nextRequest);
  }
}

async function runRequest(request: QueuedRequest): Promise<void> {
  let result: ThumbnailResult;

  try {
    const thumbnail = await getThumbnail(request.path, request.sizePixels);
    result = { kind: "ready", thumbnail };
  } catch {
    result = { kind: "failed" };
  }

  finishedResults.set(request.cacheKey, result);
  inFlightRequests.delete(request.cacheKey);
  for (const listener of request.listeners) {
    listener(result);
  }
  startNextRequests();
}

/** A thumbnail already fetched in this session, if any. */
export function cachedThumbnail(path: string, sizePixels: number): ThumbnailResult | undefined {
  return finishedResults.get(cacheKeyOf(path, sizePixels));
}

/**
 * Queues a thumbnail request and calls `onResult` when it is ready. The returned function
 * withdraws interest; a request nobody waits for any more is dropped before it starts, so rows
 * scrolled past quickly cost nothing.
 */
export function requestThumbnail(
  path: string,
  sizePixels: number,
  onResult: (result: ThumbnailResult) => void,
): () => void {
  const cacheKey = cacheKeyOf(path, sizePixels);
  const finishedResult = finishedResults.get(cacheKey);

  if (finishedResult !== undefined) {
    onResult(finishedResult);
    return () => {};
  }

  const existingRequest = waitingRequests.get(cacheKey) ?? inFlightRequests.get(cacheKey);
  const request: QueuedRequest = existingRequest ?? {
    cacheKey,
    path,
    sizePixels,
    listeners: new Set(),
  };

  request.listeners.add(onResult);
  if (existingRequest === undefined) {
    waitingRequests.set(cacheKey, request);
    startNextRequests();
  }

  return () => {
    request.listeners.delete(onResult);
    if (request.listeners.size === 0 && waitingRequests.get(cacheKey) === request) {
      waitingRequests.delete(cacheKey);
    }
  };
}
