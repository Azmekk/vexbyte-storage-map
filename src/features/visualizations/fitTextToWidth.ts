const ELLIPSIS = "…";

/**
 * Shortens `text` with an ellipsis until `measureWidth` says it fits; returns an empty string
 * when not even one character fits. Binary search keeps it cheap for long names.
 */
export function fitTextToWidth(
  text: string,
  maximumWidth: number,
  measureWidth: (candidate: string) => number,
): string {
  if (maximumWidth <= 0) {
    return "";
  }
  if (measureWidth(text) <= maximumWidth) {
    return text;
  }

  let shortestFailing = text.length;
  let longestFitting = 0;

  while (longestFitting + 1 < shortestFailing) {
    const candidateLength = Math.floor((longestFitting + shortestFailing) / 2);
    if (measureWidth(text.slice(0, candidateLength) + ELLIPSIS) <= maximumWidth) {
      longestFitting = candidateLength;
    } else {
      shortestFailing = candidateLength;
    }
  }

  return longestFitting === 0 ? "" : text.slice(0, longestFitting) + ELLIPSIS;
}
