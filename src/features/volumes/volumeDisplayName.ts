import type { DriveType, VolumeSummary } from "@/lib/backend/volumes";
import { currentPlatform } from "@/lib/platform/currentPlatform";

const FALLBACK_NAME_BY_DRIVE_TYPE: Record<DriveType, string> = {
  fixed: "Local Disk",
  removable: "Removable Disk",
  network: "Network Drive",
  optical: "Optical Drive",
  ram: "RAM Disk",
  unknown: "Drive",
};

const ROOT_FILE_SYSTEM_NAME = "File System";

/**
 * A name for a Linux mount point without a label: its last folder name, for example `USB` for
 * `/media/martin/USB`, or `File System` for `/`.
 */
export function mountPointFallbackName(mountPoint: string): string {
  const pathComponents = mountPoint.split("/").filter((component) => component.length > 0);
  return pathComponents.at(-1) ?? ROOT_FILE_SYSTEM_NAME;
}

/**
 * The volume label, or a fallback when there is none: on Windows the name Explorer shows (for
 * example `Local Disk`), on Linux the last folder name of the mount point.
 */
export function volumeDisplayName(volume: VolumeSummary): string {
  const trimmedLabel = volume.label.trim();
  if (trimmedLabel.length > 0) {
    return trimmedLabel;
  }

  return currentPlatform().os === "windows"
    ? FALLBACK_NAME_BY_DRIVE_TYPE[volume.driveType]
    : mountPointFallbackName(volume.mountPoint);
}

const TRAILING_PATH_SEPARATORS = /[\\/]+$/;

/** The mount point without its trailing separator, for example `C:` for `C:\`, or `/home` as is. */
export function volumeDriveLabel(volume: VolumeSummary): string {
  const trimmedMountPoint = volume.mountPoint.replace(TRAILING_PATH_SEPARATORS, "");
  return trimmedMountPoint.length > 0 ? trimmedMountPoint : volume.mountPoint;
}
