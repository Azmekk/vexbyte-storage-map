import type { DriveType, VolumeSummary } from "@/lib/backend/volumes";

const FALLBACK_NAME_BY_DRIVE_TYPE: Record<DriveType, string> = {
  fixed: "Local Disk",
  removable: "Removable Disk",
  network: "Network Drive",
  optical: "Optical Drive",
  ram: "RAM Disk",
  unknown: "Drive",
};

/** The label Explorer would show, for example `Windows` or `Local Disk` when there is none. */
export function volumeDisplayName(volume: VolumeSummary): string {
  const trimmedLabel = volume.label.trim();
  return trimmedLabel.length > 0 ? trimmedLabel : FALLBACK_NAME_BY_DRIVE_TYPE[volume.driveType];
}

const TRAILING_PATH_SEPARATORS = /[\\/]+$/;

/** The mount point without its trailing separator, for example `C:` for `C:\`. */
export function volumeDriveLabel(volume: VolumeSummary): string {
  const trimmedMountPoint = volume.mountPoint.replace(TRAILING_PATH_SEPARATORS, "");
  return trimmedMountPoint.length > 0 ? trimmedMountPoint : volume.mountPoint;
}
