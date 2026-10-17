const MILLISECONDS_PER_SECOND = 1000;
const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY = 24 * SECONDS_PER_HOUR;
const SECONDS_PER_WEEK = 7 * SECONDS_PER_DAY;
const SECONDS_PER_MONTH = 30 * SECONDS_PER_DAY;
const SECONDS_PER_YEAR = 365 * SECONDS_PER_DAY;

/** Files can be a little ahead of the clock (other machines, rounding); treat that as now. */
const FUTURE_TOLERANCE_SECONDS = 60;

const relativeTimeFormat = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

type RelativeUnitStep = {
  unit: Intl.RelativeTimeFormatUnit;
  secondsPerUnit: number;
  upperLimitSeconds: number;
};

const RELATIVE_UNIT_STEPS: RelativeUnitStep[] = [
  { unit: "minute", secondsPerUnit: SECONDS_PER_MINUTE, upperLimitSeconds: SECONDS_PER_HOUR },
  { unit: "hour", secondsPerUnit: SECONDS_PER_HOUR, upperLimitSeconds: SECONDS_PER_DAY },
  { unit: "day", secondsPerUnit: SECONDS_PER_DAY, upperLimitSeconds: SECONDS_PER_WEEK },
  { unit: "week", secondsPerUnit: SECONDS_PER_WEEK, upperLimitSeconds: SECONDS_PER_MONTH },
  { unit: "month", secondsPerUnit: SECONDS_PER_MONTH, upperLimitSeconds: SECONDS_PER_YEAR },
  { unit: "year", secondsPerUnit: SECONDS_PER_YEAR, upperLimitSeconds: Number.POSITIVE_INFINITY },
];

/**
 * Describes a moment relative to `nowUnixMilliseconds` in plain English: "just now",
 * "5 minutes ago", "yesterday", "3 weeks ago", "2 years ago".
 */
export function formatRelativeDate(unixMilliseconds: number, nowUnixMilliseconds: number): string {
  if (!Number.isFinite(unixMilliseconds) || !Number.isFinite(nowUnixMilliseconds)) {
    return "unknown date";
  }

  const elapsedSeconds = (nowUnixMilliseconds - unixMilliseconds) / MILLISECONDS_PER_SECOND;

  if (elapsedSeconds < -FUTURE_TOLERANCE_SECONDS) {
    return "in the future";
  }

  if (elapsedSeconds < SECONDS_PER_MINUTE) {
    return "just now";
  }

  const matchingStep =
    RELATIVE_UNIT_STEPS.find((step) => elapsedSeconds < step.upperLimitSeconds) ??
    RELATIVE_UNIT_STEPS[RELATIVE_UNIT_STEPS.length - 1];

  if (matchingStep === undefined) {
    return "unknown date";
  }

  const elapsedUnits = Math.max(1, Math.floor(elapsedSeconds / matchingStep.secondsPerUnit));
  return relativeTimeFormat.format(-elapsedUnits, matchingStep.unit);
}
