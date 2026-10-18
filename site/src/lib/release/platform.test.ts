import { describe, expect, it } from "vitest";
import {
  classifyPlatform,
  isRequestablePlatform,
  preferredLinuxPackageKind,
  unsupportedPlatformName,
} from "./platform";

const WINDOWS_CHROME =
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36";
const WINDOWS_FIREFOX =
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:144.0) Gecko/20100101 Firefox/144.0";
const MAC_SAFARI =
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";
const ANDROID_CHROME =
  "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Mobile Safari/537.36";
const IPHONE =
  "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1";
const LINUX_FIREFOX = "Mozilla/5.0 (X11; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0";
const LINUX_ARM_FIREFOX = "Mozilla/5.0 (X11; Linux aarch64; rv:144.0) Gecko/20100101 Firefox/144.0";
const LINUX_CHROME =
  "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36";
const UBUNTU_FIREFOX =
  "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0";
const FEDORA_FIREFOX =
  "Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:144.0) Gecko/20100101 Firefox/144.0";

describe("classifyPlatform", () => {
  it("reads Windows x64 from the user agent", () => {
    expect(classifyPlatform(WINDOWS_CHROME)).toBe("windows-x64");
    expect(classifyPlatform(WINDOWS_FIREFOX)).toBe("windows-x64");
  });

  it("prefers client hints, which reveal ARM behind a frozen user agent", () => {
    expect(classifyPlatform(WINDOWS_CHROME, { platform: "Windows", architecture: "arm" })).toBe(
      "windows-arm64",
    );
    expect(
      classifyPlatform(WINDOWS_CHROME, { platform: "Windows", architecture: "x86", bitness: "32" }),
    ).toBe("windows-32");
    expect(
      classifyPlatform(WINDOWS_CHROME, { platform: "Windows", architecture: "x86", bitness: "64" }),
    ).toBe("windows-x64");
  });

  it("tells x64 Linux from ARM Linux", () => {
    expect(classifyPlatform(LINUX_FIREFOX)).toBe("linux");
    expect(classifyPlatform(LINUX_ARM_FIREFOX)).toBe("linux-arm64");
    expect(classifyPlatform(LINUX_CHROME, { platform: "Linux", architecture: "x86" })).toBe(
      "linux",
    );
    expect(classifyPlatform(LINUX_CHROME, { platform: "Linux", architecture: "arm" })).toBe(
      "linux-arm64",
    );
  });

  it("recognises platforms the downloads do not support", () => {
    expect(classifyPlatform(MAC_SAFARI)).toBe("mac");
    expect(classifyPlatform(ANDROID_CHROME)).toBe("mobile");
    expect(classifyPlatform(IPHONE)).toBe("mobile");
    expect(classifyPlatform(ANDROID_CHROME, { platform: "Android" })).toBe("mobile");
  });

  it("falls back to unknown", () => {
    expect(classifyPlatform("")).toBe("unknown");
  });
});

describe("unsupportedPlatformName", () => {
  it("names every platform that can't install, and nothing else", () => {
    expect(unsupportedPlatformName("mobile")).toBe("phones and tablets");
    expect(unsupportedPlatformName("mac")).toBe("macOS");
    expect(unsupportedPlatformName("linux-arm64")).toBe("ARM Linux");
    expect(unsupportedPlatformName("linux")).toBeNull();
    expect(unsupportedPlatformName("windows-32")).toBe("32-bit Windows");
    expect(unsupportedPlatformName("windows-x64")).toBeNull();
    expect(unsupportedPlatformName("windows-arm64")).toBeNull();
    expect(unsupportedPlatformName("unknown")).toBeNull();
  });
});

describe("isRequestablePlatform", () => {
  it("lets only macOS visitors request a build", () => {
    expect(isRequestablePlatform("mac")).toBe(true);
    expect(isRequestablePlatform("linux")).toBe(false);
    expect(isRequestablePlatform("linux-arm64")).toBe(false);
    expect(isRequestablePlatform("mobile")).toBe(false);
    expect(isRequestablePlatform("windows-32")).toBe(false);
    expect(isRequestablePlatform("windows-x64")).toBe(false);
  });
});

describe("preferredLinuxPackageKind", () => {
  it("picks the package that suits the distribution in the user agent", () => {
    expect(preferredLinuxPackageKind(UBUNTU_FIREFOX)).toBe("deb");
    expect(preferredLinuxPackageKind(FEDORA_FIREFOX)).toBe("rpm");
    expect(preferredLinuxPackageKind(LINUX_FIREFOX)).toBe("appimage");
  });
});
