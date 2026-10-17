const countFormat = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

/** A whole number with thousands separators, for example `12,480`. */
export function formatCount(count: number): string {
  if (!Number.isFinite(count)) {
    return "0";
  }

  return countFormat.format(Math.round(count));
}
