import { describe, expect, it } from "vitest";
import type { DuplicateGroup } from "@/lib/backend/generated";
import { describeDuplicateGroup, estimateGroupCardHeight } from "./describeDuplicateGroup";

const now = Date.UTC(2026, 9, 17, 12);
const day = 24 * 60 * 60 * 1000;

const group: DuplicateGroup = {
  id: 7,
  fileSize: 2 * 1024 * 1024,
  wastedBytes: 4 * 1024 * 1024,
  files: [
    {
      path: "C:\\Users\\Ana\\Pictures\\beach.jpg",
      modifiedUnixMilliseconds: now - 3 * day,
      createdUnixMilliseconds: now - 3 * day,
      keep: true,
    },
    {
      path: "D:\\Backup\\beach.jpg",
      modifiedUnixMilliseconds: now - day,
      createdUnixMilliseconds: now - day,
      keep: false,
    },
    {
      path: "D:\\Backup\\Old\\beach.jpg",
      modifiedUnixMilliseconds: now - day,
      createdUnixMilliseconds: now - day,
      keep: false,
    },
  ],
};

describe("describeDuplicateGroup", () => {
  it("summarizes copies, waste and removals", () => {
    const description = describeDuplicateGroup(group, now);
    expect(description.copiesText).toBe("3 copies of 2.0 MB");
    expect(description.wastedText).toBe("4.0 MB wasted");
    expect(description.removalText).toBe("2 copies to remove");
  });

  it("labels the group's recycle button with the marked copies only", () => {
    const description = describeDuplicateGroup(group, now);
    expect(description.markedForRemovalCount).toBe(2);
    expect(description.recycleMarkedButtonText).toBe("Move 2 marked (4.0 MB) to Recycle Bin");
    expect(description.canRecycleSingleFile).toBe(true);
  });

  it("describes each copy with name, folder, date and category", () => {
    const [firstFile] = describeDuplicateGroup(group, now).files;
    expect(firstFile).toMatchObject({
      fileName: "beach.jpg",
      folderPath: "C:\\Users\\Ana\\Pictures",
      shortFolderPath: "C:\\Users\\Ana\\Pictures",
      modifiedRelativeText: "3 days ago",
      category: "images",
      keep: true,
    });
  });

  it("says when every copy is kept", () => {
    const allKept = { ...group, files: group.files.map((file) => ({ ...file, keep: true })) };
    expect(describeDuplicateGroup(allKept, now).removalText).toBe("Keeping every copy");
  });

  it("estimates taller cards for more copies", () => {
    expect(estimateGroupCardHeight(3)).toBeGreaterThan(estimateGroupCardHeight(2));
    expect(estimateGroupCardHeight(null)).toBe(estimateGroupCardHeight(2));
  });
});
