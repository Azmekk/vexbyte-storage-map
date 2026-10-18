import { afterEach, describe, expect, it } from "vitest";
import { describeNodeFlags } from "@/features/explorer/inspector/describeNodeFlags";
import { describeEmptyFolder } from "@/features/explorer/tree/describeEmptyFolder";
import type { NodeFlagSummary } from "@/lib/backend/tree";
import { initializeCurrentPlatform, resetCurrentPlatform } from "./currentPlatform";
import { platformWording, wordingForOperatingSystem } from "./platformWording";

const NO_FLAGS: NodeFlagSummary = {
  inaccessible: false,
  hardLinkCountedElsewhere: false,
  cloudPlaceholder: false,
  compressed: false,
  sparse: false,
  junction: false,
  symlink: false,
  synthetic: false,
  system: false,
  hidden: false,
};

function useLinux(): void {
  initializeCurrentPlatform({ os: "linux", pathsAreCaseSensitive: true, appVersion: "0.2.0" });
}

afterEach(() => {
  resetCurrentPlatform();
});

describe("platformWording", () => {
  it("uses the Windows names until the platform is known", () => {
    expect(platformWording().binName).toBe("Recycle Bin");
    expect(platformWording().fileManagerName).toBe("File Explorer");
  });

  it("uses Linux names on Linux", () => {
    useLinux();
    expect(platformWording()).toEqual(wordingForOperatingSystem("linux"));
    expect(platformWording().binName).toBe("Trash");
    expect(platformWording().fileManagerName).toBe("the file manager");
  });
});

describe("Linux wording in descriptions", () => {
  it("calls a junction a mount point", () => {
    expect(describeNodeFlags({ ...NO_FLAGS, junction: true })[0]?.label).toBe("Junction");
    useLinux();
    expect(describeNodeFlags({ ...NO_FLAGS, junction: true })[0]?.label).toBe("Mount point");
  });

  it("does not suggest administrator rights for a protected folder", () => {
    const facts = { flags: { ...NO_FLAGS, inaccessible: true }, onlineOnlyFileCount: 0 };
    expect(describeEmptyFolder(facts).description).toContain("administrator");
    useLinux();
    const description = describeEmptyFolder(facts);
    expect(description.title).toBe("The system did not let the scan look inside");
    expect(description.description).not.toContain("administrator");
  });
});
