const countFormatter = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

/** Formats a count with thousands separators, for example `12,345`. */
export function formatCount(count: number): string {
  if (!Number.isFinite(count)) {
    return "0";
  }

  return countFormatter.format(Math.round(count));
}
