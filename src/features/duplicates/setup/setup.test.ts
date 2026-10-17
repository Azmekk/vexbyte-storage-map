import { describe, expect, it } from "vitest";
import {
  buildDuplicateSearchSettings,
  describeInvalidSettings,
  describeSetupBlocker,
} from "./buildDuplicateSearchSettings";
import {
  ALL_FILE_CATEGORIES,
  fileCategoryForPath,
  toggleFileCategory,
} from "./fileCategoryOptions";
import {
  addCustomFolderChoice,
  folderChoicesFromSuggestions,
  removeFolderChoice,
  toggleFolderChoice,
} from "./folderChoices";
import {
  DEFAULT_MINIMUM_FILE_SIZE_BYTES,
  MINIMUM_SIZE_PRESETS,
  minimumSizePresetFromValue,
  minimumSizePresetValue,
} from "./minimumSizePresets";

describe("minimum size presets", () => {
  it("offers 64 KB to 100 MB with 64 KB as the default", () => {
    expect(MINIMUM_SIZE_PRESETS.map((preset) => preset.bytes)).toEqual([
      65_536, 262_144, 1_048_576, 10_485_760, 104_857_600,
    ]);
    expect(MINIMUM_SIZE_PRESETS.map((preset) => preset.label)).toEqual([
      "64 KB",
      "256 KB",
      "1 MB",
      "10 MB",
      "100 MB",
    ]);
    expect(DEFAULT_MINIMUM_FILE_SIZE_BYTES).toBe(65_536);
  });

  it("round-trips presets through select values", () => {
    for (const preset of MINIMUM_SIZE_PRESETS) {
      expect(minimumSizePresetFromValue(minimumSizePresetValue(preset))).toEqual(preset);
    }
  });

  it("falls back to the default for unknown values", () => {
    expect(minimumSizePresetFromValue("12345").bytes).toBe(DEFAULT_MINIMUM_FILE_SIZE_BYTES);
    expect(minimumSizePresetFromValue("not a number").label).toBe("64 KB");
  });
});

describe("file categories", () => {
  it("lists all eight categories", () => {
    expect(ALL_FILE_CATEGORIES).toHaveLength(8);
  });

  it("finds the category of a path by extension, ignoring case", () => {
    expect(fileCategoryForPath("C:\\Photos\\IMG_0001.JPG")).toBe("images");
    expect(fileCategoryForPath("C:\\Docs\\report.pdf")).toBe("pdfs");
    expect(fileCategoryForPath("C:\\Setup\\installer.msi")).toBe("archivesAndInstallers");
    expect(fileCategoryForPath("C:\\Data\\notes")).toBeNull();
    expect(fileCategoryForPath("C:\\folder.jpg\\notes")).toBeNull();
    expect(fileCategoryForPath("C:\\Data\\.gitignore")).toBeNull();
  });

  it("toggles a category and keeps the canonical order", () => {
    const withoutImages = toggleFileCategory(ALL_FILE_CATEGORIES, "images");
    expect(withoutImages).not.toContain("images");
    expect(toggleFileCategory(withoutImages, "images")).toEqual([...ALL_FILE_CATEGORIES]);
  });
});

describe("folder choices", () => {
  const suggestions = [
    { name: "Pictures", path: "C:\\Users\\Ana\\Pictures" },
    { name: "Downloads", path: "C:\\Users\\Ana\\Downloads" },
  ];

  it("selects every suggested folder", () => {
    const choices = folderChoicesFromSuggestions(suggestions);
    expect(choices.every((choice) => choice.isSelected && !choice.isCustom)).toBe(true);
  });

  it("adds a custom folder named after its last segment", () => {
    const choices = addCustomFolderChoice(
      folderChoicesFromSuggestions(suggestions),
      "D:\\Backups\\Phone\\",
    );
    expect(choices.at(-1)).toEqual({
      path: "D:\\Backups\\Phone\\",
      name: "Phone",
      isCustom: true,
      isSelected: true,
    });
  });

  it("re-selects a folder that is already listed instead of adding it twice", () => {
    const unselected = toggleFolderChoice(
      folderChoicesFromSuggestions(suggestions),
      "C:\\Users\\Ana\\Pictures",
    );
    const choices = addCustomFolderChoice(unselected, "c:\\users\\ana\\pictures\\");
    expect(choices).toHaveLength(2);
    expect(choices[0]?.isSelected).toBe(true);
  });

  it("removes a folder", () => {
    const choices = removeFolderChoice(
      folderChoicesFromSuggestions(suggestions),
      "C:\\Users\\Ana\\Downloads",
    );
    expect(choices.map((choice) => choice.name)).toEqual(["Pictures"]);
  });
});

describe("search settings", () => {
  const folderChoices = [
    { path: "C:\\A", name: "A", isCustom: false, isSelected: true },
    { path: "C:\\B", name: "B", isCustom: true, isSelected: false },
  ];

  it("sends only the selected folders", () => {
    expect(
      buildDuplicateSearchSettings({
        folderChoices,
        selectedCategories: ["images", "pdfs"],
        minimumFileSizeBytes: 1024,
      }),
    ).toEqual({ folders: ["C:\\A"], categories: ["images", "pdfs"], minimumFileSizeBytes: 1024 });
  });

  it("explains what is missing before a search can start", () => {
    const noFolders = folderChoices.map((choice) => ({ ...choice, isSelected: false }));
    expect(
      describeSetupBlocker({
        folderChoices: noFolders,
        selectedCategories: ["images"],
        minimumFileSizeBytes: 1,
      }),
    ).toBe("Choose at least one folder to search.");
    expect(
      describeSetupBlocker({ folderChoices, selectedCategories: [], minimumFileSizeBytes: 1 }),
    ).toBe("Choose at least one kind of file to compare.");
    expect(
      describeSetupBlocker({
        folderChoices,
        selectedCategories: ["audio"],
        minimumFileSizeBytes: 1,
      }),
    ).toBeNull();
  });

  it("turns backend validation details into a sentence", () => {
    expect(
      describeInvalidSettings(
        "C:\\Windows is a system or application folder and cannot be searched",
      ),
    ).toBe("C:\\Windows is a system or application folder and cannot be searched.");
    expect(describeInvalidSettings("no folders were selected")).toBe("No folders were selected.");
    expect(describeInvalidSettings("  ")).toBe("These settings can't be used for a search.");
  });
});
