import { describe, expect, it } from "vitest";
import { fallbackRelease, releaseFromGitHub } from "./latestRelease";

const INSTALLER_URL =
  "https://github.com/Azmekk/vexbyte-storage-map/releases/download/v0.2.0/Vexbyte.Storage.Map_0.2.0_x64-setup.exe";

describe("releaseFromGitHub", () => {
  it("links straight to the installer for the latest tag", () => {
    const release = releaseFromGitHub({
      // biome-ignore lint/style/useNamingConvention: GitHub API field name.
      tag_name: "v0.2.0",
      assets: [
        { name: "Vexbyte.Storage.Map_0.2.0_x64-setup.exe.sig", size: 400 },
        { name: "Vexbyte.Storage.Map_0.2.0_x64-setup.exe", size: 3_219_495 },
      ],
    });
    expect(release).toEqual({
      version: "0.2.0",
      installerUrl: INSTALLER_URL,
      sizeInBytes: 3_219_495,
    });
  });

  it("still links to the installer when its asset is missing from the list", () => {
    // biome-ignore lint/style/useNamingConvention: GitHub API field name.
    const release = releaseFromGitHub({ tag_name: "v0.2.0", assets: [] });
    expect(release.installerUrl).toBe(INSTALLER_URL);
    expect(release.sizeInBytes).toBeNull();
  });
});

describe("fallbackRelease", () => {
  it("builds the installer link from the version", () => {
    expect(fallbackRelease("0.1.3").installerUrl).toBe(
      "https://github.com/Azmekk/vexbyte-storage-map/releases/download/v0.1.3/Vexbyte.Storage.Map_0.1.3_x64-setup.exe",
    );
  });
});
