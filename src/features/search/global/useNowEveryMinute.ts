import { useEffect, useState } from "react";

const MILLISECONDS_PER_MINUTE = 60_000;

/** The current time, updated once a minute, for "5 minutes ago" labels. */
export function useNowEveryMinute(): number {
  const [nowUnixMilliseconds, setNowUnixMilliseconds] = useState(() => Date.now());

  useEffect(() => {
    const intervalId = window.setInterval(
      () => setNowUnixMilliseconds(Date.now()),
      MILLISECONDS_PER_MINUTE,
    );
    return () => window.clearInterval(intervalId);
  }, []);

  return nowUnixMilliseconds;
}
