import { describe, expect, it } from "vitest";
import { formatRelativeDate } from "./formatRelativeDate";

const now = Date.UTC(2026, 9, 17, 12, 0, 0);
const second = 1000;
const minute = 60 * second;
const hour = 60 * minute;
const day = 24 * hour;

describe("formatRelativeDate", () => {
  it("calls the last minute just now", () => {
    expect(formatRelativeDate(now, now)).toBe("just now");
    expect(formatRelativeDate(now - 59 * second, now)).toBe("just now");
  });

  it("counts minutes and hours", () => {
    expect(formatRelativeDate(now - minute, now)).toBe("1 minute ago");
    expect(formatRelativeDate(now - 5 * minute, now)).toBe("5 minutes ago");
    expect(formatRelativeDate(now - 3 * hour, now)).toBe("3 hours ago");
    expect(formatRelativeDate(now - 23 * hour, now)).toBe("23 hours ago");
  });

  it("says yesterday for one day and counts days up to a week", () => {
    expect(formatRelativeDate(now - day, now)).toBe("yesterday");
    expect(formatRelativeDate(now - 4 * day, now)).toBe("4 days ago");
  });

  it("moves to weeks, months and years", () => {
    expect(formatRelativeDate(now - 7 * day, now)).toBe("last week");
    expect(formatRelativeDate(now - 15 * day, now)).toBe("2 weeks ago");
    expect(formatRelativeDate(now - 65 * day, now)).toBe("2 months ago");
    expect(formatRelativeDate(now - 400 * day, now)).toBe("last year");
    expect(formatRelativeDate(now - 3 * 365 * day, now)).toBe("3 years ago");
  });

  it("tolerates small clock differences and flags dates far in the future", () => {
    expect(formatRelativeDate(now + 30 * second, now)).toBe("just now");
    expect(formatRelativeDate(now + day, now)).toBe("in the future");
  });

  it("handles invalid input", () => {
    expect(formatRelativeDate(Number.NaN, now)).toBe("unknown date");
  });
});
