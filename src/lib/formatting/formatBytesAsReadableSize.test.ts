import { describe, expect, it } from "vitest";
import { formatBytesAsReadableSize } from "./formatBytesAsReadableSize";

const kibibyte = 1024;
const mebibyte = kibibyte * 1024;
const gibibyte = mebibyte * 1024;
const tebibyte = gibibyte * 1024;

describe("formatBytesAsReadableSize", () => {
  it("shows whole bytes below one kilobyte", () => {
    expect(formatBytesAsReadableSize(0)).toBe("0 B");
    expect(formatBytesAsReadableSize(1)).toBe("1 B");
    expect(formatBytesAsReadableSize(1023)).toBe("1023 B");
  });

  it("uses 1024-based units with one decimal", () => {
    expect(formatBytesAsReadableSize(kibibyte)).toBe("1.0 KB");
    expect(formatBytesAsReadableSize(1536)).toBe("1.5 KB");
    expect(formatBytesAsReadableSize(5.25 * mebibyte)).toBe("5.3 MB");
    expect(formatBytesAsReadableSize(476.3 * gibibyte)).toBe("476.3 GB");
    expect(formatBytesAsReadableSize(2 * tebibyte)).toBe("2.0 TB");
  });

  it("moves to the next unit when rounding reaches 1024", () => {
    expect(formatBytesAsReadableSize(mebibyte - 1)).toBe("1.0 MB");
    expect(formatBytesAsReadableSize(gibibyte - 1)).toBe("1.0 GB");
  });

  it("keeps terabytes as the largest unit", () => {
    expect(formatBytesAsReadableSize(2048 * tebibyte)).toBe("2048.0 TB");
  });

  it("treats negative and invalid input as zero bytes", () => {
    expect(formatBytesAsReadableSize(-5)).toBe("0 B");
    expect(formatBytesAsReadableSize(Number.NaN)).toBe("0 B");
  });
});
