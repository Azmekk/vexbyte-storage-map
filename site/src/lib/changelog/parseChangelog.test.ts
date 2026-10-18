import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { parseChangelog } from "./parseChangelog";

const SAMPLE = `# Changelog

## 0.2.0 - 2026-10-20
### What's new
- Runs on Linux.
- Tells you when a new version is out
  and links to what changed.
### Developer information
- A new \`mount_table\` crate.

## 0.1.3 - 2026-10-01
### What's new
- First public release.
### Something else
- Ignored.
`;

describe("parseChangelog", () => {
  it("reads every version with its date and both bullet lists", () => {
    expect(parseChangelog(SAMPLE)).toEqual([
      {
        version: "0.2.0",
        date: "2026-10-20",
        whatsNew: [
          "Runs on Linux.",
          "Tells you when a new version is out and links to what changed.",
        ],
        developerInformation: ["A new `mount_table` crate."],
      },
      {
        version: "0.1.3",
        date: "2026-10-01",
        whatsNew: ["First public release."],
        developerInformation: [],
      },
    ]);
  });

  it("accepts Windows line endings and a heading without a date", () => {
    const entries = parseChangelog("## 1.0.0\r\n### What's new\r\n- Done.\r\n");
    expect(entries).toEqual([
      { version: "1.0.0", date: null, whatsNew: ["Done."], developerInformation: [] },
    ]);
  });

  it("ignores bullets before the first version", () => {
    expect(parseChangelog("# Changelog\n- stray\n")).toEqual([]);
  });

  it("parses the repository's CHANGELOG.md", () => {
    const markdown = readFileSync(new URL("../../../../CHANGELOG.md", import.meta.url), "utf8");
    const entries = parseChangelog(markdown);
    expect(entries.length).toBeGreaterThan(0);
    for (const entry of entries) {
      expect(entry.version).toMatch(/^\d+\.\d+\.\d+/);
      expect(entry.whatsNew.length).toBeGreaterThan(0);
    }
  });
});
