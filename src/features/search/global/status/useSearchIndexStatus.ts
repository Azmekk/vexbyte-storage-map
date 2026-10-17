import { useEffect } from "react";
import { create } from "zustand";
import {
  getSearchIndexStatus,
  type SearchIndexStatus,
  subscribeToSearchIndexStatus,
} from "@/lib/backend/search";
import { readableErrorMessage } from "@/lib/errors/readableErrorMessage";

export type SearchIndexStatusState = {
  /** `null` until the first answer arrived. */
  status: SearchIndexStatus | null;
  loadErrorMessage: string | null;
};

const useSearchIndexStatusStore = create<SearchIndexStatusState>(() => ({
  status: null,
  loadErrorMessage: null,
}));

let mountedSubscriberCount = 0;
let stopListening: (() => void) | null = null;

/**
 * Subscribes to status events first and then fetches the current status, so no change is lost
 * in between. A fetch answer that arrives after an event is older than it and is dropped.
 */
function startListening(): () => void {
  let hasReceivedEvent = false;
  let isStopped = false;
  const store = useSearchIndexStatusStore;

  const unlistenPromise = subscribeToSearchIndexStatus((status) => {
    hasReceivedEvent = true;
    store.setState({ status, loadErrorMessage: null });
  });

  unlistenPromise
    .then(() => getSearchIndexStatus())
    .then(
      (status) => {
        if (!isStopped && !hasReceivedEvent) {
          store.setState({ status, loadErrorMessage: null });
        }
      },
      (error: unknown) => {
        if (!isStopped && !hasReceivedEvent) {
          store.setState({ loadErrorMessage: readableErrorMessage(error) });
        }
      },
    );

  return () => {
    isStopped = true;
    void unlistenPromise.then(
      (unlisten) => unlisten(),
      () => undefined,
    );
  };
}

/**
 * The search index status, kept current by one shared subscription however many components use
 * it. The subscription stops when the last of them unmounts.
 */
export function useSearchIndexStatus(): SearchIndexStatusState {
  const status = useSearchIndexStatusStore((state) => state.status);
  const loadErrorMessage = useSearchIndexStatusStore((state) => state.loadErrorMessage);

  useEffect(() => {
    mountedSubscriberCount += 1;
    if (mountedSubscriberCount === 1) {
      stopListening = startListening();
    }
    return () => {
      mountedSubscriberCount -= 1;
      if (mountedSubscriberCount === 0 && stopListening !== null) {
        stopListening();
        stopListening = null;
      }
    };
  }, []);

  return { status, loadErrorMessage };
}
