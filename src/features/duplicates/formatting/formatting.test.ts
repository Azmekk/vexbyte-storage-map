import { describe, expect, it } from "vitest";
import { formatCount } from "./formatCount";
import { pluralize } from "./pluralize";

describe("formatCount", () => {
  it("rounds and adds thousands separators", () => {
    expect(formatCount(0)).toBe("0");
    expect(formatCount(12480.4)).toBe("12,480");
    expect(formatCount(Number.NaN)).toBe("0");
  });
});

describe("pluralize", () => {
  it("picks the singular only for exactly one", () => {
    expect(pluralize(1, "file", "files")).toBe("1 file");
    expect(pluralize(0, "file", "files")).toBe("0 files");
    expect(pluralize(1204, "copy", "copies")).toBe("1,204 copies");
  });
});
