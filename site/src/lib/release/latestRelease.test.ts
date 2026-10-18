import { describe, expect, it } from "vitest";
import { fallbackRelease, releaseFromGitHub } from "./latestRelease";

const DOWNLOAD_URL = "https://github.com/Azmekk/vexbyte-storage-map/releases/download/v0.2.0";
const INSTALLER_URL = `${DOWNLOAD_URL}/Vexbyte.Storage.Map_0.2.0_x64-setup.exe`;

describe("releaseFromGitHub", () => {
  it("picks every platform's file by suffix and keeps the sizes", () => {
    const release = releaseFromGitHub({
      // biome-ignore lint/style/useNamingConvention: GitHub API field name.
      tag_name: "v0.2.0",
      assets: [
        { name: "Vexbyte.Storage.Map_0.2.0_x64-setup.exe.sig", size: 400 },
        { name: "Vexbyte.Storage.Map_0.2.0_x64-setup.exe", size: 3_219_495 },
        { name: "Vexbyte.Storage.Map_0.2.0_amd64.AppImage", size: 80_000_000 },
        { name: "Vexbyte.Storage.Map_0.2.0_amd64.AppImage.sig", size: 400 },
        { name: "Vexbyte.Storage.Map-0.2.0-1.x86_64.rpm", size: 5_000_000 },
        { name: "Vexbyte.Storage.Map_0.2.0_amd64.deb", size: 4_000_000 },
      ],
    });
    expect(release).toEqual({
      version: "0.2.0",
      windowsInstaller: {
        fileName: "Vexbyte.Storage.Map_0.2.0_x64-setup.exe",
        url: INSTALLER_URL,
        sizeInBytes: 3_219_495,
      },
      linuxPackages: [
        {
          kind: "deb",
          fileName: "Vexbyte.Storage.Map_0.2.0_amd64.deb",
          url: `${DOWNLOAD_URL}/Vexbyte.Storage.Map_0.2.0_amd64.deb`,
          sizeInBytes: 4_000_000,
        },
        {
          kind: "rpm",
          fileName: "Vexbyte.Storage.Map-0.2.0-1.x86_64.rpm",
          url: `${DOWNLOAD_URL}/Vexbyte.Storage.Map-0.2.0-1.x86_64.rpm`,
          sizeInBytes: 5_000_000,
        },
        {
          kind: "appimage",
          fileName: "Vexbyte.Storage.Map_0.2.0_amd64.AppImage",
          url: `${DOWNLOAD_URL}/Vexbyte.Storage.Map_0.2.0_amd64.AppImage`,
          sizeInBytes: 80_000_000,
        },
      ],
    });
  });

  it("still links to the installer when its asset is missing from the list", () => {
    // biome-ignore lint/style/useNamingConvention: GitHub API field name.
    const release = releaseFromGitHub({ tag_name: "v0.2.0", assets: [] });
    expect(release.windowsInstaller.url).toBe(INSTALLER_URL);
    expect(release.windowsInstaller.sizeInBytes).toBeNull();
    expect(release.linuxPackages).toEqual([]);
  });
});

describe("fallbackRelease", () => {
  it("builds the installer link from the version and knows no Linux files", () => {
    const release = fallbackRelease("0.1.3");
    expect(release.windowsInstaller.url).toBe(
      "https://github.com/Azmekk/vexbyte-storage-map/releases/download/v0.1.3/Vexbyte.Storage.Map_0.1.3_x64-setup.exe",
    );
    expect(release.linuxPackages).toEqual([]);
  });
});
