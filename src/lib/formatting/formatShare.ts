const SMALLEST_SHOWN_PERCENTAGE = 0.1;

/**
 * Formats a share between 0 and 1 as a percentage with one decimal, for example `0.234` becomes
 * `23.4%`. Shares that round to zero but are not zero show as `<0.1%`; a missing share as `–`.
 */
export function formatShare(share: number | null): string {
  if (share === null || !Number.isFinite(share)) {
    return "–";
  }

  const percentage = share * 100;

  if (percentage <= 0) {
    return "0%";
  }

  if (percentage < SMALLEST_SHOWN_PERCENTAGE) {
    return "<0.1%";
  }

  if (percentage >= 99.95) {
    return "100%";
  }

  return `${percentage.toFixed(1)}%`;
}
