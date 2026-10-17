import { formatCount } from "@/lib/formatting/formatCount";

export type SearchResultCountFacts = {
  shownHitCount: number;
  totalMatchCount: number;
  totalMatchesCapped: boolean;
  elapsedMilliseconds: number;
};

/** `1 match`, `1,284 matches`. */
export function formatMatchCount(matchCount: number): string {
  return `${formatCount(matchCount)} ${Math.round(matchCount) === 1 ? "match" : "matches"}`;
}

/** `in 12 ms`, `in under 1 ms`, `in 1.4 s`. */
export function formatSearchDuration(elapsedMilliseconds: number): string {
  if (elapsedMilliseconds < 1) {
    return "in under 1 ms";
  }
  if (elapsedMilliseconds < 1000) {
    return `in ${Math.round(elapsedMilliseconds)} ms`;
  }
  return `in ${(elapsedMilliseconds / 1000).toFixed(1)} s`;
}

/** `showing the first 10,000` when more items matched than the list holds, otherwise `null`. */
export function describeShownHitLimit(facts: SearchResultCountFacts): string | null {
  if (!facts.totalMatchesCapped && facts.shownHitCount >= facts.totalMatchCount) {
    return null;
  }
  return `showing the first ${formatCount(facts.shownHitCount)}`;
}

/** The whole count line in words: `1,284 matches in 12 ms, showing the first 10,000`. */
export function describeSearchResultCount(facts: SearchResultCountFacts): string {
  const headline = `${formatMatchCount(facts.totalMatchCount)} ${formatSearchDuration(facts.elapsedMilliseconds)}`;
  const limit = describeShownHitLimit(facts);
  return limit === null ? headline : `${headline}, ${limit}`;
}
