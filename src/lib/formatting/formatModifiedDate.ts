const shortDateFormatter = new Intl.DateTimeFormat(undefined, {
  year: "numeric",
  month: "short",
  day: "numeric",
});

const fullDateFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "short",
});

/** A compact date for table cells, or `–` when unknown. */
export function formatModifiedDate(unixMilliseconds: number | null): string {
  if (unixMilliseconds === null || !Number.isFinite(unixMilliseconds) || unixMilliseconds <= 0) {
    return "–";
  }

  return shortDateFormatter.format(new Date(unixMilliseconds));
}

/** Date and time for the inspector, or `Unknown`. */
export function formatModifiedDateAndTime(unixMilliseconds: number | null): string {
  if (unixMilliseconds === null || !Number.isFinite(unixMilliseconds) || unixMilliseconds <= 0) {
    return "Unknown";
  }

  return fullDateFormatter.format(new Date(unixMilliseconds));
}
