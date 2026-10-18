import type { LinuxPackageKind } from "./repository";

export type LinuxPackageDescription = {
  label: string;
  distributions: string;
};

export const LINUX_PACKAGE_DESCRIPTIONS: Record<LinuxPackageKind, LinuxPackageDescription> = {
  deb: { label: ".deb", distributions: "Debian, Ubuntu, Mint" },
  rpm: { label: ".rpm", distributions: "Fedora, RHEL, openSUSE" },
  appimage: { label: "AppImage", distributions: "Arch and other distros" },
};
