import { describe, expect, it } from "vitest";
import { compareVersions } from "./compareVersions";
import {
  describeVersionChangeSinceLastRun,
  parseStoredTimestamp,
  shouldCheckForUpdateNow,
  shouldOfferUpdate,
  UPDATE_CHECK_INTERVAL_MILLISECONDS,
} from "./updateCheckDecisions";
import { changelogUrlForVersion, DOWNLOAD_PAGE_URL } from "./updateLinks";

describe("compareVersions", () => {
  it("compares each component as a number", () => {
    expect(compareVersions("0.10.0", "0.9.0")).toBeGreaterThan(0);
    expect(compareVersions("0.1.3", "0.1.4")).toBeLessThan(0);
    expect(compareVersions("1.2.3", "1.2.3")).toBe(0);
  });

  it("treats missing components as zero and ignores a v prefix and suffixes", () => {
    expect(compareVersions("1.2", "1.2.0")).toBe(0);
    expect(compareVersions("v0.2.0", "0.2.0")).toBe(0);
    expect(compareVersions("0.2.0-beta.1", "0.2.0")).toBe(0);
  });

  it("returns null for something that is not a version", () => {
    expect(compareVersions("nightly", "0.1.0")).toBeNull();
    expect(compareVersions("0.1.0", "")).toBeNull();
    expect(compareVersions("1..2", "1.0.2")).toBeNull();
  });
});

describe("shouldCheckForUpdateNow", () => {
  const now = 1_800_000_000_000;

  it("checks when it never checked before", () => {
    expect(shouldCheckForUpdateNow(null, now)).toBe(true);
  });

  it("checks at most once a day", () => {
    expect(shouldCheckForUpdateNow(now - 60_000, now)).toBe(false);
    expect(shouldCheckForUpdateNow(now - UPDATE_CHECK_INTERVAL_MILLISECONDS + 1, now)).toBe(false);
    expect(shouldCheckForUpdateNow(now - UPDATE_CHECK_INTERVAL_MILLISECONDS, now)).toBe(true);
  });

  it("checks when the stored time lies in the future", () => {
    expect(shouldCheckForUpdateNow(now + 60_000, now)).toBe(true);
  });
});

describe("parseStoredTimestamp", () => {
  it("reads whole positive numbers and drops everything else", () => {
    expect(parseStoredTimestamp("1800000000000")).toBe(1_800_000_000_000);
    expect(parseStoredTimestamp(null)).toBeNull();
    expect(parseStoredTimestamp("")).toBeNull();
    expect(parseStoredTimestamp("soon")).toBeNull();
    expect(parseStoredTimestamp("-5")).toBeNull();
    expect(parseStoredTimestamp("0")).toBeNull();
    expect(parseStoredTimestamp("99999999999999999999")).toBeNull();
  });
});

describe("shouldOfferUpdate", () => {
  it("offers every version except the one the user skipped", () => {
    expect(shouldOfferUpdate("0.2.0", null)).toBe(true);
    expect(shouldOfferUpdate("0.2.0", "0.2.0")).toBe(false);
    expect(shouldOfferUpdate("0.3.0", "0.2.0")).toBe(true);
  });
});

describe("describeVersionChangeSinceLastRun", () => {
  it("only records the version on a first run", () => {
    expect(describeVersionChangeSinceLastRun(null, "0.2.0")).toEqual({ kind: "firstRun" });
  });

  it("reports an update when the version went up", () => {
    expect(describeVersionChangeSinceLastRun("0.1.3", "0.2.0")).toEqual({
      kind: "updated",
      version: "0.2.0",
    });
  });

  it("stays quiet for the same, an older or an unreadable version", () => {
    expect(describeVersionChangeSinceLastRun("0.2.0", "0.2.0")).toEqual({ kind: "unchanged" });
    expect(describeVersionChangeSinceLastRun("0.3.0", "0.2.0")).toEqual({ kind: "unchanged" });
    expect(describeVersionChangeSinceLastRun("garbage", "0.2.0")).toEqual({ kind: "unchanged" });
  });
});

describe("update links", () => {
  it("point at the app's website", () => {
    expect(DOWNLOAD_PAGE_URL).toBe("https://storagemap.vexbyte.com/");
    expect(changelogUrlForVersion("0.2.0")).toBe("https://storagemap.vexbyte.com/changelog/#0.2.0");
  });
});
