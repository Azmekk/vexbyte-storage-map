import { describe, expect, it } from "vitest";
import type { DuplicateFile, DuplicateGroup } from "@/lib/backend/generated";
import { describeGroupRecycleButton, describeRecycleScope } from "./describeRecycleScope";
import {
  canRecycleSingleFile,
  groupRecycleRequest,
  singleFileRecycleRequest,
} from "./recycleRequests";

const TWO_MEGABYTES = 2 * 1024 * 1024;

function createFile(path: string, keep: boolean): DuplicateFile {
  return { path, keep, modifiedUnixMilliseconds: 0, createdUnixMilliseconds: 0 };
}

function createGroup(files: DuplicateFile[]): DuplicateGroup {
  return { id: 4, fileSize: TWO_MEGABYTES, files, wastedBytes: TWO_MEGABYTES * (files.length - 1) };
}

const threeCopies = createGroup([
  createFile("C:PhotosIMG_0001.jpg", true),
  createFile("C:BackupIMG_0001.jpg", false),
  createFile("C:BackupIMG_0001 (copy).jpg", false),
]);

describe("recycle requests", () => {
  it("moves exactly the chosen copy, even one marked Keep", () => {
    const request = singleFileRecycleRequest(threeCopies, "C:PhotosIMG_0001.jpg");

    expect(request?.filesToRemove).toEqual([
      { path: "C:PhotosIMG_0001.jpg", fileSize: TWO_MEGABYTES, category: "images" },
    ]);
    expect(request?.scope).toEqual({ kind: "file", remainingCopyCount: 2, isOnlyKeptCopy: true });
  });

  it("knows a copy marked Remove is not the kept one", () => {
    const request = singleFileRecycleRequest(threeCopies, "C:BackupIMG_0001.jpg");
    expect(request?.scope).toEqual({ kind: "file", remainingCopyCount: 2, isOnlyKeptCopy: false });
  });

  it("refuses unknown paths and groups without a second copy", () => {
    expect(singleFileRecycleRequest(threeCopies, "C:elsewhere.jpg")).toBeNull();

    const single = createGroup([createFile("C:a.jpg", true)]);
    expect(canRecycleSingleFile(single)).toBe(false);
    expect(singleFileRecycleRequest(single, "C:a.jpg")).toBeNull();
    expect(canRecycleSingleFile(threeCopies)).toBe(true);
  });

  it("moves only the copies of a group marked Remove", () => {
    const request = groupRecycleRequest(threeCopies);

    expect(request?.filesToRemove.map((file) => file.path)).toEqual([
      "C:BackupIMG_0001.jpg",
      "C:BackupIMG_0001 (copy).jpg",
    ]);
    expect(request?.scope).toEqual({ kind: "group", remainingCopyCount: 1 });
  });

  it("has nothing to move for a group with every copy kept", () => {
    const allKept = createGroup([createFile("C:a.jpg", true), createFile("C:\b.jpg", true)]);
    expect(groupRecycleRequest(allKept)).toBeNull();
  });
});

describe("describeRecycleScope", () => {
  const oneFile = { fileCount: 1, readableSize: "2.0 MB" };

  it("describes a single file", () => {
    const text = describeRecycleScope(
      { kind: "file", remainingCopyCount: 2, isOnlyKeptCopy: false },
      oneFile,
    );

    expect(text.title).toBe("Move 1 file (2.0 MB) to Recycle Bin");
    expect(text.description).toContain("Recycle Bin");
    expect(text.description).toContain("restore it");
    expect(text.note).toBe("The other 2 copies stay where they are.");
    expect(text.confirmLabel).toBe("Move 1 file to Recycle Bin");
  });

  it("says when the chosen file is the kept copy", () => {
    const text = describeRecycleScope(
      { kind: "file", remainingCopyCount: 2, isOnlyKeptCopy: true },
      oneFile,
    );
    expect(text.note).toBe(
      "This is the copy marked Keep. The other 2 copies stay where they are and are all marked Keep afterwards.",
    );
  });

  it("says when a group leaves the list", () => {
    expect(
      describeRecycleScope({ kind: "file", remainingCopyCount: 1, isOnlyKeptCopy: true }, oneFile)
        .note,
    ).toBe("The other copy stays where it is, and the group leaves the list.");
    expect(
      describeRecycleScope(
        { kind: "group", remainingCopyCount: 1 },
        { fileCount: 2, readableSize: "4.0 MB" },
      ).note,
    ).toBe("The copy marked Keep stays where it is, and the group leaves the list.");
  });

  it("describes a group's marked copies", () => {
    const text = describeRecycleScope(
      { kind: "group", remainingCopyCount: 2 },
      { fileCount: 2, readableSize: "4.0 MB" },
    );

    expect(text.title).toBe("Move 2 marked files (4.0 MB) from this group to Recycle Bin");
    expect(text.note).toBe("The 2 copies marked Keep stay where they are.");
    expect(text.confirmLabel).toBe("Move 2 files to Recycle Bin");
  });

  it("describes every marked copy, also while the list loads", () => {
    const text = describeRecycleScope(
      { kind: "allMarked" },
      { fileCount: 1204, readableSize: "3.4 GB" },
    );

    expect(text.title).toBe("Move all 1,204 marked files (3.4 GB) to Recycle Bin");
    expect(text.description).toContain("At least one copy of every file stays");
    expect(text.note).toBeNull();
    expect(describeRecycleScope({ kind: "allMarked" }, null).title).toBe(
      "Move all marked files to Recycle Bin",
    );
    expect(describeRecycleScope({ kind: "allMarked" }, oneFile).title).toBe(
      "Move 1 marked file (2.0 MB) to Recycle Bin",
    );
  });

  it("labels a group's button", () => {
    expect(describeGroupRecycleButton(2, "4.0 MB")).toBe("Move 2 marked (4.0 MB) to Recycle Bin");
    expect(describeGroupRecycleButton(0, "0 B")).toBe("Move 0 marked (0 B) to Recycle Bin");
  });
});
