import { useEffect, useState } from "react";
import { describeScanAge } from "./describeScanAge";

const MILLISECONDS_PER_MINUTE = 60_000;
/** Lands just after the minute turns, so the label never shows the previous minute. */
const REFRESH_SLACK_MILLISECONDS = 50;

/** How long ago the scan ended, refreshed whenever another minute has passed. */
export function useScanAgeLabel(endedAtUnixMilliseconds: number | null): string | null {
  const [nowUnixMilliseconds, setNowUnixMilliseconds] = useState(() => Date.now());

  useEffect(() => {
    if (endedAtUnixMilliseconds === null) {
      return;
    }
    const elapsedMilliseconds = Math.max(0, nowUnixMilliseconds - endedAtUnixMilliseconds);
    const untilNextMinute =
      MILLISECONDS_PER_MINUTE - (elapsedMilliseconds % MILLISECONDS_PER_MINUTE);
    const timeoutId = window.setTimeout(
      () => setNowUnixMilliseconds(Date.now()),
      untilNextMinute + REFRESH_SLACK_MILLISECONDS,
    );
    return () => window.clearTimeout(timeoutId);
  }, [endedAtUnixMilliseconds, nowUnixMilliseconds]);

  return describeScanAge(endedAtUnixMilliseconds, nowUnixMilliseconds);
}
