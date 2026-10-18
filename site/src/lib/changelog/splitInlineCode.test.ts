import { describe, expect, it } from "vitest";
import { splitInlineCode } from "./splitInlineCode";

describe("splitInlineCode", () => {
  it("separates code spans from the text around them", () => {
    expect(splitInlineCode("Reads `/proc/self/mountinfo` at start.")).toEqual([
      { kind: "text", text: "Reads " },
      { kind: "code", text: "/proc/self/mountinfo" },
      { kind: "text", text: " at start." },
    ]);
  });

  it("handles code at the edges and plain text", () => {
    expect(splitInlineCode("`a` and `b`")).toEqual([
      { kind: "code", text: "a" },
      { kind: "text", text: " and " },
      { kind: "code", text: "b" },
    ]);
    expect(splitInlineCode("no code")).toEqual([{ kind: "text", text: "no code" }]);
  });

  it("leaves an unmatched backtick as text", () => {
    expect(splitInlineCode("odd ` tick")).toEqual([{ kind: "text", text: "odd ` tick" }]);
  });
});
