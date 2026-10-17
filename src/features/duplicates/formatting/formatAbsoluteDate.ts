const absoluteDateFormat = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "short",
});

/** The full local date and time, for tooltips next to a relative date. */
export function formatAbsoluteDate(unixMilliseconds: number): string {
  if (!Number.isFinite(unixMilliseconds)) {
    return "Unknown date";
  }

  return absoluteDateFormat.format(new Date(unixMilliseconds));
}
