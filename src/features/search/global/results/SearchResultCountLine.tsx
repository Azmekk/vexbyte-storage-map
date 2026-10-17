import { NumberTicker } from "@/components/NumberTicker";
import type { SearchResponse } from "@/lib/backend/search";
import {
  describeSearchResultCount,
  describeShownHitLimit,
  formatMatchCount,
  formatSearchDuration,
  type SearchResultCountFacts,
} from "./describeSearchResultCount";

type SearchResultCountLineProps = {
  response: SearchResponse | null;
  isSearching: boolean;
};

export function SearchResultCountLine({ response, isSearching }: SearchResultCountLineProps) {
  if (response === null) {
    return (
      <p className="text-muted-foreground text-sm" aria-live="polite">
        {isSearching ? "Searching…" : " "}
      </p>
    );
  }

  const facts: SearchResultCountFacts = {
    shownHitCount: response.hits.length,
    totalMatchCount: response.totalMatchCount,
    totalMatchesCapped: response.totalMatchesCapped,
    elapsedMilliseconds: response.elapsedMilliseconds,
  };
  const shownHitLimit = describeShownHitLimit(facts);

  return (
    <p className="text-muted-foreground text-sm">
      <span className="sr-only" aria-live="polite">
        {describeSearchResultCount(facts)}
      </span>
      <span aria-hidden>
        <NumberTicker
          value={response.totalMatchCount}
          formatValue={formatMatchCount}
          className="font-medium text-foreground"
        />{" "}
        {formatSearchDuration(response.elapsedMilliseconds)}
        {shownHitLimit !== null && `, ${shownHitLimit}`}
      </span>
    </p>
  );
}
