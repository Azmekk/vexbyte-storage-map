const MILLISECONDS_PER_SECOND = 1000;
const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 3600;

function padToTwoDigits(value: number): string {
  return value.toString().padStart(2, "0");
}

/** A running clock for elapsed time: `0:07`, `12:45` or `1:02:03`. */
export function formatElapsedClock(elapsedMilliseconds: number): string {
  const totalSeconds = Math.max(0, Math.floor(elapsedMilliseconds / MILLISECONDS_PER_SECOND));
  const hours = Math.floor(totalSeconds / SECONDS_PER_HOUR);
  const minutes = Math.floor((totalSeconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  const seconds = totalSeconds % SECONDS_PER_MINUTE;

  if (hours > 0) {
    return `${hours}:${padToTwoDigits(minutes)}:${padToTwoDigits(seconds)}`;
  }

  return `${minutes}:${padToTwoDigits(seconds)}`;
}

/** A finished duration in words: `0.4 s`, `12 s`, `3 min 5 s`. */
export function formatDurationInWords(durationMilliseconds: number): string {
  const safeMilliseconds = Math.max(0, durationMilliseconds);

  if (safeMilliseconds < 10 * MILLISECONDS_PER_SECOND) {
    return `${(safeMilliseconds / MILLISECONDS_PER_SECOND).toFixed(1)} s`;
  }

  const totalSeconds = Math.round(safeMilliseconds / MILLISECONDS_PER_SECOND);

  if (totalSeconds < SECONDS_PER_MINUTE) {
    return `${totalSeconds} s`;
  }

  const minutes = Math.floor(totalSeconds / SECONDS_PER_MINUTE);
  const seconds = totalSeconds % SECONDS_PER_MINUTE;
  return seconds === 0 ? `${minutes} min` : `${minutes} min ${seconds} s`;
}
