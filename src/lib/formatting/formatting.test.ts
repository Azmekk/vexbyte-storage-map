import { describe, expect, it } from "vitest";
import { formatCount } from "./formatCount";
import { formatDurationInWords, formatElapsedClock } from "./formatDuration";
import { formatModifiedDate } from "./formatModifiedDate";
import { formatShare } from "./formatShare";

describe("formatCount", () => {
  it("groups thousands", () => {
    expect(formatCount(1234567)).toBe("1,234,567");
    expect(formatCount(0)).toBe("0");
    expect(formatCount(Number.NaN)).toBe("0");
  });
});

describe("formatShare", () => {
  it("formats shares as percentages", () => {
    expect(formatShare(0.234)).toBe("23.4%");
    expect(formatShare(1)).toBe("100%");
    expect(formatShare(0)).toBe("0%");
  });

  it("marks tiny and missing shares", () => {
    expect(formatShare(0.0004)).toBe("<0.1%");
    expect(formatShare(null)).toBe("–");
  });
});

describe("formatElapsedClock", () => {
  it("shows minutes and seconds, and hours when needed", () => {
    expect(formatElapsedClock(7_400)).toBe("0:07");
    expect(formatElapsedClock(765_000)).toBe("12:45");
    expect(formatElapsedClock(3_723_000)).toBe("1:02:03");
    expect(formatElapsedClock(-5)).toBe("0:00");
  });
});

describe("formatDurationInWords", () => {
  it("uses tenths for short durations and minutes for long ones", () => {
    expect(formatDurationInWords(420)).toBe("0.4 s");
    expect(formatDurationInWords(12_300)).toBe("12 s");
    expect(formatDurationInWords(185_000)).toBe("3 min 5 s");
    expect(formatDurationInWords(120_000)).toBe("2 min");
  });
});

describe("formatModifiedDate", () => {
  it("shows a dash for unknown dates", () => {
    expect(formatModifiedDate(null)).toBe("–");
    expect(formatModifiedDate(0)).toBe("–");
  });

  it("formats known dates", () => {
    expect(formatModifiedDate(Date.UTC(2024, 2, 12, 12))).toMatch(/2024/);
  });
});
