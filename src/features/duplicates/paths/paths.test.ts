import { describe, expect, it } from "vitest";
import { isPathInsideFolder, isSameFolder } from "./isPathInsideFolder";
import { splitFilePath } from "./splitFilePath";
import { truncatePathMiddle } from "./truncatePathMiddle";

describe("splitFilePath", () => {
  it("splits on the last separator of either kind", () => {
    expect(splitFilePath("C:\\Users\\Ana\\photo.jpg")).toEqual({
      fileName: "photo.jpg",
      folderPath: "C:\\Users\\Ana",
    });
    expect(splitFilePath("C:/Users/Ana/photo.jpg").fileName).toBe("photo.jpg");
    expect(splitFilePath("photo.jpg")).toEqual({ fileName: "photo.jpg", folderPath: "" });
  });
});

describe("truncatePathMiddle", () => {
  it("leaves short paths alone", () => {
    expect(truncatePathMiddle("C:\\Users", 20)).toBe("C:\\Users");
  });

  it("cuts the middle and keeps both ends", () => {
    const path = "C:\\Users\\Ana\\Pictures\\Holidays\\2024\\Portugal\\Lisbon";
    const truncated = truncatePathMiddle(path, 30);
    expect(truncated).toHaveLength(30);
    expect(truncated.startsWith("C:\\Users")).toBe(true);
    expect(truncated.endsWith("Portugal\\Lisbon")).toBe(true);
    expect(truncated).toContain("…");
  });

  it("returns only the ellipsis when there is no room", () => {
    expect(truncatePathMiddle("C:\\Users\\Ana", 1)).toBe("…");
  });
});

describe("isPathInsideFolder", () => {
  it("matches the folder and everything below it, ignoring case", () => {
    expect(isPathInsideFolder("C:\\Photos\\a.jpg", "c:\\photos\\")).toBe(true);
    expect(isPathInsideFolder("C:\\Photos", "C:\\Photos")).toBe(true);
    expect(isPathInsideFolder("C:\\Photos Old\\a.jpg", "C:\\Photos")).toBe(false);
    expect(isPathInsideFolder("C:\\Photos\\a.jpg", "")).toBe(false);
  });

  it("compares folders ignoring separator style and trailing separators", () => {
    expect(isSameFolder("C:/Photos/", "c:\\photos")).toBe(true);
    expect(isSameFolder("C:\\Photos", "C:\\Photo")).toBe(false);
  });
});
